use super::*;

// ───────── E142 第二十二次跑第二段：第二个几何 P8G（登记 5.4 ⑥(v)，7.2 A1、A5、A6，8.2） ─────────

/// 7.2 A5 的 P8G 半边：每盘 8 589 934 592 字节 ⇒ 524 288 槽、单元区 474 112 槽、7 408 个聚簇段（命令四 `label6_anchor.py` 的输出，不取任何一次产物）。
#[test]
fn p8g_parameters_carry_eight_gibibytes_per_device_and_the_registered_slot_counts() {
    let p8g = p8g_parameters();
    assert_eq!(p8g.device_bytes, 8_589_934_592);
    assert_eq!(p8g.device_count, 2);
    assert_eq!(p8g.device_slots(), 524_288);
    assert_eq!(p8g.unit_area_slots(), 474_112);
    assert_eq!(p8g.unit_area_slots() / CLUSTER_SEGMENT_SLOTS, 7_408);
    let main_point = PoolParameters::settled_two_devices();
    assert_eq!(main_point.device_bytes, 4_294_967_296);
    assert_eq!(main_point.unit_area_slots(), UNIT_AREA_SLOTS, "主点那一份仍是 211 968 槽");
    assert_eq!(main_point.device_slots(), 262_144);
    assert!(JOURNAL_RING_BYTES * 4 <= p8g.device_bytes, "环 ≤ 盘 ÷ 4（7.2 A5 journal_ring_ok）");
}

fn rows_field(rows: &[String], row_name: &str, key: &str) -> String {
    find_field_value_excluding(rows, row_name, key, &[]).unwrap_or_else(|| panic!("{row_name} 行里没有 {key}"))
}

/// 7.2 A5 与 A1：主点空闲每盘 3 472 605 184、全空段 3310 个；P8G 空闲每盘 7 767 572 480、全空段 7 406；runs 都是 4；标签 6 都是 196 760（第 25 次登记 7.1 K6）。
/// M228（单元区槽数仍按 4 GiB）、M230（段数上界写死 3312）在 P8G 这一格红，主点上不红。
#[test]
fn accounting_row_carries_the_registered_free_bytes_and_empty_segments_at_both_geometries() {
    let file = sample_file();
    for (geometry, parameters, free_bytes, empty_segments) in [
        ("主点", PoolParameters::settled_two_devices(), "3472605184", "3310"),
        ("P8G", p8g_parameters(), "7767572480", "7406"),
    ] {
        let section = stream_section(&parameters, &file);
        assert_eq!(rows_field(&section.rows, "accounting", "free_bytes_per_device"), free_bytes, "{geometry}：空闲字节");
        assert_eq!(rows_field(&section.rows, "accounting", "empty_cluster_segments_per_device"), empty_segments, "{geometry}：全空聚簇段");
        assert_eq!(rows_field(&section.rows, "accounting", "fragmentation_runs_per_device"), "4", "{geometry}：空闲 run 数");
        assert_eq!(rows_field(&section.rows, "accounting", "committed_reservation"), "196760", "{geometry}：标签 6 不随盘大小变（7.1 K6，7.2 A-T1）");
    }
}

/// 取一条 `name=<row_name>` 行（`device_region_bytes` 或 `p8g_device_region_bytes`）里 (设备, 偏移) 那一块的字节。
fn region_bytes_at(rows: &[String], row_name: &str, device: u32, offset: u64) -> Vec<u8> {
    let row = rows
        .iter()
        .find(|row| {
            let fields = split_fields(row);
            fields.iter().any(|(key, value)| *key == "name" && *value == row_name)
                && fields.iter().any(|(key, value)| *key == "device" && *value == device.to_string())
                && fields.iter().any(|(key, value)| *key == "offset" && *value == offset.to_string())
        })
        .unwrap_or_else(|| panic!("没有 device={device} offset={offset} 的区域行"));
    let hexadecimal = split_fields(row).iter().find(|(key, _)| *key == "hexadecimal").map(|(_, value)| (*value).to_string()).expect("区域行带十六进制");
    hex_decode(&hexadecimal)
}

fn little_endian_u64_at(bytes: &[u8], start: usize) -> u64 {
    u64::from_le_bytes(bytes[start..start + 8].try_into().expect("切了 8 字节"))
}

