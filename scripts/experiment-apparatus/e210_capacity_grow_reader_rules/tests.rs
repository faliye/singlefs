use super::*;

fn slot_of(table: Table) -> Slot {
    Slot::SelfVerified {
        generation: 1,
        table,
        carries_intent_marker: false,
    }
}

fn marked_slot_of(table: Table) -> Slot {
    Slot::SelfVerified {
        generation: 1,
        table,
        carries_intent_marker: true,
    }
}

fn board_of(disk_zero: [Slot; 2], disk_one: [Slot; 2]) -> Board {
    [disk_zero, disk_one]
}

fn decision(writable_mount_made: bool, read_only_mount_made: bool, named: DiskSet) -> Decision {
    Decision {
        writable_mount_made,
        read_only_mount_made,
        named_by_read_only: named,
    }
}

fn made_naming(named: DiskSet) -> Decision {
    decision(true, true, named)
}

/// 登记第七节盘面小输入用例 1：混合盘面，每盘一槽 (200, 200)、一槽 (300, 300)，探测 (200, 200)。
#[test]
fn mixed_board_at_the_probe_is_made_by_chain_and_by_probe_arm_and_refused_by_highest_and_by_the_baseline(
) {
    let board = board_of(
        [slot_of([200, 200]), slot_of([300, 300])],
        [slot_of([200, 200]), slot_of([300, 300])],
    );
    let probe = [200, 200];
    assert_eq!(
        arm_decision(Arm::ChainThenLowest, &board, probe),
        made_naming([false, false])
    );
    assert_eq!(
        arm_decision(Arm::HighestTable, &board, probe),
        decision(false, true, [true, true])
    );
    assert_eq!(
        arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
        decision(false, true, [true, true])
    );
    assert_eq!(
        arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe),
        made_naming([true, true])
    );
    assert_eq!(
        arm_decision(Arm::OnlyIfEveryTableIsEqual, &board, probe),
        WHOLE_POOL_REFUSED
    );
}

/// 小输入用例 2：四槽都 (300, 300)，探测 (200, 200)：五条臂一样，可写拒、只读点名两块盘。
#[test]
fn single_table_board_below_its_table_is_refused_writable_and_names_both_disks_on_every_arm() {
    let board = board_of(
        [slot_of([300, 300]), slot_of([300, 300])],
        [slot_of([300, 300]), slot_of([300, 300])],
    );
    for arm in ARMS {
        assert_eq!(
            arm_decision(arm, &board, [200, 200]),
            decision(false, true, [true, true]),
            "{}",
            arm_name(arm)
        );
    }
}

/// 第七节第一张表最后一行：表 (200, 200)、探测 (199, 200) 可写拒、只读点名盘 0；探测 (201, 200) 可写做成、只读不点名。
#[test]
fn shrunk_disk_is_refused_writable_and_named_read_only_while_grown_disk_is_made_and_not_named_on_every_arm(
) {
    let board = board_of(
        [slot_of([200, 200]), slot_of([200, 200])],
        [slot_of([200, 200]), slot_of([200, 200])],
    );
    for arm in ARMS {
        assert_eq!(
            arm_decision(arm, &board, [199, 200]),
            decision(false, true, [true, false]),
            "{}",
            arm_name(arm)
        );
        assert_eq!(
            arm_decision(arm, &board, [201, 200]),
            made_naming([false, false]),
            "{}",
            arm_name(arm)
        );
    }
}

/// 小输入用例 3：每盘一槽 (200, 200) 带标记、一槽 (300, 300)，探测 (200, 200)：丙取标记槽的表而做成，乙拒，甲不看标记也做成。
#[test]
fn marked_slot_makes_the_intent_marker_arm_take_the_old_table_where_the_highest_table_arm_refuses(
) {
    let board = board_of(
        [marked_slot_of([200, 200]), slot_of([300, 300])],
        [marked_slot_of([200, 200]), slot_of([300, 300])],
    );
    let probe = [200, 200];
    assert_eq!(
        arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
        made_naming([false, false])
    );
    assert_eq!(
        arm_decision(Arm::HighestTable, &board, probe),
        decision(false, true, [true, true])
    );
    assert_eq!(
        arm_decision(Arm::ChainThenLowest, &board, probe),
        made_naming([false, false])
    );
}

/// 小输入用例 4：盘 0 两槽 (200, 100)、盘 1 两槽 (100, 200)，探测 (200, 200)：不成链，甲与基线臂整池拒，乙、丙、丁做成。
#[test]
fn tables_that_do_not_form_a_chain_make_the_chain_arm_and_the_baseline_refuse_the_whole_pool() {
    let board = board_of(
        [slot_of([200, 100]), slot_of([200, 100])],
        [slot_of([100, 200]), slot_of([100, 200])],
    );
    let probe = [200, 200];
    assert_eq!(
        arm_decision(Arm::ChainThenLowest, &board, probe),
        WHOLE_POOL_REFUSED
    );
    assert_eq!(
        arm_decision(Arm::OnlyIfEveryTableIsEqual, &board, probe),
        WHOLE_POOL_REFUSED
    );
    assert_eq!(
        arm_decision(Arm::HighestTable, &board, probe),
        made_naming([false, false])
    );
    assert_eq!(
        arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
        made_naming([false, false])
    );
    assert_eq!(
        arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe),
        made_naming([false, false])
    );
}

