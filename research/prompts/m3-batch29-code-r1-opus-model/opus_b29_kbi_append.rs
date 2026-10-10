
// ───────────── m3-batch29-code-r1 攻方腿追加（只在攻方仓副本里，不入库） ─────────────
// 第 29 批 L1（走读不往下读的孩子每一份副本判 I-2.1）在层级 1 的三棵树上：类标签那一改的长度口径、一份读不出一份对得上、
// 一份落在从没写过的槽上且位置条目校验和恰是全零那一段的 CRC。每一形打印一行 RESULT，不断言判定（观测）。

/// 子指针第 `index` 条位置条目在 120 字节内部条目里的起点（盘 4、槽 6、整单元校验和 4）。
fn opus_b29_location_start(index: usize) -> usize {
    CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + POINTER_FIRST_LOCATION_OFFSET + index * POINTER_LOCATION_BYTES
}

fn opus_b29_set_location(entry: &mut [u8], index: usize, device: Option<u32>, slot: Option<u64>, checksum: Option<u32>) {
    let start = opus_b29_location_start(index);
    if let Some(device) = device {
        entry[start..start + 4].copy_from_slice(&device.to_le_bytes());
    }
    if let Some(slot) = slot {
        entry[start + 4..start + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
    }
    if let Some(checksum) = checksum {
        entry[start + 10..start + 14].copy_from_slice(&checksum.to_le_bytes());
    }
}

fn opus_b29_report(form: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let red: Vec<&str> = violated_invariant_set(&verdicts).into_iter().collect();
    let detail_of = |name: &str| match &verdicts.iter().find(|(identifier, _)| *identifier == name).expect("清单里有").1 {
        InvariantVerdict::Violated(detail) => detail.clone(),
        _ => String::new(),
    };
    let i21 = detail_of("I-2.1");
    let i21_site = if i21.is_empty() {
        "-"
    } else if i21.contains("走读不往下读的孩子") {
        "unwalked-child"
    } else {
        "elsewhere"
    };
    let i72 = detail_of("I-7.2");
    let i72_site = if i72.is_empty() {
        "-"
    } else if i72.contains("（走读不往下读的孩子）两份都与位置条目里的校验和对不上") {
        "unwalked-child-both"
    } else {
        "elsewhere"
    };
    let i21_kind = match &verdicts.iter().find(|(identifier, _)| *identifier == "I-2.1").expect("清单里有").1 {
        InvariantVerdict::Holds => "holds",
        InvariantVerdict::Violated(_) => "violated",
        InvariantVerdict::NotApplicable(_) => "not-applicable",
        InvariantVerdict::PartiallyJudged(_) => "partially-judged",
    };
    println!("RESULT kbi form={form} red=[{}] i2_1={i21_site} i2_1_verdict={i21_kind} i7_2={i72_site}", red.join(","));
}

/// 三棵树（层级 1）的干净镜像、树种类、容器那一类的类型段。
fn opus_b29_trees() -> Vec<(&'static str, MemoryPool, u16, u16)> {
    vec![
        ("inode", build_pool("opus-b29-inode").memory_pool(), TREE_KIND_INODE, PACKED_RECORD_TYPE_INODE),
        (
            "tomb",
            common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("opus-b29-tomb"),
            TREE_KIND_TOMBSTONE,
            PACKED_RECORD_TYPE_TOMBSTONE,
        ),
        ("lin", image_after_rolling_back_to_the_first_root("opus-b29-lin").image, TREE_KIND_LINEAGE, PACKED_RECORD_TYPE_LINEAGE),
    ]
}

/// 根第 0 条条目的孩子（码 3 容器，32 KiB，两盘同槽）的类标签改成 `tag`，不重封，新的整单元校验和沿引用链补到根槽。
fn opus_b29_change_the_child_class_tag(image: &mut MemoryPool, kind: u16, tag: u8) {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(image, kind, 0);
    let slot = common_pointer_head_corruption::the_slot_of(&child);
    let mut bytes = common_pointer_head_corruption::unit_bytes_at(image, slot, common_pointer_head_corruption::data_unit_bytes());
    assert_eq!(bytes[UNIT_HEADER_CLASS_TAG_OFFSET], 3, "孩子是码 3 容器");
    bytes[UNIT_HEADER_CLASS_TAG_OFFSET] = tag;
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(image, slot, &bytes);
}

/// 一个从没写过的槽（两块盘上 32 KiB 全零）。
fn opus_b29_untouched_slot(image: &MemoryPool) -> u64 {
    let slot = IMAGE_BYTES / SLOT - 8;
    for device in DEVICES {
        assert!(
            read(image, device, slot * SLOT, 32768).iter().all(|byte| *byte == 0),
            "槽 {slot} 在盘 {device} 上没写过"
        );
    }
    slot
}

#[test]
fn opus_b29_level_one_forms() {
    let zero_crc_16k = crc32_castagnoli(&vec![0u8; 16384]);
    let zero_crc_32k = crc32_castagnoli(&vec![0u8; 32768]);
    for (tree, clean, kind, registered) in opus_b29_trees() {
        let untouched = opus_b29_untouched_slot(&clean);
        let beyond = IMAGE_BYTES / SLOT + 100;
        opus_b29_report(&format!("{tree}-clean"), &clean);
        // 条目类型段三种：7（H-J3a 类型段不认得）、0（H-J3b 孩子是另一类：说码 2、孩子是码 3）、原样（登记的那一支，对照）
        let entry_types: [(&str, u16); 3] = [("type7", CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED), ("type0", CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE), ("typeR", registered)];
        // 孩子那一份怎么改（只改第二条位置条目，或改孩子单元本身）
        #[derive(Clone, Copy)]
        enum Change {
            Nothing,
            ChildTag(u8),
            SecondLocation { device: Option<u32>, slot: Option<u64>, checksum: Option<u32> },
        }
        let changes: Vec<(String, Change)> = vec![
            ("as-is".to_string(), Change::Nothing),
            ("tag1".to_string(), Change::ChildTag(1)),
            ("tag2".to_string(), Change::ChildTag(2)),
            ("tag9".to_string(), Change::ChildTag(9)),
            ("loc1dev7".to_string(), Change::SecondLocation { device: Some(7), slot: None, checksum: None }),
            ("loc1beyond".to_string(), Change::SecondLocation { device: None, slot: Some(beyond), checksum: None }),
            ("loc1zero16".to_string(), Change::SecondLocation { device: None, slot: Some(untouched), checksum: Some(zero_crc_16k) }),
            ("loc1zero32".to_string(), Change::SecondLocation { device: None, slot: Some(untouched), checksum: Some(zero_crc_32k) }),
            ("loc1zerowrong".to_string(), Change::SecondLocation { device: None, slot: Some(untouched), checksum: Some(zero_crc_16k ^ 1) }),
        ];
        for (change_name, change) in &changes {
            for (type_name, entry_type) in entry_types {
                if matches!(change, Change::Nothing) && type_name == "typeR" {
                    continue;
                }
                let mut image = clean.clone();
                if let Change::ChildTag(tag) = change {
                    opus_b29_change_the_child_class_tag(&mut image, kind, *tag);
                }
                let change = *change;
                change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
                    if entry_type != registered {
                        change_the_entry_type(entry, registered, entry_type);
                    }
                    if let Change::SecondLocation { device, slot, checksum } = change {
                        opus_b29_set_location(entry, 1, device, slot, checksum);
                    }
                });
                opus_b29_report(&format!("{tree}-L1-{change_name}-{type_name}"), &image);
            }
        }
    }
}

