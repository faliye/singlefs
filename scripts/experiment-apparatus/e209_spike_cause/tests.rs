use super::*;

// ---- 合成日志（第五节「阳性对照」）：节点抽象成编号，tree = 1..12；a 种子 {1}、拉进 {2..6}，b 种子 {7}、拉进空，c 种子 {8}、拉进 {9..12} ----

fn publish_of(name: &str) -> PublishInput {
    let (seed, pulled): (Vec<u64>, Vec<u64>) = match name {
        "a" => (vec![1], vec![2, 3, 4, 5, 6]),
        "b" => (vec![7], vec![]),
        "c" => (vec![8], vec![9, 10, 11, 12]),
        other => panic!("合成日志里没有操作 {other}"),
    };
    let final_numbers: Vec<u64> = seed.iter().chain(&pulled).copied().collect();
    PublishInput {
        seed: synthetic_set(&seed),
        pulled: synthetic_set(&pulled),
        final_set: synthetic_set(&final_numbers),
        previous_nodes: synthetic_set(&(1..=12).collect::<Vec<u64>>()),
        ..PublishInput::default()
    }
}

fn synthetic_steps(sequence: &[&str]) -> Vec<StepInput> {
    sequence
        .iter()
        .map(|name| {
            if *name == "remount" {
                StepInput::Remount(Vec::new())
            } else {
                StepInput::Operation(OperationInput { name: (*name).to_string(), selector: (*name).to_string(), final_nodes: 7, publishes: vec![publish_of(name)], ..OperationInput::default() })
            }
        })
        .collect()
}

fn synthetic_views(item: &str, sequence: &[&str]) -> Vec<OperationView> {
    compute_views(item, "synthetic", item, (&NodeSet::new(), &NodeSet::new()), 0, &synthetic_steps(sequence))
}

/// 三条生成规则：`fresh ≥ 5`、用户序号 1 或 3、`Q_stale ≥ 6`（量名在产物里叫 `stale_nodes`）。
fn with_flags(mut views: Vec<OperationView>, rule: &str) -> Vec<OperationView> {
    for view in &mut views {
        view.flagged = match rule {
            "by_nodes" => view.fresh[0] >= 5,
            "by_ordinal" => view.ordinal_user == 1 || view.ordinal_user == 3,
            "by_quantity" => view.stale_nodes >= 6,
            other => panic!("没有生成规则 {other}"),
        };
    }
    views
}

fn flagged_ordinals(views: &[OperationView]) -> Vec<u64> {
    views.iter().filter(|view| view.flagged).map(|view| view.ordinal_user).collect()
}

fn pair(rule: &str) -> (Vec<OperationView>, Vec<OperationView>) {
    (with_flags(synthetic_views("forward_script", &["a", "b", "c", "a", "c"]), rule), with_flags(synthetic_views("rotated_script", &["b", "c", "a", "c", "a"]), rule))
}

fn refs(views: &[OperationView]) -> Vec<&OperationView> {
    views.iter().collect()
}

#[test]
fn the_synthetic_logs_have_the_registered_freshness_and_stored_quantity() {
    let (forward_views, rotated_views) = pair("by_nodes");
    let column = |views: &[OperationView], variant: usize| views.iter().map(|view| view.fresh[variant]).collect::<Vec<_>>();
    assert_eq!(column(&forward_views, 0), [6, 1, 5, 0, 0], "宽读：种子与拉进都算");
    assert_eq!(column(&rotated_views, 0), [1, 5, 6, 0, 0]);
    assert_eq!(column(&forward_views, 1), [5, 0, 4, 0, 0], "窄读：只算拉进的");
    assert_eq!(forward_views.iter().map(|view| view.stale_nodes).collect::<Vec<_>>(), [12, 6, 5, 0, 0]);
    assert_eq!(rotated_views.iter().map(|view| view.stale_nodes).collect::<Vec<_>>(), [12, 11, 6, 0, 0]);
    assert_eq!(forward_views.iter().map(|view| view.ordinal_user).collect::<Vec<_>>(), [1, 2, 3, 4, 5], "序号从 1 数");
}

#[test]
fn the_three_generation_rules_flag_the_registered_ordinals() {
    let expected = [("by_nodes", vec![1, 3], vec![2, 3]), ("by_ordinal", vec![1, 3], vec![1, 3]), ("by_quantity", vec![1, 2], vec![1, 2, 3])];
    for (rule, on_forward, on_rotated) in expected {
        let (forward_views, rotated_views) = pair(rule);
        assert_eq!(flagged_ordinals(&forward_views), on_forward, "{rule} 在 X 上");
        assert_eq!(flagged_ordinals(&rotated_views), on_rotated, "{rule} 在 Y 上");
    }
}

