// ───── b30 正推腿 M2 探针（照 b29 正推腿 L2 探针，新旧两棵树各跑一遍）：发布 B 之后逐个把一个单元的两份副本清零，看 I-5.4 报什么（候选集里 A 的分配记录树还读得出） ─────
#[test]
fn b30_m2_i54_after_zeroing_each_unit_of_the_second_version() {
    let clean = image_after_the_overwrite("b30-m2-i54-each-unit");
    let clean_i54 = verdict(&clean, "I-5.4");
    {
        // 每条根的分配记录树节点槽（树表种类 3 那一条指着的树，从根往下认得出的节点）：下面清零哪个单元会让 I-5.4 变，要对着这张表读
        let newest = common_pointer_head_corruption::tree_table_entry_of_kind(&clean, TREE_KIND_ALLOCATION);
        let nodes: Vec<u64> = common_pointer_head_corruption::allocation_record_tree_nodes_as_far_as_readable(&clean, &newest.root)
            .iter().map(|pointer| pointer.locations[0].slot.0).collect();
        println!("B30I54 newest_version_allocation_record_tree_node_slots={nodes:?}");
    }
    println!("B30I54 clean I-5.4={:?}", clean_i54);
    for (slot, length) in UNITS_AFTER_OVERWRITE {
        let mut image = clean.clone();
        zero_both_copies_of_the_unit(&mut image, slot, length);
        let verdicts = check_pool_image(&image);
        let i54 = verdicts.iter().find(|(n, _)| *n == "I-5.4").map(|(_, v)| match v {
            InvariantVerdict::Holds => "Holds".to_string(),
            InvariantVerdict::Violated(t) => format!("Violated({})", t.chars().take(60).collect::<String>()),
            InvariantVerdict::NotApplicable(t) => format!("NotApplicable({})", t.chars().take(40).collect::<String>()),
            InvariantVerdict::PartiallyJudged(t) => format!("PartiallyJudged({})", t.chars().take(80).collect::<String>()),
        }).unwrap();
        println!("B30I54 slot={slot} len={length} violated={:?} I-5.4={i54}", violated_invariants(&verdicts));
    }
}
