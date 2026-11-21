//! 布局字段行的锚点、小端读取、各类单元与水位线与记录与日志头的字段行、字段计数与阳性对照、行的输出。

use super::*;

// ───────────────────────── E142 第二十一次跑：R56 字段表（登记第七节 7.1 甲表 L1–L21、J1–J24，共 174 行） ─────────────────────────
//
// 每一行一个（单元，字段）：**期望值**是登记甲表写的字面量（`layout/01` 原文写了值的取原文，原文没写这一项的取
// D8 / D18 / D23 的条文），**实测值**从这条流写出的字节里按字面字段偏移解出来——偏移、槽号、设备内偏移都是登记 7.2 B5–B11 的字面量，
// 不取装置的任何格式常量与解析函数（下面只用几个小端读数的小函数）。`layout/01` 没给具体值的字段（点名项里的出生树、指针里的位置条目校验和、
// t13 的载荷校验和）不在表里，由 R58 的逐字节比对罩着。

/// 登记 7.2 B1：t13 的反向链 = CRC32C(新 w4 的 487 字节头，`header_csum` [46, 78) 按零参与)。值取自 `back_chain_487.py`
/// 在第二十一次跑步 ①(4) 带输入（N19R 产物里取的 fsid 8 字节与实例表指针 86 字节）算出的 `B1  t13.back_chain` 那一行
/// （`research/results/e142-r21-anchor-b1-2026-10-04.out`）——是锚点，不是装置的任何一次产物；单测钉它。
pub(super) const ANCHOR_BACK_CHAIN_OF_THE_TRANSACTION_JOURNAL_RECORD: u32 = 2_660_362_816;
/// 登记 7.2 B1b：w4 的反向链 = CRC32C(新 w1 的 487 字节头)，同一次运行打出的 `B1b w4.back_chain` 那一行。
pub(super) const ANCHOR_BACK_CHAIN_OF_THE_WARM_UP_RECORD_AT_COUNTER_TWO: u32 = 750_278_362;

/// 字段表的一行。`expected` 与 `actual` 都写成不含空格的文本，好让结果行按空格切段。
pub(super) struct LayoutFieldRow {
    pub(super) row_identifier: &'static str,
    pub(super) record_or_unit: String,
    pub(super) field: String,
    pub(super) layout_line: &'static str,
    pub(super) expected: String,
    pub(super) actual: String,
}

/// 字段表主表与 PC-e 注入副本共用的比较函数：期望与实测逐字相同。M209 把它改成恒判相等。
pub(super) fn layout_field_equal(expected: &str, actual: &str) -> bool {
    expected == actual
}

fn le_u16_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
}

fn le_u32_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from(u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("切了 4 字节")))
}

fn le_48_bit_unsigned_at(bytes: &[u8], offset: usize) -> u64 {
    let mut eight = [0u8; 8];
    eight[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(eight)
}

fn le_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("切了 8 字节"))
}

fn is_all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 两条位置条目的（设备，槽号）：`offsets` = [第 0 条设备, 第 0 条槽号, 第 1 条设备, 第 1 条槽号] 在 `bytes` 里的偏移（槽号 6 字节、设备 4 字节）。
fn location_pair_text(bytes: &[u8], offsets: [usize; 4]) -> String {
    format!("({},{}),({},{})", le_u32_at(bytes, offsets[0]), le_48_bit_unsigned_at(bytes, offsets[1]), le_u32_at(bytes, offsets[2]), le_48_bit_unsigned_at(bytes, offsets[3]))
}

/// 登记 7.2 B11 的落点：槽号乘 16384（设备内字节偏移）。
pub(super) fn layout_slot_offset(slot: u64) -> u64 {
    slot * 16384
}

fn layout_bytes(image: &dyn BlockReader, device: u32, offset: u64, length: usize) -> Vec<u8> {
    image.read(DeviceIdentity(device), DeviceOffset(offset), length)
}