#[test]
fn the_freshness_judge_row_of_the_matrix_is_green_red_red() {
    let (forward_views, rotated_views) = pair("by_nodes");
    let reading = judge_freshness_rule(0, &refs(&forward_views), &refs(&rotated_views), &[]);
    assert_eq!((reading.threshold, reading.agrees(), reading.script.mismatches.len()), (Some(5), true, 0), "gen_by_nodes 绿：0 处不符");
    let (ordinal_forward_views, ordinal_rotated_views) = pair("by_ordinal");
    let ordinal_reading = judge_freshness_rule(0, &refs(&ordinal_forward_views), &refs(&ordinal_rotated_views), &[]);
    assert!(!ordinal_reading.agrees());
    assert!(ordinal_reading.defining_failures.is_empty());
    assert_eq!(ordinal_reading.script.mismatches.len(), 2, "gen_by_ordinal 红：Y 上 2 处不符");
    assert!(ordinal_reading.script.mismatches[0].contains("#1:") && ordinal_reading.script.mismatches[1].contains("#2:"), "序号 1、2：{:?}", ordinal_reading.script.mismatches);
    let (quantity_forward_views, quantity_rotated_views) = pair("by_quantity");
    let quantity_reading = judge_freshness_rule(0, &refs(&quantity_forward_views), &refs(&quantity_rotated_views), &[]);
    assert_eq!(quantity_reading.threshold, Some(1), "m 取旗上操作新鲜度的最小值：6、1 取 1");
    assert_eq!(quantity_reading.defining_failures.len(), 1, "gen_by_quantity 红：X 上定规则失败 1 处");
    assert!(quantity_reading.defining_failures[0].contains("#3:fresh=5"), "序号 3 的 fresh = 5：{:?}", quantity_reading.defining_failures);
    assert_eq!(quantity_reading.verdict(), RuleVerdict::DoesNotHold);
}

#[test]
fn the_threshold_boundary_turns_the_green_cell_red_outside_two_to_five() {
    let (forward_views, _) = pair("by_nodes");
    let wrong = |threshold: u64| mismatches_at(&refs(&forward_views), 0, threshold).mismatches.len();
    for threshold in 2..=5 {
        assert_eq!(wrong(threshold), 0, "m = {threshold} 在有效区间里");
    }
    assert_eq!(wrong(6), 1, "m = 6：c 的 fresh = 5 被判不旗");
    assert_eq!(wrong(1), 1, "m = 1：b 的 fresh = 1 被判旗");
}

#[test]
fn the_ordinal_judge_row_of_the_matrix_is_red_green_red() {
    let first_difference = |rule: &str| {
        let (forward_views, rotated_views) = pair(rule);
        let by_script = BTreeMap::from([("forward_script".to_string(), refs(&forward_views)), ("rotated_script".to_string(), refs(&rotated_views))]);
        judge_ordinal_equality(&by_script).first_difference
    };
    assert_eq!(first_difference("by_nodes"), Some("forward_script/rotated_script#1".to_string()), "首处不同在序号 1");
    assert_eq!(first_difference("by_ordinal"), None, "gen_by_ordinal 绿");
    assert_eq!(first_difference("by_quantity"), Some("forward_script/rotated_script#3".to_string()));
}

#[test]
fn a_pair_of_scripts_of_different_lengths_is_not_compared_past_the_shorter_one() {
    let long = with_flags(synthetic_views("long", &["a", "b", "c", "a", "c", "a"]), "by_ordinal");
    let short = with_flags(synthetic_views("short", &["a", "b", "c", "a"]), "by_ordinal");
    let by_script = BTreeMap::from([("long".to_string(), refs(&long)), ("short".to_string(), refs(&short))]);
    let reading = judge_ordinal_equality(&by_script);
    assert_eq!((reading.pairs, reading.first_difference), (1, None), "短的那边没有值的序号不报不同");
}

