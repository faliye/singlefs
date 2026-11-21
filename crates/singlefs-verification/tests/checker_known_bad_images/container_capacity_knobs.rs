//! 墓碑、谱系、退役三种容器在容量旋钮下的个数与编号，以及读不回时容量不变量不判。

use super::*;

// ───────────────────────── G2、G3：容器号从 1 起、逐片加一、每片至少一条（I-9.21、I-9.22、I-9.36、I-9.37） ─────────────────────────

/// I-9.21 / I-9.22 / I-9.36 / I-9.37 判的三棵容器树（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G2、G3）：
/// 墓碑树判 I-9.21 / I-9.22，谱系树与退役树判 I-9.36 / I-9.37。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerTreeOfTheCapacityInvariants {
    Tombstone,
    Lineage,
    Retirement,
}

impl ContainerTreeOfTheCapacityInvariants {
    fn tree_kind(self) -> u16 {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => TREE_KIND_TOMBSTONE,
            ContainerTreeOfTheCapacityInvariants::Lineage => TREE_KIND_LINEAGE,
            ContainerTreeOfTheCapacityInvariants::Retirement => TREE_KIND_RETIREMENT,
        }
    }

    /// 带旋钮值的入口判这一棵的那一条：墓碑树 I-9.22，谱系树与退役树 I-9.37。
    fn knob_capacity_invariant(self) -> &'static str {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => {
                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
            }
            ContainerTreeOfTheCapacityInvariants::Lineage
            | ContainerTreeOfTheCapacityInvariants::Retirement => {
                LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
            }
        }
    }

    /// 违例说明里怎么称呼这一棵（`walk.rs` 的说明照这个写）。
    fn name_in_the_violation(self) -> &'static str {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => "墓碑树",
            ContainerTreeOfTheCapacityInvariants::Lineage => "谱系树",
            ContainerTreeOfTheCapacityInvariants::Retirement => "退役树",
        }
    }

    /// 池级 checker 带旋钮值的入口：只这一棵装 `records_per_container` 条一片的旋钮，另两棵当没装。
    fn checker_knobs(
        self,
        records_per_container: u64,
    ) -> singlefs_checker::walk::ContainerCapacityKnobs {
        let is_this_tree = |tree: ContainerTreeOfTheCapacityInvariants| tree == self;
        singlefs_checker::walk::ContainerCapacityKnobs {
            tombstone: is_this_tree(ContainerTreeOfTheCapacityInvariants::Tombstone).then(|| {
                singlefs_checker::walk::TombstoneContainerCapacityKnob::new(records_per_container)
                    .expect("1..=815")
            }),
            lineage: is_this_tree(ContainerTreeOfTheCapacityInvariants::Lineage).then(|| {
                singlefs_checker::walk::LineageContainerCapacityKnob::new(records_per_container)
                    .expect("1..=1359")
            }),
            retirement: is_this_tree(ContainerTreeOfTheCapacityInvariants::Retirement).then(|| {
                singlefs_checker::walk::RetirementContainerCapacityKnob::new(records_per_container)
                    .expect("1..=959")
            }),
        }
    }

    /// 写入口：只这一棵压到 `records_per_container` 条一片，另两棵与节点照产品。
    fn write_capacities(
        self,
        records_per_container: u64,
    ) -> singlefs_core::container_tree::ContainerTreeCapacities {
        let is_this_tree = |tree: ContainerTreeOfTheCapacityInvariants| tree == self;
        singlefs_core::container_tree::ContainerTreeCapacities {
            tombstone_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Tombstone) {
                singlefs_core::tombstone_tree::TombstoneContainerCapacity::CappedForTests(
                    singlefs_core::tombstone_tree::TombstoneRecordsPerContainerCappedForTests::new(
                        records_per_container,
                    )
                    .expect("1..=815"),
                )
            } else {
                singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat
            },
            lineage_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Lineage) {
                singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
                    singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
                        records_per_container,
                    )
                    .expect("1..=1359"),
                )
            } else {
                singlefs_core::lineage_tree::LineageContainerCapacity::FromTheContainerFormat
            },
            retirement_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Retirement)
            {
                singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                    singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
                        records_per_container,
                    )
                    .expect("1..=959"),
                )
            } else {
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat
            },
            node_entries:
                singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
        }
    }

    /// 这一版这一棵每片容器的记录数，按叶序。
    fn record_counts_of_each_container(self, output: &TransactionOutput) -> Vec<usize> {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => output
                .tombstone_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            ContainerTreeOfTheCapacityInvariants::Lineage => output
                .lineage_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            ContainerTreeOfTheCapacityInvariants::Retirement => output
                .retirement_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
        }
    }
}