fn push_layout_row(rows: &mut Vec<LayoutFieldRow>, row_identifier: &'static str, record_or_unit: &str, field: &str, layout_line: &'static str, expected: &str, actual: &str) {
    rows.push(LayoutFieldRow { row_identifier, record_or_unit: record_or_unit.to_string(), field: field.to_string(), layout_line, expected: expected.to_string(), actual: actual.to_string() });
}

/// 树表第 `index` 条的六个字段（树 ID、种类、根指针落点槽号或 `zero`、头 ID、诞生 txg、previous_snapshot_txg）：
/// 条目从 131 + 200 × index 起；根指针 86 字节在条目内偏移 14，第 0 条位置条目的槽号在指针内偏移 54（条目内 68）。
fn tree_table_entry_texts(tree_table_unit: &[u8], index: usize) -> [String; 6] {
    let base = 131 + 200 * index;
    let root_text = if is_all_zero(&tree_table_unit[base + 14..base + 100]) { "zero".to_string() } else { le_48_bit_unsigned_at(tree_table_unit, base + 68).to_string() };
    [
        le_u64_at(tree_table_unit, base).to_string(),
        le_u16_at(tree_table_unit, base + 10).to_string(),
        root_text,
        le_u64_at(tree_table_unit, base + 116).to_string(),
        le_u64_at(tree_table_unit, base + 108).to_string(),
        le_u64_at(tree_table_unit, base + 100).to_string(),
    ]
}