/// 小输入用例 5：盘 0 两槽 (100, 100)、盘 1 两槽 (200, 200)（盘 0 被换回早镜像），探测 (100, 200)，落点上界 (200, 200)：
/// 甲可写与只读都漏放行；乙、丙拒可写、只读点名盘 0；丁可写做成（放弃）、只读点名盘 0，可写那一半漏放行。
#[test]
fn swapped_in_old_disk_leaks_through_the_chain_arm_both_ways_and_through_the_probe_arm_writable_only(
) {
    let board = board_of(
        [slot_of([100, 100]), slot_of([100, 100])],
        [slot_of([200, 200]), slot_of([200, 200])],
    );
    let probe = [100, 200];
    let beyond = beyond_the_end([200, 200], probe);
    assert_eq!(beyond, [true, false]);
    let chain = arm_decision(Arm::ChainThenLowest, &board, probe);
    assert_eq!(chain, made_naming([false, false]));
    assert!(
        is_leak_through_writable_mount(chain, beyond)
            && is_leak_through_read_only_mount(chain, beyond)
    );
    assert_eq!(
        arm_decision(Arm::HighestTable, &board, probe),
        decision(false, true, [true, false])
    );
    assert_eq!(
        arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
        decision(false, true, [true, false])
    );
    let probe_arm = arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe);
    assert_eq!(probe_arm, made_naming([true, false]));
    assert!(
        is_leak_through_writable_mount(probe_arm, beyond)
            && !is_leak_through_read_only_mount(probe_arm, beyond)
    );
    assert_eq!(
        arm_decision(Arm::OnlyIfEveryTableIsEqual, &board, probe),
        WHOLE_POOL_REFUSED
    );
}

/// 小输入用例 7：盘 0 一槽 (100, 100) 带标记、另一槽 (300, 300)；盘 1 一槽 (200, 200) 带标记、另一槽 (300, 300)；探测 (100, 100)：
/// 丙取各带标记槽的表逐项最小 (100, 100) 而做成；乙取 (300, 300) 而拒；甲取 (100, 100) 而做成。
#[test]
fn two_marked_slots_make_the_intent_marker_arm_take_their_lowest_table() {
    let board = board_of(
        [marked_slot_of([100, 100]), slot_of([300, 300])],
        [marked_slot_of([200, 200]), slot_of([300, 300])],
    );
    let probe = [100, 100];
    assert_eq!(
        arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
        made_naming([false, false])
    );
    assert_eq!(
        arm_decision(Arm::HighestTable, &board, probe),
        decision(false, true, [true, true])
    );
    assert_eq!(
        arm_decision(Arm::ChainThenLowest, &board, probe),
        made_naming([false, false])
    );
}

/// 丁的放弃要求探测值不低于生效表（逐项最小）：每盘一槽 (200, 200)、一槽 (300, 300)，探测 (199, 300) 低于 (200, 200)，放弃不成，可写拒。
#[test]
fn probe_arm_refuses_abandoning_when_the_probe_is_below_the_lowest_table() {
    let board = board_of(
        [slot_of([200, 200]), slot_of([300, 300])],
        [slot_of([200, 200]), slot_of([300, 300])],
    );
    assert_eq!(
        arm_decision(
            Arm::LowestToReadHighestToProbeWithAbandon,
            &board,
            [199, 300]
        ),
        decision(false, true, [true, false])
    );
    assert_eq!(
        arm_decision(
            Arm::LowestToReadHighestToProbeWithAbandon,
            &board,
            [200, 300]
        ),
        made_naming([true, false])
    );
}
/// 丁的放弃分支去掉「有效槽的表不全相同」是等价变异（登记第九节）：表全相同时探测值低于表必然低于生效表，放弃的另一个条件已不成立。
/// 把等价性写成测试：枚举全部表全相同的盘面与探测值。
#[test]
fn abandoning_is_never_reachable_when_every_table_is_the_same() {
    for table in [[100, 100], [200, 200], [300, 100], [100, 300]] {
        let board = board_of(
            [slot_of(table), slot_of(table)],
            [slot_of(table), slot_of(table)],
        );
        for first_probe in [table[0] - 1, table[0], table[0] + 1] {
            for second_probe in [table[1] - 1, table[1], table[1] + 1] {
                let probe = [first_probe, second_probe];
                let outcome = standard_outcome(table, probe);
                let probe_reaches_lowest_table =
                    (0..DISK_COUNT).all(|disk| probe[disk] >= table[disk]);
                assert!(
                    outcome.writable_mount_made || !probe_reaches_lowest_table,
                    "table {table:?} probe {probe:?}"
                );
                assert_eq!(
                    arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe),
                    outcome
                );
            }
        }
    }
}

