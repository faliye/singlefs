use super::*;

fn the_copied_elements(unit_area_slots: u64) -> u64 {
    2 * elements_per_disk(unit_area_slots)
}

fn no_elements(_unit_area_slots: u64) -> u64 {
    0
}

fn constant_seven_elements(_unit_area_slots: u64) -> u64 {
    7
}

#[test]
fn the_counts_positive_control_reads_the_steps_and_not_the_scale() {
    // 第一段真产物的形态：步数 0、scale 是元素数。阳性对照读步数，所以过不了（读成 scale 的错法在这一半红）
    let mut elements_only_in_the_scale = String::new();
    controls_report(
        &read_product(&whole_copy_rows_text(no_elements, the_copied_elements)),
        None,
        &mut elements_only_in_the_scale,
    );
    for state in STATES {
        assert!(
            elements_only_in_the_scale.contains(&format!("name=verdict control=b1_counts_whole_copy_alpha_at_least_0_95 state={state} passed=false alpha=0.000000")),
            "{elements_only_in_the_scale}"
        );
    }
    // 反过来：步数是元素数、scale 是一个不随 U 变的数。阳性对照照样过（读成 scale 的错法在这一半也红）
    let mut steps_only = String::new();
    controls_report(
        &read_product(&whole_copy_rows_text(
            the_copied_elements,
            constant_seven_elements,
        )),
        None,
        &mut steps_only,
    );
    for state in STATES {
        assert!(
            steps_only.contains(&format!("name=verdict control=b1_counts_whole_copy_alpha_at_least_0_95 state={state} passed=true alpha=1.000000")),
            "{steps_only}"
        );
    }
}

#[test]
fn the_copy_elements_check_reads_the_scale_and_not_the_steps() {
    // 第七节 ③1 拿站点另读的 scale 比 U 那一部分：scale 是元素数时 0 处不符，步数是多少都不管
    let mut elements_only_in_the_scale = String::new();
    counts_report(
        &read_product(&whole_copy_rows_text(no_elements, the_copied_elements)),
        &mut elements_only_in_the_scale,
    );
    assert!(
        elements_only_in_the_scale
            .contains("name=verdict quantity=three_one_copy_elements disagreements=0"),
        "{elements_only_in_the_scale}"
    );
    // scale 是 7（比 U 那一部分小）而步数是元素数：三个点 × 两种状态 6 格都不符（读成步数的错法在这里红）
    let mut scale_too_small = String::new();
    counts_report(
        &read_product(&whole_copy_rows_text(
            the_copied_elements,
            constant_seven_elements,
        )),
        &mut scale_too_small,
    );
    assert!(
        scale_too_small
            .contains("name=verdict quantity=three_one_copy_elements disagreements=6"),
        "{scale_too_small}"
    );
}

/// 对账用的一格（base、同一次挂载、覆盖写第 1 轮）：两块盘、固定点 2 轮，入口、出口读数一条都没有，行读数只有 `rows` 给的
/// （`rows` 里的每行带不带走到命中为止那两样，照第三段给的原样写）。
fn reconciliation_text(rows: &[(&str, &str)]) -> String {
    let at = "point=base state=same_mount kind=overwrite round=1";
    let mut text = String::from("E203RESULT name=point point=base unit_area_slots=131072 unlinks=10 data_units=10 files=10\n");
    push_line(&mut text, &format!("name=state {at} devices=2"));
    push_line(
        &mut text,
        &format!("name=publish {at} had_caller_segment=true fixed_point_rounds=2"),
    );
    for (row, readings) in rows {
        push_line(
            &mut text,
            &format!("name=row {at} segment=between row={row} {readings}"),
        );
    }
    text
}