/// L1–L16：事务写的各单元里的树 ID、树表条目（设备 0 那一份，槽号 = 登记第零节写清单）。
fn push_layout_rows_for_units(image: &dyn BlockReader, rows: &mut Vec<LayoutFieldRow>) {
    let unit = |slot: u64, length: usize| layout_bytes(image, 0, layout_slot_offset(slot), length);
    let data_unit_bytes = unit(50180, 32768);
    push_layout_row(rows, "L1", "t1_data_unit", "five_tuple_tree_identifier@43", "182", "65536", &le_u64_at(&data_unit_bytes, 43).to_string());
    let extent_root_bytes = unit(50240, 16384);
    push_layout_row(rows, "L2", "t2_extent_root", "tree_identifier@42", "282", "65536", &le_u64_at(&extent_root_bytes, 42).to_string());
    // 条目区从 163 起（头 86 + 2 × 24 + 29），条目 = key 24 + 标签 1 + 数据指针 88，指针内偏移 34 是出生树。
    push_layout_row(rows, "L3", "t2_extent_root", "entry0_data_pointer_birth_tree@222", "208", "65536", &le_u64_at(&extent_root_bytes, 222).to_string());
    let inode_leaf_container_bytes = unit(50242, 32768);
    push_layout_row(rows, "L4", "t3_inode_leaf_container", "class_identity_birth_tree@43", "260", "65537", &le_u64_at(&inode_leaf_container_bytes, 43).to_string());
    let inode_root_bytes = unit(50244, 16384);
    push_layout_row(rows, "L5", "t4_inode_root", "tree_identifier@42", "256", "65537", &le_u64_at(&inode_root_bytes, 42).to_string());
    // 条目区从 131 起，条目 = 分隔 key 8 + 身份引用 26 + 子指针 86；身份引用 = 出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8。
    let identity_reference = format!("({},{},{},{})", le_u64_at(&inode_root_bytes, 139), le_u16_at(&inode_root_bytes, 147), le_u64_at(&inode_root_bytes, 149), le_u64_at(&inode_root_bytes, 157));
    push_layout_row(rows, "L6", "t4_inode_root", "entry0_identity_reference@139", "258", "(65537,2,1,3)", &identity_reference);
    for (name, slot) in [
        ("t5_allocation_leaf_of_device_0", 50245u64),
        ("t6_allocation_leaf_of_device_1", 50246),
        ("t7_allocation_internal_of_device_0", 50247),
        ("t8_allocation_internal_of_device_1", 50248),
        ("t9_allocation_root", 50249),
    ] {
        push_layout_row(rows, "L7", name, "tree_identifier@42", "298", "65538", &le_u64_at(&unit(slot, 16384), 42).to_string());
    }
    let accounting_root_bytes = unit(50250, 16384);
    push_layout_row(rows, "L8", "t10_accounting_root", "tree_identifier@42", "302", "65539", &le_u64_at(&accounting_root_bytes, 42).to_string());
    // 记账条目区从 159 起（头 86 + 2 × 22 + 29），条目 34 字节，key 22 = 统计量 2 + 树 ID 8 + 设备 4 + 代 8；第 15 条（下标 14）是 inode 号水位。
    let watermark_key = format!("({},{},{},{})", le_u16_at(&accounting_root_bytes, 159 + 14 * 34), le_u64_at(&accounting_root_bytes, 159 + 14 * 34 + 2), le_u32_at(&accounting_root_bytes, 159 + 14 * 34 + 10), le_u64_at(&accounting_root_bytes, 159 + 14 * 34 + 14));
    push_layout_row(rows, "L9", "t10_accounting_root", "entry14_key_inode_watermark@635", "272+303", "(12,65537,4294967295,3)", &watermark_key);
    let mapping_root_bytes = unit(50251, 16384);
    push_layout_row(rows, "L10", "t11_mapping_root", "tree_identifier@42", "77", "65540", &le_u64_at(&mapping_root_bytes, 42).to_string());
    // 映射条目区从 169 起（头 86 + 2 × 27 + 29），条目一律 55，key 27 = 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10（实例代号 4 + 事务号 6）。
    let code_one_key = (0..10)
        .map(|index| 169 + 55 * index)
        .find(|offset| mapping_root_bytes[*offset] == 1)
        .map_or("absent".to_string(), |offset| format!("({},{},{},({},{}))", mapping_root_bytes[offset], le_u64_at(&mapping_root_bytes, offset + 1), le_u64_at(&mapping_root_bytes, offset + 9), le_u32_at(&mapping_root_bytes, offset + 17), le_48_bit_unsigned_at(&mapping_root_bytes, offset + 21)));
    push_layout_row(rows, "L11", "t11_mapping_root", "code_one_entry_key", "228", "(1,65536,3,(1,1))", &code_one_key);
    let tree_table_first_version_bytes = unit(50252, 16384);
    push_layout_row(rows, "L12", "t12_tree_table_first_version", "tree_identifier@42", "386", "0", &le_u64_at(&tree_table_first_version_bytes, 42).to_string());
    push_layout_row(rows, "L13", "t12_tree_table_first_version", "entry_count@98", "386+49+78", "12", &le_u16_at(&tree_table_first_version_bytes, 98).to_string());
    push_layout_row(rows, "L14", "t12_tree_table_first_version", "declared_length@8", "386", "2400", &le_u16_at(&tree_table_first_version_bytes, 8).to_string());
    let field_names = ["tree_identifier", "kind", "root_slot", "head_identifier", "birth_txg", "previous_snapshot_txg"];
    let first_seven: [[&str; 6]; 7] = [
        ["65536", "1", "50240", "65537", "3", "0"],
        ["65537", "2", "50244", "65537", "3", "0"],
        ["65538", "3", "50249", "0", "3", "0"],
        ["65539", "4", "50250", "0", "3", "0"],
        ["65541", "6", "zero", "0", "3", "0"],
        ["65542", "7", "zero", "0", "3", "0"],
        ["65543", "8", "zero", "0", "3", "0"],
    ];
    // 第 8–12 条：`deleted_inodes`、墓碑树、谱系树、logged_ops 与退役树（`layout/03`「树表四条新条目」「树表退役树条目」两行）。
    let last_five: [[&str; 6]; 5] = [
        ["65544", "9", "zero", "65537", "3", "0"],
        ["65545", "10", "zero", "65537", "3", "0"],
        ["65546", "11", "zero", "0", "3", "0"],
        ["65547", "12", "zero", "0", "3", "0"],
        ["65548", "13", "zero", "65537", "3", "0"], // 退役树
    ];
    for (index, expected) in first_seven.iter().enumerate() {
        let actual = tree_table_entry_texts(&tree_table_first_version_bytes, index);
        for (field_index, field_name) in field_names.iter().enumerate() {
            push_layout_row(rows, "L15", "t12_tree_table_first_version", &format!("entry{index}_{field_name}"), "387+389-391", expected[field_index], &actual[field_index]);
        }
    }
    for (offset_index, expected) in last_five.iter().enumerate() {
        let index = 7 + offset_index;
        let actual = tree_table_entry_texts(&tree_table_first_version_bytes, index);
        for (field_index, field_name) in field_names.iter().enumerate() {
            push_layout_row(rows, "L16", "t12_tree_table_first_version", &format!("entry{index}_{field_name}"), "387", expected[field_index], &actual[field_index]);
        }
    }
}