/// 小输入用例 6：同一份落点上界 (200, 200) 与探测下，做成不点名、做成点名、拒三种结局 × 越界空 / 非空 / 类里有越界，
/// 六个谓词各给出手算的 0 或 1。
#[test]
fn predicates_give_the_hand_computed_answers_on_every_outcome_and_overrun_combination() {
    let landing = [200, 200];
    assert_eq!(
        beyond_the_end(landing, [200, 200]),
        [false, false],
        "落点正好在盘尾不算越界"
    );
    assert_eq!(beyond_the_end(landing, [199, 200]), [true, false]);
    assert_eq!(beyond_the_end(landing, [199, 199]), [true, true]);
    let made_unnamed = made_naming([false, false]);
    let made_named = made_naming([true, false]);
    let refused = decision(false, true, [false, false]);
    let nobody = [false, false];
    let disk_zero = [true, false];
    assert!(!is_leak_through_writable_mount(made_unnamed, nobody));
    assert!(is_leak_through_writable_mount(made_unnamed, disk_zero));
    assert!(is_leak_through_writable_mount(made_named, disk_zero));
    assert!(!is_leak_through_writable_mount(refused, disk_zero));
    assert!(!is_leak_through_read_only_mount(made_unnamed, nobody));
    assert!(is_leak_through_read_only_mount(made_unnamed, disk_zero));
    assert!(!is_leak_through_read_only_mount(made_named, disk_zero));
    assert!(is_leak_through_read_only_mount(made_named, [true, true]));
    assert!(!is_leak_through_read_only_mount(
        WHOLE_POOL_REFUSED,
        disk_zero
    ));
    assert!(is_literal_misrefusal(refused, nobody));
    assert!(!is_literal_misrefusal(refused, disk_zero));
    assert!(!is_literal_misrefusal(made_unnamed, nobody));
    assert!(is_avoidable_misrefusal(refused, nobody));
    assert!(
        !is_avoidable_misrefusal(refused, disk_zero),
        "类里有越界的历史，拒不算可避免"
    );
    assert!(!is_avoidable_misrefusal(made_unnamed, nobody));
}

fn plan_counts(with_intent_marker_round: bool, new_table_rounds: usize) -> (usize, usize) {
    count_writes_and_pool_barriers(&acquisition_write_plan(
        with_intent_marker_round,
        new_table_rounds,
    ))
}

/// 取号写 = 2 轮 × 2 盘 = 4 次槽写，轮后各一道池屏障，开头那道前面没有写的屏障不算：2 道（D22（单元原子性怎么合成） 已定项 8）。
#[test]
fn acquisition_write_plan_of_two_rounds_has_four_writes_and_two_barriers_after_writes() {
    assert_eq!(plan_counts(false, 2), (2 + 2, 1 + 1));
    assert_eq!(
        acquisition_write_plan(false, 2).len(),
        1 + (2 + 1) + (2 + 1),
        "开头一道屏障，之后每轮两次写加一道屏障"
    );
}

/// 丙的写序多一轮标记轮：6 次槽写、3 道屏障（行 2 的读数）。
#[test]
fn plan_with_the_intent_marker_round_has_six_writes_and_three_barriers() {
    assert_eq!(plan_counts(true, 2), (2 + 2 + 2, 1 + 1 + 1));
}

/// 行 2 的对照：每盘每轮一写、轮后一道屏障的四轮写序是 8 次写、4 道屏障；开头前面没有写的那一道不记。
#[test]
fn four_round_control_plan_has_eight_writes_and_four_barriers() {
    assert_eq!(plan_counts(false, 4), (4 + 4, 1 + 1 + 1 + 1));
    assert_eq!(
        count_writes_and_pool_barriers(&[PlanStep::PoolBarrier, PlanStep::PoolBarrier]),
        (0, 0)
    );
}

/// 每条臂的写序读数：除丙之外是 4 与 2，丙是 6 与 3。
#[test]
fn write_plan_of_every_arm_has_the_registered_write_and_barrier_counts() {
    for arm in ARMS {
        let expected = if arm == Arm::OldTableWhileIntentMarkerIsOnTheDisk {
            (6, 3)
        } else {
            (4, 2)
        };
        assert_eq!(
            count_writes_and_pool_barriers(&write_plan_of(arm)),
            expected,
            "{}",
            arm_name(arm)
        );
    }
}

/// 槽世代号规则（D22 已定项 16）：从两槽世代号都是 1 起连写，第一次世代号 2 落槽 0，第二次世代号 3 落槽 1，第三次世代号 4 落槽 0。
#[test]
fn slot_generation_rule_writes_slot_zero_then_slot_one_then_slot_zero_from_the_formatted_pair()
{
    let mut board = board_of(
        [slot_of([1, 1]), slot_of([1, 1])],
        [slot_of([1, 1]), slot_of([1, 1])],
    );
    let mut written = Vec::new();
    for _write in 0..3 {
        let (generation, slot_index) = next_write_target(&board, 0);
        board[0][slot_index] = Slot::SelfVerified {
            generation,
            table: [1, 1],
            carries_intent_marker: false,
        };
        written.push((generation, slot_index));
    }
    assert_eq!(written, vec![(2, 0), (3, 1), (4, 0)]);
    let torn_board = board_of(
        [Slot::Unreadable, Slot::Unreadable],
        [slot_of([1, 1]), slot_of([1, 1])],
    );
    assert_eq!(
        next_write_target(&torn_board, 0),
        (1, 1),
        "两槽都读不出时世代号从 1 起"
    );
}

