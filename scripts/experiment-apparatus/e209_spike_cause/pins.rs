use super::*;

fn position(level: u32, device: u32, index_in_device: u64) -> Node {
    Node::Position { level, device, index_in_device }
}

#[test]
fn the_position_arithmetic_is_pinned_on_the_registered_slots() {
    assert_eq!(LEAF_SLOTS * INTERNAL_FANOUT, 137_228, "第七节 ③ 第 2 条");
    for (slot, leaf) in [(0, 0), (811, 0), (812, 1), (137_227, 168), (137_228, 169)] {
        assert_eq!(leaf_of_slot(0, slot), position(0, 0, leaf), "槽 {slot}");
    }
    assert_eq!(parent_of(position(0, 0, 168), 3), Some(position(1, 0, 0)));
    assert_eq!(parent_of(position(0, 0, 169), 3), Some(position(1, 0, 1)), "叶 169 是第二个第一层节点的头一个孩子");
    assert_eq!(parent_of(position(1, 2, 7), 2), Some(Node::Root), "父的层级等于根的层级时父是根");
    assert_eq!(parent_of(Node::Root, 2), None);
    let per_device = |slots_per_device: u64| {
        let leaves = slots_per_device.div_ceil(LEAF_SLOTS);
        (leaves, leaves.div_ceil(INTERNAL_FANOUT))
    };
    assert_eq!(per_device(1 << 17), (162, 1));
    assert_eq!(per_device(1 << 19), (646, 4));
    assert_eq!(per_device(1 << 20), (1292, 8));
}

#[test]
fn the_history_user_changes_are_pinned_on_the_registered_points() {
    let notes: Vec<u64> = [10, 75, 100, 200].map(scripts::notes_of_the_history).to_vec();
    assert_eq!(notes, [42, 172, 222, 422], "第四节第 5 条 `python3 -I` 现算的值");
    assert_eq!(scripts::sample_point_named("files_100_units_100").map(|point| (point.files, point.units_per_file)), Some((100, 100)));
}

fn graph_with_two_devices() -> PointerGraph {
    PointerGraph::from([
        (position(0, 0, 0), vec![(0, 812 * 2 + 5), (1, 7)]),
        (position(0, 0, 2), vec![(0, 812 * 5)]),
    ])
}

#[test]
fn the_closure_model_follows_both_old_locations_and_adds_the_ancestors_and_the_root() {
    let closure = release_chain_closure(&graph_with_two_devices(), 2, &BTreeSet::from([position(0, 0, 0)]));
    let expected: NodeSet = [position(0, 0, 0), position(0, 0, 2), position(0, 0, 5), position(0, 1, 0), position(1, 0, 0), position(1, 1, 0), Node::Root].into();
    assert_eq!(closure, expected, "第二块盘的落点拉进 (0,1,0) 与它的祖先，叶 (0,0,2) 自己的旧落点又拉进叶 5；每次都爬到根");
    assert!(closure.contains(&Node::Root));
    assert_eq!(release_chain_closure(&graph_with_two_devices(), 2, &BTreeSet::from([position(0, 0, 9)])), BTreeSet::from([position(0, 0, 9)]), "上一版没有的节点没有旧落点，走到它为止");
    assert_eq!(release_chain_depth(&graph_with_two_devices(), 2, position(0, 0, 0)), 3);
    assert_eq!(release_chain_depth(&graph_with_two_devices(), 2, position(0, 0, 9)), 1);
}

#[test]
fn the_pointer_graph_text_gives_both_locations_of_every_node() {
    let graph = parse_pointer_graph("0/0/1@0.5+1.900,root@0.9+1.19").expect("读得出");
    assert_eq!(graph.get(&position(0, 0, 1)), Some(&vec![(0, 5), (1, 900)]), "两份落点都读");
    assert_eq!(graph.get(&Node::Root), Some(&vec![(0, 9), (1, 19)]));
    assert_eq!(parse_pointer_graph("-"), Some(PointerGraph::new()));
    assert_eq!(parse_pointer_graph("0/0/1@0.x"), None, "读不出整串不读");
    assert_eq!(parse_node_set("0/0/1,root"), Some(BTreeSet::from([position(0, 0, 1), Node::Root])));
    assert_eq!(parse_node_set("0/0/1,bad"), None);
}