#[test]
fn the_stored_quantity_judge_row_of_the_matrix_is_red_red_green() {
    let pooled = |rule: &str| {
        let (forward_views, rotated_views) = pair(rule);
        forward_views.iter().chain(&rotated_views).map(|view| (view.stale_nodes as f64, view.flagged)).collect::<Vec<_>>()
    };
    assert_eq!(judge_stored_quantity(&pooled("by_nodes"), &[]).verdict, QuantityVerdict::DoesNotHold, "Q = 12 在 X 上旗上、在 Y 上不旗");
    assert_eq!(judge_stored_quantity(&pooled("by_ordinal"), &[]).verdict, QuantityVerdict::DoesNotHold, "Q = 6 在 X 上不旗、在 Y 上旗");
    let reading = judge_stored_quantity(&pooled("by_quantity"), &[]);
    assert_eq!((reading.pooled.threshold, reading.pooled.errors, reading.pooled.flagged_values, reading.pooled.unflagged_values), (6.0, 0, 3, 2), "θ = 6，旗上取值 {{12,11,6}}，不旗取值 {{5,0}}");
    assert_eq!(reading.verdict, QuantityVerdict::TestGeometryNotRun, "检验几何还没跑");
}

#[test]
fn the_threshold_of_the_stored_quantity_is_not_refitted_on_the_test_geometry() {
    let training = [(12.0, true), (11.0, true), (6.0, true), (5.0, false), (0.0, false)];
    let doubled_scale = [(24.0, true), (12.0, true), (10.0, false), (0.0, false)];
    let reading = judge_stored_quantity(&training, &doubled_scale);
    assert_eq!((reading.geometry_errors, reading.verdict), (1, QuantityVerdict::OnlyInTheTrainingGeometry), "量程乘 2 之后门槛 6 把 10 错判成旗上");
}

#[test]
fn the_freshness_mount_variants_are_told_apart_by_a_remount() {
    let since_mount = |rule: &str| {
        let mut views = synthetic_views("w", &["a", "remount", "a", "b"]);
        for view in &mut views {
            view.flagged = match rule {
                "since_mount" => view.fresh[0] >= 5,
                "since_creation" => view.fresh[2] >= 5,
                other => panic!("{other}"),
            };
        }
        views
    };
    let mount_views = since_mount("since_mount");
    assert_eq!(mount_views.iter().map(|view| view.fresh[0]).collect::<Vec<_>>(), [6, 6, 1], "挂载清零：重挂载之后的 a 又是 6");
    assert_eq!(mount_views.iter().map(|view| view.fresh[2]).collect::<Vec<_>>(), [6, 0, 1], "不清零：重挂载之后的 a 是 0");
    assert_eq!(mount_views.iter().map(|view| view.epoch).collect::<Vec<_>>(), [0, 1, 1]);
    assert_eq!(mount_views.iter().map(|view| view.ordinal_user).collect::<Vec<_>>(), [1, 1, 2], "重挂载之后用户序号从 1 重数");
    let creation_views = since_mount("since_creation");
    let check = |views: &[OperationView], variant: usize| judge_freshness_rule(variant, &refs(views), &[], &[]);
    assert!(check(&mount_views, 0).agrees() && check(&creation_views, 2).agrees(), "各配各的判器绿");
    assert_eq!(check(&creation_views, 0).defining_failures.len(), 1, "挂载判器对不清零的生成规则红 1 处");
    assert_eq!(check(&mount_views, 2).defining_failures.len(), 1, "不清零判器对清零的生成规则红 1 处");
}

#[test]
fn the_spike_flag_uses_the_lower_median_and_the_floor() {
    assert_eq!((SPIKE_FLOOR, SPIKE_MULTIPLE), (28, 4), "同 E207 装置的两个常量");
    assert_eq!(spike_line(SPIKE_FLOOR, &[7, 7, 7, 433, 7]), 28);
    assert_eq!(spike_line(SPIKE_FLOOR, &[3, 3, 3, 3, 20]), 28);
    assert_eq!(lower_median(&[1, 2, 3, 4]), 2, "下中位数");
    let flags = flags_by_line(&[7, 7, 7, 60, 7]);
    assert_eq!(flags.iter().map(|row| row[MAIN_FLOOR_LINE_POSITION]).collect::<Vec<_>>(), [false, false, false, true, false], "下中位数 7 → 旗线 28，60 旗上；均值会把旗线抬到 68");
}