/// 持久段与原始状态数（D13（验证路线） 已定项 4）：每段两次写、每次取三态，四写序 18 = 9 + 9，六写序 27 = 9 + 9 + 9。
#[test]
fn raw_crash_state_counts_are_eighteen_for_the_four_write_plan_and_twenty_seven_for_the_six_write_plan(
) {
    let segment_lengths = |steps: &[PlanStep]| {
        persistence_segments(steps)
            .iter()
            .map(|segment| segment.len())
            .collect::<Vec<usize>>()
    };
    assert_eq!(
        segment_lengths(&acquisition_write_plan(false, 2)),
        vec![2, 2]
    );
    assert_eq!(
        segment_lengths(&acquisition_write_plan(true, 2)),
        vec![2, 2, 2]
    );
    let start = start_board([1, 1], [200, 200]);
    let four_write_points = crash_points(
        &start,
        &acquisition_write_plan(false, 2),
        [200, 200],
        [300, 300],
    );
    let six_write_points = crash_points(
        &start,
        &acquisition_write_plan(true, 2),
        [200, 200],
        [300, 300],
    );
    assert_eq!(four_write_points.len(), 9 + 9 + 1, "原始状态加终点已用");
    assert_eq!(six_write_points.len(), 9 + 9 + 9 + 1);
    assert_eq!(
        four_write_points
            .iter()
            .filter(|point| point.is_end_of_use)
            .count(),
        1
    );
    assert_eq!(outcomes_of_one_slot_write().len(), 3);
}

/// 第三态「新旧都读不出」：第一持久段两次写都撕裂（态序号 2 × 3 + 2 = 8）时，盘 0 与盘 1 取号落的槽都读不出，另一槽仍装前表。
#[test]
fn torn_write_leaves_the_target_slot_unreadable_and_the_other_slot_with_the_front_table() {
    let start = start_board([1, 1], [200, 200]);
    let points = crash_points(
        &start,
        &acquisition_write_plan(false, 2),
        [200, 200],
        [300, 300],
    );
    let board = points[8].board;
    for disk_board in board.iter().take(DISK_COUNT) {
        assert_eq!(
            disk_board[0],
            Slot::Unreadable,
            "世代号 4 落槽 0，撕裂后读不出，旧内容也没了"
        );
        assert_eq!(
            disk_board[1],
            Slot::SelfVerified {
                generation: 3,
                table: [200, 200],
                carries_intent_marker: false
            }
        );
    }
    let nothing_persisted = points[0].board;
    assert_eq!(nothing_persisted, start, "序号 0 = 没有任何写持久");
}
/// 层级跨度（登记第七节）：S_0 = 812，S_1 = 137228，S_2 = 23191532，S_3 = 3919368908。
#[test]
fn level_spans_are_the_draft_values() {
    assert_eq!(
        [
            span_in_slots(0),
            span_in_slots(1),
            span_in_slots(2),
            span_in_slots(3)
        ],
        [812, 137228, 23191532, 3919368908]
    );
    assert_eq!(137228, 812 * 169);
}

/// 两块等大盘的根层级边界（每盘整槽数）：层级 1 到 68208（= 84 × 812）、层级 2 到 11527152、层级 3 到 1948088688；边界上根层级不变，加一槽就进下一层。
#[test]
fn equal_disk_boundaries_sit_exactly_where_the_next_root_level_starts() {
    assert_eq!(
        [
            equal_disks_boundary_slots(1),
            equal_disks_boundary_slots(2),
            equal_disks_boundary_slots(3)
        ],
        [68208, 11527152, 1948088688]
    );
    assert_eq!(68208, 84 * 812);
    for (level, boundary) in [(1, 68208u64), (2, 11527152), (3, 1948088688)] {
        assert_eq!(root_level_of(&[boundary, boundary]), level);
        assert_eq!(root_level_of(&[boundary + 1, boundary + 1]), level + 1);
    }
}

/// 与 `crates/` 钉死读数对拍的四个输入（`allocation_record_tree.rs` 的三条单元测试）：每盘 262144 槽 → 层级 2，每盘 67108864 槽 → 层级 3，
/// 每盘 68208 槽 → 层级 1，每盘 68209 槽 → 层级 2。
#[test]
fn root_level_arithmetic_agrees_with_the_four_readings_pinned_in_the_crates() {
    assert_eq!(root_level_of(&[262144, 262144]), 2);
    assert_eq!(root_level_of(&[67108864, 67108864]), 3);
    assert_eq!(root_level_of(&[68208, 68208]), 1);
    assert_eq!(root_level_of(&[68209, 68209]), 2);
    assert_eq!(CRATES_ROOT_LEVEL_READINGS.len(), 4);
}

/// 六个取样点（登记第四节第 6 条表）：（旧层级，新层级，甲总数，乙总数，乙的逐盘摊法）。
#[test]
fn root_level_points_have_the_draft_levels_totals_and_splits() {
    let expected: [(u32, u32, u64, u64, Table); 6] = [
        (1, 1, 136416, 136416, [68208, 68208]),
        (1, 2, 120000, 137228, [68208, 69020]),
        (1, 3, 120000, 137228, [68208, 69020]),
        (1, 2, 136416, 137228, [68208, 69020]),
        (1, 1, 136417, 136417, [68209, 68208]),
        (1, 2, 120000, 136417, [68208, 68209]),
    ];
    for (sampling_point, expected_point) in ROOT_LEVEL_POINTS.iter().zip(expected.iter()) {
        let point = root_level_point(sampling_point.1, sampling_point.2);
        assert_eq!(
            (
                point.old_level,
                point.new_level,
                point.chain_total,
                point.highest_split[0] + point.highest_split[1],
                point.highest_split
            ),
            *expected_point,
            "{}",
            sampling_point.0
        );
    }
}