/// 在这一棵树里写 `records` 条记录、写入口只这一棵压到 `records_per_container` 条一片（另两棵照产品）：
/// 墓碑树——建 `records` 个空 inode（2 号起），逐个 unlink 到底，每个一条墓碑；
/// 谱系树——建 1 个空 inode，分配器装上谱系旋钮，挂着回退到第一个文件那一版 `records` 次，每次一条谱系记录；
/// 退役树——再建 232 个空 inode（叶 [1..=233]、[234]），unlink 234 到底（最右那片删空、退役一条），之后每次建 1 个（最左那片满了，开新叶）、
/// unlink 它到底（又退役一条），共 `records` 次。
fn pool_with_records_in_one_container_tree_under_its_knob(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    records_per_container: u64,
    records: u64,
) -> BuiltPool {
    pool_with_records_in_one_container_tree_written_under_these_knobs(
        tree,
        tag,
        &vec![records_per_container; usize::try_from(records).expect("装得进 usize")],
    )
}

/// 同 [`pool_with_records_in_one_container_tree_under_its_knob`]，第 k 条记录写的时候写入口只这一棵压到 `records_per_container_of_each_record[k]` 条一片
/// （另两棵照产品）：旋钮在一份镜像的历史中途换值。写侧每一步都照当时的旋钮「满了才关」，换小之后下一条开新片、换大之后接着往开放的那片里装，
/// 写得出「不是最右的那片少一条」（J3）。
fn pool_with_records_in_one_container_tree_written_under_these_knobs(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    records_per_container_of_each_record: &[u64],
) -> BuiltPool {
    let records =
        u64::try_from(records_per_container_of_each_record.len()).expect("记录数装得进 u64");
    let pool = match tree {
        ContainerTreeOfTheCapacityInvariants::Tombstone => {
            let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, records);
            // 迭代上界：`records`。
            for (inode, records_per_container) in
                (2..2 + records).zip(records_per_container_of_each_record)
            {
                unlink_an_empty_inode_to_the_end_under_the_capacities(
                    &mut pool,
                    inode,
                    tree.write_capacities(*records_per_container),
                );
            }
            pool
        }
        ContainerTreeOfTheCapacityInvariants::Lineage => {
            let mut pool = build_pool(tag);
            let first_file_version = pool.output.clone();
            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            // 迭代上界：`records`。
            for records_per_container in records_per_container_of_each_record {
                let capacities = tree.write_capacities(*records_per_container);
                pool.allocator
                    .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
                        batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
                        tombstone_container_capacity: capacities.tombstone_container,
                        lineage_container_capacity: capacities.lineage_container,
                        retirement_container_capacity: capacities.retirement_container,
                        container_tree_node_entries: capacities.node_entries,
                    });
                common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
            }
            pool
        }
        ContainerTreeOfTheCapacityInvariants::Retirement => {
            let leaf_records = singlefs_format::INODE_LEAF_RECORDS;
            let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, leaf_records);
            // 迭代上界：`records`。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
            for (retirement_index, records_per_container) in
                records_per_container_of_each_record.iter().enumerate()
            {
                if retirement_index > 0 {
                    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(
                        &mut pool, 1,
                    );
                }
                let inode = pool.output.inode_number_watermark() - 1;
                unlink_an_empty_inode_to_the_end_under_the_capacities(
                    &mut pool,
                    inode,
                    tree.write_capacities(*records_per_container),
                );
            }
            pool
        }
    };
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output)
            .iter()
            .sum::<usize>(),
        usize::try_from(records).expect("装得进"),
        "{tree:?} 里一共 {records} 条记录"
    );
    pool
}

/// 同上，再接着建 30 次空 inode，把写容器树的那几版都转出回退候选集：留在候选集里的每一版都是同一棵容器树（照抄、不重写），
/// 改坏最新那一版的容器树就是改坏候选集里每一版的，没有一版还拿着没改坏的身份或记录。交回没改坏的镜像，带旋钮值的入口上一条都不红、
/// 那一条（I-9.22 / I-9.37）真被评估过且成立。
fn image_with_records_in_one_container_tree_and_the_root_ring_turned_over(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    records: u64,
) -> MemoryPool {
    let mut pool = pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, records);
    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        vec![1; usize::try_from(records).expect("装得进")],
        "{tree:?} 一片 1 条：{records} 片"
    );
    let image = pool.memory_pool();
    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        &image,
        tree.checker_knobs(1),
    );
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "改之前（{tree:?}，带旋钮值）一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict_among(&verdicts, tree.knob_capacity_invariant()),
        InvariantVerdict::Holds,
        "改之前 {} 真被评估过且成立",
        tree.knob_capacity_invariant()
    );
    image
}