const PRODUCT: &str = "\
E209RESULT name=header for_item=files_100_units_100__head__mount_then_first twin=no
E209RESULT name=point item=files_100_units_100__head__mount_then_first point=files_100_units_100 arm=head script=mount_then_first build=counts unit_area_slots=524288 unlinks=10 data_units=10 files=100 units_per_small_file=100 shape=small_files_of_units
E209RESULT name=constants item=files_100_units_100__head__mount_then_first leaf_slots=812 internal_fanout=169 settling_rounds_at_most=64 disks=2
E209RESULT name=history item=files_100_units_100__head__mount_then_first user_changes=222 settle_calls=222
E209RESULT name=history_carried item=files_100_units_100__head__mount_then_first ever_final=0/0/1,0/0/2 ever_pulled=-
E209RESULT name=script_step item=files_100_units_100__head__mount_then_first step_index=1 operation=remount mount_epoch=0
E209RESULT name=settle item=files_100_units_100__head__mount_then_first phase=remount mount_epoch=1 operation=remount operation_index=1 settle_call=223 judgement=0 rounds=1 outcome=settled final_nodes=2
E209RESULT name=nodes item=files_100_units_100__head__mount_then_first phase=remount mount_epoch=1 operation=remount operation_index=1 settle_call=223 publish_kind=write_row publish_order=0 first_merge_site=none seed=0/0/1 pulled=0/0/3 final=0/0/1,0/0/3 released_records_pre=4 edges=0/0/1>0/0/3 edges_walk_matches_the_implementation=1
E209RESULT name=pointer_graph item=files_100_units_100__head__mount_then_first phase=remount mount_epoch=1 operation=remount operation_index=1 settle_call=223 publish_kind=write_row publish_order=0 root_level=2 graph=0/0/1@0.2437+1.5
E209RESULT name=script_step item=files_100_units_100__head__mount_then_first step_index=2 operation=overwrite_first mount_epoch=1
E209RESULT name=declared item=files_100_units_100__head__mount_then_first phase=script mount_epoch=1 operation=overwrite_first operation_index=2 user_units=1 old_units=1 new_units=1
E209RESULT name=settle item=files_100_units_100__head__mount_then_first phase=script mount_epoch=1 operation=overwrite_first operation_index=2 settle_call=224 judgement=0 rounds=2 outcome=settled final_nodes=3
E209RESULT name=nodes item=files_100_units_100__head__mount_then_first phase=script mount_epoch=1 operation=overwrite_first operation_index=2 settle_call=224 publish_kind=user publish_order=0 first_merge_site=release_not_covered seed=0/0/1 pulled=0/0/2,0/0/3 final=0/0/1,0/0/2,0/0/3 released_records_pre=4 edges=- edges_walk_matches_the_implementation=1
E209RESULT name=item_done item=files_100_units_100__head__mount_then_first
";

#[test]
fn a_product_is_read_back_with_the_remount_publishes_and_the_seed_and_pulled_sets_apart() {
    let item = read_item_text(PRODUCT).expect("读得出");
    assert_eq!((item.name.as_str(), item.point.as_str(), item.script.as_str(), item.done), ("files_100_units_100__head__mount_then_first", "files_100_units_100", "mount_then_first", true));
    assert_eq!(item.remounts.len(), 1, "重挂载自己的发布读进来：重挂载阶段不暂停读数");
    let write_row = &item.remounts[0].publishes[0];
    assert_eq!((write_row.kind.as_str(), write_row.seed.clone(), write_row.pulled.clone()), ("write_row", synthetic_set(&[1]), synthetic_set(&[3])), "种子与拉进各归各");
    assert_eq!(write_row.edges, [(synthetic_node(1), synthetic_node(3))]);
    assert_eq!(write_row.graph.get(&synthetic_node(1)), Some(&vec![(0, 2437), (1, 5)]));
    let operation = &item.operations[0];
    assert_eq!((operation.epoch, operation.step_index, operation.settles[0].final_nodes, operation.publishes[0].final_set.len()), (1, 2, 3, 3));
    assert!(item.body_lines.iter().all(|line| !line.contains("name=header")), "孪生对拍去掉产物头");
    assert_eq!(item.history_carried_final, synthetic_set(&[1, 2]));
}