/// 行 3 的对照点：旧 (68208, 68208) → 新 (262144, 262144)：甲 136416 = 2 × 68208，乙 137228 = 169 × 812，差 812；必不跨边界的点两做法差是 0。
#[test]
fn synthetic_crossing_point_gives_136416_and_137228_and_a_difference_of_812_while_a_non_crossing_point_gives_zero(
) {
    let crossing = root_level_point([68208, 68208], [262144, 262144]);
    let highest_total = crossing.highest_split[0] + crossing.highest_split[1];
    assert_eq!(crossing.chain_total, 2 * 68208);
    assert_eq!(highest_total, 169 * 812);
    assert_eq!(highest_total, crossing.chain_total + 812);
    let non_crossing = root_level_point([60000, 60000], [68208, 68208]);
    assert_eq!(
        non_crossing.highest_split[0] + non_crossing.highest_split[1],
        non_crossing.chain_total
    );
}

/// 乙的逐盘按盘号贪心变体（第七节）：旧 (60000, 60000) → 新 (262144, 262144)：盘 0 取到 77140、盘 1 不动，总数 137140，
/// 总数最大的读法是 137228，差 88；边界加一（两盘）贪心 128209、总数最大 136417。
#[test]
fn greedy_per_disk_variant_gives_137140_against_the_best_total_137228_and_128209_one_over_the_boundary(
) {
    assert_eq!(
        greedy_split([60000, 60000], [262144, 262144]),
        [77140, 60000]
    );
    assert_eq!(77140 + 60000, 137140);
    assert_eq!(95 * 812, 77140);
    assert_eq!(137228 - 137140, 88);
    assert_eq!(
        best_total_split([60000, 60000], [262144, 262144]),
        [68208, 69020]
    );
    assert_eq!(greedy_split([60000, 60000], [68209, 68209]), [68209, 60000]);
    assert_eq!(68209 + 60000, 128209);
    assert_eq!(
        best_total_split([60000, 60000], [68209, 68209]),
        [68208, 68209]
    );
    assert_eq!(
        best_total_split([60000, 60000], [262144, 60000]),
        [77140, 60000]
    );
}

/// 单元区整槽数 = 表项 − 单元区起点，两种口径下乙减甲的差相同。
#[test]
fn unit_area_slots_subtract_the_unit_area_start_from_each_entry() {
    assert_eq!(unit_area_slots(136416), 136416 - 2 * 50176);
    assert_eq!(
        unit_area_slots(137228) - unit_area_slots(120000),
        137228 - 120000
    );
}

/// 历史空间的元组数（登记第四节第 7 条）：四写序可写 1487（族 D 1221、族 S 266）、只读同数，合计 2974；
/// 六写序可写 2207（族 D 1815、族 S 392），合计 4414；原始状态数 18 / 27，去重后盘面数 17 / 25；全部在模型内。
#[test]
fn history_space_has_the_registered_tuple_and_state_counts_for_every_arm() {
    assert_eq!(1221 * 2 + 266 * 2, 2974);
    assert_eq!(1815 * 2 + 392 * 2, 4414);
    for arm in ARMS {
        let set = build_history_set(
            &write_plan_of(arm),
            NEWEST_SLOTS_OF_THE_MAIN_GRID,
            Numbering::Main,
        );
        let six_write = arm == Arm::OldTableWhileIntentMarkerIsOnTheDisk;
        let grown_family = grown_family_history_count(&set);
        let observed = (
            set.raw_state_count,
            set.distinct_board_count,
            set.crash_point_count,
            set.observations.len(),
            grown_family,
            set.observations.len() - grown_family,
            set.out_of_model_histories,
        );
        let expected = if six_write {
            (27, 25, 28, 2207, 1815, 392, 0)
        } else {
            (18, 17, 19, 1487, 1221, 266, 0)
        };
        assert_eq!(observed, expected, "{}", arm_name(arm));
    }
}

/// 世代号奇偶的四个组合都读到同样的原始状态数与去重后盘面数。
#[test]
fn raw_and_distinct_board_counts_do_not_depend_on_which_slot_holds_the_newest_generation() {
    let combinations = [NEWEST_SLOTS_OF_THE_MAIN_GRID, [0, 0], [0, 1], [1, 0]];
    for newest_slots in combinations {
        let four_write = build_history_set(
            &acquisition_write_plan(false, 2),
            newest_slots,
            Numbering::Main,
        );
        let six_write = build_history_set(
            &acquisition_write_plan(true, 2),
            newest_slots,
            Numbering::Main,
        );
        assert_eq!(
            (four_write.raw_state_count, four_write.distinct_board_count),
            (18, 17),
            "{newest_slots:?}"
        );
        assert_eq!(
            (six_write.raw_state_count, six_write.distinct_board_count),
            (27, 25),
            "{newest_slots:?}"
        );
    }
}
fn main_analyses_with(threads: usize) -> Vec<ArmAnalysis> {
    analyse_arms(NEWEST_SLOTS_OF_THE_MAIN_GRID, Numbering::Main, threads)
}

/// 阳性对照（对每一条臂的历史集都跑）：放行一切臂的漏放行（可写与只读）> 0；一律拒臂的字面误拒与可避免误拒 > 0。
#[test]
fn allow_everything_leaks_and_reject_every_writable_mount_misrefuses_on_the_history_set_of_every_arm(
) {
    for analysis in &main_analyses_with(4) {
        let allow_everything =
            control_readings_of(analysis, ControlReader::AllowEverything).counts;
        assert!(
            allow_everything[ReadingKind::LeakThroughWritableMount as usize] > 0,
            "{}",
            arm_name(analysis.arm)
        );
        assert!(
            allow_everything[ReadingKind::LeakThroughReadOnlyMount as usize] > 0,
            "{}",
            arm_name(analysis.arm)
        );
        let reject_every_writable_mount =
            control_readings_of(analysis, ControlReader::RejectEveryWritableMount).counts;
        assert!(
            reject_every_writable_mount[ReadingKind::LiteralMisrefusal as usize] > 0,
            "{}",
            arm_name(analysis.arm)
        );
        assert!(
            reject_every_writable_mount[ReadingKind::AvoidableMisrefusal as usize] > 0,
            "{}",
            arm_name(analysis.arm)
        );
    }
}

