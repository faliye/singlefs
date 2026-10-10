
// ───────────── m3-batch29-code-r1 攻方腿追加（只在攻方仓副本里，不入库） ─────────────
// 第 29 批 L1 在层级 2 的墓碑树上（旋钮压小：一片 1 条、一个节点 2 条，unlink 2、3、4 之后根是层级 2、第 0 条条目类型段 0 指着层级 1 的码 2 节点）。
// 每一形打印一行 RESULT，不断言判定（观测）。

const OPUS_B29_CHILD_POINTER_OFFSET: usize = 34;
const OPUS_B29_FIRST_LOCATION_OFFSET: usize = 50;
const OPUS_B29_LOCATION_BYTES: usize = 14;
const OPUS_B29_ENTRY_BYTES: usize = 120;
const OPUS_B29_ENTRY_TYPE_OFFSET: usize = 16;

fn opus_b29_level_two_image() -> singlefs_harness::memory_pool::MemoryPool {
    let mut pool = pool_with_new_inodes("opus-b29-tombstone-level-two", 5);
    let capacities = capacities_of_one_record_and_two_entries();
    unlink(&mut pool, 2, capacities);
    unlink(&mut pool, 3, capacities);
    unlink(&mut pool, 4, capacities);
    assert_eq!(pool.output.tombstone_tree_index_nodes.shape.root_level(), 2);
    pool.memory_pool()
}

fn opus_b29_report_given_the_knob(form: &str, image: &singlefs_harness::memory_pool::MemoryPool) {
    let verdicts = check_pool_image_given_the_tombstone_container_capacity_knob(
        image,
        Some(tombstone_container_capacity_knob(TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB)),
    );
    let mut red: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    red.sort_unstable();
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
    println!("RESULT ctg form={form} red=[{}] i2_1={i21_site} i2_1_verdict={i21_kind} i7_2={i72_site}", red.join(","));
}

fn opus_b29_root_slot(image: &singlefs_harness::memory_pool::MemoryPool) -> singlefs_core::address::SlotNumber {
    let root = common_pointer_head_corruption::tree_table_entry_of_kind(image, singlefs_core::records::TREE_KIND_TOMBSTONE).root;
    common_pointer_head_corruption::the_slot_of(&root)
}

fn opus_b29_child_slot_of_entry_zero(image: &singlefs_harness::memory_pool::MemoryPool) -> singlefs_core::address::SlotNumber {
    let root = common_pointer_head_corruption::unit_bytes_at(image, opus_b29_root_slot(image), common_pointer_head_corruption::node_bytes());
    assert_eq!(root[50], 2, "墓碑树根是层级 2");
    let start = 115 + 2 * usize::from(root[51]);
    let entry = &root[start..start + OPUS_B29_ENTRY_BYTES];
    assert_eq!(u16::from_le_bytes([entry[OPUS_B29_ENTRY_TYPE_OFFSET], entry[OPUS_B29_ENTRY_TYPE_OFFSET + 1]]), 0, "第 0 条条目类型段 0");
    let location = OPUS_B29_CHILD_POINTER_OFFSET + OPUS_B29_FIRST_LOCATION_OFFSET;
    let mut slot = [0u8; 8];
    slot[..6].copy_from_slice(&entry[location + 4..location + 10]);
    singlefs_core::address::SlotNumber(u64::from_le_bytes(slot))
}

fn opus_b29_change_entry_zero_of_the_root(image: &mut singlefs_harness::memory_pool::MemoryPool, change: impl FnOnce(&mut [u8])) {
    let root_slot = opus_b29_root_slot(image);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        common_pointer_head_corruption::node_bytes(),
        |root| {
            let start = 115 + 2 * usize::from(root[51]);
            change(&mut root[start..start + OPUS_B29_ENTRY_BYTES]);
        },
    );
}

#[test]
fn opus_b29_level_two_tombstone_forms() {
    let clean = opus_b29_level_two_image();
    let zero_crc_16k = singlefs_core::checksum::crc32_castagnoli(&vec![0u8; 16384]);
    let untouched = singlefs_core::address::SlotNumber((4u64 << 30) / 16384 - 8);
    for device in [0u32, 1] {
        let bytes = clean
            .devices
            .get(&singlefs_core::address::DeviceIdentity(device))
            .expect("盘")
            .read(untouched.to_device_offset(), 32768);
        assert!(bytes.iter().all(|byte| *byte == 0), "槽没写过");
    }
    opus_b29_report_given_the_knob("tomb-L2-clean", &clean);
    #[derive(Clone, Copy)]
    enum Change {
        Nothing,
        ChildTag(u8),
        SecondLocation { device: Option<u32>, slot: Option<u64>, checksum: Option<u32> },
        FlipSums { first: bool, second: bool },
    }
    let changes: Vec<(&str, Change)> = vec![
        ("as-is", Change::Nothing),
        ("tag1", Change::ChildTag(1)),
        ("tag3", Change::ChildTag(3)),
        ("tag9", Change::ChildTag(9)),
        ("loc1dev7", Change::SecondLocation { device: Some(7), slot: None, checksum: None }),
        ("loc1zero16", Change::SecondLocation { device: None, slot: Some(untouched.0), checksum: Some(zero_crc_16k) }),
        ("sum01bad", Change::FlipSums { first: true, second: true }),
        ("sum0bad", Change::FlipSums { first: true, second: false }),
    ];
    // 类型段：7（不认得）、1（说孩子是码 3 容器、孩子是码 2：孩子是另一类）、0（原样，登记的那一支，对照）
    for (change_name, change) in changes {
        for (type_name, entry_type) in [("type7", 7u16), ("type1", 1u16), ("type0", 0u16)] {
            if matches!(change, Change::Nothing) && type_name == "type0" {
                continue;
            }
            let mut image = clean.clone();
            if let Change::ChildTag(tag) = change {
                let child_slot = opus_b29_child_slot_of_entry_zero(&image);
                let mut bytes = common_pointer_head_corruption::unit_bytes_at(&image, child_slot, common_pointer_head_corruption::node_bytes());
                assert_eq!(bytes[6], 2, "孩子是码 2 节点");
                bytes[6] = tag;
                common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(&mut image, child_slot, &bytes);
            }
            opus_b29_change_entry_zero_of_the_root(&mut image, |entry| {
                entry[OPUS_B29_ENTRY_TYPE_OFFSET..OPUS_B29_ENTRY_TYPE_OFFSET + 2].copy_from_slice(&entry_type.to_le_bytes());
                let location = |index: usize| OPUS_B29_CHILD_POINTER_OFFSET + OPUS_B29_FIRST_LOCATION_OFFSET + index * OPUS_B29_LOCATION_BYTES;
                match change {
                    Change::SecondLocation { device, slot, checksum } => {
                        let start = location(1);
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
                    Change::FlipSums { first, second } => {
                        if first {
                            entry[location(0) + 10] ^= 0xff;
                        }
                        if second {
                            entry[location(1) + 10] ^= 0xff;
                        }
                    }
                    Change::Nothing | Change::ChildTag(_) => {}
                }
            });
            opus_b29_report_given_the_knob(&format!("tomb-L2-{change_name}-{type_name}"), &image);
        }
    }
}