#[test]
fn the_reconciliation_covers_every_scan_row_and_counts_the_bounded_rows_apart() {
    assert_eq!(scan_rows().len(), 29, "27 行扫类加两行整份拷");
    let totals = |text: &str| {
        let mut out = String::new();
        (reconciliation_report(&read_product(text), &mut out), out)
    };
    // 入口、出口读数一条都没有：吃它们的 15 行读不到；记账行那一行只吃盘数（0 次调用、0 步）核过；调用处那十行没有行读数，0 步对 0 表长核过
    let (empty, out) = totals(&reconciliation_text(&[]));
    assert_eq!(
        empty,
        ReconciliationTotals {
            cells: 12,
            disagreements: 0,
            over_the_bound: 0,
            unread: 15,
        },
        "{out}"
    );
    assert!(
        out.contains("name=verdict quantity=three_two cells_checked=12 disagreements=0 cells_over_the_bound=0 cells_with_the_scale_unread=15 rows_reconciled=27 rows_equal_only=18 rows_with_a_bounded_part=8 rows_steps_only=1 rows_not_reconciled=0"),
        "{out}"
    );
    assert!(
        out.contains("name=three_two_rows_not_reconciled count=0 rows=\n"),
        "{out}"
    );
    // 插计数脚本修订 4 之前的行读数（没带走到命中为止那两样）：那一格读不到，不拿 0 补
    let (old_product, old_product_report) = totals(&reconciliation_text(&[(
        "root_header_linear_find",
        "calls=3 steps=12",
    )]));
    assert_eq!((old_product.cells, old_product.unread), (11, 16), "{old_product_report}");
    // 走到命中为止那一部分超了上界：算不符，另记超上界；不符那一行带两边四个数
    let (over, over_report) = totals(&reconciliation_text(&[(
        "record_intersection_filter",
        "calls=3 steps=10 scale=6 steps_to_the_hit=4 scale_to_the_hit=3",
    )]));
    assert_eq!((over.disagreements, over.over_the_bound), (1, 1), "{over_report}");
    assert!(
        over_report.contains("name=three_two_disagreement point=base state=same_mount kind=overwrite round=1 row=record_intersection_filter relation=WholeTableAndUpToTheHitAtTheCallSite steps=10 steps_to_the_hit=4 whole_table_lengths=6 lengths_to_the_hit=3 over_the_bound=true calls=3"),
        "{over_report}"
    );
    assert!(
        over_report.contains("name=three_two_row row=record_intersection_filter relation=WholeTableAndUpToTheHitAtTheCallSite cells_checked=1 disagreements=1 over_the_bound=1"),
        "{over_report}"
    );
}

/// 修订 3 ② 那 13 行对账用的一格（base、同一次挂载、第 1 轮）：两块盘，固定点 `rounds` 轮、每轮分配记录树节点数取 `nodes_each_round`，
/// 入口读数记在落点判之前那一段、出口读数记在准入之后那一段，这一行的读数都记在两道之间那一段。
fn entry_and_exit_text(
    kind: &str,
    rounds: u64,
    nodes_each_round: &[u64],
    readings: &[(&str, u64)],
    row: &str,
    calls: u64,
    steps: u64,
) -> String {
    let at = format!("point=base state=same_mount kind={kind} round=1");
    let mut text = String::from("E203RESULT name=point point=base unit_area_slots=131072 unlinks=10 data_units=10 files=10\n");
    push_line(&mut text, &format!("name=state {at} devices=2"));
    push_line(
        &mut text,
        &format!("name=publish {at} had_caller_segment=true fixed_point_rounds={rounds}"),
    );
    for (key, value) in readings {
        let segment = if key.starts_with("entry_") {
            "before_landing"
        } else {
            "after"
        };
        push_line(
            &mut text,
            &format!(
                "name=delta {at} segment={segment} fixed_point_round=0 key={key} value={value}"
            ),
        );
    }
    for (index, nodes) in nodes_each_round.iter().enumerate() {
        push_line(
            &mut text,
            &format!(
                "name=delta {at} segment=between fixed_point_round={} key=allocation_plan_nodes_after value={nodes}",
                index + 1
            ),
        );
    }
    push_line(
        &mut text,
        &format!("name=row {at} segment=between row={row} calls={calls} steps={steps}"),
    );
    text
}

