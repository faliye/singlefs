use super::*;

/// 登记第七节 ③3 的 Δ0 写成产物行：从产物里读 B8 的输入，再算出来的合计必须还是 1336（读数接线与式子一起钉住）。
fn delta_zero_product_text() -> String {
    let at = "point=base state=same_mount kind=overwrite round=1";
    let mut text = String::new();
    let mut line = |rest: &str| {
        let _ = writeln!(text, "E203RESULT {rest}");
    };
    line("name=point point=base unit_area_slots=16777216 unlinks=10 data_units=10 files=10");
    line(&format!("name=state {at} units=3000 mapped_units=1500 allocation_records=2000 other_files=1000 inode_leaf_containers=8 deadlist_entries=0 extent_upper_nodes=9 extent_upper_levels=2 accounting_tree_levels=2 deadlist_tree_levels=1 allocation_tree_levels=3 central_mapping_levels=2"));
    line(&format!(
        "name=publish {at} had_caller_segment=true fixed_point_rounds=2"
    ));
    for (row, calls) in [
        ("files_other_than_the_first_plan", 2),
        ("file_other_than_the_first_linear_find", 2),
        ("inodes_with_an_extent_entry_set", 2),
        ("extent_upper_segment_plan", 2),
        ("extent_roles_rewritten_scan", 2),
        ("root_header_linear_find", 8),
        ("checkpoint_cost_accounting_tree_plan", 2),
        ("cluster_segment_sum", 4),
        ("inode_number_watermark_steps", 4),
        ("allocator_release_linear_find", 10),
        ("user_data_slot_linear_scan", 2),
        ("record_intersection_filter", 40),
        ("empty_segment_linear_scan", 4),
        ("accounting_entries_of_this_publish", 2),
    ] {
        line(&format!(
            "name=row {at} segment=before_landing row={row} calls={calls} steps=1"
        ));
    }
    line(&format!("name=row {at} segment=between row=release_judgement_version_indexes calls=1 query_levels=138"));
    let mut delta = |segment: &str, fixed_point_round: u32, key: &str, value: u32| {
        line(&format!("name=delta {at} segment={segment} fixed_point_round={fixed_point_round} key={key} value={value}"));
    };
    delta("before_landing", 0, "touched_files", 1);
    delta("before_landing", 0, "lower_nodes_of_touched_files", 1);
    delta(
        "before_landing",
        0,
        "accounting_rows_written_by_the_built_on_publish",
        15,
    );
    delta("between", 0, "deadlist_entries_changed", 0);
    delta("between", 0, "moved_entries", 0);
    delta("between", 0, "intersecting_records", 40);
    for round in [1, 2] {
        delta("between", round, "changed_leaves", 4);
        delta("between", round, "round_placements_released", 29);
        delta("between", round, "round_roles_rewritten", 1);
        delta("between", round, "container_units_rewritten", 0);
        for keys_changed in [15, 0, 2] {
            delta("between", round, "code_two_keys_changed", keys_changed);
        }
        for _ in 0..2 {
            delta("between", round, "dirty_records", 40);
        }
    }
    text
}

#[test]
fn the_by_delta_inputs_read_from_a_product_reproduce_the_registered_total() {
    let product = read_product(&delta_zero_product_text());
    let input = by_delta_input_from_the_product(
        &product,
        "base",
        "same_mount",
        "overwrite",
        &SEGMENTS_BEFORE_ADMISSION,
    )
    .expect("Δ0 的每个读数都给了");
    // 登记第七节 ③3 的 Δ0 写着「s36 每轮 dm 2、查询 0」，合计 1336（`delta_zero` 那一份直接喂式子，照旧钉 1336）。
    // 从产物读时 s36 的查询次数照修订 7 取这一轮进出 `mapped_units` 的角色数：两轮各 2 个，各查 lg(1500) = 11 步，
    // s36 = 4 + 4 × 11 = 48，合计 1336 − 4 + 48 = 1380
    assert_eq!(by_delta_total(&input), 1380);
    let by_row: BTreeMap<&str, u64> = by_delta_rows(&input).into_iter().collect();
    assert_eq!(by_row["release_judgement_version_indexes"], 138);
    assert_eq!(by_row["multi_level_tree_planning"], 68);
    assert_eq!(by_row["record_intersection_filter"], 480);
    assert_eq!(by_row["mapped_units_whole_table"], 48);
}

