
// ─── 攻方 m3-batch32-code-r1（Opus）探针：只在攻方副本里，不进仓 ───
// 臂在同一个进程里逐条换（walk.rs 副本里的 `opus_set_key_arm`）：每个起点镜像只建一次，每条臂判同一批镜像，各打一行 PROBE。

/// 七条臂（walk.rs 副本里的 `OpusKeyArm`），同一个进程里逐条换。
const OPUS_ARMS: [&str; 7] = ["today", "b31", "birthrb", "allrb", "allrbmap", "both", "gpuemu"];

fn opus_verdict_short(verdicts: &[(&'static str, InvariantVerdict)], name: &str) -> String {
    match verdicts.iter().find(|(identifier, _)| *identifier == name).map(|(_, v)| v) {
        Some(InvariantVerdict::Holds) => "H".to_string(),
        Some(InvariantVerdict::Violated(detail)) => {
            format!("V<{}>", detail.chars().take(std::env::var("OPUS_DETAIL_CHARS").ok().and_then(|v| v.parse().ok()).unwrap_or(160)).collect::<String>())
        }
        Some(InvariantVerdict::NotApplicable(_)) => "NA".to_string(),
        Some(InvariantVerdict::PartiallyJudged(_)) => "PJ".to_string(),
        None => "-".to_string(),
    }
}

fn opus_red(verdicts: &[(&'static str, InvariantVerdict)]) -> String {
    let set = violated_invariant_set(verdicts);
    format!("[{}]", set.into_iter().collect::<Vec<_>>().join(","))
}

fn opus_kind_name(kind: u16) -> &'static str {
    match kind {
        TREE_KIND_TOMBSTONE => "tomb",
        TREE_KIND_LINEAGE => "lineage",
        TREE_KIND_INODE => "inode",
        TREE_KIND_EXTENT => "extent",
        _ => "other",
    }
}

/// 一个槽在盘 `device` 上一个扇区都没写过。
fn opus_slot_unwritten(image: &MemoryPool, device: u32, slot: u64) -> bool {
    image
        .devices
        .get(&DeviceIdentity(device))
        .expect("盘")
        .written_sectors_in(DeviceOffsetInBytes(slot * SLOT), SLOT)
        .is_empty()
}

/// O3 那一族（同实现员 `image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width`），
/// 多一个开关：`record_bytes` 为 None 时容器不改记录宽（只有位置条目撞到先走过的节点）。
fn opus_o3_image(
    clean: &MemoryPool,
    kind: u16,
    record_bytes: Option<u64>,
    kind_walked_before: u16,
    location_index: usize,
) -> MemoryPool {
    let mut image = clean.clone();
    if let Some(record_bytes) = record_bytes {
        common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
            &mut image,
            kind,
            record_width_one_below(record_bytes),
        );
    }
    let walked_node_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, kind_walked_before).root;
    let walked_node_slot = common_pointer_head_corruption::the_slot_of(&walked_node_pointer);
    let wrong_checksum = crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
        &image,
        walked_node_slot,
        common_pointer_head_corruption::node_bytes(),
    )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        set_the_slot_in_a_child_location(entry, location_index, walked_node_slot.0);
        set_the_whole_unit_checksum_in_a_child_location(entry, location_index, wrong_checksum);
    });
    image
}

fn opus_probe_o3_base(base: &str) {
    let (clean, kind, record_bytes, walked_kinds): (MemoryPool, u16, u64, Vec<u16>) = match base {
        "tomb" => (
            common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("opus-b32-o3-tomb"),
            TREE_KIND_TOMBSTONE,
            singlefs_format::TOMBSTONE_RECORD_BYTES,
            vec![TREE_KIND_INODE, TREE_KIND_EXTENT, TREE_KIND_TOMBSTONE],
        ),
        "inode" => (
            build_pool("opus-b32-o3-inode").memory_pool(),
            TREE_KIND_INODE,
            singlefs_format::INODE_RECORD_BYTES,
            vec![TREE_KIND_EXTENT, TREE_KIND_INODE],
        ),
        "lineage" => (
            image_after_rolling_back_to_the_first_root("opus-b32-o3-lineage").image,
            TREE_KIND_LINEAGE,
            singlefs_format::LINEAGE_RECORD_BYTES,
            vec![TREE_KIND_INODE, TREE_KIND_EXTENT, TREE_KIND_LINEAGE],
        ),
        other => panic!("{other}"),
    };
    // 起点镜像只建一次；每一形从它 clone 出来改（`opus_o3_image`），每条臂判同一批镜像。
    let mut images = Vec::new();
    for walked in &walked_kinds {
        for (variant, width) in [("bad", Some(record_bytes)), ("good", None)] {
            for location_index in 0..2 {
                images.push((
                    *walked,
                    variant,
                    location_index,
                    opus_o3_image(&clean, kind, width, *walked, location_index),
                ));
            }
        }
    }
    for arm in OPUS_ARMS {
        singlefs_checker::walk::opus_set_key_arm(arm);
        for (walked, variant, location_index, image) in &images {
            let verdicts = check_pool_image(image);
            println!(
                "PROBE p1o3 arm={arm} base={base} walked={} variant={variant} collide=loc{location_index} red={} i1_7={} i4_9={} i3_5={}",
                opus_kind_name(*walked),
                opus_red(&verdicts),
                opus_verdict_short(&verdicts, "I-1.7").chars().take(1).collect::<String>(),
                opus_verdict_short(&verdicts, UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT),
                opus_verdict_short(&verdicts, "I-3.5"),
            );
        }
    }
}

#[test]
fn opus_probe_p1_o3_tomb() { opus_probe_o3_base("tomb"); }
#[test]
fn opus_probe_p1_o3_inode() { opus_probe_o3_base("inode"); }
#[test]
fn opus_probe_p1_o3_lineage() { opus_probe_o3_base("lineage"); }