#[test]
fn the_closure_check_reads_the_product_publishes_and_reports_the_final_node_count() {
    let item = read_item_text(PRODUCT).expect("读得出");
    let check = closure_check(&item);
    assert_eq!((check.publishes, check.final_not_covering, check.final_nodes_mismatches, check.walk_mismatches), (2, 0, 0, 0));
    assert_eq!(check.closure_mismatches.len(), 2, "这份产物的指针图只有一个节点、第二个发布没有图：模型闭包与测到的不等，对拍红");
}

#[test]
fn the_mount_judge_runs_on_a_product_with_the_remount_publishes() {
    let items = vec![read_item_text(PRODUCT).expect("读得出")];
    let world = World::new(&items);
    assert!(mount_reading_at(&world, "files_100_units_100").is_none(), "没有 walk_forward 项：取不到");
    let views = world.views_of("files_100_units_100", "mount_then_first").expect("有视图");
    assert_eq!((views.len(), views[0].epoch, views[0].ordinal_user), (1, 1, 1));
    assert_eq!(views[0].fresh[0], 1, "宽读：{{1,2,3}} 去掉重挂载带出的 {{1,3}} 只剩 {{2}}");
    assert_eq!(views[0].fresh[2], 0, "不清零：历史里带出过 {{1,2}}、重挂载带出 {{1,3}}，一个不剩");
}

#[test]
fn the_declared_sequence_is_checked_against_the_script_table() {
    let mut item = MeasuredItem { script: "old_mount_old".to_string(), ..MeasuredItem::default() };
    let steps = [(1, "overwrite_old", 0), (2, "remount", 0), (3, "overwrite_old", 1), (4, "overwrite_first", 1)];
    item.step_lines = steps.iter().map(|(index, name, epoch)| (*index, (*name).to_string(), *epoch)).collect();
    item.operations = [(1, "overwrite_old"), (3, "overwrite_old"), (4, "overwrite_first")].iter().map(|(index, name)| Operation { step_index: *index, name: (*name).to_string(), ..Operation::default() }).collect();
    assert_eq!(declared_sequence_mismatch(&item), None);
    item.step_lines[3].1 = "overwrite_young".to_string();
    assert!(declared_sequence_mismatch(&item).is_some_and(|text| text.contains("第 4 步")), "少一步、多一步、选择子取错都报");
    item.step_lines.pop();
    assert!(declared_sequence_mismatch(&item).is_some_and(|text| text.contains("步数")));
}

#[test]
fn the_constants_and_the_history_are_checked_against_the_registered_values() {
    let mut item = read_item_text(PRODUCT).expect("读得出");
    assert!(constants_mismatches(&item).is_empty() && history_mismatch(&item).is_none());
    item.constants.insert("leaf_slots".to_string(), "811".to_string());
    item.history_user_changes = Some(221);
    assert_eq!(constants_mismatches(&item).len(), 1);
    item.arm = "plain".to_string();
    assert!(constants_mismatches(&item).is_empty(), "plain 版没有常量行：不核常量");
    item.arm = "head".to_string();
    assert!(history_mismatch(&item).is_some());
    assert!(live_units_shortfall(&item).is_some(), "没有规模读数：不当 0");
    item.first_state = fields_of("E209RESULT name=state mapped_units=10010");
    assert_eq!(live_units_shortfall(&item), None, "F × K + 10 = 10010");
}

