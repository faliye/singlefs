
// ─── m3-batch31-code-r1 攻方探针：只在攻方副本里追加到 checker_known_bad_images.rs 末尾，不进库 ───
// 每个探针在一份干净镜像上改一处，跑池级 checker，打一行 `PROBE …`（红的集合、I-2.1 与 I-7.2 第一条违例说明的分类）。
mod opus_b31_probe {
    use super::*;
    use std::sync::OnceLock;

    #[derive(Clone, Copy, PartialEq)]
    enum CopyState {
        /// 位置条目里的整单元校验和翻一字节
        Bad,
        /// 位置条目的槽改到盘尾之外，读不出
        Gone,
        /// 校验和换成容器前 16 KiB 的 CRC（头里仍是码 3）
        C16,
        /// 那块盘上孩子的槽写进这棵树根（码 2）的原样字节，校验和换成那 16 KiB 的 CRC
        Node,
        /// 不动（码 3 容器，按 32 KiB 自证得过）
        Ok,
    }
    const COPY_STATES: [CopyState; 5] = [
        CopyState::Bad,
        CopyState::Gone,
        CopyState::C16,
        CopyState::Node,
        CopyState::Ok,
    ];
    fn copy_state_name(state: CopyState) -> &'static str {
        match state {
            CopyState::Bad => "bad",
            CopyState::Gone => "gone",
            CopyState::C16 => "c16",
            CopyState::Node => "node",
            CopyState::Ok => "ok",
        }
    }

    #[derive(Clone, Copy)]
    enum EntryTypeCase {
        Record,
        Zero,
        Seven,
    }
    fn entry_type_case_name(case: EntryTypeCase) -> &'static str {
        match case {
            EntryTypeCase::Record => "rec",
            EntryTypeCase::Zero => "t0",
            EntryTypeCase::Seven => "t7",
        }
    }

    #[derive(Clone, Copy)]
    struct ProbeTree {
        label: &'static str,
        kind: u16,
        record_type: u16,
    }
    const TOMB: ProbeTree = ProbeTree {
        label: "tomb",
        kind: TREE_KIND_TOMBSTONE,
        record_type: PACKED_RECORD_TYPE_TOMBSTONE,
    };
    const LINEAGE: ProbeTree = ProbeTree {
        label: "lineage",
        kind: TREE_KIND_LINEAGE,
        record_type: PACKED_RECORD_TYPE_LINEAGE,
    };
    const INODE: ProbeTree = ProbeTree {
        label: "inode",
        kind: TREE_KIND_INODE,
        record_type: PACKED_RECORD_TYPE_INODE,
    };

    fn clean_image(tree: &ProbeTree) -> MemoryPool {
        static TOMB_IMAGE: OnceLock<MemoryPool> = OnceLock::new();
        static LINEAGE_IMAGE: OnceLock<MemoryPool> = OnceLock::new();
        static INODE_IMAGE: OnceLock<MemoryPool> = OnceLock::new();
        if tree.kind == TREE_KIND_TOMBSTONE {
            TOMB_IMAGE
                .get_or_init(|| {
                    common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
                        "opus-b31-probe-tomb",
                    )
                })
                .clone()
        } else if tree.kind == TREE_KIND_LINEAGE {
            LINEAGE_IMAGE
                .get_or_init(|| image_after_rolling_back_to_the_first_root("opus-b31-probe-lineage").image)
                .clone()
        } else {
            INODE_IMAGE
                .get_or_init(|| build_pool("opus-b31-probe-inode").memory_pool())
                .clone()
        }
    }

    fn location_field(index: usize) -> usize {
        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
            + POINTER_FIRST_LOCATION_OFFSET
            + index * POINTER_LOCATION_BYTES
    }
    fn set_location_checksum(entry: &mut [u8], index: usize, checksum: u32) {
        let field = location_field(index) + LOCATION_CHECKSUM_OFFSET;
        entry[field..field + 4].copy_from_slice(&checksum.to_le_bytes());
    }
    fn set_location_slot(entry: &mut [u8], index: usize, slot: u64) {
        let field = location_field(index) + 4;
        entry[field..field + 6].copy_from_slice(&slot.to_le_bytes()[..6]);
    }
    fn slot_beyond_the_device() -> u64 {
        IMAGE_BYTES / SLOT + 4096
    }

    fn report(probe: &str, image: &MemoryPool) -> std::collections::BTreeSet<&'static str> {
        let verdicts = check_pool_image(image);
        let red = violated_invariant_set(&verdicts);
        let detail_of = |invariant: &str| {
            verdicts
                .iter()
                .find(|(identifier, _)| *identifier == invariant)
                .map(|(_, found)| match found {
                    InvariantVerdict::Violated(detail) => detail.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default()
        };
        let walk = detail_of("I-7.2");
        let walk_tag = if walk.contains("有类型段 0 的条目") {
            "fl0"
        } else if walk.contains("（走读不往下读的孩子）两份都") {
            "nogo2"
        } else if walk.is_empty() {
            "-"
        } else {
            "other"
        };
        let checksum = detail_of("I-2.1");
        let checksum_tag = if checksum.contains("走读不往下读的孩子") {
            "nogo"
        } else if checksum.is_empty() {
            "-"
        } else {
            "read"
        };
        let red_list: Vec<&str> = red.iter().copied().collect();
        let packed_container_verdict = verdicts
            .iter()
            .find(|(identifier, _)| *identifier == "I-1.7")
            .map(|(_, found)| format!("{found:?}").split('(').next().unwrap_or("").to_string())
            .unwrap_or_default();
        println!(
            "PROBE {probe} red=[{}] i2_1={checksum_tag} i7_2={walk_tag} i1_7={packed_container_verdict}",
            red_list.join(",")
        );
        if std::env::var_os("OPUS_B31_DETAIL").is_some() {
            for (identifier, found) in &verdicts {
                if let InvariantVerdict::Violated(detail) = found {
                    println!("DETAIL {probe} {identifier}: {detail}");
                }
            }
        }
        red
    }

    fn set_the_root_level(image: &mut MemoryPool, kind: u16, level: u8) {
        let root = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
        let slot = common_pointer_head_corruption::the_slot_of(&root);
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            image,
            slot,
            common_pointer_head_corruption::node_bytes(),
            |node| node[INDEX_NODE_LEVEL_OFFSET] = level,
        );
    }

    /// 树根第一条条目：类型段按 `case` 改（类型段 0 时身份三段清零），两条位置条目各按 `states` 改；`root_level` 给了就先把根头里的层级改成它。
    fn matrix_probe(
        tree: &ProbeTree,
        case: EntryTypeCase,
        states: [CopyState; 2],
        root_level: Option<u8>,
        prefix: &str,
    ) {
        let clean = clean_image(tree);
        let mut image = clean.clone();
        let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, tree.kind, 0);
        let child_slot = common_pointer_head_corruption::the_slot_of(&child);
        let container = common_pointer_head_corruption::unit_bytes_at(
            &clean,
            child_slot,
            common_pointer_head_corruption::data_unit_bytes(),
        );
        let first_sixteen = crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
        let node = root_node_bytes_of_the_tree_of_kind(&clean, tree.kind);
        let node_checksum = crc32_castagnoli(&node);
        for (index, state) in states.iter().enumerate() {
            if *state == CopyState::Node {
                write(&mut image, DEVICES[index], child_slot.0 * SLOT, &node);
            }
        }
        if let Some(level) = root_level {
            set_the_root_level(&mut image, tree.kind, level);
        }
        let beyond = slot_beyond_the_device();
        let level = change_an_entry_of_the_root_of_kind(&mut image, tree.kind, 0, |entry| {
            match case {
                EntryTypeCase::Record => {}
                EntryTypeCase::Zero => {
                    change_the_entry_type(entry, tree.record_type, 0);
                    zero_the_three_identity_fields(entry);
                }
                EntryTypeCase::Seven => change_the_entry_type(
                    entry,
                    tree.record_type,
                    CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
                ),
            }
            for (index, state) in states.iter().enumerate() {
                match state {
                    CopyState::Bad => flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, index),
                    CopyState::Gone => set_location_slot(entry, index, beyond),
                    CopyState::C16 => set_location_checksum(entry, index, first_sixteen),
                    CopyState::Node => set_location_checksum(entry, index, node_checksum),
                    CopyState::Ok => {}
                }
            }
        });
        report(
            &format!(
                "{prefix} tree={} level={level} type={} c0={} c1={}",
                tree.label,
                entry_type_case_name(case),
                copy_state_name(states[0]),
                copy_state_name(states[1])
            ),
            &image,
        );
    }

    fn whole_matrix(tree: &ProbeTree, case: EntryTypeCase) {
        for first in COPY_STATES {
            for second in COPY_STATES {
                matrix_probe(tree, case, [first, second], None, "matrix");
            }
        }
    }

    #[test]
    fn opus_b31_clean_images() {
        for tree in [TOMB, LINEAGE, INODE] {
            report(&format!("clean tree={}", tree.label), &clean_image(&tree));
        }
    }
    #[test]
    fn opus_b31_matrix_tomb_rec() { whole_matrix(&TOMB, EntryTypeCase::Record); }
    #[test]
    fn opus_b31_matrix_tomb_t0() { whole_matrix(&TOMB, EntryTypeCase::Zero); }
    #[test]
    fn opus_b31_matrix_tomb_t7() { whole_matrix(&TOMB, EntryTypeCase::Seven); }
    #[test]
    fn opus_b31_matrix_lineage_rec() { whole_matrix(&LINEAGE, EntryTypeCase::Record); }
    #[test]
    fn opus_b31_matrix_lineage_t0() { whole_matrix(&LINEAGE, EntryTypeCase::Zero); }
    #[test]
    fn opus_b31_matrix_lineage_t7() { whole_matrix(&LINEAGE, EntryTypeCase::Seven); }
    #[test]
    fn opus_b31_matrix_inode_rec() { whole_matrix(&INODE, EntryTypeCase::Record); }
    #[test]
    fn opus_b31_matrix_inode_t0() { whole_matrix(&INODE, EntryTypeCase::Zero); }
    #[test]
    fn opus_b31_matrix_inode_t7() { whole_matrix(&INODE, EntryTypeCase::Seven); }

    /// 层级 0 的根（墓碑树、谱系树）：根头里层级 1 → 0，条目不动、类型段 0 两种副本各一组。
    #[test]
    fn opus_b31_level_zero_root() {
        for tree in [TOMB, LINEAGE] {
            let mut image = clean_image(&tree);
            set_the_root_level(&mut image, tree.kind, 0);
            report(&format!("l0root tree={} type=rec c0=ok c1=ok untouched", tree.label), &image);
            for first in COPY_STATES {
                for second in COPY_STATES {
                    matrix_probe(&tree, EntryTypeCase::Zero, [first, second], Some(0), "l0root");
                }
            }
        }
    }

    /// 码 3 打包记录单元头里记录宽（2 字节）的偏移。
    const RECORD_WIDTH_OFFSET: usize = 71;

    /// O3：树根第一条条目的孩子（码 3 容器）带一个 I-1.7 违例（记录宽 − 1，重封、补到根）；再把子指针第 `collide_on` 条位置条目的槽
    /// 指到这一遍先走过的一个码 2 节点 X 的槽上（盘不动：位置条目 0 在盘 0、1 在盘 1，两盘同槽）。`variant`：
    /// `fw` 校验和是 X 那 16 KiB 的 CRC（那一份自证成码 2，另一份是码 3 容器，F-W）；`bad` 校验和是坏的；`badself` X 就是这棵树自己的根、校验和坏的。
    fn o3_probe(tree: &ProbeTree, variant: &str, collide_on: Option<usize>) {
        let mut image = clean_image(tree);
        let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, tree.kind, 0);
        let child_slot = common_pointer_head_corruption::the_slot_of(&child);
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            &mut image,
            child_slot,
            common_pointer_head_corruption::data_unit_bytes(),
            |container| {
                let width = u16::from_le_bytes([container[RECORD_WIDTH_OFFSET], container[RECORD_WIDTH_OFFSET + 1]]);
                container[RECORD_WIDTH_OFFSET..RECORD_WIDTH_OFFSET + 2].copy_from_slice(&(width - 1).to_le_bytes());
            },
        );
        let x_kind = if variant == "badself" {
            tree.kind
        } else if tree.kind == TREE_KIND_INODE {
            TREE_KIND_EXTENT
        } else {
            TREE_KIND_INODE
        };
        let x_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(&image, x_kind).root;
        let x_slot = common_pointer_head_corruption::the_slot_of(&x_pointer);
        let x_checksum = crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            x_slot,
            common_pointer_head_corruption::node_bytes(),
        ));
        if let Some(index) = collide_on {
            change_an_entry_of_the_root_of_kind(&mut image, tree.kind, 0, |entry| {
                set_location_slot(entry, index, x_slot.0);
                let checksum = if variant == "fw" { x_checksum } else { x_checksum ^ 0x5a5a_5a5a };
                set_location_checksum(entry, index, checksum);
            });
        }
        let collide = collide_on.map_or("none".to_string(), |index| format!("loc{index}"));
        report(
            &format!("o3 tree={} variant={variant} collide={collide} x_kind={x_kind}", tree.label),
            &image,
        );
    }

    #[test]
    fn opus_b31_o3_visited_key() {
        for tree in [TOMB, LINEAGE, INODE] {
            o3_probe(&tree, "none", None);
            for variant in ["fw", "bad", "badself"] {
                for index in [0usize, 1] {
                    o3_probe(&tree, variant, Some(index));
                }
            }
        }
    }

    /// N2：extent 树上段的根。三个单元的文件 unlink 到底之后（根兼叶、零条目），根头改几种形状各判一次。
    #[test]
    fn opus_b31_extent_upper_root() {
        let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file("opus-b31-extent-empty");
        common_deletion_in_progress::unlink_to_the_end(&mut pool, 1);
        let image = pool.memory_pool();
        let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
        let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
        let root_bytes = common_pointer_head_corruption::unit_bytes_at(&image, root_slot, common_pointer_head_corruption::node_bytes());
        let root = singlefs_core::unit::parse_index_node(&root_bytes).expect("extent 根解得开");
        let key_width = usize::from(root_bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
        let header_end = 86 + 2 * key_width;
        println!(
            "PROBE-INFO extent-root level={} entries={} key_width={key_width} entry_width={}",
            root.level,
            root.entries.len(),
            u16::from_le_bytes([root_bytes[header_end - 2], root_bytes[header_end - 1]])
        );
        report("extent shape=empty-l0-legal", &image);
        let shapes: [(&str, Box<dyn Fn(&mut Vec<u8>)>); 5] = [
            ("empty-l1-entry113", Box::new(|node: &mut Vec<u8>| node[INDEX_NODE_LEVEL_OFFSET] = 1)),
            ("empty-l1-entry110", Box::new(move |node: &mut Vec<u8>| {
                node[INDEX_NODE_LEVEL_OFFSET] = 1;
                node[header_end - 2..header_end].copy_from_slice(&110u16.to_le_bytes());
            })),
            ("empty-l2-entry110", Box::new(move |node: &mut Vec<u8>| {
                node[INDEX_NODE_LEVEL_OFFSET] = 2;
                node[header_end - 2..header_end].copy_from_slice(&110u16.to_le_bytes());
            })),
            ("empty-l0-entry112", Box::new(move |node: &mut Vec<u8>| {
                node[header_end - 2..header_end].copy_from_slice(&112u16.to_le_bytes());
            })),
            ("empty-l0-keywidth-tag", Box::new(|node: &mut Vec<u8>| {
                // 头里 key 宽这一字节改成 25（头长随之变，条目区解法跟着变）
                node[INDEX_NODE_KEY_WIDTH_OFFSET] = 25;
            })),
        ];
        for (shape, change) in shapes {
            let mut changed = image.clone();
            common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
                &mut changed,
                root_slot,
                common_pointer_head_corruption::node_bytes(),
                |node| change(node),
            );
            report(&format!("extent shape={shape}"), &changed);
        }
    }
}