/// 7.2 A4 的偏移（t10 在设备 0 的 823 296 000，第 3 / 4 行 value 在 [249, 257) / [283, 291)，第 10 行 [487, 495)，第 13 / 14 行 [589, 597) / [623, 631)）
/// 读 P8G 那一段打出的区域字节：盘上记的就是 P8G 的空闲与段数，标签 6 仍是 196 760（7.1 K6）。M229（P8G 一段打成主点那条流）在这里红。
#[test]
fn p8g_accounting_unit_bytes_carry_the_p8g_free_bytes_and_segment_counts_at_the_registered_offsets() {
    let (rows, _) = p8g_section_rows(&sample_file());
    let accounting_unit = region_bytes_at(&rows, "p8g_device_region_bytes", 0, 823_296_000);
    assert_eq!(little_endian_u64_at(&accounting_unit, 249), 7_767_572_480, "第 3 行 free_bytes_dev0");
    assert_eq!(little_endian_u64_at(&accounting_unit, 283), 7_767_572_480, "第 4 行 free_bytes_dev1");
    assert_eq!(little_endian_u64_at(&accounting_unit, 487), 196_760, "第 10 行已承诺预留");
    assert_eq!(little_endian_u64_at(&accounting_unit, 589), 7_406, "第 13 行 empty_cluster_segments_dev0");
    assert_eq!(little_endian_u64_at(&accounting_unit, 623), 7_406, "第 14 行 empty_cluster_segments_dev1");
    let main_point_section = stream_section(&PoolParameters::settled_two_devices(), &sample_file());
    let main_point_accounting_unit = region_bytes_at(&main_point_section.rows, "device_region_bytes", 0, 823_296_000);
    assert_eq!(little_endian_u64_at(&main_point_accounting_unit, 249), 3_472_605_184);
    assert_eq!(little_endian_u64_at(&main_point_accounting_unit, 589), 3_310);
}

/// 登记 8.2：两个几何的 29 次窗口写按（设备，偏移，长度）配得上，变的恰是 7.2 A6 那九个区域
/// （t10 记账树根的值变，t11 中央映射、t12 树表、t13 journal 记录、t14 根记录带着校验和变），t1–t9 与系统配置、暖机以外的写不变。
#[test]
fn p8g_and_main_point_windows_differ_in_exactly_the_nine_registered_regions() {
    let file = sample_file();
    let main_point_section = stream_section(&PoolParameters::settled_two_devices(), &file);
    let p8g_section = stream_section(&p8g_parameters(), &file);
    let digests = |rows: &[String]| -> BTreeMap<(u32, u64, u64), String> {
        rows.iter()
            .filter(|row| split_fields(row).iter().any(|(key, value)| *key == "name" && *value == "device_region_bytes"))
            .map(|row| {
                let fields = split_fields(row);
                let value = |wanted: &str| fields.iter().find(|(key, _)| *key == wanted).map(|(_, value)| (*value).to_string()).expect("区域行字段");
                ((value("device").parse().expect("设备"), value("offset").parse().expect("偏移"), value("length").parse().expect("长度")), value("sha256"))
            })
            .collect()
    };
    let main_point_digests = digests(&main_point_section.rows);
    let p8g_digests = digests(&p8g_section.rows);
    assert_eq!(main_point_digests.len(), 29);
    assert_eq!(p8g_digests.len(), 29);
    assert_eq!(main_point_digests.keys().collect::<Vec<_>>(), p8g_digests.keys().collect::<Vec<_>>(), "两个几何的写按（设备，偏移，长度）一一对应");
    let differing: Vec<(u32, u64, u64)> = main_point_digests.iter().filter(|(key, digest)| p8g_digests[*key] != **digest).map(|(key, _)| *key).collect();
    let registered_nine = vec![
        (0, 1_052_672, 512),
        (0, 16_785_408, 4096),
        (0, 823_296_000, 16384),
        (0, 823_312_384, 16384),
        (0, 823_328_768, 16384),
        (1, 16_785_408, 4096),
        (1, 823_296_000, 16384),
        (1, 823_312_384, 16384),
        (1, 823_328_768, 16384),
    ];
    assert_eq!(differing, registered_nine);
}

/// R69、R72：180 行字段表在 P8G 上同一份期望值全相等，甲类 12 个判决字段全真；主点那条流同一个函数跑出的也全真（与主点 `name=verdict` 里同名字段同值）。
#[test]
fn layout_field_table_is_equal_and_all_twelve_stream_verdict_fields_hold_at_both_geometries() {
    let file = sample_file();
    for (geometry, parameters) in [("主点", PoolParameters::settled_two_devices()), ("P8G", p8g_parameters())] {
        let section = stream_section(&parameters, &file);
        assert_eq!(section.rows.iter().filter(|row| row.starts_with("name=layout_field ")).count(), 180, "{geometry}：字段表行数");
        assert_eq!(rows_field(&section.rows, "layout_field_summary", "rows"), "180", "{geometry}");
        assert_eq!(rows_field(&section.rows, "layout_field_summary", "unequal"), "0", "{geometry}：字段表有不等的行");
        for (name, value) in section.verdict.as_named_fields() {
            assert!(value, "{geometry}：判决字段 {name} 不是 true");
        }
        assert_eq!(section.rows.iter().filter(|row| row.starts_with("name=device_region_bytes ")).count(), 29, "{geometry}：窗口写 29 条（7.1 K5）");
        assert!(section.rows.iter().all(|row| row.starts_with("name=")), "{geometry}：每一行都以 name= 起头");
    }
}