/// 对账之后这一行那一条 `name=three_two_row` 的「核了几格、不符几格」。
fn cells_and_disagreements_of(text: &str, row: &str) -> (u64, u64) {
    let mut out = String::new();
    reconciliation_report(&read_product(text), &mut out);
    let line = out
        .lines()
        .find(|line| line.starts_with(&format!("name=three_two_row row={row} ")))
        .unwrap_or_else(|| panic!("没有 {row} 那一行：{out}"));
    let fields = parse_line(&format!("E203RESULT {line}")).expect("这一行是 name= 起头的");
    (
        number(&fields, "cells_checked").expect("核了几格"),
        number(&fields, "disagreements").expect("不符几格"),
    )
}

/// 一行在同一份读数上：步数等于式子值时核过、多一步时不符（登记第七节 ③2 逐格相等，不按上界放过）。
fn assert_the_row_reconciles_exactly(
    kind: &str,
    rounds: u64,
    nodes_each_round: &[u64],
    readings: &[(&str, u64)],
    row: &str,
    calls: u64,
    expected_steps: u64,
) {
    let cell = |steps: u64| {
        cells_and_disagreements_of(
            &entry_and_exit_text(kind, rounds, nodes_each_round, readings, row, calls, steps),
            row,
        )
    };
    assert_eq!(
        cell(expected_steps),
        (1, 0),
        "{row}：{expected_steps} 步该与式子相等"
    );
    assert_eq!(cell(expected_steps + 1), (1, 1), "{row}：多一步该记不符");
}

#[test]
fn the_inode_record_find_walks_to_the_position_of_the_inode_whose_size_changes() {
    // 3 次调用，每次走到排第 7 的那条记录：3 × 7
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &[("entry_position_of_the_inode_whose_size_changes", 7)],
        "inode_record_linear_find",
        3,
        21,
    );
}

#[test]
fn the_extent_roles_scan_looks_at_every_node_of_the_planned_extent_tree() {
    // 3 次调用，发出来的那一版 extent 树下段、上段与别的文件的下段节点共 5 个：3 × 5
    assert_the_row_reconciles_exactly(
        "unlink_final",
        2,
        &[],
        &[("exit_extent_tree_nodes", 5)],
        "extent_roles_rewritten_scan",
        3,
        15,
    );
}

#[test]
fn the_checkpoint_cost_accounting_plan_counts_two_calls_per_judgement() {
    let readings = [
        ("entry_accounting_rows", 28),
        ("exit_accounting_tree_keys", 29),
        ("entry_accounting_tree_keys", 27),
        ("exit_accounting_tree_nodes", 3),
        ("entry_accounting_tree_nodes", 2),
    ];
    // 两次 ckpt（4 次调用）：每次 28 行 + 29 把 key + 规划 (27 + 29 + 2 × 3 + 2) = 121，共 2 × 121
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &readings,
        "checkpoint_cost_accounting_tree_plan",
        4,
        242,
    );
    let odd = entry_and_exit_text(
        "overwrite",
        2,
        &[],
        &readings,
        "checkpoint_cost_accounting_tree_plan",
        5,
        242,
    );
    assert_eq!(
        cells_and_disagreements_of(&odd, "checkpoint_cost_accounting_tree_plan"),
        (0, 0),
        "调用次数是奇数：有一次规划没走到，式子不成立，记读不到，不按一半算"
    );
}

#[test]
fn the_allocation_record_tree_node_set_takes_two_steps_per_node_of_the_previous_tree() {
    // 1 次调用，上一版分配记录树 7 个节点：1 × 2 × 7
    assert_the_row_reconciles_exactly(
        "unlink_intent",
        2,
        &[],
        &[("entry_allocation_record_tree_nodes", 7)],
        "allocation_record_tree_node_set",
        1,
        14,
    );
    let twice = entry_and_exit_text(
        "unlink_intent",
        2,
        &[],
        &[
            ("entry_allocation_record_tree_nodes", 7),
            ("entry_allocation_record_tree_nodes", 7),
        ],
        "allocation_record_tree_node_set",
        1,
        14,
    );
    assert_eq!(
        cells_and_disagreements_of(&twice, "allocation_record_tree_node_set"),
        (0, 0),
        "一次发布读到两条入口读数：不知道是哪一次的，记读不到"
    );
}