#[test]
fn the_reproduction_gate_tells_the_low_line_from_the_main_line() {
    assert_eq!(reproduction_gate(&[3, 3, 3, 20, 3]), ReproductionGate { flagged_by_line: [1, 0, 0], verdict: "only_at_the_low_line" }, "中位数 3：L = 28 不旗、L = 14 旗上");
    assert_eq!(reproduction_gate(&[3, 3, 3, 60, 3]).verdict, "reproduced");
    assert_eq!(reproduction_gate(&[7, 7, 7, 7, 7]).verdict, "not_reproduced");
    assert_eq!(reproduction_gate(&[7, 7, 7, 20, 7]).verdict, "not_reproduced", "中位数 7 时 4 × 7 = 28 盖过 14，低旗线也不旗");
}

#[test]
fn the_median_is_inflated_when_more_than_half_are_flagged() {
    assert!(median_is_inflated(&[true, true, true, false, false]));
    assert!(!median_is_inflated(&[true, true, false, false]));
}

fn mount_steps(carries: &str) -> Vec<(String, NodeSet)> {
    let (write_row, warm_up) = match carries {
        "carries" => (vec![1, 7], vec![2, 3, 4, 5, 6, 8]),
        "none" => (vec![7, 8], vec![7, 8]),
        "spread" => (vec![1, 2, 3, 7], vec![4, 5, 6, 8]),
        other => panic!("{other}"),
    };
    vec![("write_row".to_string(), synthetic_set(&write_row)), ("warm_up_1".to_string(), synthetic_set(&warm_up))]
}

#[test]
fn the_mount_judge_is_green_red_amber_on_the_synthetic_mounts() {
    let spike = synthetic_set(&[2, 3, 4, 5, 6]);
    let carries = judge_mount_carries(false, &spike, &mount_steps("carries"));
    assert_eq!((carries.verdict, carries.located_step.as_deref(), carries.one_step_verdict), ("mount_carried_the_nodes", Some("warm_up_1"), "located_one_step"));
    let none = judge_mount_carries(true, &spike, &mount_steps("none"));
    assert_eq!(none.verdict, "mount_did_not_stop_the_spike", "挂载后 T 旗上：前提不成立");
    let spread = judge_mount_carries(false, &spike, &mount_steps("spread"));
    assert_eq!((spread.verdict, spread.located_step, spread.one_step_verdict), ("mount_carried_the_nodes", None, "spread_over_several_steps"));
    assert_eq!(spread.shares_by_step, [("write_row".to_string(), 2), ("warm_up_1".to_string(), 3)], "各占 40% 与 60%");
}

#[test]
fn the_extra_steps_judge_separates_located_spread_and_unresolved() {
    let rows = |values: &[i64]| values.iter().enumerate().map(|(position, value)| (format!("row_{position}"), *value)).collect::<Vec<_>>();
    let (share, top) = top_rows_share(&rows(&[700, 250, 30, 20]), 1000);
    assert_eq!((top.len(), (share * 1000.0).round()), (3, 980.0), "前三大行占 98%");
    assert_eq!(classify_extra(1000, 0, share), "located_in_the_top_rows");
    let (spread_share, _) = top_rows_share(&rows(&[100; 10]), 1000);
    assert_eq!(classify_extra(1000, 0, spread_share), "spread_over_many_rows", "十行各 100，前三大行占 30%");
    let (concentrated_share, _) = top_rows_share(&rows(&[600, 200, 150, 50]), 1000);
    assert_eq!(classify_extra(1000, 0, concentrated_share), "located_in_the_top_rows", "前三大 95%、前两大只有 80%");
    assert!(extra_is_resolved(1000, 5) && !extra_is_resolved(1000, 150), "c2 − c1 = 5 分辨够、150 分辨不够（6.7 < 10）");
    assert_eq!(classify_extra(1000, 150, 0.98), "not_resolved");
    assert_eq!(classify_extra(1000, 0, 0.98), "located_in_the_top_rows", "噪声 0 算分辨够");
    let thresholds = (ONE_STEP_SHARE_AT_LEAST, MOUNT_SHARE_CARRIED_AT_LEAST, TOP_ROWS_SHARE_AT_LEAST);
    assert_eq!(thresholds, (0.9, 0.9, 0.9), "三条占比线都是 0.9，挪到 0.99 或 0.5 的自证由变异表走");
}

#[test]
fn the_steps_per_extra_node_exists_only_for_a_positive_extra_in_steps_and_in_nodes() {
    assert_eq!(steps_per_extra_node(7428, 46), Some(161.478), "7428 ÷ 46 保留 3 位");
    assert_eq!(steps_per_extra_node(-10730, 26), None, "多出的步数为负：没有这个比");
    assert_eq!(steps_per_extra_node(0, 26), None, "多出的步数为 0：没有这个比");
    assert_eq!(steps_per_extra_node(100, -5), None, "节点反而少了：没有这个比");
    assert_eq!(steps_per_extra_node(100, 0), None, "节点没多：没有这个比");
}