#[test]
fn the_twin_and_the_plain_comparisons_catch_a_different_line_and_a_different_disk() {
    let original = read_item_text(PRODUCT).expect("读得出");
    let mut twin = original.clone();
    assert_eq!(twin_difference(&original, &twin), None);
    twin.body_lines[3] = "E209RESULT name=history item=x user_changes=223 settle_calls=222".to_string();
    assert_eq!(twin_difference(&original, &twin), Some(4));
    let with_images = |history_end: &str, script_end: &str| MeasuredItem { images: vec![("history_end".to_string(), history_end.to_string()), ("script_end".to_string(), script_end.to_string())], ..MeasuredItem::default() };
    assert!(plain_matches_counts(&with_images("aa", "bb"), &with_images("aa", "bb")));
    assert!(!plain_matches_counts(&with_images("aa", "bb"), &with_images("aa", "cc")));
    assert!(!plain_matches_counts(&with_images("aa", "bb"), &MeasuredItem::default()), "plain 没有读数：不当相同");
}

#[test]
fn the_row_positions_are_found_by_function_name_and_unknown_ones_stay_pending() {
    assert_eq!(ROW_FUNCTIONS.len(), 28);
    let rows: Vec<&str> = by_delta_rows(&ByDeltaInput::default()).iter().map(|(row, _)| *row).collect();
    assert_eq!(rows, ROW_FUNCTIONS.iter().map(|(row, _, _)| *row).collect::<Vec<_>>(), "28 行的名字与次序同 by_delta_rows");
    let source = "pub fn other() {}\n    pub fn inode_record_of(&self) {}\nfn twice() {}\nfn twice(x: u64) {}\n";
    assert_eq!(line_of_the_function(source, "inode_record_of"), Some(2));
    assert_eq!(line_of_the_function(source, "twice"), None, "两行都有：不认");
    assert_eq!(line_of_the_function(source, ""), None);
    assert_eq!(position_of_the_row("mapped_units_whole_table", Some("/nonexistent")), "position_pending");
    assert_eq!(position_of_the_row("no_such_row", None), "position_pending");
}

#[test]
fn the_by_delta_rows_are_pinned_on_a_constructed_input() {
    let input = ByDeltaInput { settling_rounds: 3, changed_leaves_per_round: vec![2, 3], allocation_tree_levels: 4, ..ByDeltaInput::default() };
    let rows: BTreeMap<&str, u64> = by_delta_rows(&input).into_iter().collect();
    assert_eq!(rows["allocation_record_tree_node_set"], 20, "5 片叶 × 4 层");
    assert_eq!(by_delta_total(&input), 40, "node_set 与 plan 两行各 20");
}

#[test]
fn the_registered_item_names_round_trip_and_the_segments_add_up() {
    let items = scripts::registered_items();
    let in_segment = |segment: &str| items.iter().filter(|(_, candidate)| *candidate == segment).count();
    assert_eq!((in_segment("first"), in_segment("second"), in_segment("gate"), in_segment("third"), in_segment("fourth")), (6, 20, 1, 47, 4));
    assert_eq!(list_items().len(), items.len());
}

#[test]
fn the_report_output_file_is_new_and_holds_the_same_lines() {
    let arguments: Vec<String> = ["--crates-root", "/root", "--out", "/new.out", "a.out", "b.out"].iter().map(|text| text.to_string()).collect();
    assert_eq!(split_report_arguments(&arguments), (Some("/root".to_string()), Some("/new.out".to_string()), vec!["a.out".to_string(), "b.out".to_string()]));
    let directory = std::env::temp_dir().join(format!("e209-report-out-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("report.out").to_string_lossy().to_string();
    let lines = vec!["E209RESULT name=a".to_string(), "E209RESULT name=b".to_string()];
    write_new_report_file(&path, &lines).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "E209RESULT name=a\nE209RESULT name=b\n");
    assert!(write_new_report_file(&path, &["E209RESULT name=c".to_string()]).is_err(), "同名文件已在：拒，不覆盖");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "E209RESULT name=a\nE209RESULT name=b\n", "拒了之后原文件不变");
    std::fs::remove_dir_all(&directory).unwrap();
}