/// L17–L21：journal 记录 t13 与 w1、w4 新根段里的水位、根记录里的水位、mkfs 与暖机根的水位、第 0 版树表单元的条目数。
/// `after_mkfs_image` 是 mkfs 刚写完那一刻的池：区域 1、2 的槽 0 之后被暖机的第 1、2 代根覆盖，第 0 代根只在那份里读得到。
fn push_layout_rows_for_watermarks(final_image: &dyn BlockReader, after_mkfs_image: &dyn BlockReader, rows: &mut Vec<LayoutFieldRow>) {
    // journal 环 16 MiB 起，记录 4096：计数器 1、2、3 落在 16777216、16781312、16785408（登记 7.2 B11）。
    let warm_up_record_at_counter_one = layout_bytes(final_image, 0, 16_777_216, 4096);
    let warm_up_record_at_counter_two = layout_bytes(final_image, 0, 16_781_312, 4096);
    let transaction_journal_record = layout_bytes(final_image, 0, 16_785_408, 4096);
    // 新根段从记录内偏移 99 起：树表指针 86、映射指针 86，树 ID 水位在 99 + 172 = 271。
    push_layout_row(rows, "L17", "t13_journal_record", "new_root_segment_tree_identifier_watermark@271", "325+353", "65549", &le_u64_at(&transaction_journal_record, 271).to_string());
    push_layout_row(rows, "L18", "w1_journal_record", "new_root_segment_tree_identifier_watermark@271", "325+353", "65536", &le_u64_at(&warm_up_record_at_counter_one, 271).to_string());
    push_layout_row(rows, "L18", "w4_journal_record", "new_root_segment_tree_identifier_watermark@271", "325+353", "65536", &le_u64_at(&warm_up_record_at_counter_two, 271).to_string());
    // 根记录 512 槽：t14 = 区域 0 的槽 1（设备 0，1 MiB + 4096），水位在记录内偏移 122（4 + 16 + 4 + 4 + 8 + 86）。
    let transaction_root_record = layout_bytes(final_image, 0, 1_052_672, 512);
    push_layout_row(rows, "L19", "t14_root_record", "tree_identifier_watermark@122", "375", "65549", &le_u64_at(&transaction_root_record, 122).to_string());
    // 三个区域的槽 0：区域 r 在 1 MiB + r × 3 MiB，区域设备 0 / 1 / 0。
    let mkfs_root_region_0 = layout_bytes(after_mkfs_image, 0, 1_048_576, 512);
    let mkfs_root_region_1 = layout_bytes(after_mkfs_image, 1, 4_194_304, 512);
    let mkfs_root_region_2 = layout_bytes(after_mkfs_image, 0, 7_340_032, 512);
    let warm_up_root_region_1 = layout_bytes(final_image, 1, 4_194_304, 512);
    let warm_up_root_region_2 = layout_bytes(final_image, 0, 7_340_032, 512);
    for (name, root) in [
        ("mkfs_root_region_0", &mkfs_root_region_0),
        ("mkfs_root_region_1", &mkfs_root_region_1),
        ("mkfs_root_region_2", &mkfs_root_region_2),
        ("warm_up_root_region_1", &warm_up_root_region_1),
        ("warm_up_root_region_2", &warm_up_root_region_2),
    ] {
        push_layout_row(rows, "L20", name, "tree_identifier_watermark@122", "375", "65536", &le_u64_at(root, 122).to_string());
    }
    // mkfs 的第 0 版树表单元（槽 50178）：条目数在码 2 头的偏移 98。
    let tree_table_genesis = layout_bytes(final_image, 0, layout_slot_offset(50178), 16384);
    push_layout_row(rows, "L21", "mkfs_tree_table_genesis", "entry_count@98", "57", "0", &le_u16_at(&tree_table_genesis, 98).to_string());
}

