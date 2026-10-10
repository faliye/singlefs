
// ─── 正推腿（Sonnet）m3-batch31-code-r1 的探针：追加在 checker_known_bad_images.rs 副本末尾，只读 CPU 池级 checker 的判定、不断言 ───
// 每行 PROBE 开头；rerun.sh 的 compare 只比这些行。

fn probe_line(label: &str, image: &MemoryPool) {
    // 违例说明是「；」接起来的一串。N1 的调用只在说明里留「走读不往下读的孩子」那几句，数它们，别的 I-2.1（映射条目那一处，探针改了孩子槽带出来的）不算
    let verdicts = check_pool_image(image);
    let red: Vec<&'static str> = violated_invariant_set(&verdicts).into_iter().collect();
    let items_with = |invariant: &str, needle: &str| -> usize {
        verdicts
            .iter()
            .find(|(identifier, _)| *identifier == invariant)
            .map(|(_, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => {
                    detail.split('；').filter(|item| item.contains(needle)).count()
                }
                _ => 0,
            })
            .unwrap_or(0)
    };
    let child_i21 = items_with("I-2.1", "走读不往下读的孩子");
    let child_walk_failure = items_with("I-7.2", "（走读不往下读的孩子）两份都与位置条目里的校验和对不上");
    let level_failure = items_with("I-7.2", "层级 1 的节点有类型段 0 的条目");
    let i72_first_item = verdicts
        .iter()
        .find(|(identifier, _)| *identifier == "I-7.2")
        .map(|(_, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => detail.split('；').next().unwrap_or("").chars().take(70).collect::<String>(),
            _ => String::from("-"),
        })
        .unwrap_or_default();
    let i21_first_item = verdicts
        .iter()
        .find(|(identifier, _)| *identifier == "I-2.1")
        .map(|(_, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => detail.split('；').next().unwrap_or("").chars().take(20).collect::<String>(),
            _ => String::from("-"),
        })
        .unwrap_or_default();
    println!(
        "PROBE {label} red={red:?} i2_1_items_of_the_child_the_walk_does_not_go_into={child_i21} i2_1_first_item={i21_first_item} i7_2_items_both_copies_wrong={child_walk_failure} i7_2_items_level_one_type_zero={level_failure} i7_2_first_item={i72_first_item}"
    );
}

fn zero_the_identity_and_make_the_type_zero(entry: &mut [u8], registered_type: u16) {
    change_the_entry_type(entry, registered_type, CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE);
    zero_the_three_identity_fields(entry);
}

/// 孩子两份都换成这棵树根的码 2 字节（两条位置条目的整单元校验和都改成它的 CRC），`unreadable_location` 给了就把那一条位置条目的盘改成不在池里的 7。
fn image_whose_type_zero_entry_names_two_copies_holding_a_node(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    unreadable_location: Option<usize>,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let root_bytes = root_node_bytes_of_the_tree_of_kind(clean, kind);
    let checksum = crc32_castagnoli(&root_bytes);
    let mut image = clean.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        zero_the_identity_and_make_the_type_zero(entry, registered_type);
        set_the_whole_unit_checksum_in_a_child_location(entry, 0, checksum);
        set_the_whole_unit_checksum_in_a_child_location(entry, 1, checksum);
        if let Some(location) = unreadable_location {
            set_the_device_in_a_child_location(entry, location, DEVICE_IDENTITY_NOT_IN_THE_POOL);
        }
    });
    assert_eq!(level, 1, "根层级 1");
    for location_index in 0..2 {
        write(
            &mut image,
            child.locations[location_index].device.0,
            common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
            &root_bytes,
        );
    }
    image
}

#[test]
fn probe_of_the_sonnet_leg_for_batch31() {
    let tomb = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "probe-b31-tombstone",
    );
    let lineage = image_after_rolling_back_to_the_first_root("probe-b31-lineage").image;
    for (name, clean, kind, registered_type) in [
        ("tomb", &tomb, TREE_KIND_TOMBSTONE, PACKED_RECORD_TYPE_TOMBSTONE),
        ("lineage", &lineage, TREE_KIND_LINEAGE, PACKED_RECORD_TYPE_LINEAGE),
    ] {
        // A：F-W 型，类型段 0，一份自证成码 2、另一份是码 3 容器（码 2 那份在位置条目 0 / 1）
        for location_index in 0..2 {
            let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
                clean,
                kind,
                location_index,
                &|entry| zero_the_identity_and_make_the_type_zero(entry, registered_type),
            );
            probe_line(&format!("A-{name}-type0-fw-node-copy-at-location-{location_index}"), &image);
        }
        // D：两份都是码 2 且对得上（孩子读得出）
        let image = image_whose_type_zero_entry_names_two_copies_holding_a_node(clean, kind, registered_type, None);
        probe_line(&format!("D-{name}-type0-both-copies-valid-node"), &image);
        // E：一份是码 2 对得上、另一份在不在池里的盘上
        let image = image_whose_type_zero_entry_names_two_copies_holding_a_node(clean, kind, registered_type, Some(1));
        probe_line(&format!("E-{name}-type0-one-valid-node-one-unreadable"), &image);
        // C：对照，没改的干净镜像
        probe_line(&format!("C-{name}-clean"), clean);
    }
    // G：墓碑树根的层级字节 1 → 0（根下直接是类型段 0 的条目），类型段 0、身份清零、两条位置条目校验和都坏：N1 的条件写的是 level <= 1，层级 0 也走；
    // G1 是同一份改动不动层级字节（就是 harness 里 F-L0 那条坏镜像），拿来对照
    for (suffix, level_byte) in [("level-zero", Some(0u8)), ("level-one", None)] {
        let mut image = tomb.clone();
        let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
        let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            &mut image,
            root_slot,
            common_pointer_head_corruption::node_bytes(),
            |root| {
                if let Some(level) = level_byte {
                    root[INDEX_NODE_LEVEL_OFFSET] = level;
                }
                let start = 115 + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET]);
                let entry = &mut root[start..start + INTERNAL_ENTRY_BYTES];
                zero_the_identity_and_make_the_type_zero(entry, PACKED_RECORD_TYPE_TOMBSTONE);
                flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
                flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
            },
        );
        probe_line(&format!("G-tomb-root-{suffix}-type0-two-location-checksums-wrong"), &image);
        let verdicts = check_pool_image(&image);
        for invariant in ["I-2.1", "I-4.8"] {
            if let Some((_, InvariantVerdict::Violated(detail))) = verdicts.iter().find(|(identifier, _)| *identifier == invariant) {
                println!("PROBE G-{suffix}-detail-of-{invariant} {}", detail.chars().take(300).collect::<String>());
            }
        }
    }
    // B：inode 树类型段 0 的 F-W（不走 N1 那一支）对照
    let inode = build_pool("probe-b31-inode").memory_pool();
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            &inode,
            TREE_KIND_INODE,
            location_index,
            &|entry| {
                change_the_entry_type(
                    entry,
                    PACKED_RECORD_TYPE_INODE,
                    CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
                );
                zero_the_three_identity_fields(entry);
            },
        );
        probe_line(&format!("B-inode-type0-fw-node-copy-at-location-{location_index}"), &image);
    }
}