#[test]
fn the_watermark_steps_walk_the_rows_twice_and_compare_the_sorted_steps_once_each() {
    let readings = |steps_rows: u64| {
        [
            ("entry_accounting_rows", 28),
            ("entry_watermark_rows_before_its_txg", steps_rows),
        ]
    };
    // 3 次调用：2 × 28 行 + 13 条台阶行排好序的输入比 12 次，3 × 68
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &readings(13),
        "inode_number_watermark_steps",
        3,
        204,
    );
    // 一条台阶行都没有：排序不比，3 × 56（不按 0 − 1 下溢）
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &readings(0),
        "inode_number_watermark_steps",
        3,
        168,
    );
}

#[test]
fn the_deadlist_changes_build_the_file_index_only_on_the_extra_call() {
    let readings = [
        ("entry_deadlist_entries", 1),
        ("entry_kept_mapping_entries", 2),
        ("entry_mapped_units", 37),
        ("entry_files_other_than_the_first", 12),
    ];
    // 2 轮、6 次调用（每轮多一次：按号的文件索引）：2 × (1 + 2 + 37 + 12)
    assert_the_row_reconciles_exactly(
        "unlink_final",
        2,
        &[],
        &readings,
        "deadlist_changes_indexes",
        6,
        104,
    );
    // 2 轮、4 次调用（不建文件索引）：2 × (1 + 2 + 37)
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &readings,
        "deadlist_changes_indexes",
        4,
        80,
    );
}

/// s20 那一格的读数：三棵码 2 树入口、出口各两样，deadlist 树出口 key 数由调用方给。
fn multi_level_readings(deadlist_keys_after: u64) -> Vec<(&'static str, u64)> {
    vec![
        ("exit_allocation_record_tree_nodes", 17),
        ("exit_accounting_tree_keys", 28),
        ("entry_accounting_tree_keys", 28),
        ("exit_accounting_tree_nodes", 1),
        ("entry_accounting_tree_nodes", 1),
        ("exit_deadlist_tree_keys", deadlist_keys_after),
        ("entry_deadlist_tree_keys", 3),
        ("exit_deadlist_tree_nodes", 1),
        ("entry_deadlist_tree_nodes", 2),
        ("exit_central_mapping_tree_keys", 35),
        ("entry_central_mapping_tree_keys", 34),
        ("exit_central_mapping_tree_nodes", 1),
        ("entry_central_mapping_tree_nodes", 1),
    ]
}

#[test]
fn the_multi_level_planning_sums_the_three_code_two_trees_in_every_round() {
    // 每轮：记账 28 + (28 + 28 + 2 × 1 + 1) = 87；deadlist 删到零根只走上一版 2 个节点 = 0 + 2；
    // 映射 35 + (34 + 35 + 2 × 1 + 1) = 107；2 轮 × 196
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[17, 17],
        &multi_level_readings(0),
        "multi_level_tree_planning",
        6,
        392,
    );
    // deadlist 这一版还有 4 把 key：4 + (3 + 4 + 2 × 1 + 2) = 15；2 轮 × (87 + 15 + 107)
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[17, 17],
        &multi_level_readings(4),
        "multi_level_tree_planning",
        6,
        418,
    );
    let moved = entry_and_exit_text(
        "overwrite",
        2,
        &[16, 17],
        &multi_level_readings(0),
        "multi_level_tree_planning",
        6,
        392,
    );
    assert_eq!(
        cells_and_disagreements_of(&moved, "multi_level_tree_planning"),
        (0, 0),
        "第一轮的分配记录树少一个节点：那一轮的映射单元不是出口那一份，记读不到"
    );
}