#[test]
fn the_mapped_units_queries_of_the_by_delta_arm_are_the_roles_moved_and_not_the_whole_table() {
    // B1 那一行照抄每个角色逐个查上一版的索引：产物里这一行的 query_levels 是整表的量（这里写 1500 次 × 11 = 16500）。
    // B8 只问这一轮进出 `mapped_units` 的角色（两轮各 2 个，M = 1500，lg 11）：s36 = 4 + 4 × 11 = 48，与产物里那个整表的数无关
    let text = delta_zero_product_text()
        + "E203RESULT name=row point=base state=same_mount kind=overwrite round=1 segment=between row=mapped_units_whole_table calls=2 steps=3000 query_levels=16500\n";
    let input = by_delta_input_from_the_product(
        &read_product(&text),
        "base",
        "same_mount",
        "overwrite",
        &SEGMENTS_BEFORE_ADMISSION,
    )
    .expect("Δ0 的每个读数都给了");
    assert_eq!(input.mapped_units_changed_over_all_rounds, 4);
    assert_eq!(input.mapped_units_query_levels, 44);
    let by_row: BTreeMap<&str, u64> = by_delta_rows(&input).into_iter().collect();
    assert_eq!(by_row["mapped_units_whole_table"], 48);
    assert_eq!(
        mapped_units_queries_by_delta(0, 1500),
        0,
        "这一轮没有角色进出就一次不问"
    );
    assert_eq!(mapped_units_queries_by_delta(6, 10_145), 6 * 14);
}

#[test]
fn an_unread_value_is_a_gap_and_not_a_zero() {
    let text =
        delta_zero_product_text().replace("key=changed_leaves", "key=changed_leaves_renamed");
    let product = read_product(&text);
    let gaps = by_delta_input_from_the_product(
        &product,
        "base",
        "same_mount",
        "overwrite",
        &SEGMENTS_BEFORE_ADMISSION,
    )
    .expect_err("少了每轮内容变了的叶数");
    assert!(
        gaps.iter()
            .any(|gap| gap.starts_with("changed_leaves@round")),
        "{gaps:?}"
    );
}

#[test]
fn fixed_point_round_with_two_code_two_trees_is_a_gap() {
    // 第一段真产物的形态：deadlist 树走零根那一支没记，每轮只有记账树与中央映射树两条。每轮要三条，少了记读不到，不拿 0 补
    let text = delta_zero_product_text().replace(
        "key=code_two_keys_changed value=0\n",
        "key=code_two_keys_changed_not_recorded value=0\n",
    );
    let product = read_product(&text);
    let gaps = by_delta_input_from_the_product(
        &product,
        "base",
        "same_mount",
        "overwrite",
        &SEGMENTS_BEFORE_ADMISSION,
    )
    .expect_err("每轮只有两棵码 2 树的读数");
    assert!(
        gaps.iter()
            .any(|gap| gap == "delta.code_two_keys_changed:4_for_2_rounds"),
        "{gaps:?}"
    );
}

#[test]
fn the_deadlist_tree_keys_are_weighted_by_the_deadlist_tree_height() {
    // 每轮三条按记账树、deadlist 树、中央映射树的次序读，各乘各自的层数（Δ0：2、1、2）。
    // deadlist 那一条从 0 改成 3：每轮多 3 × 1，两轮共多 6，s20 从 68 到 74；deadlist 与中央映射两条的层数配反了会是 76
    let text = delta_zero_product_text().replace(
        "key=code_two_keys_changed value=0\n",
        "key=code_two_keys_changed value=3\n",
    );
    let input = by_delta_input_from_the_product(
        &read_product(&text),
        "base",
        "same_mount",
        "overwrite",
        &SEGMENTS_BEFORE_ADMISSION,
    )
    .expect("每轮三条都在");
    let by_row: BTreeMap<&str, u64> = by_delta_rows(&input).into_iter().collect();
    assert_eq!(by_row["multi_level_tree_planning"], 68 + 2 * 3);
}