/// 最新的根那一版这一棵树（根层级 1、直接管容器）每片容器的槽，按叶序。
fn container_slots_of_the_newest_tree(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
) -> Vec<singlefs_core::address::SlotNumber> {
    let root_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(image, tree.tree_kind()).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("容器树根解得开");
    assert_eq!(root.level, 1, "{tree:?} 的根直接管容器");
    root.entries
        .iter()
        .map(|entry| {
            common_pointer_head_corruption::the_slot_of(
                &singlefs_core::pointer::NodePointer::read_from(
                    &mut singlefs_core::bytes::ByteReader::at(
                        entry,
                        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
                    ),
                ),
            )
        })
        .collect()
}

/// 最新的根那一版这一棵树每片容器头里的容器号与根里指它那条条目的身份容器号一起改成 `container_numbers_in_leaf_order` 里的号
/// （身份照旧与子头相符；分隔 key 与子指针不动），改过的单元重封、新的整单元校验和沿引用链补到根。
fn renumber_the_containers_of_the_newest_tree(
    image: &mut MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
    container_numbers_in_leaf_order: &[u64],
) {
    let container_slots = container_slots_of_the_newest_tree(image, tree);
    assert_eq!(
        container_slots.len(),
        container_numbers_in_leaf_order.len(),
        "{tree:?} 的容器片数与要写的号数相同"
    );
    for (entry_index, (container_slot, container_number)) in container_slots
        .into_iter()
        .zip(container_numbers_in_leaf_order)
        .enumerate()
    {
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            image,
            container_slot,
            common_pointer_head_corruption::data_unit_bytes(),
            |container| {
                set_u64(
                    container,
                    PACKED_UNIT_CONTAINER_NUMBER_OFFSET,
                    *container_number,
                )
            },
        );
        change_an_entry_of_the_root_of_kind(image, tree.tree_kind(), entry_index, |entry| {
            set_u64(
                entry,
                CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
                *container_number,
            );
        });
    }
}

/// 最新的根那一版这一棵树最右那片容器清空：记录数与声明长度写 0、记录区写 0（容器号、身份不动），重封、补校验和。
fn empty_the_rightmost_container_of_the_newest_tree(
    image: &mut MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
) {
    let rightmost_slot = *container_slots_of_the_newest_tree(image, tree)
        .last()
        .expect("至少一片容器");
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        rightmost_slot,
        common_pointer_head_corruption::data_unit_bytes(),
        |container| {
            assert_eq!(
                u16::from_le_bytes([
                    container[PACKED_UNIT_RECORD_COUNT_OFFSET],
                    container[PACKED_UNIT_RECORD_COUNT_OFFSET + 1]
                ]),
                1,
                "最右那片容器一条记录"
            );
            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 0);
            set_u16(container, PACKED_UNIT_DECLARED_LENGTH_OFFSET, 0);
            container[PACKED_UNIT_RECORDS_START..].fill(0);
        },
    );
}

/// 带旋钮值的入口（只这一棵装一片 1 条）上只红这一棵的那一条（I-9.22 / I-9.37），违例说明里有 `detail_fragment`。
fn assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
    detail_fragment: &str,
) {
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
        image,
        tree,
        1,
        detail_fragment,
    );
}

/// 带旋钮值的入口（只这一棵装一片 `records_per_container` 条）上只红这一棵的那一条（I-9.22 / I-9.37），违例说明里有 `detail_fragment`。
fn assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
    records_per_container: u64,
    detail_fragment: &str,
) {
    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        image,
        tree.checker_knobs(records_per_container),
    );
    assert_eq!(
        violated_invariants(&verdicts),
        [tree.knob_capacity_invariant()],
        "{tree:?}（旋钮值 {records_per_container}）：只红 {}：{verdicts:?}",
        tree.knob_capacity_invariant()
    );
    assert!(
        matches!(
            verdict_among(&verdicts, tree.knob_capacity_invariant()),
            InvariantVerdict::Violated(detail)
                if detail.contains(tree.name_in_the_violation()) && detail.contains(detail_fragment)
        ),
        "{tree:?}：违例说明写出「{detail_fragment}」：{:?}",
        verdict_among(&verdicts, tree.knob_capacity_invariant())
    );
}