/// 第一个落在 `from` 之后、两块盘上自己与前后各一槽都没写过的槽。
fn opus_free_slot_after(image: &MemoryPool, _from: u64) -> u64 {
    let last_written = DEVICES
        .iter()
        .filter_map(|device| {
            image
                .devices
                .get(&DeviceIdentity(*device))
                .expect("盘")
                .written_sectors()
                .keys()
                .map(|sector| sector * 512 / SLOT)
                .filter(|slot| *slot >= UNIT_AREA_START_SLOT_OF_THE_POOL)
                .max()
        })
        .max()
        .expect("单元区写过");
    last_written + 16
}

/// 「第 0 条那一份对不上、它的落点没有别的单元占着」那一形（实现员报告「停下」第 1 条第二段，推的）：回退到 A 之后的镜像，
/// D（最新）照抄 A 的 inode 树。inode 树根 X 第一条条目指的叶容器 L：`raise_birth` 时把 L 头里的诞生代号抬到 D 的 txg（> A 的 txg，
/// 重封、沿引用链补到根；指针与映射里的出生身份不改，I-1.2 / I-1.11 会跟着红，与 I-4.9 无关）；`corrupt_first_copy` 时把 L 在盘 0 那一份
/// 写成别的字节（不补校验和，那个槽上没有别的单元）；`direct` 时把 X 原样另写一份到没写过的槽 X'，D 的树表改指 X'（D 与 A 各自新走
/// 自己的 inode 树根，L 由两版的树根直接引用，A 走到 L 时在 F-KEY 那一处「走过」命中、按那一处的键合并）；不 `direct` 时 A 在 X 上就命中、
/// 按 X 下面记下的落点（L 的两个落点都在）合并。
fn opus_split_image(
    rolled: &ImageAfterTheRollbackToTheFirstRoot,
    raise_birth: bool,
    corrupt_first_copy: bool,
    direct: bool,
) -> (MemoryPool, u64, u64, u64) {
    let mut image = rolled.image.clone();
    let leaf = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    let leaf_slot = common_pointer_head_corruption::the_slot_of(&leaf);
    let data_unit_bytes = common_pointer_head_corruption::data_unit_bytes();
    if raise_birth {
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            &mut image,
            leaf_slot,
            data_unit_bytes,
            |bytes| {
                let birth = u64::from_le_bytes(bytes[73..81].try_into().expect("8 字节"));
                assert!(birth <= rolled.rollback_target_txg, "L 的诞生代号 {birth} 本来不晚于 A");
                bytes[73..81].copy_from_slice(&rolled.rollback_publish_txg.to_le_bytes());
            },
        );
    }
    if direct {
        let inode_root = common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
        let x_slot = common_pointer_head_corruption::the_slot_of(&inode_root);
        let x_bytes = common_pointer_head_corruption::unit_bytes_at(
            &image,
            x_slot,
            common_pointer_head_corruption::node_bytes(),
        );
        let x_prime = opus_free_slot_after(&image, leaf_slot.0 + 64);
        for device in DEVICES {
            write(&mut image, device, x_prime * SLOT, &x_bytes);
        }
        let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&inode_root);
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            &mut image,
            singlefs_core::address::SlotNumber(rolled.tree_table_slot_of_the_rollback_publish),
            common_pointer_head_corruption::node_bytes(),
            |table| {
                let starts: Vec<usize> = (0..=table.len() - pointer_bytes.len())
                    .filter(|start| table[*start..*start + pointer_bytes.len()] == pointer_bytes[..])
                    .collect();
                assert_eq!(starts.len(), 1, "D 的树表里 inode 树根指针恰好一处");
                for location_index in 0..2 {
                    let at = starts[0] + POINTER_FIRST_LOCATION_OFFSET + location_index * POINTER_LOCATION_BYTES + LOCATION_DEVICE_BYTES;
                    table[at..at + LOCATION_SLOT_BYTES].copy_from_slice(&x_prime.to_le_bytes()[..LOCATION_SLOT_BYTES]);
                }
            },
        );
    }
    if corrupt_first_copy {
        let garbage = vec![0xa5u8; data_unit_bytes];
        write(&mut image, leaf.locations[0].device.0, leaf_slot.0 * SLOT, &garbage);
    }
    (image, leaf_slot.0, rolled.rollback_target_txg, rolled.rollback_publish_txg)
}

#[test]
fn opus_probe_p1_split() {
    // 回退到 A 之后的起点镜像只建一次，八形各从它 clone 出来改。
    let rolled = image_after_rolling_back_to_the_first_root("opus-b32-split");
    let mut images = Vec::new();
    for direct in [false, true] {
        for raise_birth in [false, true] {
            for corrupt_first_copy in [false, true] {
                images.push((
                    direct,
                    raise_birth,
                    corrupt_first_copy,
                    opus_split_image(&rolled, raise_birth, corrupt_first_copy, direct),
                ));
            }
        }
    }
    for arm in OPUS_ARMS {
        singlefs_checker::walk::opus_set_key_arm(arm);
        for (direct, raise_birth, corrupt_first_copy, (image, leaf_slot, a_txg, d_txg)) in &images {
            let verdicts = check_pool_image(image);
            println!(
                "PROBE p1split arm={arm} direct={direct} raise_birth={raise_birth} corrupt_loc0={corrupt_first_copy} leaf_slot={leaf_slot} a_txg={a_txg} d_txg={d_txg} red={} i4_9={} i3_5={}",
                opus_red(&verdicts),
                opus_verdict_short(&verdicts, UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT),
                opus_verdict_short(&verdicts, "I-3.5"),
            );
        }
    }
}