/// 阴性对照（对每一条臂的历史集都跑）：按类判官臂的漏放行（可写与只读）与可避免误拒都是 0。
#[test]
fn class_oracle_has_no_leak_and_no_avoidable_misrefusal_on_the_history_set_of_every_arm() {
    for analysis in &main_analyses_with(4) {
        let class_oracle =
            control_readings_of(analysis, ControlReader::NameWhatTheClassOverruns).counts;
        assert_eq!(
            class_oracle[ReadingKind::LeakThroughWritableMount as usize],
            0,
            "{}",
            arm_name(analysis.arm)
        );
        assert_eq!(
            class_oracle[ReadingKind::LeakThroughReadOnlyMount as usize],
            0,
            "{}",
            arm_name(analysis.arm)
        );
        assert_eq!(
            class_oracle[ReadingKind::AvoidableMisrefusal as usize],
            0,
            "{}",
            arm_name(analysis.arm)
        );
    }
}

/// 决定是观测的函数：每个读者（臂自己与三条对照读者）在每个观测类里的决定全相同，不同的类数是 0；类确实建出来了。
#[test]
fn every_reader_decides_the_same_within_each_observation_class_on_every_arm() {
    for analysis in &main_analyses_with(4) {
        assert!(analysis.arm_readings.classes > 0);
        assert_eq!(
            analysis.arm_readings.classes_with_differing_decisions,
            0,
            "{}",
            arm_name(analysis.arm)
        );
        assert_eq!(analysis.control_readings.len(), CONTROL_READERS.len());
        for (control, readings) in &analysis.control_readings {
            assert_eq!(
                readings.classes_with_differing_decisions,
                0,
                "{} {}",
                arm_name(analysis.arm),
                control_reader_name(*control)
            );
        }
    }
}

/// 观测类的算法生效：有类里既有越界成员、又有自己不越界的成员（单一表 T_后、探测 = 前尺寸：终点未用与终点已用同类），
/// 所以字面误拒读数减可避免误拒读数在至少一条臂上 ≥ 1。
#[test]
fn some_class_mixes_overrunning_and_non_overrunning_histories_so_literal_exceeds_avoidable_on_some_arm(
) {
    let analyses = main_analyses_with(4);
    let set = &analysis_of(&analyses, Arm::HighestTable).set;
    let mixed_class_exists = set.observations.iter().any(|observation| {
        beyond_the_end(observation.landing, observation.probe_sizes) == NOBODY_NAMED
            && set.class_union_of(observation) != NOBODY_NAMED
    });
    assert!(mixed_class_exists);
    assert!(analyses.iter().any(|analysis| {
        analysis.arm_readings.counts[ReadingKind::LiteralMisrefusal as usize]
            > analysis.arm_readings.counts[ReadingKind::AvoidableMisrefusal as usize]
    }));
}

/// 并行单位：五条臂各一个自己的读者单位，加三条对照读者在每一条臂上各一个，共 5 + 15 = 20 个，每条臂都有三条对照。
#[test]
fn every_arm_gets_its_own_reader_unit_and_all_three_control_readers() {
    let units = reader_units();
    assert_eq!(units.len(), 5 + 5 * 3);
    for (arm_index, arm) in ARMS.iter().enumerate() {
        let own = units
            .iter()
            .filter(|unit| unit.0 == arm_index && matches!(unit.1, Reader::Arm(_)))
            .count();
        let controls = units
            .iter()
            .filter(|unit| unit.0 == arm_index && matches!(unit.1, Reader::Control(_)))
            .count();
        assert_eq!((own, controls), (1, 3), "{}", arm_name(*arm));
    }
    for analysis in &main_analyses_with(4) {
        assert_eq!(
            analysis.control_readings.len(),
            3,
            "{}",
            arm_name(analysis.arm)
        );
    }
}

/// 读者规则不看槽号与世代号也不看数值，只比大小次序：世代号奇偶的另三个组合与把「再变大」的探测值一并换数的另一组整数，
/// 全部读数与主格逐项相同；五个变体每条臂各一行，只有「再变大」直接加 100 的那一支允许不同。
#[test]
fn readings_do_not_change_with_the_slot_parity_or_with_integers_replaced_on_every_number() {
    let main_analyses = main_analyses_with(4);
    let check = invariance_lines(&main_analyses, 4);
    let details_outside_the_fixed_growth_step_variant = check
        .lines
        .iter()
        .filter(|line| {
            line.starts_with("name=invariance_detail ")
                && !line.contains(" variant=alternative_integers_with_fixed_growth_step ")
        })
        .cloned()
        .collect::<Vec<String>>()
        .join("\n");
    assert_eq!(
        check.mismatches_in_order_preserving_check(),
        0,
        "{details_outside_the_fixed_growth_step_variant}"
    );
    let summary_lines: Vec<&String> = check
        .lines
        .iter()
        .filter(|line| line.starts_with("name=invariance arm="))
        .collect();
    assert_eq!(summary_lines.len(), 5 * ARMS.len());
    for line in summary_lines {
        if !line.contains("variant=alternative_integers_with_fixed_growth_step") {
            assert!(line.ends_with("equal_to_main=true"), "{line}");
        }
    }
    for analysis in &main_analyses {
        assert!(analyses_agree(analysis, analysis));
        assert!(differing_fields(analysis, analysis).is_empty());
    }
}