/// 攻方 `mut-w-gap2` 那一形（写侧下一个号 = 最右号 + 2）：两片容器 1、2 号改成 1、3 号。
fn assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[1, 3]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序容器号不是逐片加一：容器 1 之后是容器 3",
    );
}

/// 攻方 `mut-w-start2` 那一形（零根时的号写成 2）：两片容器 1、2 号改成 2、3 号。
fn assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[2, 3]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序第一片容器的容器号是 2，不是 1",
    );
}

/// 攻方 `mut-w-eager` 那一形（满了就先开下一片、最右多一片 0 条）：三片各 1 条，最右那片清空。
fn assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 3);
    empty_the_rightmost_container_of_the_newest_tree(&mut image, tree);
    match tree {
        ContainerTreeOfTheCapacityInvariants::Tombstone
        | ContainerTreeOfTheCapacityInvariants::Lineage => {
            assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
                &image,
                tree,
                "容器 3 一条记录都没有",
            );
        }
        ContainerTreeOfTheCapacityInvariants::Retirement => {
            // 抹空的那片退役容器里那条退役记录没了：扫描重建按退役树认领叶容器，
            // 那条记录点名的叶容器最后那一版失去认领、它的记录被扫描重建收进
            // inode 记录集合，I-1.13 跟着红——`scan_rebuild_reads_lineage_and_retirement_records.rs` 抹掉退役记录那个必红世界的同一形。
            let verdicts =
                singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
                    &image,
                    tree.checker_knobs(1),
                );
            assert_eq!(
                violated_invariants(&verdicts),
                [
                    tree.knob_capacity_invariant(),
                    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX
                ],
                "{tree:?}（旋钮值 1）：红 {} 与 I-1.13：{verdicts:?}",
                tree.knob_capacity_invariant()
            );
            assert!(
                matches!(
                    verdict_among(&verdicts, tree.knob_capacity_invariant()),
                    InvariantVerdict::Violated(detail)
                        if detail.contains(tree.name_in_the_violation())
                            && detail.contains("容器 3 一条记录都没有")
                ),
                "{tree:?}：违例说明写出「容器 3 一条记录都没有」：{:?}",
                verdict_among(&verdicts, tree.knob_capacity_invariant())
            );
        }
    }
}

/// 攻方 `mut-w-same` 那一形（下一个号 = 最右号）：两片容器 1、2 号改成 1、1 号。
fn assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[1, 1]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序容器号不是逐片加一：容器 1 之后是容器 1",
    );
}

#[test]
fn tombstone_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-numbered-one-and-three",
    );
}

#[test]
fn lineage_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-one-and-three",
    );
}

#[test]
fn retirement_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-one-and-three",
    );
}

#[test]
fn tombstone_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-numbered-two-and-three",
    );
}

#[test]
fn lineage_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-two-and-three",
    );
}

#[test]
fn retirement_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-two-and-three",
    );
}

#[test]
fn tombstone_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-with-an-empty-rightmost-one",
    );
}

#[test]
fn lineage_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-with-an-empty-rightmost-one",
    );
}

#[test]
fn retirement_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-with-an-empty-rightmost-one",
    );
}

/// G3 ①：谱系树两片同号（墓碑树那一形是 `two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant`）。
#[test]
fn lineage_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-one-and-one",
    );
}

/// G3 ①：退役树两片同号。
#[test]
fn retirement_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-one-and-one",
    );
}

/// H3（旋钮支那一形；正推 K2.2）：两片容器 1、2 号改成 0、1 号——
/// 相邻加一成立、每片 1 条，只「沿叶序第一片容器号是 1」那句不成立（判成「第一片 ≤ 1」就放过它）。
fn assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[0, 1]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序第一片容器的容器号是 0，不是 1",
    );
}

#[test]
fn tombstone_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-numbered-zero-and-one",
    );
}

#[test]
fn lineage_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-zero-and-one",
    );
}

#[test]
fn retirement_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-zero-and-one",
    );
}