/// J 表一格：w1、w4、t13 三条记录各一行（设备 0 那一份），期望值按记录给；`None` 表示这一格不含那条记录（J14 只有 w1、w4）。
fn push_layout_rows_for_records(
    rows: &mut Vec<LayoutFieldRow>,
    records: &[(&'static str, Vec<u8>); 3],
    row_identifier: &'static str,
    field: &str,
    layout_line: &'static str,
    expected: [Option<&str>; 3],
    decode: &dyn Fn(&[u8]) -> String,
) {
    for ((label, bytes), expected_text) in records.iter().zip(expected) {
        if let Some(expected_text) = expected_text {
            push_layout_row(rows, row_identifier, label, field, layout_line, expected_text, &decode(bytes));
        }
    }
}

/// J1–J24：记录头（w1、w4、t13 三条，设备 0 那一份），偏移照 D23 已定项 4 / 15 的字段表（登记 7.2 B5、B6：新根段 [99, 463)，
/// fsid 463、MAC 471、头末 487）。J13 的 w4 与 t13 两格取锚点 B1b、B1（`back_chain_487.py` 的输出，不取任何产物）。
fn push_layout_rows_for_journal_headers(image: &dyn BlockReader, rows: &mut Vec<LayoutFieldRow>) {
    let records: [(&'static str, Vec<u8>); 3] = [
        ("w1_journal_record", layout_bytes(image, 0, 16_777_216, 4096)),
        ("w4_journal_record", layout_bytes(image, 0, 16_781_312, 4096)),
        ("t13_journal_record", layout_bytes(image, 0, 16_785_408, 4096)),
    ];
    fn all(text: &str) -> [Option<&str>; 3] {
        [Some(text); 3]
    }
    let zero_or_hex = |bytes: &[u8]| if is_all_zero(bytes) { "zero".to_string() } else { lowercase_hex(bytes) };
    let back_chain_of_the_warm_up_record_at_counter_two = ANCHOR_BACK_CHAIN_OF_THE_WARM_UP_RECORD_AT_COUNTER_TWO.to_string();
    let back_chain_of_the_transaction_journal_record = ANCHOR_BACK_CHAIN_OF_THE_TRANSACTION_JOURNAL_RECORD.to_string();
    // 三条记录各自已发布的根记录（设备，设备内偏移）：w1 对 w2（区域 1 槽 0）、w4 对 w5（区域 2 槽 0）、t13 对 t14（区域 0 槽 1）。
    let published_root_hex: Vec<String> = [(1u32, 4_194_304u64), (0, 7_340_032), (0, 1_052_672)]
        .iter()
        .map(|(device, offset)| lowercase_hex(&layout_bytes(image, *device, *offset, 512)[170..256]))
        .collect();
    let data_unit_fsid_hex = lowercase_hex(&layout_bytes(image, 0, layout_slot_offset(50180), 512)[83..91]);
    {
        let mut push = |row_identifier: &'static str, field: &str, layout_line: &'static str, expected: [Option<&str>; 3], decode: &dyn Fn(&[u8]) -> String| {
            push_layout_rows_for_records(rows, &records, row_identifier, field, layout_line, expected, decode);
        };
        push("J1", "magic@0", "338", all("5346534a"), &|record: &[u8]| lowercase_hex(&record[0..4]));
        push("J2", "record_type@4", "339", all("1"), &|record: &[u8]| le_u16_at(record, 4).to_string());
        push("J3", "algorithm_type@6", "340", all("0"), &|record: &[u8]| record[6].to_string());
        push("J4", "record_flags@7", "341", all("1"), &|record: &[u8]| record[7].to_string());
        push("J5", "record_length@8", "342", all("4096"), &|record: &[u8]| le_u32_at(record, 8).to_string());
        push("J6", "named_count@12", "343+61+64", [Some("0"), Some("0"), Some("12")], &|record: &[u8]| le_u32_at(record, 12).to_string());
        push("J7", "jsn_instance_counter@16", "344+61+64", [Some("(1,1)"), Some("(1,2)"), Some("(1,3)")], &|record: &[u8]| format!("({},{})", le_u32_at(record, 16), le_48_bit_unsigned_at(record, 20)));
        push("J8", "checkpoint_txg@26", "345+61+64", [Some("1"), Some("2"), Some("3")], &|record: &[u8]| le_u64_at(record, 26).to_string());
        push("J9", "nonce@34", "346", all("zero"), &|record: &[u8]| zero_or_hex(&record[34..46]));
        push("J10", "transaction@78", "348+321", [Some("0"), Some("0"), Some("1")], &|record: &[u8]| le_u64_at(record, 78).to_string());
        push("J11", "commit_marker@86", "349+321", all("1"), &|record: &[u8]| record[86].to_string());
        push("J12", "ordinal_within_publish@87", "350+322", all("1"), &|record: &[u8]| le_u32_at(record, 87).to_string());
        push("J13", "back_chain@91", "351+323", [Some("0"), Some(back_chain_of_the_warm_up_record_at_counter_two.as_str()), Some(back_chain_of_the_transaction_journal_record.as_str())], &|record: &[u8]| le_u32_at(record, 91).to_string());
        push("J14", "payload_checksum@95", "352", [Some("0"), Some("0"), None], &|record: &[u8]| le_u32_at(record, 95).to_string());
        push(
            "J15",
            "tree_table_pointer_locations@149",
            "353+325",
            [Some("(0,50178),(1,50178)"), Some("(0,50178),(1,50178)"), Some("(0,50252),(1,50252)")],
            &|record: &[u8]| location_pair_text(record, [149, 153, 163, 167]),
        );
        push(
            "J16",
            "mapping_root_pointer@185",
            "353+325+379",
            [Some("zero"), Some("zero"), Some("(0,50251),(1,50251)")],
            &|record: &[u8]| if is_all_zero(&record[185..271]) { "zero".to_string() } else { location_pair_text(record, [235, 239, 249, 253]) },
        );
        push("J17", "rollback_floor@279", "353+325", all("0"), &|record: &[u8]| le_u64_at(record, 279).to_string());
        push("J18", "instance_table_pointer_locations@337", "378", all("(0,50176),(1,50176)"), &|record: &[u8]| location_pair_text(record, [337, 341, 351, 355]));
        push(
            "J19",
            "instance_table_pointer[287,373)_vs_published_root_record[170,256)",
            "62+378",
            [Some(published_root_hex[0].as_str()), Some(published_root_hex[1].as_str()), Some(published_root_hex[2].as_str())],
            &|record: &[u8]| lowercase_hex(&record[287..373]),
        );
        push("J20", "allocation_record_tree_root_pointer@373", "380", all("zero"), &|record: &[u8]| zero_or_hex(&record[373..459]));
        push("J21", "unmount_marker@459", "371", all("0"), &|record: &[u8]| le_u32_at(record, 459).to_string());
        push("J22", "fsid[463,471)_vs_data_unit_header[83,91)", "354+326+187", all(data_unit_fsid_hex.as_str()), &|record: &[u8]| lowercase_hex(&record[463..471]));
        push("J23", "mac@471", "355+327", all("zero"), &|record: &[u8]| zero_or_hex(&record[471..487]));
    }
    // J24：t13 的 12 个点名项，第 k 项起点 487 + 56k，两条位置条目各 14 字节（设备 4 + 槽号 6 + 校验和 4），第 1 条从项内偏移 14 起；
    // 槽号依次是 t1..t12 的落点（登记第零节写清单），设备 0、1。
    let transaction_journal_record = &records[2].1;
    for (named_index, slot) in [50180u64, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250, 50251, 50252].iter().enumerate() {
        let start = 487 + 56 * named_index;
        push_layout_row(
            rows,
            "J24",
            "t13_journal_record",
            &format!("named{named_index}_locations@{start}"),
            "329+361",
            &format!("(0,{slot}),(1,{slot})"),
            &location_pair_text(transaction_journal_record, [start, start + 4, start + 14, start + 18]),
        );
    }
}

/// 字段表整张（180 行：第二十一次跑的 174 行加第二十五次跑的退役树条目 6 行）。`final_image` 是整条流写完之后的池，`after_mkfs_image` 是 mkfs 刚写完那一刻的池（L20 要第 0 代根）。
pub(super) fn layout_field_rows(final_image: &dyn BlockReader, after_mkfs_image: &dyn BlockReader) -> Vec<LayoutFieldRow> {
    let mut rows = Vec::new();
    push_layout_rows_for_units(final_image, &mut rows);
    push_layout_rows_for_watermarks(final_image, after_mkfs_image, &mut rows);
    push_layout_rows_for_journal_headers(final_image, &mut rows);
    rows
}

/// （相等行数，不等行数），经同一个比较函数。
pub(super) fn layout_field_tally(rows: &[LayoutFieldRow]) -> (usize, usize) {
    let equal = rows.iter().filter(|row| layout_field_equal(&row.expected, &row.actual)).count();
    (equal, rows.len() - equal)
}

/// PC-e（第二十一次跑登记第五节 5.2）：把实测值那一列拷一份、改四处——L19（t14 根记录水位）加 1、L16 墓碑树那一条的头 ID 改 0、
/// L13（树表条目数）改 7、J18 里 t13 那一行第一条位置条目的槽号改 0——再经同一个比较函数过一遍。
/// 返回（注入的行数，比较结果与主表相比变了的行数，变了的恰是注入的那几行吗）；主表那一份不受影响。
pub(super) fn layout_field_positive_control(rows: &[LayoutFieldRow]) -> (usize, usize, bool) {
    let mut actuals: Vec<String> = rows.iter().map(|row| row.actual.clone()).collect();
    let mut injected_rows: Vec<usize> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let injected = match (row.row_identifier, row.field.as_str(), row.record_or_unit.as_str()) {
            ("L19", _, _) => Some(row.actual.parse::<u64>().map_or("injected".to_string(), |watermark| (watermark + 1).to_string())),
            ("L16", "entry8_head_identifier", _) => Some("0".to_string()),
            ("L13", _, _) => Some("7".to_string()),
            ("J18", _, "t13_journal_record") => Some("(0,0),(1,50176)".to_string()),
            _ => None,
        };
        if let Some(injected) = injected {
            actuals[index] = injected;
            injected_rows.push(index);
        }
    }
    let changed_rows: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(index, row)| layout_field_equal(&row.expected, &row.actual) != layout_field_equal(&row.expected, &actuals[*index]))
        .map(|(index, _)| index)
        .collect();
    (injected_rows.len(), changed_rows.len(), changed_rows == injected_rows)
}

// ───────────────────────── main：发结果行 ─────────────────────────

pub(super) fn emit(emitter: &mut Emitter, body: &str) {
    let line = emitter.emit_raw(body);
    EMITTED_LINES.with(|lines| lines.borrow_mut().push(line.clone()));
    println!("{line}");
}

#[cfg(test)]
#[path = "layout_field_rows_and_anchors_tests.rs"]
mod tests;