/// 打出去的 P8G 一段：第一行 config、最后一行 verdict，中间每行都带 `p8g_` 前缀，各类行条数是登记 V22e 点名的数。
#[test]
fn p8g_section_rows_are_prefixed_and_carry_the_registered_row_counts() {
    let (rows, verdict) = p8g_section_rows(&sample_file());
    assert!(rows[0].starts_with("name=p8g_config devices=2 "), "{}", rows[0]);
    assert!(rows[0].contains(" device_bytes=8589934592 "), "{}", rows[0]);
    assert!(rows[0].contains(" unit_area_slots=474112 "), "{}", rows[0]);
    assert!(rows.last().expect("有行").starts_with("name=p8g_verdict write_list_ok=true recover_full_ok=true "));
    assert!(rows.iter().all(|row| row.starts_with("name=p8g_")), "每一行都带前缀");
    let count = |row_name: &str| rows.iter().filter(|row| row.starts_with(&format!("name=p8g_{row_name} "))).count();
    assert_eq!(count("device_region_bytes"), 29);
    assert_eq!(count("layout_field"), 180);
    assert_eq!(count("layout_field_summary"), 1);
    assert_eq!(count("accounting"), 1);
    assert_eq!(count("verdict"), 1);
    assert!(verdict.as_named_fields().iter().all(|(_, value)| *value));
    let verdict_row = rows.last().expect("有行");
    for (name, _) in verdict.as_named_fields() {
        assert!(verdict_row.contains(&format!(" {name}=true")), "{name}");
    }
}

#[test]
fn prefix_row_name_with_p8g_renames_only_the_first_name_field() {
    assert_eq!(prefix_row_name_with_p8g("name=gap id=G22 text=name=x"), "name=p8g_gap id=G22 text=name=x");
    assert_eq!(prefix_row_name_with_p8g("name=accounting entries=15"), "name=p8g_accounting entries=15");
}

/// 对照那一格的比较函数分得出差别：主点打印的 `name=verdict` 行里一个字段翻成 false、一个字段缺失，各被点名；其余相等。
#[test]
fn printed_verdict_comparison_names_exactly_the_flipped_and_the_missing_fields() {
    let all_true = StreamVerdictFields {
        write_list_ok: true,
        recover_full_ok: true,
        positive_control_main_geometry_ok: true,
        role_label_contradictions_zero: true,
        role_labels_all_defined: true,
        text_number_mismatches_zero: true,
        text_numbers_all_classified: true,
        named_instances_ok: true,
        clause_fields_system_configuration_ok: true,
        clause_fields_root_flags_ok: true,
        clause_fields_rollback_floor_ok: true,
        clause_fields_unmount_marker_ok: true,
    };
    let full_line = format!("E7RESULT name=verdict width_mismatches=0 {} control_states_ok=true", all_true.as_named_fields().iter().map(|(name, _)| format!("{name}=true")).collect::<Vec<_>>().join(" "));
    assert_eq!(compare_stream_verdict_fields_to_printed_verdict(&full_line, &all_true), (12, vec![]));
    let flipped_line = full_line.replace("r18_g8_ok=true", "r18_g8_ok=false").replace(" named_instances_ok=true", "");
    assert_eq!(compare_stream_verdict_fields_to_printed_verdict(&flipped_line, &all_true), (10, vec!["named_instances_ok", "r18_g8_ok"]));
    let one_false = StreamVerdictFields { write_list_ok: false, ..all_true };
    assert_eq!(compare_stream_verdict_fields_to_printed_verdict(&full_line, &one_false), (11, vec!["write_list_ok"]));
}

/// 读路径按池的每盘字节数定根的格数（D8 已定项 14：⌈盘上槽数 ÷ (812 × 169)⌉）：主点 262 144 槽 ⇒ 2 格，P8G 524 288 槽 ⇒ 4 格
/// （命令四 `label6_anchor.py` 的 A5 槽数除以 137 228 向上取整；7.2 A5 只钉槽数，格数是这一步的算术）。M231 在这里红。
#[test]
fn read_path_root_cells_per_device_follow_the_device_bytes_of_the_pool() {
    let (main_point_pool, _) = mkfs(&PoolParameters::settled_two_devices());
    let (p8g_pool, _) = mkfs(&p8g_parameters());
    assert_eq!(root_cells_per_device_of_reader(&main_point_pool.pool, 2), 262_144u64.div_ceil(812 * 169));
    assert_eq!(root_cells_per_device_of_reader(&main_point_pool.pool, 2), 2);
    assert_eq!(root_cells_per_device_of_reader(&p8g_pool.pool, 2), 524_288u64.div_ceil(812 * 169));
    assert_eq!(root_cells_per_device_of_reader(&p8g_pool.pool, 2), 4);
}