#[test]
fn the_container_trees_put_out_every_container_and_node_in_every_round() {
    // 3 轮，三棵容器树发出来共 14 片容器与节点：3 × 14
    assert_the_row_reconciles_exactly(
        "unlink_intent",
        3,
        &[],
        &[("exit_container_tree_units", 14)],
        "mapped_units_of_container_trees",
        9,
        42,
    );
}

#[test]
fn the_moved_entries_tail_filter_looks_at_every_node_of_the_previous_mapping_tree() {
    // 2 次调用，上一版中央映射树 4 个节点：2 × 4（不是 key 数 9）
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &[
            ("entry_central_mapping_tree_nodes", 4),
            ("entry_central_mapping_tree_keys", 9),
        ],
        "moved_entries_path_tail_filter",
        2,
        8,
    );
}

#[test]
fn the_release_judgement_builds_its_indexes_by_the_number_of_extra_calls() {
    let readings = [
        ("entry_deadlist_entries", 1),
        ("exit_deadlist_entries", 3),
        ("entry_mapped_units", 37),
        ("entry_units", 39),
        ("entry_files_other_than_the_first", 12),
    ];
    // 2 轮、10 次调用（每轮留存索引一次加三份索引）：2 × (2 + 37 + 39 + 12)
    assert_the_row_reconciles_exactly(
        "unlink_final",
        2,
        &[],
        &readings,
        "release_judgement_version_indexes",
        10,
        180,
    );
    // 2 轮、8 次调用（没查到别的文件的单元，不建文件索引）：2 × (2 + 37 + 39)
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &readings,
        "release_judgement_version_indexes",
        8,
        156,
    );
}

#[test]
fn the_accounting_entries_of_a_publish_are_three_pool_rows_and_six_per_device() {
    // 2 次调用，两块盘：2 × (3 + 6 × 2)
    assert_the_row_reconciles_exactly(
        "unlink_final",
        2,
        &[],
        &[],
        "accounting_entries_of_this_publish",
        2,
        30,
    );
}

#[test]
fn the_instance_table_chain_walks_every_page_of_the_replaced_chain() {
    // 2 次调用，被换下的旧链 3 片：2 × 3
    assert_the_row_reconciles_exactly(
        "overwrite",
        2,
        &[],
        &[("entry_instance_table_chain_replaced", 3)],
        "instance_table_chain",
        2,
        6,
    );
}

#[test]
fn the_mapped_units_table_indexes_the_previous_version_and_puts_out_this_one_in_every_round() {
    let readings = [
        ("exit_allocation_record_tree_nodes", 7),
        ("entry_mapped_units", 35),
        ("exit_mapped_units", 37),
    ];
    // 2 轮：上一版 35 条建索引 + 这一版 37 条，2 × 72
    assert_the_row_reconciles_exactly(
        "unlink_intent",
        2,
        &[7, 7],
        &readings,
        "mapped_units_whole_table",
        4,
        144,
    );
}

#[test]
fn the_files_plan_walks_every_other_file_of_the_previous_version_in_the_last_batch_too() {
    // s5，修订 4 换吃入口读数：2 次调用，上一版别的文件 1000 个：2 × 1000（最后一批也对）
    assert_the_row_reconciles_exactly(
        "unlink_final",
        2,
        &[],
        &[("entry_files_other_than_the_first", 1000)],
        "files_other_than_the_first_plan",
        2,
        2000,
    );
}

#[test]
fn the_inodes_with_an_entry_take_the_first_file_and_every_other_file_in_the_last_batch_too() {
    // s7，修订 4 换吃入口读数：2 次调用，每次第一个文件一步加别的文件 1000 步：2 × 1001
    assert_the_row_reconciles_exactly(
        "unlink_final",
        2,
        &[],
        &[("entry_files_other_than_the_first", 1000)],
        "inodes_with_an_extent_entry_set",
        2,
        2002,
    );
}

#[test]
fn the_cluster_segment_sum_walks_every_cluster_segment_once_per_device() {
    // s14，修订 4 由上界收严成相等：两块盘各调一次，入口聚簇段 7 段：2 × 7（最后一批也对）
    assert_the_row_reconciles_exactly(
        "unlink_final",
        2,
        &[],
        &[("entry_cluster_segments", 7)],
        "cluster_segment_sum",
        2,
        14,
    );
}

