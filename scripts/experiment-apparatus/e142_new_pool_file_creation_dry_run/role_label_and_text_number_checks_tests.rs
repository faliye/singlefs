use super::*;

/// M157 的会红检查：G22 的文本把三个占位符换成真给的记账数，不是写死的旧字面。
#[test]
fn render_free_bytes_gap_text_fills_in_the_current_accounting_numbers() {
    let template = GAPS.iter().find(|(gap_identifier, _)| *gap_identifier == "G22").expect("GAPS 里有 G22").1;
    let text = render_free_bytes_gap_text(template, 42, 7, 3);
    assert_eq!(extract_number_after_marker(&text, "空闲字节"), Some(42));
    assert!(!text.contains("3472670720"), "不许把旧字面留在模板替换的结果里");
}

/// PC3 之一（角色名自检，R18/R20）：合成一份产物——`name=role_labels` 把 `t9` 定义成单元，
/// 另一行把 `t9` 当 journal 记录用（`record=t9`），第三行用一个没定义过的 `t99`。
/// M158（矛盾判定恒为 false）会让第一个断言变红。
#[test]
fn role_label_self_check_catches_a_contradiction_and_an_undefined_label() {
    let dirty: Vec<String> = vec![
        "E7RESULT name=role_labels scope=main_transaction units=t1,t9 journal_record=t13 root=t14 system_configuration=t15 warm_up_journal_records=w1,w4".to_string(),
        "E7RESULT name=journal_header_flag_diff device=0 offset=1 length=1 old=00 new=01 record=t9 field=record_flags offset_in_record=7".to_string(),
        "E7RESULT name=some_other_line odd_field=t99_suffix".to_string(),
    ];
    let check = check_role_labels(&dirty);
    assert_eq!(check.contradictions, 1, "{:?}", check.contradiction_rows);
    assert_eq!(check.undefined, 1, "{:?}", check.undefined_rows);

    let clean: Vec<String> = vec![
        "E7RESULT name=role_labels scope=main_transaction units=t1,t9 journal_record=t13 root=t14 system_configuration=t15 warm_up_journal_records=w1,w4".to_string(),
        "E7RESULT name=journal_header_flag_diff device=0 offset=1 length=1 old=00 new=01 record=t13 field=record_flags offset_in_record=7".to_string(),
    ];
    let clean_check = check_role_labels(&clean);
    assert_eq!(clean_check.contradictions, 0);
    assert_eq!(clean_check.undefined, 0);
}

/// PC3 之二（文本数字自检，R19）：合成一份产物——一行有 `bound_count=6`、自由文本里嵌的却是 5
/// （绑定但对不上）；一行有一个既没绑定、也不在字面白名单里的数（`text=用了99次`）。
/// M159（把每个数都归进字面白名单）会让第二个断言变红；M157（G22 绑定退回字面）就是第一种情形的真实版本。
#[test]
fn text_number_self_check_catches_a_bound_mismatch_and_an_unclassified_number() {
    let dirty: Vec<String> = vec![
        "E7RESULT name=gap id=G22 text=空闲字节5 bound_free_bytes_per_device=6".to_string(),
        "E7RESULT name=probe_summary text=这一批一共用了99次".to_string(),
    ];
    let check = check_text_numbers(&dirty);
    assert_eq!(check.bound_mismatched, 1, "{:?}", check.mismatch_rows);
    assert_eq!(check.unclassified, 1, "{:?}", check.unclassified_rows);

    let clean: Vec<String> = vec!["E7RESULT name=gap id=G22 text=空闲字节6 bound_free_bytes_per_device=6".to_string()];
    let clean_check = check_text_numbers(&clean);
    assert_eq!(clean_check.bound_mismatched, 0);
    assert_eq!(clean_check.unclassified, 0);
}

/// Q142.29 两处点名（问题单第 8 行）：从真实产物文本（不是 Rust 变量）里现取 journal_header 与
/// write_list 两处的记录标签、G22 与记账两处的空闲字节，逐字节相同。
#[test]
fn named_instances_journal_header_and_write_list_agree_and_the_free_bytes_gap_matches_accounting() {
    let lines: Vec<String> = vec![
        "E7RESULT name=role_labels scope=main_transaction units=t1,t2 journal_record=t13 root=t14 system_configuration=t15 warm_up_journal_records=w1,w4".to_string(),
        "E7RESULT name=journal_header_record_bytes arm=F record=w1 device=0 flag_byte=0x00 ordinal_bytes=1 back_chain=0".to_string(),
        "E7RESULT name=journal_header_record_bytes arm=F record=t13 device=0 flag_byte=0x01 ordinal_bytes=1 back_chain=123".to_string(),
        "E7RESULT name=accounting entries=1 allocated_bytes_per_device=1 free_bytes_per_device=42 empty_cluster_segments_per_device=1 fragmentation_runs_per_device=1 inode_watermark=1 unreclaimable_per_device=0 defer_queue_per_device=0 pending_delete=0 committed_reservation=0 sequence_all_one=true generation=1".to_string(),
        "E7RESULT name=gap id=G22 text=空闲字节42、全空聚簇段数1、runs1三个数都随它变 bound_free_bytes_per_device=42 bound_empty_cluster_segments_per_device=1 bound_fragmentation_runs_per_device=1".to_string(),
    ];
    let journal_header_record_label = find_field_value_excluding(&lines, "journal_header_record_bytes", "record", &["w1", "w4"]).unwrap();
    let write_list_journal_record_label = find_field_value_excluding(&lines, "role_labels", "journal_record", &[]).unwrap();
    assert_eq!(journal_header_record_label, write_list_journal_record_label);
    let free_bytes_gap_text = split_fields(lines.iter().find(|line| line.contains("id=G22")).unwrap()).iter().find(|(field_key, _)| *field_key == "text").unwrap().1.to_string();
    let free_bytes_in_the_gap_text = extract_number_after_marker(&free_bytes_gap_text, "空闲字节").unwrap();
    let accounting_free_bytes: u64 = find_field_value_excluding(&lines, "accounting", "free_bytes_per_device", &[]).unwrap().parse().unwrap();
    assert_eq!(free_bytes_in_the_gap_text, accounting_free_bytes);
}

/// 宽度表（M219 的会红检查）：30 行逐行预期等于实测，记录头三行钉在 D23 已定项 4 / 15 / 17 今天的数。
#[test]
fn width_rows_are_all_equal_and_pin_the_three_record_header_numbers() {
    let rows = width_rows();
    assert_eq!(rows.len(), 30);
    for (structure, expected, actual) in rows {
        assert_eq!(expected, actual, "{structure}");
    }
    let expected_of = |name: &str| rows.iter().find(|(structure, _, _)| *structure == name).map(|(_, expected, _)| *expected);
    assert_eq!(expected_of("journal_header"), Some(487));
    assert_eq!(expected_of("journal_new_root_segment"), Some(364));
    assert_eq!(expected_of("journal_named_entries_per_record"), Some(64));
}
