// ─── 正推腿（Sonnet）m3-batch32-code-r1 的探针：追加在 checker_known_bad_images.rs 副本末尾，只读 CPU 池级 checker 的判定、不断言 ───
// 每行 PROBE 开头；rerun.sh 的 compare 只比这些行。

fn p32_probe_line(label: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let red: Vec<&'static str> = violated_invariant_set(&verdicts).into_iter().collect();
    println!("PROBE {label} red={red:?}");
}

/// 一份副本的六种状态（P2 的两两扫）：S3 自证成码 3（原样，32 KiB 校验和）、S2 自证成码 2（盘上那一槽头 16 KiB 写成这棵树根的字节、校验和 16 KiB 的 CRC）、
/// C16 头里是码 3 而校验和只对得上前 16 KiB、BAD 校验和翻位、UNREAD 盘身份不在池里、ZERO 槽指向没写过的 16 KiB 而校验和是全零 16 KiB 的 CRC。
#[derive(Clone, Copy, Debug)]
enum P32CopyState {
    S3,
    S2,
    C16,
    Bad,
    Unread,
    Zero,
}

const P32_ALL_STATES: [(P32CopyState, &str); 6] = [
    (P32CopyState::S3, "S3"),
    (P32CopyState::S2, "S2"),
    (P32CopyState::C16, "C16"),
    (P32CopyState::Bad, "BAD"),
    (P32CopyState::Unread, "UNREAD"),
    (P32CopyState::Zero, "ZERO"),
];

const P32_SLOT_NOTHING_WROTE: u64 = 190_000;

fn p32_image_with_the_two_copies_in(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    states: [P32CopyState; 2],
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let child_slot = common_pointer_head_corruption::the_slot_of(&child);
    let container = common_pointer_head_corruption::unit_bytes_at(
        clean,
        child_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let node = root_node_bytes_of_the_tree_of_kind(clean, kind);
    let crc_container = crc32_castagnoli(&container);
    let crc_first_sixteen = crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
    let crc_node = crc32_castagnoli(&node);
    let crc_zero_sixteen = crc32_castagnoli(&vec![0u8; SLOT as usize]);
    let mut image = clean.clone();
    for (index, state) in states.iter().enumerate() {
        if let P32CopyState::S2 = state {
            write(&mut image, child.locations[index].device.0, child_slot.0 * SLOT, &node);
        }
    }
    change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry_type(entry, registered_type, CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE);
        zero_the_three_identity_fields(entry);
        for (index, state) in states.iter().enumerate() {
            match state {
                P32CopyState::S3 => set_the_whole_unit_checksum_in_a_child_location(entry, index, crc_container),
                P32CopyState::S2 => set_the_whole_unit_checksum_in_a_child_location(entry, index, crc_node),
                P32CopyState::C16 => set_the_whole_unit_checksum_in_a_child_location(entry, index, crc_first_sixteen),
                P32CopyState::Bad => set_the_whole_unit_checksum_in_a_child_location(entry, index, crc_container ^ 0x5a5a_5a5a),
                P32CopyState::Unread => set_the_device_in_a_child_location(entry, index, DEVICE_IDENTITY_NOT_IN_THE_POOL),
                P32CopyState::Zero => {
                    set_the_slot_in_a_child_location(entry, index, P32_SLOT_NOTHING_WROTE);
                    set_the_whole_unit_checksum_in_a_child_location(entry, index, crc_zero_sixteen);
                }
            }
        }
    });
    image
}

fn p32_matrix_of_the_tree(label: &str, clean: &MemoryPool, kind: u16, registered_type: u16, identity_invariant: &str) {
    for device in 0..2 {
        assert!(
            read(clean, device, P32_SLOT_NOTHING_WROTE * SLOT, 2 * SLOT as usize).iter().all(|byte| *byte == 0),
            "槽 {P32_SLOT_NOTHING_WROTE} 起 32 KiB 在盘 {device} 上没写过"
        );
    }
    for (first, first_name) in P32_ALL_STATES {
        for (second, second_name) in P32_ALL_STATES {
            let image = p32_image_with_the_two_copies_in(clean, kind, registered_type, [first, second]);
            let verdicts = check_pool_image(&image);
            let red: Vec<&'static str> = violated_invariant_set(&verdicts).into_iter().collect();
            let entry_red = red.contains(&identity_invariant);
            println!("PROBE p2 {label} loc0={first_name} loc1={second_name} identity_red={entry_red} red={red:?}");
        }
    }
}

