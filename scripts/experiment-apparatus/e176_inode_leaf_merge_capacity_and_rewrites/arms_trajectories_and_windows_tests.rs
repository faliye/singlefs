use super::*;

#[test]
fn one_record_per_leaf_anchor_leaves_drop_empty_only_with_134_plus_233_live_inodes() {
    let observed = anchor_one_record_per_leaf_then_insert_until_refused(ARM_DROP_EMPTY_ONLY);
    assert_eq!(observed, format!("refused_insert={} live={} containers=135 merges=0 borrows=0", 31_455 + 233, 134 + 233));
}

fn assert_three_threads_render_the_same_lines_as_one(arms: &[ArmVariant], windows: &[(&'static str, [usize; 3])]) {
    let sequences: Vec<RegisteredSequence> = registered_sequences()
        .into_iter()
        .filter(|sequence| ["wr_p055_seed2", "ws_a1000_d500_smallest_first", "wa_keep1of2_a1000", "wa_keep3of4_a1000"].contains(&sequence.name))
        .collect();
    assert_eq!(sequences.len(), 4);
    let render = |results: Vec<SequenceResult>| results.iter().flat_map(sequence_lines).collect::<Vec<_>>().join("\n");
    let single = render(run_sequences(&sequences, arms, windows, 1, 200_000));
    let multi = render(run_sequences(&sequences, arms, windows, 3, 200_000));
    assert!(!single.is_empty());
    assert_eq!(single, multi, "按序列分线程跑，合并结果与单线程逐字相同（{} 个臂变体）", arms.len());
}

#[test]
fn multithreaded_and_single_threaded_runs_are_byte_identical() {
    assert_three_threads_render_the_same_lines_as_one(&SEGMENT_ONE_ARMS, &SEGMENT_ONE_WINDOWS);
}

#[test]
fn segment_two_multithreaded_and_single_threaded_runs_are_byte_identical() {
    assert_three_threads_render_the_same_lines_as_one(&SEGMENT_TWO_ARMS, &SEGMENT_TWO_WINDOWS);
}

fn two_leaf_tree(policy: MergePolicy, left: Vec<u64>, right_number: u64, right: Vec<u64>) -> InodeLeafTree {
    InodeLeafTree::from_containers(
        policy,
        LeafContainerLimit::OneRootNode,
        vec![LeafContainer { container_number: left[0], inode_numbers: left }, LeafContainer { container_number: right_number, inode_numbers: right }],
    )
}

#[test]
fn structural_check_turns_red_on_each_broken_shape() {
    let overlapping = two_leaf_tree(MergePolicy::DropEmptyOnly, vec![1, 5], 3, vec![4, 6]);
    assert!(check_structure_after_publish(&overlapping, &[]).is_err(), "左片最大号 5 ≥ 右片最小号 4 要红");
    let third_sentence = two_leaf_tree(MergePolicy::DropEmptyOnly, vec![1, 5], 3, vec![6, 7]);
    assert!(check_structure_after_publish(&third_sentence, &[]).is_err(), "M1 上左片最大号 5 ≥ 右片容器号 3 要红");
    let borrowed = two_leaf_tree(ARM_ABSORB_AND_BORROW_MAIN.policy, ascending_run(1, 120), 100, ascending_run(121, 240));
    assert_eq!(check_structure_after_publish(&borrowed, &[]), Ok(1), "M3 上同一形态只计数：1 对");
    let thin_pair = two_leaf_tree(ARM_ABSORB_RIGHT_MAIN.policy, ascending_run(1, 100), 101, ascending_run(101, 233));
    assert!(check_structure_after_publish(&thin_pair, &[]).is_err(), "M2 上相邻两片之和 233 < 234 要红");
    let unsorted = two_leaf_tree(MergePolicy::DropEmptyOnly, vec![1, 3, 2], 10, vec![10]);
    assert!(check_structure_after_publish(&unsorted, &[0]).is_err(), "动过的那片片内不升序要红");
}

#[test]
fn merge_and_borrow_arms_keep_adjacent_pairs_above_one_leaf_through_a_random_run() {
    for arm in [ARM_ABSORB_RIGHT_MAIN, ARM_ABSORB_AND_BORROW_MAIN, ARM_ABSORB_AND_BORROW_FLOOR_59, ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR, ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST, ARM_ABSORB_AND_BORROW_LEFT_PAIR_FIRST] {
        let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
        let mut generator = WorkloadGenerator::new(Workload::RandomInsertDelete { insert_probability_percent: 60 }, 7);
        for _ in 0..50_000 {
            let touched = match generator.next_operation() {
                Operation::Create(inode) => {
                    assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "五万步到不了 135 片");
                    vec![tree.containers.len() - 1]
                }
                Operation::Unlink(inode) => {
                    let rewrites = tree.remove(inode);
                    assert!(rewrites.content_notation_count() <= content_notation_bound_per_removal(&tree));
                    rewrites.content_rewritten_positions
                }
            };
            check_structure_after_publish(&tree, &touched).unwrap_or_else(|violation| panic!("臂 {}：{violation}", arm.name));
        }
    }
}

#[test]
fn segment_two_windows_put_each_registered_variant_against_drop_empty_only() {
    let names = |triple: [usize; 3]| triple.map(|arm_index| SEGMENT_TWO_ARMS[arm_index].name);
    assert_eq!(SEGMENT_TWO_WINDOWS.map(|(label, _)| label), ["main", "just_enough", "left_first"]);
    assert_eq!(names(SEGMENT_TWO_WINDOWS[0].1), ["M1", "M2", "M3"]);
    assert_eq!(names(SEGMENT_TWO_WINDOWS[1].1), ["M1", "M2", "M3_just_enough"]);
    assert_eq!(names(SEGMENT_TWO_WINDOWS[2].1), ["M1", "M2_left_first", "M3_left_first"]);
    assert_eq!(
        ARM_ABSORB_AND_BORROW_LEFT_PAIR_FIRST.policy,
        MergePolicy::AbsorbRightAndBorrowFromRight { order: NeighbourCheckOrder::LeftPairFirst, floor: 117, amount: BorrowAmount::HalfTheDifference }
    );
    assert_eq!(
        ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR.policy,
        MergePolicy::AbsorbRightAndBorrowFromRight { order: NeighbourCheckOrder::RightPairFirst, floor: 117, amount: BorrowAmount::JustEnoughForTheFloor }
    );
    assert_eq!(ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST.policy, MergePolicy::AbsorbRightWhenTheyFit { order: NeighbourCheckOrder::LeftPairFirst });
}

fn three_leaf_tree_whose_middle_leaf_fits_with_either_neighbour(arm: ArmVariant) -> InodeLeafTree {
    InodeLeafTree::from_containers(
        arm.policy,
        LeafContainerLimit::OneRootNode,
        vec![
            LeafContainer { container_number: 1, inode_numbers: ascending_run(1, 117) },
            LeafContainer { container_number: 234, inode_numbers: ascending_run(234, 350) },
            LeafContainer { container_number: 467, inode_numbers: ascending_run(467, 583) },
        ],
    )
}

fn removal_outcome(arm: ArmVariant) -> (Vec<usize>, Vec<u64>, Vec<usize>, Vec<usize>) {
    let mut tree = three_leaf_tree_whose_middle_leaf_fits_with_either_neighbour(arm);
    let rewrites = tree.remove(350);
    check_structure_after_publish(&tree, &rewrites.content_rewritten_positions).unwrap_or_else(|violation| panic!("臂 {}：{violation}", arm.name));
    let sizes = tree.containers.iter().map(|container| container.inode_numbers.len()).collect();
    let numbers = tree.containers.iter().map(|container| container.container_number).collect();
    (sizes, numbers, rewrites.content_rewritten_positions, rewrites.position_rewritten_positions)
}

/// 右边那一对先查：中间那片吸收右邻，摘的是最右一格，位置记法只记中间那片。
fn assert_the_middle_leaf_absorbs_its_right_neighbour(arm: ArmVariant) {
    assert_eq!(removal_outcome(arm), (vec![117, 233], vec![1, 234], vec![1], vec![1]), "臂 {}", arm.name);
}

/// 左边那一对先查：左邻吸收中间那片，摘的是中间一格，右邻位置左移，位置记法比右边先查多 1 片。
fn assert_the_left_neighbour_absorbs_the_middle_leaf(arm: ArmVariant) {
    assert_eq!(removal_outcome(arm), (vec![233, 117], vec![1, 467], vec![0], vec![0, 1]), "臂 {}", arm.name);
}

#[test]
fn absorb_right_checking_the_right_pair_first_lets_the_middle_leaf_absorb_its_right_neighbour() {
    assert_the_middle_leaf_absorbs_its_right_neighbour(ARM_ABSORB_RIGHT_MAIN);
}

#[test]
fn absorb_and_borrow_checking_the_right_pair_first_lets_the_middle_leaf_absorb_its_right_neighbour() {
    assert_the_middle_leaf_absorbs_its_right_neighbour(ARM_ABSORB_AND_BORROW_MAIN);
}

#[test]
fn absorb_right_checking_the_left_pair_first_lets_the_left_neighbour_absorb_the_middle_leaf_and_shifts_one_more_position() {
    assert_the_left_neighbour_absorbs_the_middle_leaf(ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST);
}

#[test]
fn absorb_and_borrow_checking_the_left_pair_first_lets_the_left_neighbour_absorb_the_middle_leaf_and_shifts_one_more_position() {
    assert_the_left_neighbour_absorbs_the_middle_leaf(ARM_ABSORB_AND_BORROW_LEFT_PAIR_FIRST);
}
