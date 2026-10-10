
// ════════════ 正推腿 m3-batch24-code-r1 的探针（接在 checker_known_bad_images.rs 去掉 `//!` 行的副本后面，借它的搭建函数）════════════
// 只跑 CPU 池级 checker（harness 档）。四种树 × 条目类型段 × 孩子头里的类标签：每一格先由条款推「该红的是不是目标」，再与 CPU 实际比。
#[derive(Clone, Copy, PartialEq, Debug)]
enum ZsFixture {
    TombstoneLevelOne,
    LineageLevelOne,
    InodeLevelOne,
    TombstoneLevelTwo,
}

const ZS_FIXTURES: [ZsFixture; 4] = [
    ZsFixture::TombstoneLevelOne,
    ZsFixture::LineageLevelOne,
    ZsFixture::InodeLevelOne,
    ZsFixture::TombstoneLevelTwo,
];

fn zs_base(fixture: ZsFixture) -> MemoryPool {
    match fixture {
        ZsFixture::TombstoneLevelOne => {
            common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file("zs-tomb1")
        }
        ZsFixture::LineageLevelOne => image_after_rolling_back_to_the_first_root("zs-lineage1").image,
        ZsFixture::InodeLevelOne => build_pool("zs-inode1").memory_pool(),
        ZsFixture::TombstoneLevelTwo => {
            pool_with_empty_inodes_unlinked_under_the_tombstone_knob("zs-tomb2", &[2, 3, 4])
                .memory_pool()
        }
    }
}

fn zs_kind(fixture: ZsFixture) -> u16 {
    match fixture {
        ZsFixture::TombstoneLevelOne | ZsFixture::TombstoneLevelTwo => TREE_KIND_TOMBSTONE,
        ZsFixture::LineageLevelOne => TREE_KIND_LINEAGE,
        ZsFixture::InodeLevelOne => TREE_KIND_INODE,
    }
}

fn zs_target(fixture: ZsFixture) -> &'static str {
    match fixture {
        ZsFixture::TombstoneLevelOne | ZsFixture::TombstoneLevelTwo => "I-9.23",
        ZsFixture::LineageLevelOne => "I-9.17",
        ZsFixture::InodeLevelOne => "I-9.2",
    }
}

/// 这棵树里条目类型段「指码 3 容器」的那个值：墓碑 1、谱系 5、inode 2。
fn zs_container_type(fixture: ZsFixture) -> u16 {
    match fixture {
        ZsFixture::TombstoneLevelOne | ZsFixture::TombstoneLevelTwo => 1,
        ZsFixture::LineageLevelOne => 5,
        ZsFixture::InodeLevelOne => 2,
    }
}

/// 条款之外的类型段：一个任何一棵树都不登记的 7，与另一棵树的容器类型值（谱系树里放 1、别的放 5）。
fn zs_unregistered_types(fixture: ZsFixture) -> [u16; 2] {
    match fixture {
        ZsFixture::LineageLevelOne | ZsFixture::InodeLevelOne => [7, 1],
        ZsFixture::TombstoneLevelOne | ZsFixture::TombstoneLevelTwo => [7, 5],
    }
}

fn zs_first_child(image: &MemoryPool, kind: u16) -> (singlefs_core::address::SlotNumber, usize, u8) {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root = singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
        image,
        common_pointer_head_corruption::the_slot_of(&root_pointer),
        common_pointer_head_corruption::node_bytes(),
    ))
    .expect("根解得开");
    let child_pointer =
        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            &root.entries[0],
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
        ));
    let slot = common_pointer_head_corruption::the_slot_of(&child_pointer);
    let header = common_pointer_head_corruption::unit_bytes_at(
        image,
        slot,
        common_pointer_head_corruption::node_bytes(),
    );
    let tag = header[6];
    let length = if tag == 3 || tag == 1 {
        common_pointer_head_corruption::data_unit_bytes()
    } else {
        common_pointer_head_corruption::node_bytes()
    };
    (slot, length, tag)
}