#[test]
fn the_round_shares_are_shares_of_the_sum_of_the_round_differences() {
    let rounds = |values: &[(&str, u64)]| values.iter().map(|(exit, steps)| (exit.to_string(), *steps)).collect::<Vec<_>>();
    let shares = round_shares(&rounds(&[("release_not_covered", 300), ("settled", 700)]), &rounds(&[("release_not_covered", 100), ("settled", 100)]));
    assert_eq!(shares, [(1, "release_not_covered".to_string(), 0.25), (2, "settled".to_string(), 0.75)], "差 200 与 600，和 800：占 25% 与 75%，两份加起来是 1");
    let settled_only_shares = round_shares(&rounds(&[("settled", 500)]), &rounds(&[("settled", 100), ("settled", 400)]));
    assert_eq!(settled_only_shares, [(1, "settled".to_string(), 0.0), (2, "none".to_string(), 0.0)], "差的和是 0：都记 0，缺的轮出口记 none");
}

fn comparable_view(item: &str, flagged: bool) -> OperationView {
    OperationView { item: item.to_string(), point: "p".to_string(), selector: "first".to_string(), old_units: 1, new_units: 1, flagged, step_index: 1, ..OperationView::default() }
}

#[test]
fn the_comparables_are_the_first_two_unflagged_operations_by_item_name() {
    let target = comparable_view("p__head__walk_forward", true);
    let candidates = vec![
        comparable_view("p__head__walk_forward", true),
        comparable_view("p__head__c_script", false),
        comparable_view("p__head__a_script", false),
        comparable_view("p__head__b_script", false),
        comparable_view("p__head__flagged_script", true),
    ];
    let names: Vec<&str> = pick_comparables(&target, &candidates).iter().map(|view| view.item.as_str()).collect();
    assert_eq!(names, ["p__head__a_script", "p__head__b_script"]);
}

#[test]
fn the_ordinal_shift_judge_separates_following_the_operation_from_following_the_ordinal() {
    let view = |ordinal: u64, selector: &str, flagged: bool| OperationView { ordinal_user: ordinal, selector: selector.to_string(), flagged, ..OperationView::default() };
    let walk = [view(1, "first", true), view(2, "old", false)];
    let walk_refs: Vec<&OperationView> = walk.iter().collect();
    let filler = |count: u64, flagged_ordinal: u64| -> (String, u64, Vec<OperationView>) {
        let views = (1..=count + 1).map(|ordinal| view(ordinal, if ordinal == count + 1 { "first" } else { "new_file" }, ordinal == flagged_ordinal)).collect();
        (format!("filler_create_{count}"), count, views)
    };
    let judged = |fillers: Vec<(String, u64, Vec<OperationView>)>| {
        let refs: Vec<(String, u64, Vec<&OperationView>)> = fillers.iter().map(|(name, count, views)| (name.clone(), *count, views.iter().collect())).collect();
        judge_ordinal_shift(&walk_refs, &refs).verdict
    };
    assert_eq!(judged(vec![filler(1, 2), filler(4, 5)]), "follows_the_operation", "first 在序号 n + 1 旗上");
    assert_eq!(judged(vec![filler(1, 1), filler(4, 1)]), "follows_the_ordinal", "旗留在序号 1");
    assert_eq!(judged(vec![filler(1, 2), filler(4, 1)]), "neither");
}

#[test]
fn the_presweep_judge_needs_both_sides_of_the_threshold() {
    let point = |count: u64, flagged: bool| SweepPoint { presweep_count: count, flagged, stale_nodes: 0, publishes_since_flag: 0, final_nodes: 0, first_flagged_ordinal: None, first_flagged_selector: String::new() };
    let reading = judge_presweep(&[point(0, true), point(1, true), point(4, false), point(8, false)]);
    assert_eq!((reading.between, reading.verdict), (Some((1, 4)), "threshold_between_two_sample_points"));
    assert_eq!(judge_presweep(&[point(0, true), point(1, true)]).verdict, "threshold_sides_not_both_sampled");
    assert_eq!(judge_presweep(&[point(0, false), point(1, true)]).verdict, "not_monotone_in_the_presweep_count");
}