/// H1：
/// 只这一棵在旋钮 1 下写出两片各 1 条（另两棵照产品）。写侧在旋钮 K 下装满 K 条才关（D18（块里携带什么信息） 已定项 24 第 3 条「满了才关；
/// 关闭路径加只供测试的容量旋钮」），旋钮 2、3 写不出「不是最右的那片只装 1 条」。带旋钮值的入口：报写的时候那个旋钮值 1 一条不红、
/// 这一棵的那一条（I-9.22 / I-9.37）判过而成立；报旋钮 2、3——每片都不超旋钮值、容器号从 1 逐片加一、每片至少一条，只「不是最右的那片装满
/// 旋钮值」那句不成立，只红这一棵的那一条。
fn assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let pool = pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, 2);
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [1, 1],
        "{tree:?} 两片各 1 条"
    );
    let image = pool.memory_pool();
    let verdicts_given_the_written_knob =
        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
            &image,
            tree.checker_knobs(1),
        );
    assert_eq!(
        violated_invariants(&verdicts_given_the_written_knob),
        Vec::<&str>::new(),
        "{tree:?}：报写的时候那个旋钮值 1 一条不红：{verdicts_given_the_written_knob:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts_given_the_written_knob,
            tree.knob_capacity_invariant()
        ),
        InvariantVerdict::Holds,
        "{tree:?}：报旋钮值 1 时 {} 判过而成立",
        tree.knob_capacity_invariant()
    );
    // 迭代上界：两个旋钮值。
    for larger_knob in [2, 3] {
        assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
            &image,
            tree,
            larger_knob,
            &format!("不是最右的容器 1 装了 1 条，不是旋钮值 {larger_knob}"),
        );
    }
}

#[test]
fn two_tombstone_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
) {
    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-closed-before-a-larger-knob",
    );
}

#[test]
fn two_lineage_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
) {
    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-closed-before-a-larger-knob",
    );
}

#[test]
fn two_retirement_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
) {
    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-closed-before-a-larger-knob",
    );
}

/// J3 那几张盘的旋钮值：写的时候在 2 与 1 之间换，判的时候报 2。
const KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES: u64 = 2;

/// J3 那几张盘怎么造：一片几条、按叶序每片记录数、
/// 少一条的那片的容器号、判之前要不要把根环转过一圈。
struct ContainersUnderChangingKnobs {
    records_per_container_of_each_record: &'static [u64],
    record_counts_in_leaf_order: &'static [usize],
    short_container: u64,
    root_ring: RootRingBeforeTheJudgement,
}

/// 判之前根环转没转过一圈。
#[derive(Clone, Copy, Debug)]
enum RootRingBeforeTheJudgement {
    /// 写容器树的那几版都还在回退候选集里，池级 checker 逐版判。
    KeepsTheVersionsThatWroteTheContainerTree,
    /// 再建 30 次空 inode（`common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes`）：留在候选集里的每一版都是写完的那棵容器树。
    TurnedOverByCreatingInodes,
}

/// 中间那片少一条：旋钮 2、2、1、1 写出 [2, 1, 1]（攻方 `k4_middle_short_container` 那一形）。只判叶序第一片（攻方 `h1-first-only`）、
/// 只判最左片（正推 MB2）在它上面照绿，正确判法红在容器 2。
const MIDDLE_CONTAINER_SHORT: ContainersUnderChangingKnobs = ContainersUnderChangingKnobs {
    records_per_container_of_each_record: &[2, 2, 1, 1],
    record_counts_in_leaf_order: &[2, 1, 1],
    short_container: 2,
    root_ring: RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree,
};

/// 倒数第二片之前全满、倒数第二片少一条：旋钮 2、2、2、2、1、1 写出 [2, 2, 1, 1]。只判叶序第一片、只判最左片照绿，正确判法红在容器 3。
const ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER: ContainersUnderChangingKnobs =
    ContainersUnderChangingKnobs {
        records_per_container_of_each_record: &[2, 2, 2, 2, 1, 1],
        record_counts_in_leaf_order: &[2, 2, 1, 1],
        short_container: 3,
        root_ring: RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree,
    };

/// 少一条的那片之后还有一片满的：旋钮 2、2、1、1、2、1 写出 [2, 1, 2, 1]，再把根环转过一圈。只判倒数第二片（正推 MC）在它上面照绿——
/// 写的中途有一版是 [2, 1, 1]，容器 2 在那一版是倒数第二片，那一版留在候选集里时只判倒数第二片也红，所以要转出去；
/// 只判叶序第一片、只判最左片同样照绿，正确判法红在容器 2。
const A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: ContainersUnderChangingKnobs =
    ContainersUnderChangingKnobs {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 1],
        short_container: 2,
        root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,
    };

