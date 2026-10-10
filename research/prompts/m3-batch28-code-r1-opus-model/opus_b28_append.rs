
// ════════════════════════════ m3-batch28-code-r1 攻方原型（追加到 checker_known_bad_images.rs 的拷贝末尾；只在仓副本上编、跑） ════════════════════════════
// 用例名都以 opus_b28_ 打头，输出一律是 RESULT 打头的行；OPUS_CODE 是这一份副本的代号（new = 冻结副本原样，old = walk.rs 反打第 28 批补丁）。

fn opus_b28_code() -> String {
    std::env::var("OPUS_CODE").unwrap_or_else(|_| "?".to_string())
}

fn opus_b28_kind_of(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "H",
        InvariantVerdict::Violated(_) => "V",
        InvariantVerdict::NotApplicable(_) => "NA",
        _ => "P",
    }
}

fn opus_b28_verdict_tag(verdicts: &[(&'static str, InvariantVerdict)], invariant: &str) -> String {
    verdicts
        .iter()
        .find(|(name, _)| *name == invariant)
        .map(|(_, verdict)| opus_b28_kind_of(verdict).to_string())
        .unwrap_or_else(|| "-".to_string())
}

/// H-V 的造法（与第 28 批坏镜像 `image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device` 同一个做法，这里另写一份，
/// 不依赖第 28 批新加的帮手）：种类 `kind` 那棵树的根第一条条目先照 `change_the_entry` 改，再把它子指针第 `location_index` 条位置条目的
/// 整单元校验和换成这棵树根（码 2）原样 16 KiB 的 CRC、重封到根，那块盘上孩子的槽起写进这 16 KiB；另一块盘那一份不动（是 `image` 里原来那片码 3 容器）。
fn opus_b28_one_copy_verifying_as_a_node(
    image: &MemoryPool,
    kind: u16,
    location_index: usize,
    change_the_entry: &dyn Fn(&mut [u8]),
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(image, kind, 0);
    let root_bytes = root_node_bytes_of_the_tree_of_kind(image, kind);
    let checksum_of_the_root_bytes = crc32_castagnoli(&root_bytes);
    let mut changed = image.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut changed, kind, 0, |entry| {
        change_the_entry(entry);
        let pointer_field = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
            ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES;
        let mut pointer = singlefs_core::pointer::NodePointer::read_from(
            &mut singlefs_core::bytes::ByteReader::at(entry, pointer_field.start),
        );
        pointer.locations[location_index].unit_checksum = checksum_of_the_root_bytes;
        entry[pointer_field].copy_from_slice(&common_pointer_head_corruption::node_pointer_bytes(&pointer));
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    write(
        &mut changed,
        child.locations[location_index].device.0,
        common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
        &root_bytes,
    );
    changed
}

/// 这棵树根第一条条目指的那片容器（两份都）照 `change` 改、重封，新的整单元校验和沿引用链补到根槽。
fn opus_b28_change_the_first_container(image: &mut MemoryPool, kind: u16, change: impl FnOnce(&mut Vec<u8>)) {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(image, kind, 0);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        common_pointer_head_corruption::the_slot_of(&child),
        common_pointer_head_corruption::data_unit_bytes(),
        change,
    );
}

/// 码 3 容器里第 `index` 条记录（宽 `width`）`offset` 起那 8 字节改成 `value(旧值, 容器字节)`。
fn opus_b28_set_record_u64(container: &mut [u8], width: usize, index: usize, offset: usize, value: impl FnOnce(u64, &[u8]) -> u64) {
    let at = 136 + index * width + offset;
    let old = get_u64(container, at);
    let new = value(old, container);
    container[at..at + 8].copy_from_slice(&new.to_le_bytes());
}

type OpusB28Violation = (&'static str, &'static str, fn(&mut MemoryPool, u16));

/// 每棵树的「真身那一份里的违例」：名字、该红的不变量、改法（两份一起改、重封、补到根；之后 H-V 那一步只换另一块盘那一份）。
fn opus_b28_violations(kind: u16) -> Vec<OpusB28Violation> {
    let none: OpusB28Violation = ("none", "-", |_, _| {});
    let width: OpusB28Violation = ("width", "I-1.7", |image, kind| {
        opus_b28_change_the_first_container(image, kind, |c| {
            let w = u16::from_le_bytes([c[71], c[72]]);
            c[71..73].copy_from_slice(&(w - 1).to_le_bytes());
        })
    });
    let empty: OpusB28Violation = ("empty", "I-9.13", |image, kind| {
        opus_b28_change_the_first_container(image, kind, |c| {
            c[69..71].copy_from_slice(&0u16.to_le_bytes());
        })
    });
    if kind == TREE_KIND_INODE {
        vec![
            none,
            ("reserved", "I-9.7", |image, kind| {
                opus_b28_change_the_first_container(image, kind, |c| opus_b28_set_record_u64(c, 140, 0, 112, |_, _| 1))
            }),
            ("blocks", "I-9.15", |image, kind| {
                opus_b28_change_the_first_container(image, kind, |c| opus_b28_set_record_u64(c, 140, 0, 48, |old, _| old + 1))
            }),
            ("change-count", "I-9.26", |image, kind| {
                opus_b28_change_the_first_container(image, kind, |c| {
                    opus_b28_set_record_u64(c, 140, 0, 88, |_, container| get_u64(container, 73) + 1)
                })
            }),
            ("inode-below-container", "I-9.4", |image, kind| {
                // 第 0 条记录的 inode 号改到容器号之下（I-9.4 第一句）
                opus_b28_change_the_first_container(image, kind, |c| {
                    let container_number = get_u64(c, 53);
                    if container_number > 0 {
                        opus_b28_set_record_u64(c, 140, 0, 0, |_, _| container_number - 1);
                    }
                })
            }),
            width,
            empty,
        ]
    } else if kind == TREE_KIND_TOMBSTONE {
        vec![
            none,
            ("range", "I-9.20", |image, _kind| {
                common_deletion_in_progress::change_the_first_tombstone_of_the_newest_root(image, |t| {
                    common_deletion_in_progress::set_tombstone_field(t, common_deletion_in_progress::TOMBSTONE_RECORD_RANGE_END_OFFSET, 0)
                })
            }),
            ("death-txg", "I-9.20", |image, _kind| {
                common_deletion_in_progress::change_the_first_tombstone_of_the_newest_root(image, |t| {
                    common_deletion_in_progress::set_tombstone_field(t, common_deletion_in_progress::TOMBSTONE_RECORD_DEATH_TXG_OFFSET, 1_000_000)
                })
            }),
            width,
            empty,
        ]
    } else {
        vec![none, width, empty]
    }
}

/// H-V 段：三棵树 × 真身那一份里的违例 × 条目两种（不改 / 改成类型段 0、身份清零）× 另一类那一份在位置条目 0 或 1（再加不做 H-V 的对照）。
/// 每形一行：违例该红的那一条的判定、红的全集。
#[test]
fn opus_b28_hv_subtree_of_the_verifying_copy() {
    let code = opus_b28_code();
    let pools: Vec<(&str, u16, u16, &str, MemoryPool)> = vec![
        ("inode", TREE_KIND_INODE, PACKED_RECORD_TYPE_INODE, "I-9.2", build_pool("opus-b28-hv-inode").memory_pool()),
        (
            "tomb",
            TREE_KIND_TOMBSTONE,
            PACKED_RECORD_TYPE_TOMBSTONE,
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("opus-b28-hv-tomb"),
        ),
        (
            "lineage",
            TREE_KIND_LINEAGE,
            PACKED_RECORD_TYPE_LINEAGE,
            LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            image_after_rolling_back_to_the_first_root("opus-b28-hv-lineage").image,
        ),
    ];
    for (tree, kind, record_type, entry_invariant, clean) in &pools {
        for (violation, target, apply) in opus_b28_violations(*kind) {
            let mut base = clean.clone();
            apply(&mut base, *kind);
            let entry_variants: Vec<(&str, Box<dyn Fn(&mut [u8])>)> = vec![
                ("keep", Box::new(|_entry: &mut [u8]| {})),
                (
                    "type0",
                    Box::new(move |entry: &mut [u8]| {
                        change_the_entry_type(entry, *record_type, CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE);
                        zero_the_three_identity_fields(entry);
                    }),
                ),
            ];
            for (entry_name, change_the_entry) in &entry_variants {
                let mut shapes: Vec<(String, MemoryPool)> = Vec::new();
                let mut without_hv = base.clone();
                change_an_entry_of_the_root_of_kind(&mut without_hv, *kind, 0, |entry| change_the_entry(entry));
                shapes.push(("none".to_string(), without_hv));
                for location_index in 0..2 {
                    shapes.push((
                        format!("loc{location_index}"),
                        opus_b28_one_copy_verifying_as_a_node(&base, *kind, location_index, change_the_entry.as_ref()),
                    ));
                }
                for (hv, image) in &shapes {
                    let verdicts = check_pool_image(image);
                    let reds: Vec<&str> = violated_invariants(&verdicts);
                    // core 的恢复读者（`recovery::read_unit_via_locations`：按条目那一类的长度、第一份对得上的）对这张镜像给什么
                    let core = match recover(image, JournalPolicy::Consult).outcome {
                        singlefs_core::recovery::RecoveryOutcome::FileRead { content, .. } => format!("FileRead({})", content.len()),
                        singlefs_core::recovery::RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
                        singlefs_core::recovery::RecoveryOutcome::Failed { failure, .. } => {
                            format!("Failed({})", format!("{failure:?}").chars().take(40).collect::<String>())
                        }
                    };
                    println!(
                        "RESULT HV code={code} tree={tree} violation={violation} entry={entry_name} other_class_copy={hv} target={target}:{} entry_invariant={entry_invariant}:{} I-2.1:{} core={core} reds={reds:?}",
                        if target == "-" { "-".to_string() } else { opus_b28_verdict_tag(&verdicts, target) },
                        opus_b28_verdict_tag(&verdicts, entry_invariant),
                        opus_b28_verdict_tag(&verdicts, "I-2.1"),
                    );
                }
            }
        }
    }
}

/// J1 / CPU 与 GPU 第二次抽样的 CPU 期望：新池新建文件那条根（txg 3）里实例表指针头部的出生 txg / 实例代号 / 尾段（出生序号）各改一处、重封根槽，
/// 记录新根段跟着改（与 `an_instance_table_pointer_whose_birth_txg_differs_from_the_instance_table_header_reddens_only_the_birth_identity_invariant` 同一造法）。
/// 每形一行：CPU 池级 checker 红的全集与 I-1.2 的说明（GPU 走读内核的实例表链那一支不判 I-1.2，读码见报告）。
#[test]
fn opus_b28_instance_table_pointer_birth_identity() {
    let code = opus_b28_code();
    let clean = build_pool("opus-b28-instance-table-pointer").memory_pool();
    let fields: [(&str, usize, usize); 4] = [("none", 0, 0), ("birth_txg", 42, 8), ("instance", 78, 4), ("tail", 82, 4)];
    for (name, field, width) in fields {
        let mut image = clean.clone();
        if width > 0 {
            let (device, offset) = NEW_POOL_FILE_CREATION_ROOT_SLOT;
            let mut root = read(&image, device, offset, 512);
            let at = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + field;
            root[at] = root[at].wrapping_add(1);
            let digest = wide_checksum_with_field_zeroed(&root, 512, 138);
            root[138..170].copy_from_slice(&digest);
            write(&mut image, device, offset, &root);
            copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
        }
        let verdicts = check_pool_image(&image);
        let detail = verdicts
            .iter()
            .find(|(n, _)| *n == "I-1.2")
            .map(|(_, v)| match v {
                InvariantVerdict::Violated(text) => text.chars().take(120).collect::<String>(),
                other => opus_b28_kind_of(other).to_string(),
            })
            .unwrap_or_default();
        let core = match recover(&image, JournalPolicy::Consult).outcome {
            singlefs_core::recovery::RecoveryOutcome::FileRead { content, .. } => format!("FileRead({})", content.len()),
            singlefs_core::recovery::RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
            singlefs_core::recovery::RecoveryOutcome::Failed { failure, .. } => {
                format!("Failed({})", format!("{failure:?}").chars().take(40).collect::<String>())
            }
        };
        println!(
            "RESULT IT code={code} field={name} core={core} reds={:?} I-1.2={detail}",
            violated_invariants(&verdicts)
        );
    }
}