fn zs_unit_length_of_class(class: u8) -> Option<usize> {
    match class {
        1 | 3 => Some(32768),
        2 => Some(16384),
        _ => None,
    }
}

#[test]
fn zs_matrix_of_entry_type_against_the_class_tag_of_the_child() {
    let tags: [Option<u8>; 6] = [None, Some(1), Some(2), Some(3), Some(9), Some(0)];
    let mut rows = 0usize;
    let mut deviations = 0usize;
    for fixture in ZS_FIXTURES {
        let base = zs_base(fixture);
        let kind = zs_kind(fixture);
        let (slot, length, original_tag) = zs_first_child(&base, kind);
        println!(
            "ZS_FIXTURE {fixture:?} child_slot={} child_length={length} child_original_tag={original_tag}",
            slot.0
        );
        let container_type = zs_container_type(fixture);
        let [unregistered_a, unregistered_b] = zs_unregistered_types(fixture);
        let types: [(u16, bool); 6] = [
            (0, true),
            (0, false),
            (container_type, false),
            (unregistered_a, false),
            (unregistered_b, false),
            (container_type, true),
        ];
        for (entry_type, zero_identity) in types {
            for tag in tags {
                let mut image = base.clone();
                if let Some(tag_value) = tag {
                    let mut bytes = common_pointer_head_corruption::unit_bytes_at(&image, slot, length);
                    bytes[6] = tag_value;
                    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
                        &mut image, slot, &bytes,
                    );
                }
                change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
                    set_u16(entry, CONTAINER_TREE_ENTRY_TYPE_OFFSET, entry_type);
                    if zero_identity {
                        zero_the_three_identity_fields(entry);
                    }
                });
                let verdicts = if fixture == ZsFixture::TombstoneLevelTwo {
                    check_pool_image_under_the_tombstone_knob(&image)
                } else {
                    check_pool_image(&image)
                };
                let reds = violated_invariants(&verdicts);
                let target = zs_target(fixture);
                let observed_target_red = reds.contains(&target);
                // 条款推出来的期望
                let effective_tag = tag.unwrap_or(original_tag);
                let named_class: Option<u8> = if entry_type == 0 {
                    Some(2)
                } else if entry_type == container_type {
                    Some(3)
                } else {
                    None
                };
                let self_verifying_class: Option<u8> = zs_unit_length_of_class(effective_tag)
                    .filter(|class_length| *class_length == length)
                    .map(|_| effective_tag);
                let identity_ok = if entry_type == 0 {
                    zero_identity || fixture == ZsFixture::TombstoneLevelTwo
                } else if entry_type == container_type {
                    fixture != ZsFixture::TombstoneLevelTwo && !zero_identity
                } else {
                    false
                };
                let expected_target_red: Option<bool> = match named_class {
                    None => Some(true),
                    Some(named) => {
                        if let Some(class) = self_verifying_class {
                            Some(class != named || !identity_ok)
                        } else if zs_unit_length_of_class(named) == Some(length) {
                            Some(effective_tag != named || !identity_ok)
                        } else if entry_type == 0 {
                            Some(!identity_ok)
                        } else {
                            None
                        }
                    }
                };
                let only_target_when_the_child_verifies = self_verifying_class.is_none()
                    || reds.iter().all(|red| *red == target);
                let mut flags = String::new();
                if let Some(expected) = expected_target_red {
                    if expected != observed_target_red {
                        flags.push_str(" DEVIATION_TARGET");
                    }
                }
                if !only_target_when_the_child_verifies {
                    flags.push_str(" DEVIATION_EXTRA_RED");
                }
                if !flags.is_empty() {
                    deviations += 1;
                }
                rows += 1;
                println!(
                    "ZS_CPU {fixture:?} type={entry_type} zero_identity={zero_identity} tag={tag:?} verifies_as={self_verifying_class:?} expected_target_red={expected_target_red:?} reds={reds:?}{flags}"
                );
            }
        }
    }
    println!("ZS_SUMMARY rows={rows} deviations={deviations}");
}