/// 少一条的那片离最右三片：旋钮 2、2、1、1、2、2、2、1 写出 [2, 1, 2, 2, 1]，再把根环转过一圈（L2）。上面三张盘少的那片离最右
/// 都不超过两片，只判最右之前两片（`w-last-two`）在它们上面与正确判法同判；
/// 这一张的容器 2 在倒数第三片之前，那一形不判它、照绿，正确判法红在容器 2。写的中途 [2, 1, 1]、[2, 1, 2]、[2, 1, 2, 1] 那几版里容器 2
/// 离最右不超过两片，那几版留在候选集里时那一形也红，所以要转出去。
const A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST: ContainersUnderChangingKnobs =
    ContainersUnderChangingKnobs {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 2, 1],
        short_container: 2,
        root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,
    };

/// J3：只这一棵按 `layout` 逐条换旋钮写出三片以上的容器（另两棵照产品），带旋钮值的入口报这一棵旋钮 2：每片都不超旋钮值、
/// 容器号从 1 逐片加一、每片至少一条，只「不是最右的容器 `short_container` 装了 1 条，不是旋钮值 2」不成立，只红这一棵的那一条（I-9.22 / I-9.37）。
/// H1 的会红用例都只有两片容器，两片时只判最左片、只判倒数第二片、只判叶序第一片与正确写法等价，这几张盘把它们分开。
fn assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    layout: &ContainersUnderChangingKnobs,
) {
    let mut pool = pool_with_records_in_one_container_tree_written_under_these_knobs(
        tree,
        tag,
        layout.records_per_container_of_each_record,
    );
    match layout.root_ring {
        RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree => {}
        RootRingBeforeTheJudgement::TurnedOverByCreatingInodes => {
            common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
        }
    }
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        layout.record_counts_in_leaf_order,
        "{tree:?} 按叶序每片的记录数"
    );
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
        &pool.memory_pool(),
        tree,
        KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES,
        &format!(
            "不是最右的容器 {} 装了 1 条，不是旋钮值 {KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES}",
            layout.short_container
        ),
    );
}

#[test]
fn tombstone_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-one-and-one-records",
        &MIDDLE_CONTAINER_SHORT,
    );
}

#[test]
fn lineage_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-one-and-one-records",
        &MIDDLE_CONTAINER_SHORT,
    );
}

#[test]
fn retirement_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-one-and-one-records",
        &MIDDLE_CONTAINER_SHORT,
    );
}

#[test]
fn tombstone_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-two-one-and-one-records",
        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
    );
}

#[test]
fn lineage_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-two-one-and-one-records",
        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
    );
}

#[test]
fn retirement_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-two-one-and-one-records",
        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
    );
}

#[test]
fn tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-one-two-and-one-records",
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
    );
}

#[test]
fn lineage_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-one-two-and-one-records",
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
    );
}

#[test]
fn retirement_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-one-two-and-one-records",
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
    );
}

#[test]
fn tombstone_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-one-two-two-and-one-records",
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
    );
}

#[test]
fn lineage_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-one-two-two-and-one-records",
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
    );
}

#[test]
fn retirement_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-one-two-two-and-one-records",
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
    );
}

/// G3 ③：退役容器压到一片 2 条写出两条退役记录（一片 2 条），带旋钮值的入口报退役旋钮 1：那片装了 2 条、多于旋钮值，只红 I-9.37；
/// 报写的时候那个旋钮值 2 一条不红。
#[test]
fn a_retirement_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
) {
    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
    let pool = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-retirement-container-past-the-knob",
        2,
        2,
    );
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [2],
        "退役树一片容器 2 条"
    );
    let image = pool.memory_pool();
    let verdicts_given_the_written_knob =
        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
            &image,
            tree.checker_knobs(2),
        );
    assert_eq!(
        violated_invariants(&verdicts_given_the_written_knob),
        Vec::<&str>::new(),
        "报写的时候那个旋钮值 2：一条不红：{verdicts_given_the_written_knob:?}"
    );
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "装了 2 条，多于旋钮值 1",
    );
}