/// 「再变大」在换过的后尺寸上直接加 100，拆开了主格里 200 加 100 等于 300 这类跨形状的相等（四条写序臂与丙在终点已用都是全部槽装后表，
/// 与另一个形状变大前的盘面加相同探测值落进同一个观测类）：每条臂的观测类变多、与主格不同的读数逐字段列出（含类数）；
/// 把探测值一并换数的写法不拆，类数不变。
#[test]
fn fixed_growth_step_variant_splits_classes_that_the_main_numbers_merge() {
    let main_analyses = main_analyses_with(4);
    let fixed = analyse_arms(
        NEWEST_SLOTS_OF_THE_MAIN_GRID,
        Numbering::OrderPreservingWithFixedGrowthStep,
        4,
    );
    let preserved = analyse_arms(
        NEWEST_SLOTS_OF_THE_MAIN_GRID,
        Numbering::OrderPreservingOnEveryNumber,
        4,
    );
    for main in &main_analyses {
        let fixed_arm = analysis_of(&fixed, main.arm);
        assert!(
            fixed_arm.arm_readings.classes > main.arm_readings.classes,
            "{}",
            arm_name(main.arm)
        );
        assert!(!analyses_agree(main, fixed_arm), "{}", arm_name(main.arm));
        assert!(differing_fields(main, fixed_arm)
            .iter()
            .any(|difference| difference.reader == "arm" && difference.field == "classes"));
        let preserved_arm = analysis_of(&preserved, main.arm);
        assert_eq!(
            preserved_arm.arm_readings.classes,
            main.arm_readings.classes,
            "{}",
            arm_name(main.arm)
        );
    }
    let check = invariance_lines(&main_analyses, 4);
    assert_eq!(check.mismatches_in_registered_clause(), ARMS.len());
    let detail_lines: Vec<&String> = check
        .lines
        .iter()
        .filter(|line| line.starts_with("name=invariance_detail "))
        .collect();
    assert!(detail_lines
        .iter()
        .any(|line| line.contains(" compared_reader=arm field=classes ")));
    assert!(detail_lines
        .iter()
        .all(|line| line.contains(" variant=alternative_integers_with_fixed_growth_step ")));
}

/// 另一组整数：每块盘一张严格递增的换数表，主格的每个整数在所有形状里换成同一个数；五个形状换数之后是手算的值
/// （含登记写的早 (10, 90)、前 (20, 95)、后 (30, 99)）；探测值「再变大」一支把主格里的后尺寸加 100 也换数，另一支在换过的数上加 100。
#[test]
fn alternative_numbers_are_one_increasing_table_per_disk_and_give_the_hand_computed_shapes() {
    for (disk, table) in ALTERNATIVE_NUMBER_OF_MAIN_NUMBER.iter().enumerate().take(DISK_COUNT) {
        for pair in table.windows(2) {
            assert!(pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1, "{disk}");
        }
    }
    let hand_computed: [(Table, Table, Table); 5] = [
        ([10, 90], [20, 95], [30, 99]),
        ([10, 90], [20, 95], [30, 95]),
        ([10, 90], [20, 90], [20, 99]),
        ([10, 90], [10, 90], [20, 95]),
        ([10, 90], [10, 90], [20, 90]),
    ];
    for numbering in [
        Numbering::OrderPreservingOnEveryNumber,
        Numbering::OrderPreservingWithFixedGrowthStep,
    ] {
        let shapes = shapes_in(numbering);
        assert_eq!(shapes.len(), hand_computed.len());
        for (shape, expected) in shapes.iter().zip(hand_computed) {
            assert_eq!((shape.early, shape.front, shape.back), expected);
        }
    }
    let main_shapes = shapes_in(Numbering::Main);
    assert_eq!(
        (
            main_shapes[2].early,
            main_shapes[2].front,
            main_shapes[2].back
        ),
        ([100, 100], [200, 100], [200, 300])
    );
    assert_eq!(grown_size(Numbering::Main, 0, 300), 400);
    assert_eq!(
        grown_size(Numbering::OrderPreservingWithFixedGrowthStep, 0, 30),
        130
    );
    assert_eq!(
        grown_size(Numbering::OrderPreservingOnEveryNumber, 0, 30),
        40
    );
    assert_eq!(
        grown_size(Numbering::OrderPreservingOnEveryNumber, 1, 90),
        95
    );
    assert_eq!(
        grown_size(Numbering::OrderPreservingOnEveryNumber, 1, 95),
        99
    );
    assert_eq!(
        grown_size(Numbering::OrderPreservingOnEveryNumber, 1, 99),
        120
    );
}

