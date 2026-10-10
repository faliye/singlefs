// ───── b29 正推腿 L4 探针：树表里两条同一种类（种类码 ≥ 32 与 < 32），池级 checker 与 core 只读恢复各说什么 ─────
#[test]
fn b29_l4_duplicate_kind_probe() {
    use singlefs_core::recovery::{recover, JournalPolicy};
    let clean = build_pool("b29-l4-dup-kind").memory_pool();
    let base_report = recover(&clean, JournalPolicy::Consult);
    println!("L4PROBE clean root={:?} outcome={}", base_report.effective_root, format!("{:?}", base_report.outcome).chars().take(120).collect::<String>());
    for (label, kind) in [("kind7", 7u16), ("kind13", 13u16), ("kind31", 31u16), ("kind32", 32u16), ("kind40", 40u16)] {
        let mut image = clean.clone();
        mutate_unit(
            &mut image,
            TREE_TABLE,
            |bytes| {
                set_the_kind_of_tree_table_entry(bytes, DEADLIST_TREE_TABLE_ENTRY_INDEX, TREE_KIND_DEADLIST, kind);
                set_the_kind_of_tree_table_entry(bytes, LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX, TREE_KIND_LOGGED_OPERATIONS, kind);
            },
            true,
        );
        let verdicts = check_pool_image(&image);
        let i916 = verdicts.iter().find(|(name, _)| *name == "I-9.16").map(|(_, v)| format!("{v:?}").chars().take(150).collect::<String>());
        let report = recover(&image, JournalPolicy::Consult);
        println!(
            "L4PROBE {label} violated={:?} i916={:?} root={:?} outcome={}",
            violated_invariants(&verdicts),
            i916,
            report.effective_root,
            format!("{:?}", report.outcome).chars().take(160).collect::<String>()
        );
    }
}


// ───── b29 正推腿 L2 探针：发布 B 之后逐个把一个单元的两份副本清零，看 I-5.4 报什么（候选集里 A 的分配记录树还读得出） ─────
#[test]
fn b29_l2_i54_after_zeroing_each_unit_of_the_second_version() {
    let clean = image_after_the_overwrite("b29-l2-i54-each-unit");
    let clean_i54 = verdict(&clean, "I-5.4");
    println!("L2PROBE clean I-5.4={:?}", clean_i54);
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
        println!("L2PROBE slot={slot} len={length} violated={:?} I-5.4={i54}", violated_invariants(&verdicts));
    }
}