#[test]
fn probe_of_the_sonnet_leg_for_batch32_tomb() {
    p32_matrix_of_the_tree(
        "tomb",
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("p32-probe-tomb-matrix"),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

#[test]
fn probe_of_the_sonnet_leg_for_batch32_lineage() {
    p32_matrix_of_the_tree(
        "lineage",
        &image_after_rolling_back_to_the_first_root("p32-probe-lineage-matrix").image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

#[test]
fn probe_of_the_sonnet_leg_for_batch32_zero_slot() {
    // P4：类型段 7 的墓碑树根第一条条目、第 1 条位置条目的槽（盘 1）写成各个值、校验和是全零 16 KiB 的 CRC。CPU 读的是 4 GiB 的盘：
    // 槽 190000 在盘内没写过（读出全零、I-2.1 成立）；槽 ≥ 2^18 越过盘尾读不到；槽 2^32 + 190000、2^33 + 190000、2^36 + 190000 在 CPU 里都读不到
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("p32-probe-zero-slot");
    let crc_zero_sixteen = crc32_castagnoli(&vec![0u8; SLOT as usize]);
    for (label, slot) in [
        ("slot190000", 190_000u64),
        ("slot262143-last-slot-of-the-disk", 262_143),
        ("slot262144-one-past-the-disk", 262_144),
        ("slot2p32+190000", (1u64 << 32) + 190_000),
        ("slot2p33+190000", (1u64 << 33) + 190_000),
        ("slot2p36+190000", (1u64 << 36) + 190_000),
    ] {
        let mut image = clean.clone();
        change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
            change_the_entry_type(entry, PACKED_RECORD_TYPE_TOMBSTONE, 7);
            set_the_slot_in_a_child_location(entry, 1, slot);
            set_the_whole_unit_checksum_in_a_child_location(entry, 1, crc_zero_sixteen);
        });
        p32_probe_line(&format!("p4 zero-slot-crc {label}"), &image);
    }
}

/// P1：F-KEY 那一族三形（谱系 / 墓碑 / inode 叶）两种盘序（先走过的节点在位置条目 0 / 1）各一行：红的那一批与 I-4.9 的说明开头。
/// 补丁 F1（extra_mutations.tsv 第 9、10 行）在场时拿它看谱系 loc0 那一形还多不多红 I-4.9。
#[test]
fn probe_of_the_sonnet_leg_for_batch32_i49() {
    let lineage = image_after_rolling_back_to_the_first_root("p32-probe-i49-lineage").image;
    let tomb = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("p32-probe-i49-tomb");
    let inode = build_pool("p32-probe-i49-inode").memory_pool();
    for (label, clean, kind, record_bytes, walked_before) in [
        ("lineage", &lineage, TREE_KIND_LINEAGE, singlefs_format::LINEAGE_RECORD_BYTES, TREE_KIND_INODE),
        ("tomb", &tomb, TREE_KIND_TOMBSTONE, singlefs_format::TOMBSTONE_RECORD_BYTES, TREE_KIND_INODE),
        ("inode", &inode, TREE_KIND_INODE, singlefs_format::INODE_RECORD_BYTES, TREE_KIND_EXTENT),
    ] {
        for location_index in 0..2 {
            let image = image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width(
                clean, kind, record_bytes, walked_before, location_index,
            );
            let verdicts = check_pool_image(&image);
            let red: Vec<&'static str> = violated_invariant_set(&verdicts).into_iter().collect();
            let i49 = verdicts
                .iter()
                .find(|(identifier, _)| *identifier == "I-4.9")
                .map(|(_, verdict)| match verdict {
                    InvariantVerdict::Violated(detail) => format!("Violated:{}", detail.chars().take(60).collect::<String>()),
                    other => format!("{other:?}"),
                })
                .unwrap_or_default();
            let i35 = verdicts
                .iter()
                .find(|(identifier, _)| *identifier == "I-3.5")
                .map(|(_, verdict)| match verdict {
                    InvariantVerdict::Violated(detail) => format!("Violated:{}", detail.chars().take(160).collect::<String>()),
                    other => format!("{other:?}"),
                })
                .unwrap_or_default();
            println!("PROBE i49 {label} named_walked_node_in_location={location_index} red={red:?} I-4.9={i49} I-3.5={i35}");
        }
    }
}

/// 全零 16 KiB / 32 KiB 的 CRC-32C（core 的实现）：与模型目录里 Python 独立算的 CRC 逐位比（F-零槽 表里两条全零单元的检查字就是这个函数算的）。
#[test]
fn probe_of_the_sonnet_leg_for_batch32_crc() {
    println!("PROBE crc zeros16384={:08x} zeros32768={:08x}", crc32_castagnoli(&vec![0u8; 16384]), crc32_castagnoli(&vec![0u8; 32768]));
}
