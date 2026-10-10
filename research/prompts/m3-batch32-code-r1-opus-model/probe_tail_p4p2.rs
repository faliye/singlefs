
/// P4（F-零槽）：墓碑树根第一条条目改成类型段 `entry_type`（7 不认得；0 走 F-L0 那一支），位置条目 1 改指盘 1 上一个没写过的槽 g、
/// 整单元校验和换成 `checksum_of`(g) 给的值。`neighbour_written` 说 g 之后那一槽 g+1 在盘 1 上写过没有。
/// 两份都由 `judge_every_copy_of_a_child_the_walk_does_not_go_into` 判（CPU `copy_matches_its_location_checksum_at_the_length_its_header_names`：
/// 头里类标签 0 没登记，按 16 KiB 与 32 KiB 两种长度各读一次比）。
fn opus_zero_slot_image(
    clean: &MemoryPool,
    entry_type: u16,
    neighbour_written: bool,
    checksum_kind: &str,
) -> (MemoryPool, u64, u32) {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_TOMBSTONE, 0);
    let child_slot = common_pointer_head_corruption::the_slot_of(&child).0;
    let device = child.locations[1].device.0;
    let written: std::collections::BTreeSet<u64> = clean
        .devices
        .get(&DeviceIdentity(device))
        .expect("盘")
        .written_sectors()
        .keys()
        .map(|sector| sector * 512 / SLOT)
        .filter(|slot| *slot >= UNIT_AREA_START_SLOT_OF_THE_POOL)
        .collect();
    let _ = child_slot;
    let gap = if neighbour_written {
        written
            .iter()
            .map(|slot| slot - 1)
            .find(|slot| *slot >= UNIT_AREA_START_SLOT_OF_THE_POOL && !written.contains(slot))
            .expect("找得到下一槽写过的空槽")
    } else {
        written.iter().next_back().expect("写过") + 8
    };
    assert!(!written.contains(&gap) && written.contains(&(gap + 1)) == neighbour_written);
    let zeros = vec![0u8; 32768];
    let neighbour = common_pointer_head_corruption::unit_bytes_at(
        clean,
        singlefs_core::address::SlotNumber(gap + 1),
        16384,
    );
    let checksum = match checksum_kind {
        "zero16" => crc32_castagnoli(&zeros[..16384]),
        "zero32" => crc32_castagnoli(&zeros),
        "zero16+next" => crc32_castagnoli(&[&zeros[..16384], &neighbour[..]].concat()),
        other => panic!("{other}"),
    };
    let mut image = clean.clone();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(entry, 1, entry_type);
        if entry_type == 0 {
            zero_the_three_identity_fields(entry);
        }
        set_the_slot_in_a_child_location(entry, 1, gap);
        set_the_whole_unit_checksum_in_a_child_location(entry, 1, checksum);
    });
    (image, gap, checksum)
}

#[test]
fn opus_probe_p4_zero_slot_next_to_a_written_slot() {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "opus-b32-p4",
    );
    println!(
        "PROBE p4const crc_zero16={:#010x} crc_zero32={:#010x}",
        crc32_castagnoli(&[0u8; 16384]),
        crc32_castagnoli(&[0u8; 32768])
    );
    for entry_type in [7u16, 0] {
        for neighbour_written in [true, false] {
            for checksum_kind in ["zero16", "zero32", "zero16+next"] {
                let (image, gap, checksum) =
                    opus_zero_slot_image(&clean, entry_type, neighbour_written, checksum_kind);
                let verdicts = check_pool_image(&image);
                println!(
                    "PROBE p4 type={entry_type} next_written={neighbour_written} sum={checksum_kind}({checksum:#010x}) gap_slot={gap} red={} i2_1={}",
                    opus_red(&verdicts),
                    opus_verdict_short(&verdicts, "I-2.1"),
                );
            }
        }
    }
}

/// P2（F-ID-b）副本组合：墓碑树根（层级 `level`）第一条条目类型段 1 → 0、身份三段清零，两条位置条目各放一种副本：
/// node（这棵树根的 16 KiB 原样、校验和按它）、c3（容器原样、校验和照旧，按 32 KiB 自证得过）、c16（容器原样、校验和换成前 16 KiB 的 CRC）、
/// bad（容器原样、校验和翻几位）。
fn opus_copy_matrix_image(clean: &MemoryPool, level: u8, copies: [&str; 2]) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_TOMBSTONE, 0);
    let child_slot = common_pointer_head_corruption::the_slot_of(&child);
    let container = common_pointer_head_corruption::unit_bytes_at(clean, child_slot, 32768);
    let node = root_node_bytes_of_the_tree_of_kind(clean, TREE_KIND_TOMBSTONE);
    let mut image = clean.clone();
    let mut checksums = [0u32; 2];
    for (index, copy) in copies.iter().enumerate() {
        checksums[index] = match *copy {
            "node" => {
                write(&mut image, child.locations[index].device.0, child_slot.0 * SLOT, &node);
                crc32_castagnoli(&node)
            }
            "c3" => child.locations[index].unit_checksum,
            "c16" => crc32_castagnoli(&container[..16384]),
            "bad" => child.locations[index].unit_checksum ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY,
            other => panic!("{other}"),
        };
    }
    if level != 1 {
        set_the_level_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, level);
    }
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(entry, 1, 0);
        zero_the_three_identity_fields(entry);
        for index in 0..2 {
            set_the_whole_unit_checksum_in_a_child_location(entry, index, checksums[index]);
        }
    });
    image
}

#[test]
fn opus_probe_p2_copy_matrix() {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "opus-b32-p2",
    );
    let kinds = ["node", "c3", "c16", "bad"];
    for level in [1u8, 0] {
        for first in kinds {
            for second in kinds {
                let image = opus_copy_matrix_image(&clean, level, [first, second]);
                let verdicts = check_pool_image(&image);
                println!(
                    "PROBE p2 level={level} c0={first} c1={second} red={} i9_23={}",
                    opus_red(&verdicts),
                    opus_verdict_short(&verdicts, "I-9.23"),
                );
            }
        }
    }
}