/// 每条臂每个读数的轨迹加起来等于该读数的计数；终点已用那一点与最后持久段全持久那一点的下标在写序里是预期的位置。
#[test]
fn trajectory_sums_to_the_counts_and_marks_the_end_of_use_and_the_last_fully_persisted_point() {
    for analysis in &main_analyses_with(4) {
        for reading in ALL_READINGS {
            let summed: usize = analysis
                .arm_readings
                .trajectory
                .iter()
                .map(|counts| counts[reading as usize])
                .sum();
            assert_eq!(
                summed,
                analysis.arm_readings.counts[reading as usize],
                "{}",
                arm_name(analysis.arm)
            );
        }
        let six_write = analysis.arm == Arm::OldTableWhileIntentMarkerIsOnTheDisk;
        let (crash_points, last_fully_persisted) = if six_write { (28, 22) } else { (19, 13) };
        assert_eq!(
            (
                analysis.set.crash_point_count,
                analysis.set.last_segment_fully_persisted_index
            ),
            (crash_points, last_fully_persisted)
        );
    }
}

/// 同一份装置同一条命令跑两次、线程数 1 与线程数多时，除配置行里的线程数之外逐字相同（确定性；多线程与单线程对拍）。
#[test]
fn output_lines_are_identical_across_runs_and_across_thread_counts_except_the_configured_threads(
) {
    let reader_rules_single = reader_rules_on_histories_lines(1);
    assert_eq!(
        reader_rules_single[1..],
        reader_rules_on_histories_lines(1)[1..]
    );
    assert_eq!(
        reader_rules_single[1..],
        reader_rules_on_histories_lines(8)[1..]
    );
    let write_counts_single = write_counts_and_root_level_lines(1);
    assert_eq!(
        write_counts_single[1..],
        write_counts_and_root_level_lines(8)[1..]
    );
}

/// 行 2 与行 3 的输出：每条臂一行写数（丙 6 与 3，其余 4 与 2）、四轮写序对照读数 8 与 4，四个与 `crates/` 对拍的输入，六个取样点，判决行全过。
#[test]
fn write_counts_and_root_level_output_has_the_registered_lines() {
    let lines = write_counts_and_root_level_lines(4);
    let count_of = |needle: &str| lines.iter().filter(|line| line.contains(needle)).count();
    assert_eq!(count_of("name=write_count arm="), 5);
    assert_eq!(
        count_of("arm=old_table_while_intent_marker writes=6 pool_barriers=3 "),
        1
    );
    assert_eq!(count_of(" writes=4 pool_barriers=2 "), 4);
    assert_eq!(count_of("name=write_count_control plan=four_rounds writes=8 pool_barriers=4 expected_writes=8 expected_pool_barriers=4 matches=true"), 1);
    assert_eq!(count_of("name=crates_cross_check "), 4);
    assert_eq!(count_of("name=root_level_point "), 6);
    assert_eq!(count_of("matches=false"), 0);
    assert_eq!(count_of("name=not_run_after_sufficient item="), 3);
    for verdict in [
        "name=verdict check=write_counts reference_writes=4 reference_pool_barriers=2 four_round_control_matches=true arms_that_write_more_than_the_reference=old_table_while_intent_marker",
        "name=verdict check=root_level_cross_check crates_reading_mismatches=zero",
        "name=verdict check=independent_arithmetic draft_mismatches=zero",
        "name=verdict check=root_level_points synthetic_control_matches_hand_values=true non_crossing_points_have_zero_difference=true",
    ] {
        assert_eq!(count_of(verdict), 1, "{verdict}");
    }
}

/// 行 1 的输出：每条臂一行历史空间、五个读数各一行、三条对照各一行（五条臂各自的三条）、判决行里的对照与保序不变性与计数都过；
/// 登记原文的不变性判决行在五条臂上各不符一处（「再变大」直接加 100 那一支，明细行逐字段列出）。
#[test]
fn reader_rules_output_has_every_arm_and_every_control_and_the_passing_verdicts() {
    let lines = reader_rules_on_histories_lines(4);
    let count_of = |needle: &str| lines.iter().filter(|line| line.contains(needle)).count();
    assert_eq!(count_of("name=history_space arm="), 5);
    assert_eq!(count_of("name=reading arm="), 5 * 5);
    assert_eq!(count_of("name=control arm="), 5 * 3);
    for control in CONTROL_READERS {
        assert_eq!(
            count_of(&format!(" reader={} ", control_reader_name(control))),
            5 + 5,
            "对照读者的 control 行与 class_consistency 行各五条"
        );
    }
    assert_eq!(count_of("name=trajectory arm="), 5 * 5);
    assert_eq!(count_of("name=hand_derivation claim="), 7);
    assert_eq!(count_of("name=hand_derivation_witness claim="), 2);
    assert_eq!(count_of("name=not_run_after_sufficient item="), 2);
    assert_eq!(count_of("name=not_run_this_segment item="), 2);
    for verdict in [
        "name=verdict check=positive_controls allow_everything_leaks_on_every_arm=true reject_every_writable_mount_misrefuses_on_every_arm=true",
        "name=verdict check=negative_controls negative_control_violations=zero",
        "name=verdict check=decision_is_function_of_observation differing_class_violations=zero",
        "name=verdict check=literal_minus_avoidable at_least_one_arm_literal_exceeds_avoidable=true",
        "name=verdict check=invariance_order_preserving order_preserving_mismatches=zero",
        "name=verdict check=out_of_model out_of_model_violations=zero",
        "name=verdict check=history_counts history_count_mismatches=zero",
    ] {
        assert_eq!(count_of(verdict), 1, "{verdict}");
    }
    assert_eq!(
        count_of(&format!(
            "name=verdict check=invariance invariance_mismatches={}",
            ARMS.len()
        )),
        1
    );
    assert_eq!(count_of("name=invariance arm="), 5 * ARMS.len());
    assert!(count_of("name=invariance_detail ") > 0);
}