/// G3 ②：产品容量下退役 960 片叶（每次建一个号开新叶、再删到底）：退役树 [959, 1]，同时 960 条墓碑把墓碑树写成 [815, 145]。
/// 产品入口一条不红，I-9.36（退役树：第一片装满 959、最右那片不要求满）与 I-9.21 真被评估过且成立——959 与谱系的 1359 对调、
/// 最右那片也要求满，这里就红。
#[test]
fn nine_hundred_sixty_retirements_at_product_capacity_fill_every_retirement_container_but_the_rightmost_and_redden_nothing(
) {
    const RETIREMENTS_THAT_OPEN_A_SECOND_CONTAINER: u64 = 960;
    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
    let leaf_records = singlefs_format::INODE_LEAF_RECORDS;
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(
        "known-good-retirement-containers-at-product-capacity",
        leaf_records - 1,
    );
    // 迭代上界：常量 960。每一轮建一个号（最左那片叶满了，开新叶），再删到底：那片叶删空、退役一条，墓碑一条。
    for _ in 0..RETIREMENTS_THAT_OPEN_A_SECOND_CONTAINER {
        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
        let inode = pool.output.inode_number_watermark() - 1;
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            singlefs_core::container_tree::ContainerTreeCapacities::FROM_THE_FORMAT,
        );
    }
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [959, 1],
        "退役树第一片装满 959、第二片 1 条"
    );
    assert_eq!(
        ContainerTreeOfTheCapacityInvariants::Tombstone
            .record_counts_of_each_container(&pool.output),
        [815, 145],
        "墓碑树第一片装满 815、第二片 145 条"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "产品容量的多容器合法镜像：一条不红：{verdicts:?}"
    );
    for invariant in [
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    ] {
        assert_eq!(
            verdict_among(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant} 真被评估过且成立"
        );
    }
}

/// G3 ②：产品容量下挂着回退 1360 次（每次退到紧前那一版）：谱系树 [1359, 1]。产品入口一条不红，I-9.36（谱系树：第一片装满 1359、
/// 最右那片不要求满）真被评估过且成立——1359 与退役的 959 对调、最右那片也要求满，这里就红。
#[test]
fn one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing(
) {
    const ROLLBACKS_THAT_OPEN_A_SECOND_CONTAINER: u64 = 1360;
    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
    let mut pool = build_pool("known-good-lineage-containers-at-product-capacity");
    let mut previous = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    // 迭代上界：常量 1360。每一次退到紧前那一版（两版轮流），一次一条谱系记录。
    for _ in 0..ROLLBACKS_THAT_OPEN_A_SECOND_CONTAINER {
        let target = previous;
        previous = pool.output.clone();
        common_deletion_in_progress::roll_back_to(&mut pool, &target);
    }
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [1359, 1],
        "谱系树第一片装满 1359、第二片 1 条"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "产品容量的多容器合法镜像：一条不红：{verdicts:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts,
            NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
        ),
        InvariantVerdict::Holds,
        "I-9.36 真被评估过且成立"
    );
}

/// G3 ④ 判不了那一句的理由（`walk.rs` 的 `DeletionScanGap::reason`）里点得出是哪一形的那一段。
const NOT_JUDGED_LINEAGE_TREE_UNREADABLE: &str = "谱系树读不全";

const NOT_JUDGED_RETIREMENT_TREE_UNREADABLE: &str = "退役树读不全";

const NOT_JUDGED_TREE_TABLE_UNREADABLE: &str = "树表读不出";

/// 带旋钮值的入口上 I-9.37 报的部分没判里逐条列出的没判到的部分（判出违例、成立或不适用都不是这一形）：这几份坏镜像都是射程里有一版读不出，
/// C701（别的判法静默跳过读不出的部分），一律报部分没判。
fn reason_the_knob_capacity_invariant_is_not_judged(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
) -> String {
    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        image,
        tree.checker_knobs(1),
    );
    match verdict_among(&verdicts, tree.knob_capacity_invariant()) {
        InvariantVerdict::PartiallyJudged(unjudged) => unjudged,
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "{tree:?}：{} 射程里有一版读不出，要报部分没判，实际 {other:?}",
            tree.knob_capacity_invariant()
        ),
    }
}

/// G3 ④ 谱系树读不全：旋钮下回退两次（谱系 [1, 1]），第二片谱系容器两份全零（不补引用链）。回退一次那一版的谱系树读得全、照判；
/// 最新那一版读不全，I-9.37 报射程里判不了、不报成立。
#[test]
fn an_unreadable_lineage_container_leaves_the_knob_capacity_invariant_not_judged() {
    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
    let mut image = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-lineage-container-unreadable",
        1,
        2,
    )
    .memory_pool();
    let second_container_slot = container_slots_of_the_newest_tree(&image, tree)[1];
    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
        &mut image,
        second_container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
    assert!(
        reason.contains(NOT_JUDGED_LINEAGE_TREE_UNREADABLE),
        "谱系树读不全的那一版 I-9.37 判不了：{reason}"
    );
}