#[test]
fn the_file_find_walks_at_most_every_other_file_at_each_call() {
    // s6：2 次调用，每次调用那一刻那一版别的文件 1000 个：上界 2000，全走到命中为止
    assert_the_call_site_row_reconciles(
        "unlink_final",
        "file_other_than_the_first_linear_find",
        0,
        2000,
    );
}

#[test]
fn the_extent_upper_plan_walks_every_path_and_previous_node_and_at_most_the_previous_nodes_per_contains(
) {
    // s8：3 个有条目的号 × (1 + 最高层 2) = 9 步路径，换下那一遍 filter 走上一版 4 个节点：全表扫 13；
    // 3 次 contains 没被短路，每次那一刻上一版 4 个节点：上界 12
    assert_the_call_site_row_reconciles("unlink_intent", "extent_upper_segment_plan", 13, 12);
}

#[test]
fn the_root_header_find_walks_at_most_every_unit_at_each_call() {
    // s12：8 次调用，每次那一版 30 个单元：上界 240
    assert_the_call_site_row_reconciles("overwrite", "root_header_linear_find", 0, 240);
}

#[test]
fn the_allocation_record_tree_plan_walks_both_node_sets_in_every_round() {
    // s16：两轮，规划各走这一轮的 7 个节点与上一版 7 个（28），intersection 首轮 3 + 7、次轮 5 + 7（22）：全表扫 50
    assert_the_call_site_row_reconciles("unlink_final", "allocation_record_tree_plan", 50, 0);
}

#[test]
fn the_release_find_walks_at_most_the_records_of_that_moment() {
    // s25：两次找，那一刻记录 10 条、12 条：上界 22
    assert_the_call_site_row_reconciles(
        "unlink_intent",
        "allocator_release_linear_find",
        0,
        22,
    );
}

#[test]
fn the_user_data_slot_scan_walks_at_most_the_set_segments_and_one_segments_pairs() {
    // s26：两次调用，每次调用那一刻集合里 10 个段、答案那一段里至多 32 对：上界 2 × (10 + 32) = 84
    assert_the_call_site_row_reconciles("overwrite", "user_data_slot_linear_scan", 0, 84);
}

#[test]
fn the_record_intersection_filters_and_retains_every_record_and_finds_within_them() {
    // s27：filter 那一刻 40 条、retain 那一刻 39 条：全表扫 79；find 那一刻 39 条：上界 39
    assert_the_call_site_row_reconciles("unlink_final", "record_intersection_filter", 79, 39);
}

#[test]
fn the_empty_segment_scan_walks_at_most_every_whole_segment() {
    // s28：4 次调用，每次那一刻 2048 个整段：上界 8192
    assert_the_call_site_row_reconciles("unlink_intent", "empty_segment_linear_scan", 0, 8192);
}

#[test]
fn the_fallback_slot_scan_walks_at_most_every_candidate_and_slot_of_its_span() {
    // s29：一次调用，10 个候选 × (1 + 跨度 2 槽)：上界 30
    assert_the_call_site_row_reconciles("overwrite", "fallback_slot_linear_scan", 0, 30);
}

#[test]
fn the_pooled_fallback_row_reconciles_only_on_calls_with_steps() {
    // s37：每次调用至少走一片叶：调过且有步、没调过且没有步核过；调过没有步、没调过却有步都不符
    let cell = |readings: &str| {
        cells_and_disagreements_of(
            &reconciliation_text(&[("pooled_fallback_by_the_leaf", readings)]),
            "pooled_fallback_by_the_leaf",
        )
    };
    assert_eq!(cell("calls=2 steps=9"), (1, 0));
    assert_eq!(cell("calls=0 steps=0"), (1, 0));
    assert_eq!(cell("calls=2 steps=0"), (1, 1));
    assert_eq!(cell("calls=0 steps=5"), (1, 1));
}