#[test]
fn the_extrapolation_and_the_summary_rules_hold() {
    let points = [(10.0, 1.0), (100.0, 2.0), (1000.0, 3.0)];
    assert_eq!(value_on_axis(&points, 100.0), Some(2.0), "量过的点直接取");
    assert_eq!(value_on_axis(&points, 550.0), Some(2.5), "两点之间线性内插");
    assert_eq!(
        value_on_axis(&points, 10_000.0),
        Some(13.0),
        "最后两点的斜率线性外推"
    );
    assert_eq!(value_on_axis(&points[..1], 20.0), None, "一个点外推不出");
    let grows = Some(Band::GrowsWithTheAxis);
    let flat = Some(Band::DoesNotGrow);
    assert_eq!(summarize(&[flat, grows], false), Summary::GrowsOnSomeAxis);
    assert_eq!(
        summarize(&[flat, flat], false),
        Summary::DoesNotGrowOnAnyAxis
    );
    assert_eq!(
        summarize(&[flat, Some(Band::Between)], false),
        Summary::BetweenOnSomeAxis
    );
    assert_eq!(
        summarize(&[flat], true),
        Summary::SomeInputUnread,
        "有格读不到不记「不随」"
    );
    assert_eq!(summarize(&[], false), Summary::SomeInputUnread);
}

#[test]
fn the_pooled_fallback_steps_join_the_today_total_and_mark_the_by_delta_side_when_called() {
    let flat = synthetic_counts_text(&synthetic_unit_area_points(4, 8));
    let at = "point=unit_area_2_pow_24 state=same_mount kind=overwrite round=1";
    let called = format!(
        "{flat}E203RESULT name=row {at} segment=between row=pooled_fallback_by_the_leaf calls=2 steps=40\n\
             E203RESULT name=row {at} segment=between row=fallback_slot_linear_scan calls=0 steps=7\n"
    );
    // s29 那一行合成输入里原本没有；调用次数记 0 让 B8 的输入不变，只给它 7 步好读出来
    let (flat_counts, called_counts) = (read_product(&flat), read_product(&called));
    let (mut flat_out, mut called_out) = (String::new(), String::new());
    // pooled_fallback_calls = 0 的点：一格都不标
    assert_eq!(pooled_fallback_report(&flat_counts, &mut flat_out), (0, 12), "{flat_out}");
    assert!(
        flat_out.contains("name=verdict quantity=by_delta_pooled_fallback cells_checked=12 cells_with_pooled_fallback_calls=0"),
        "{flat_out}"
    );
    // pooled_fallback_calls > 0 的点：只标这一格，带 s37 与 s29 的步数；B8 合计不变（s37 没有式子），B1 合计多 40 步
    assert_eq!(pooled_fallback_report(&called_counts, &mut called_out), (1, 12), "{called_out}");
    let marked: Vec<&str> = called_out
        .lines()
        .filter(|line| line.contains("by_delta_arms_total=missing_pooled_fallback_row_only_for_the_listed_steps"))
        .collect();
    assert_eq!(marked.len(), 1, "{called_out}");
    assert!(
        marked[0].starts_with("name=by_delta_pooled_fallback point=unit_area_2_pow_24 state=same_mount kind=overwrite pooled_fallback_calls=2 pooled_fallback_steps=40 per_disk_fallback_steps=7 by_delta_total="),
        "{}",
        marked[0]
    );
    let unmarked_total = flat_out
        .lines()
        .find(|line| line.contains("point=unit_area_2_pow_24 state=same_mount kind=overwrite "))
        .and_then(|line| line.split("by_delta_total=").nth(1))
        .and_then(|rest| rest.split(' ').next())
        .expect("读得到 B8 合计");
    assert_ne!(unmarked_total, "unread", "{flat_out}");
    assert!(marked[0].contains(&format!("by_delta_total={unmarked_total} ")), "{}", marked[0]);
    let today = |counts: &Product| {
        scan_steps(counts, "unit_area_2_pow_24", "same_mount", "overwrite", &SEGMENTS_BEFORE_ADMISSION)
    };
    assert_eq!(today(&called_counts), today(&flat_counts) + 40 + 7);
}

#[test]
fn the_records_of_each_leaf_take_every_record_and_every_leaf_of_the_union() {
    // s30：四次分叶，那一刻记录 900、905、905、910 条（3620），两次比叶各 5 片与 15 片（20）：全表扫 3640
    assert_the_call_site_row_reconciles("unlink_final", "records_of_each_leaf", 3640, 0);
}