/// G3 ④ 退役树读不全：旋钮下退役两次（退役 [1, 1]），第二片退役容器两份全零。I-9.37 报射程里判不了。
#[test]
fn an_unreadable_retirement_container_leaves_the_knob_capacity_invariant_not_judged() {
    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
    let mut image = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-retirement-container-unreadable",
        1,
        2,
    )
    .memory_pool();
    let second_container_slot = container_slots_of_the_newest_tree(&image, tree)[1];
    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
        &mut image,
        second_container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
    assert!(
        reason.contains(NOT_JUDGED_RETIREMENT_TREE_UNREADABLE),
        "退役树读不全的那一版 I-9.37 判不了：{reason}"
    );
}

/// G3 ④ 整版树表读不出：旋钮下回退一次（谱系 [1]），再建一个号；回退那一版的树表两份全零（不补引用链）。那一版谱系与退役都判不了，
/// I-9.37 报射程里判不了；最新那一版照判。
#[test]
fn an_unreadable_tree_table_leaves_the_knob_capacity_invariant_not_judged() {
    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
    let mut pool = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-tree-table-unreadable-under-the-lineage-knob",
        1,
        1,
    );
    let rolled_back_once_tree_table_slot = pool.output.unit(TransactionUnit::TreeTable).slot;
    let rolled_back_once_txg = pool.output.root.checkpoint_txg;
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let mut image = pool.memory_pool();
    assert_ne!(
        common_deletion_in_progress::tree_table_slot_of_the_newest_root(&image),
        rolled_back_once_tree_table_slot,
        "最新那一版（txg {} 之后又建了一个号）的树表另写一份",
        rolled_back_once_txg.0
    );
    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
        &mut image,
        rolled_back_once_tree_table_slot,
        common_pointer_head_corruption::node_bytes(),
    );
    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
    assert!(
        reason.contains(NOT_JUDGED_TREE_TABLE_UNREADABLE),
        "树表读不出的那一版 I-9.37 判不了：{reason}"
    );
}

/// 旧的墓碑旋钮入口（`check_pool_image_given_the_tombstone_container_capacity_knob`）只给墓碑旋钮：谱系树与退役树当没装旋钮。
/// 这一棵在旋钮下写成两片各 1 条，交给旧的墓碑旋钮入口（墓碑旋钮给产品容量 815：退役那份历史 unlink 两次写的两条墓碑同在一片，旋钮给得再小
/// I-9.22 会跟着红）：这一棵按产品容量判，只红 I-9.36、说明点名这一棵；I-9.37 报不适用。
fn assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let image =
        pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, 2).memory_pool();
    let verdicts =
        singlefs_checker::walk::check_pool_image_given_the_tombstone_container_capacity_knob(
            &image,
            Some(
                singlefs_checker::walk::TombstoneContainerCapacityKnob::new(
                    singlefs_format::TOMBSTONE_CONTAINER_RECORDS,
                )
                .expect("1..=815"),
            ),
        );
    assert_eq!(
        violated_invariants(&verdicts),
        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "{tree:?}：旧的墓碑旋钮入口按产品容量判这一棵，只红 I-9.36：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_among(
                &verdicts,
                NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
            ),
            InvariantVerdict::Violated(detail) if detail.contains(tree.name_in_the_violation())
        ),
        "{tree:?}：I-9.36 的说明点名这一棵"
    );
    assert!(
        matches!(
            verdict_among(
                &verdicts,
                LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "{tree:?}：I-9.37 报不适用"
    );
}

/// G3 ⑤：旧的墓碑旋钮入口给谱系那一棵 `None`。
#[test]
fn the_old_tombstone_knob_entry_judges_the_lineage_tree_by_its_product_capacity() {
    assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-old-tombstone-knob-entry-lineage",
    );
}

/// G3 ⑤：旧的墓碑旋钮入口给退役那一棵 `None`。
#[test]
fn the_old_tombstone_knob_entry_judges_the_retirement_tree_by_its_product_capacity() {
    assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-old-tombstone-knob-entry-retirement",
    );
}
