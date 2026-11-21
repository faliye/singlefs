use super::*;

/// R56 字段表：180 行（L1–L21 共 100 行、J1–J24 共 80 行，各号的行数逐个钉；L16 含第二十五次跑加的退役树一条 6 行），主几何的字节与登记甲表的期望值逐行相等；
/// 每一行的期望值是登记里的字面量，不取装置的常量（M204–M207、M210–M213 的会红检查也落在这里）。
#[test]
fn layout_field_table_has_174_rows_and_every_row_equals_the_registered_value() {
    let BuiltPool { recording, .. } = built_pool();
    let rows = layout_field_rows(&recording.pool, &after_mkfs_pool());
    let mut rows_per_identifier: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &rows {
        *rows_per_identifier.entry(row.row_identifier).or_default() += 1;
    }
    let expected_rows_per_identifier: BTreeMap<&str, usize> = [
        ("L1", 1), ("L2", 1), ("L3", 1), ("L4", 1), ("L5", 1), ("L6", 1), ("L7", 5), ("L8", 1), ("L9", 1), ("L10", 1), ("L11", 1),
        ("L12", 1), ("L13", 1), ("L14", 1), ("L15", 42), ("L16", 30), ("L17", 1), ("L18", 2), ("L19", 1), ("L20", 5), ("L21", 1),
        ("J1", 3), ("J2", 3), ("J3", 3), ("J4", 3), ("J5", 3), ("J6", 3), ("J7", 3), ("J8", 3), ("J9", 3), ("J10", 3), ("J11", 3),
        ("J12", 3), ("J13", 3), ("J14", 2), ("J15", 3), ("J16", 3), ("J17", 3), ("J18", 3), ("J19", 3), ("J20", 3), ("J21", 3),
        ("J22", 3), ("J23", 3), ("J24", 12),
    ]
    .into_iter()
    .collect();
    assert_eq!(rows_per_identifier, expected_rows_per_identifier);
    assert_eq!(rows.len(), 174 + 6, "7.2 A-T5：退役树那一条多 6 行");
    assert_eq!(rows.iter().filter(|row| row.row_identifier.starts_with('L')).count(), 100);
    assert_eq!(rows.iter().filter(|row| row.row_identifier.starts_with('J')).count(), 80);
    let unequal: Vec<String> = rows
        .iter()
        .filter(|row| !layout_field_equal(&row.expected, &row.actual))
        .map(|row| format!("{} {} {} expected={} actual={}", row.row_identifier, row.record_or_unit, row.field, row.expected, row.actual))
        .collect();
    assert!(unequal.is_empty(), "字段表不等的行：{unequal:#?}");
    assert_eq!(layout_field_tally(&rows), (180, 0));
    // 几行抽查实测值（不经比较函数）：树表条目数、声明长度、水位、退役树那一条、两条反向链。
    let actual_of = |row_identifier: &str, record_or_unit: &str, field: &str| rows.iter().find(|row| row.row_identifier == row_identifier && row.record_or_unit == record_or_unit && row.field == field).map(|row| row.actual.clone());
    assert_eq!(actual_of("L13", "t12_tree_table_first_version", "entry_count@98").as_deref(), Some("12"));
    assert_eq!(actual_of("L14", "t12_tree_table_first_version", "declared_length@8").as_deref(), Some("2400"));
    assert_eq!(actual_of("L17", "t13_journal_record", "new_root_segment_tree_identifier_watermark@271").as_deref(), Some("65549"));
    assert_eq!(actual_of("L19", "t14_root_record", "tree_identifier_watermark@122").as_deref(), Some("65549"));
    let retirement_entry_actuals: Vec<Option<String>> = ["tree_identifier", "kind", "root_slot", "head_identifier", "birth_txg", "previous_snapshot_txg"]
        .iter()
        .map(|field| actual_of("L16", "t12_tree_table_first_version", &format!("entry11_{field}")))
        .collect();
    let retirement_entry_expected: Vec<Option<String>> = ["65548", "13", "zero", "65537", "3", "0"].iter().map(|value| Some((*value).to_string())).collect(); // 退役树（7.1 K2）
    assert_eq!(retirement_entry_actuals, retirement_entry_expected, "L16 第 11 条：退役树");
    assert_eq!(actual_of("J13", "t13_journal_record", "back_chain@91").as_deref(), Some("2660362816"));
    assert_eq!(actual_of("J13", "w4_journal_record", "back_chain@91").as_deref(), Some("750278362"));
}

/// 字段表的行名过得了产物自检：角色名没有未定义的、没有矛盾的（`record_or_unit=` 里的 t1..t13 都在 `name=role_labels` 的定义里）。
#[test]
fn layout_field_lines_pass_the_role_label_self_check() {
    let BuiltPool { recording, .. } = built_pool();
    let rows = layout_field_rows(&recording.pool, &after_mkfs_pool());
    let mut lines: Vec<String> = vec![
        "E7RESULT name=role_labels scope=main_transaction units=t1,t2,t3,t4,t5,t6,t7,t8,t9,t10,t11,t12 journal_record=t13 root=t14 system_configuration=t15 warm_up_journal_records=w1,w4".to_string(),
    ];
    for row in &rows {
        lines.push(format!(
            "E7RESULT name=layout_field id={} record_or_unit={} field={} expected={} actual={} equal=true layout_line={}",
            row.row_identifier, row.record_or_unit, row.field, row.expected, row.actual, row.layout_line
        ));
    }
    let check = check_role_labels(&lines);
    assert_eq!(check.undefined, 0, "{:?}", check.undefined_rows);
    assert_eq!(check.contradictions, 0, "{:?}", check.contradiction_rows);
    assert_eq!(check_text_numbers(&lines).unclassified, 0, "字段表的行里没有自由文本");
}

/// PC-e（M209 的会红检查）：注入四处，比较结果与主表相比恰变四行；比较函数恒判相等就是 0 行（`detected=0`、`exact=false`）。
#[test]
fn layout_field_positive_control_detects_exactly_the_four_injected_rows() {
    let BuiltPool { recording, .. } = built_pool();
    let rows = layout_field_rows(&recording.pool, &after_mkfs_pool());
    assert_eq!(layout_field_positive_control(&rows), (4, 4, true));
    // 主表那一份不受影响：注入只动副本。
    assert_eq!(layout_field_tally(&rows), (180, 0));
}

/// 字段表分得出一个字节：把 t14 根记录里的水位低字节取反（0x0d → 0xf2，65549 → 65778，7.2 A-T4），恰有 L19 一行变不等。
#[test]
fn layout_field_table_notices_one_flipped_watermark_byte() {
    let BuiltPool { recording, .. } = built_pool();
    let mut damaged = recording.pool.clone();
    flip_byte(&mut damaged, DeviceIdentity(0), DeviceOffset(1_052_672), 122);
    let rows = layout_field_rows(&damaged, &after_mkfs_pool());
    let unequal: Vec<(&str, String)> = rows.iter().filter(|row| !layout_field_equal(&row.expected, &row.actual)).map(|row| (row.row_identifier, row.actual.clone())).collect();
    assert_eq!(unequal, vec![("L19", "65778".to_string())]);
}
