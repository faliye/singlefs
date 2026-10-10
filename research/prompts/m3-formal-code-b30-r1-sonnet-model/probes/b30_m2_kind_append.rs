// ───── b30 正推腿 M2 探针（b29 正推腿 L4 探针原样，在 b30 树上再跑）：树表里两条同一种类（种类码 ≥ 32 与 < 32），池级 checker 与 core 只读恢复各说什么 ─────
#[test]
fn b30_m2_duplicate_kind_probe() {
    use singlefs_core::recovery::{recover, JournalPolicy};
    let clean = build_pool("b30-m2-dup-kind").memory_pool();
    let base_report = recover(&clean, JournalPolicy::Consult);
    println!("B30KIND clean root={:?} outcome={}", base_report.effective_root, format!("{:?}", base_report.outcome).chars().take(120).collect::<String>());
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
            "B30KIND {label} violated={:?} i916={:?} root={:?} outcome={}",
            violated_invariants(&verdicts),
            i916,
            report.effective_root,
            format!("{:?}", report.outcome).chars().take(160).collect::<String>()
        );
    }
}