/// 层级 1 节点里不往下走孩子的第三支（墓碑树、谱系树类型段 0：孩子会是层级 0 的码 2，`walk_the_internal_node_below_a_container_tree_entry`
/// 只记走读失败）与第 29 批那两支并排：子指针两条位置条目的校验和都坏（或只坏一条），孩子本身不动。
#[test]
fn opus_b29_level_one_third_branch_forms() {
    for (tree, clean, kind, registered) in opus_b29_trees() {
        let entry_types: [(&str, u16, bool); 4] = [
            ("type7", CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED, false),
            ("type0", CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE, false),
            ("type0z", CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE, true),
            ("typeR", registered, false),
        ];
        for (sums_name, flip_first, flip_second) in [("sum01bad", true, true), ("sum0bad", true, false), ("sum1bad", false, true), ("sumok", false, false)] {
            for (type_name, entry_type, zero_identity) in entry_types {
                let mut image = clean.clone();
                change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
                    if entry_type != registered {
                        change_the_entry_type(entry, registered, entry_type);
                    }
                    if zero_identity {
                        zero_the_three_identity_fields(entry);
                    }
                    if flip_first {
                        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
                    }
                    if flip_second {
                        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
                    }
                });
                opus_b29_report(&format!("{tree}-L1T-{sums_name}-{type_name}"), &image);
            }
        }
    }
}
