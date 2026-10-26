//! harness 测试文件 `crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{file_content, parameters, FILE_BYTES};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_tree_split::{
    capacities, propagate_the_new_checksum, shape_text,
    write_unit_to_the_same_slot_on_both_devices, TreeSplitPool,
};
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::allocation_record_tree::{
    AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::allocator::Placement;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::code_two_tree::{CodeTwoTreeNodeContents, CodeTwoTreeNodeOrigin};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::transaction::{
    multi_level_tree_of_role, MultiLevelCodeTwoTree, PoolVersion, TransactionOutput,
    TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_harness::memory_pool::MemoryPool;

/// 两棵树都压小：记账树 15 行按叶 4 条长成两层（四片叶），中央映射树按叶 3 条长成两层。
pub const SMALL_ACCOUNTING: (usize, usize) = (4, 3);

pub const SMALL_CENTRAL_MAPPING: (usize, usize) = (3, 3);

/// 这一版里某棵多层码 2 树的全部节点单元，按 bump 次序（`units` 里记账树 / 中央映射树那一族角色）。
pub fn node_units(
    output: &TransactionOutput,
    tree: MultiLevelCodeTwoTree,
) -> Vec<&singlefs_core::transaction::PublishedUnit> {
    output
        .units
        .iter()
        .filter(|unit| multi_level_tree_of_role(unit.identity) == Some(tree))
        .collect()
}

/// ① 第一个文件版本在压小的容量下写出两棵多层树：每个节点头里的层级等于它在树里的层级、区间是子树覆盖区间、
/// 出生序号按 bump 次序（先叶后根、同层按 key 升序）从 0 发，树高从根节点头现读；冷走读读回文件、池级 checker 全绿、
/// 挂载态把整棵映射树读进来、从盘上重建出来的两棵树与发布写出的逐节点相同。
pub fn under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts(
    images: &mut PoolCheckerImages,
) {
    let pool = TreeSplitPool::with_the_first_file_version_under(capacities(
        SMALL_ACCOUNTING,
        SMALL_CENTRAL_MAPPING,
    ));
    let output = &pool.output;
    for tree in [
        MultiLevelCodeTwoTree::Accounting,
        MultiLevelCodeTwoTree::CentralMapping,
    ] {
        let version = output.multi_level_tree(tree);
        assert!(
            version.shape.height() >= 2,
            "{tree:?} 压小之后是多层：{}",
            shape_text(version)
        );
        assert_eq!(
            output.height_read_from_the_root_node_header(tree),
            version.shape.height(),
            "{tree:?}：树高从根节点头里的层级现读，等于形状的高"
        );
        let units = node_units(output, tree);
        assert_eq!(
            units.len(),
            version.shape.nodes().len(),
            "{tree:?}：每个节点一个单元"
        );
        for (sequence, ((node, unit), pointer)) in version
            .shape
            .nodes()
            .iter()
            .zip(&units)
            .zip(&version.pointers)
            .enumerate()
        {
            let header = parse_index_node(&unit.bytes).expect("节点解得开");
            assert_eq!(
                header.level, node.position.level,
                "{tree:?} {:?}：头里的层级是它在树里的层级",
                node.position
            );
            assert_eq!(
                header.birth_sequence.0,
                u32::try_from(sequence).expect("出生序号"),
                "{tree:?}：出生序号按 bump 次序从 0 发（D19 已定项 9：树内先叶后根、同层按 key 升序）"
            );
            assert_eq!(pointer.birth_sequence, header.birth_sequence);
            assert_eq!(
                crc32_castagnoli(&unit.bytes),
                pointer.locations[0].unit_checksum,
                "{tree:?}：指针罩着这个单元的整单元校验和"
            );
            let (smallest, largest) = version
                .shape
                .key_range_of_the_subtree(node.position)
                .expect("节点不空");
            assert_eq!(
                (
                    header.smallest_key.as_slice(),
                    header.largest_key.as_slice()
                ),
                (smallest.bytes(), largest.bytes()),
                "{tree:?} {:?}：头里的 key 区间是子树覆盖区间（D18 已定项 2）",
                node.position
            );
        }
    }
    // bump 次序：记账树的节点先叶后根、根是 `AccountingTree`，之后中央映射树同样，树表最末。
    let tree_roles: Vec<TransactionUnit> = output
        .rewritten
        .iter()
        .copied()
        .filter(|identity| multi_level_tree_of_role(*identity).is_some())
        .collect();
    let accounting_nodes = output.accounting_tree.node_count();
    assert_eq!(
        tree_roles[accounting_nodes - 1],
        TransactionUnit::AccountingTree,
        "记账树的根在它那一族的最末"
    );
    assert_eq!(
        tree_roles.last(),
        Some(&TransactionUnit::MappingTree),
        "中央映射树的根在它那一族的最末"
    );
    assert_eq!(output.rewritten.last(), Some(&TransactionUnit::TreeTable));
    assert_eq!(
        output.mapping_keys.len(),
        4 + output.allocation_record_tree.nodes.len() + accounting_nodes,
        "数据单元、extent 根、inode 叶、inode 根各一条，分配记录树（按位置寻址，D8（核心索引结构） 已定项 14）与记账树每个节点一条"
    );

    let image = pool.memory_pool();
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (output.root.instance, output.root.checkpoint_txg),
            content: file_content(),
        },
        "冷走读沿两棵多层树读回文件"
    );
    images.hand_out(
        "池级 checker 全绿",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let mounted = mount_read_only(&image).expect("整池只读挂载");
    assert_eq!(
        mounted
            .mounted
            .central_mapping_tree_reads_at_open()
            .node_reads,
        u64::try_from(output.central_mapping_tree.node_count()).expect("节点数"),
        "挂载态把整棵映射树读进来"
    );
    let read_back = mounted
        .mounted
        .open_file(&image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件")
        .read_at(
            &image,
            FileOffsetInBytes(0),
            u64::try_from(FILE_BYTES).expect("3000"),
        )
        .expect("读整个文件");
    assert_eq!(read_back.bytes, file_content());

    let RebuiltVersion::WithFile(rebuilt) =
        rebuild_version(&image, &output.root, Some(output.record.clone())).expect("从盘上重建")
    else {
        panic!("这一版有文件");
    };
    assert_eq!(rebuilt.accounting_tree, output.accounting_tree);
    assert_eq!(rebuilt.central_mapping_tree, output.central_mapping_tree);
    assert_eq!(rebuilt.accounting_entries, output.accounting_entries);
    assert_eq!(rebuilt.mapping_keys, output.mapping_keys);
    assert_eq!(rebuilt.mapped_units, output.mapped_units);
    for tree in [
        MultiLevelCodeTwoTree::Accounting,
        MultiLevelCodeTwoTree::CentralMapping,
    ] {
        assert_eq!(
            node_units(&rebuilt, tree),
            node_units(output, tree),
            "{tree:?}：每个节点的角色、落点与字节逐个相同（重建出来的这一版多一个实例表单元，与树无关）"
        );
    }
}

/// ② 一次空发布只换分配记录树与记账树节点的映射 key：中央映射树里没碰到的叶照抄——字节、落点不动、不进这次写出的角色，
/// 角色跟着这一版的位置走——被换下的节点（上一版的分配记录树、记账树的每个节点、中央映射树改了的那条路径、树表）正好释放，
/// 照抄的那几片在分配器里仍是已分配。
pub fn an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes(
    images: &mut PoolCheckerImages,
) {
    let small = capacities(SMALL_ACCOUNTING, SMALL_CENTRAL_MAPPING);
    let mut pool = TreeSplitPool::with_the_first_file_version_under(small);
    pool.empty_publish(small);
    let before = pool.version_before_the_current_one().clone();
    let after = &pool.output;
    let plan_origins: Vec<(TransactionUnit, CodeTwoTreeNodeOrigin)> = after
        .central_mapping_tree
        .shape
        .nodes()
        .iter()
        .map(|node| {
            let role = MultiLevelCodeTwoTree::CentralMapping
                .role_of_node(node.position, &after.central_mapping_tree.shape);
            let origin = if after.rewritten.contains(&role) {
                CodeTwoTreeNodeOrigin::RewrittenThisPublish
            } else {
                let unit = after.unit(role);
                let previous_node = before
                    .central_mapping_tree
                    .shape
                    .nodes()
                    .iter()
                    .find(|previous| {
                        before
                            .unit(MultiLevelCodeTwoTree::CentralMapping.role_of_node(
                                previous.position,
                                &before.central_mapping_tree.shape,
                            ))
                            .slot
                            == unit.slot
                    })
                    .expect("没重写的节点在上一版里有同一个落点");
                CodeTwoTreeNodeOrigin::CarriedFrom(previous_node.position)
            };
            (role, origin)
        })
        .collect();
    let carried: Vec<&(TransactionUnit, CodeTwoTreeNodeOrigin)> = plan_origins
        .iter()
        .filter(|(_, origin)| matches!(origin, CodeTwoTreeNodeOrigin::CarriedFrom(_)))
        .collect();
    assert!(
        !carried.is_empty(),
        "中央映射树有叶没被这次空发布碰到（{}）",
        shape_text(&after.central_mapping_tree)
    );
    for (role, origin) in &carried {
        let CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) = origin else {
            unreachable!("上面筛过")
        };
        let previous_role = MultiLevelCodeTwoTree::CentralMapping
            .role_of_node(*previous_position, &before.central_mapping_tree.shape);
        assert_eq!(
            after.unit(*role).bytes,
            before.unit(previous_role).bytes,
            "{role:?} 照抄 {previous_role:?}：字节一个不动"
        );
        assert!(
            pool.allocator
                .record_for(
                    singlefs_core::address::DeviceIdentity(0),
                    after.unit(*role).slot
                )
                .is_some_and(|record| !record.is_released),
            "{role:?} 照抄的那一片在分配器里仍是已分配"
        );
    }
    // 被换下的：上一版的分配记录树根与根之下这次重写了的节点（按位置寻址，D8（核心索引结构） 已定项 14：这一版同一个位置的指针换了）、
    // 记账树的每个节点、中央映射树里这次没照抄的节点、树表。
    let carried_slots: Vec<u64> = carried
        .iter()
        .map(|(role, _)| after.unit(*role).slot.0)
        .collect();
    let allocation_record_tree_node_below_the_root_was_rewritten =
        |position: AllocationRecordTreeNodePosition, slot: u64| {
            after
                .allocation_record_tree
                .pointer_of(AllocationRecordTreeNode::BelowTheRoot(position))
                .is_none_or(|pointer| pointer.locations[0].slot.0 != slot)
        };
    let mut expected_released: Vec<Placement> = before
        .units
        .iter()
        .filter(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::AllocationTree | TransactionUnit::TreeTable
            ) || matches!(
                unit.identity,
                TransactionUnit::AllocationTreeNodeBelowTheRoot(position)
                    if allocation_record_tree_node_below_the_root_was_rewritten(position, unit.slot.0)
            ) || multi_level_tree_of_role(unit.identity) == Some(MultiLevelCodeTwoTree::Accounting)
                || (multi_level_tree_of_role(unit.identity)
                    == Some(MultiLevelCodeTwoTree::CentralMapping)
                    && !carried_slots.contains(&unit.slot.0))
        })
        .map(|unit| Placement {
            slot: unit.slot,
            span: unit.identity.span_slots(),
        })
        .collect();
    expected_released.sort_by_key(|placement| placement.slot);
    let mut released = after.released.clone();
    released.sort_by_key(|placement| placement.slot);
    assert_eq!(
        released, expected_released,
        "被换下的节点正好释放掉，照抄的一个不释放"
    );
    images.hand_out(
        "池级 checker 全绿",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// ③ 多层之后进程退出、重开可写挂载：从盘上重建两棵树（形状与每个节点的落点），写行与暖机按产品容量接着发——
/// 记账树整批换代、按产品容量重新长成一个节点；中央映射树只改变了的那条路径。挂载之后冷走读读回文件、池级 checker 全绿。
pub fn pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on(
    images: &mut PoolCheckerImages,
) {
    let small = capacities(SMALL_ACCOUNTING, SMALL_CENTRAL_MAPPING);
    let mut pool = TreeSplitPool::with_the_first_file_version_under(small);
    pool.empty_publish(small);
    let before_the_mount = pool.output.clone();
    let mut devices = pool.reopened_devices();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let PoolVersion::WithFile(current) = &mounted.current else {
        panic!("挂载之后的现行那一版带文件");
    };
    assert_eq!(
        current.accounting_tree.node_count(),
        1,
        "写行与暖机按产品容量发：15 行装得进一个节点，记账树整批换代之后回到根兼叶"
    );
    assert_eq!(
        mounted.output.chosen_root, before_the_mount.root,
        "所选根是多层那一版的根"
    );
    pool.devices = devices;
    let image = pool.memory_pool();
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (current.root.instance, current.root.checkpoint_txg),
            content: file_content(),
        }
    );
    images.hand_out(
        "池级 checker 全绿",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 已知坏镜像的改法：只改记账树或中央映射树的根，把根的新字节写回原槽，再把新校验和补到根槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootDamage {
    /// 根头里的层级加一：孩子的层级不再是父层级减一。
    LevelOneAboveItsChildren,
    /// 第 1 条分隔 key 抬到第 1 个孩子的最小 key 之上（仍小于第 2 个孩子的）。
    SeparatorAboveTheSmallestKeyOfItsChild,
    /// 第 1 条分隔 key 压到等于第 0 个孩子的最大 key：不再大于左邻孩子的最大 key。
    SeparatorNotAboveTheLargestKeyOfTheLeftNeighbour,
    /// 根头里的最大 key 写成最后一条分隔 key：不是子树覆盖区间。
    HeaderRangeNotCoveringTheSubtree,
    /// 根的条目宽加一个字节（记账树内部条目 108 → 109）。
    EntryWiderThanTheInternalEntryFieldTable,
}

/// 在一份干净镜像上按 `damage` 改一棵树的根、补好引用链，交回改过的镜像。
pub fn image_with_the_root_damaged(
    pool: &TreeSplitPool,
    tree: MultiLevelCodeTwoTree,
    damage: RootDamage,
) -> MemoryPool {
    let output = &pool.output;
    let mut image = pool.memory_pool();
    let root_role = match tree {
        MultiLevelCodeTwoTree::Accounting => TransactionUnit::AccountingTree,
        MultiLevelCodeTwoTree::CentralMapping => TransactionUnit::MappingTree,
        MultiLevelCodeTwoTree::Deadlist => TransactionUnit::DeadlistTree,
    };
    let root_unit = output.unit(root_role);
    let root = parse_index_node(&root_unit.bytes).expect("根解得开");
    assert!(
        root.level >= 1 && root.entries.len() >= 2,
        "根是装着至少两个孩子的内部节点"
    );
    let key_width = root.key_width;
    let version = output.multi_level_tree(tree);
    let root_node = version.shape.root().expect("有根");
    let CodeTwoTreeNodeContents::Internal { children } = &root_node.contents else {
        panic!("根是内部节点");
    };
    let (_, child_zero_largest) = version
        .shape
        .key_range_of_the_subtree(children[0].child)
        .expect("孩子不空");
    let (child_one_smallest, _) = version
        .shape
        .key_range_of_the_subtree(children[1].child)
        .expect("孩子不空");
    let mut level = root.level;
    let mut largest_key = root.largest_key.clone();
    let mut entries = root.entries.clone();
    let mut entry_width = root.entry_width;
    match damage {
        RootDamage::LevelOneAboveItsChildren => level += 1,
        RootDamage::SeparatorAboveTheSmallestKeyOfItsChild => {
            // 记账 key 的最后一个字段（代 8，小端）加 2^48：比孩子的最小 key 大，仍小于下一个孩子的最小 key
            // （记账行按（统计量, 树, 盘）各一行，下一个孩子的 key 前三个字段就更大）。只用在记账树上。
            let mut raised = child_one_smallest.bytes().to_vec();
            raised[key_width - 2] = raised[key_width - 2].wrapping_add(1);
            entries[1][..key_width].copy_from_slice(&raised);
        }
        RootDamage::SeparatorNotAboveTheLargestKeyOfTheLeftNeighbour => {
            entries[1][..key_width].copy_from_slice(child_zero_largest.bytes());
        }
        RootDamage::HeaderRangeNotCoveringTheSubtree => {
            largest_key = entries[entries.len() - 1][..key_width].to_vec();
        }
        RootDamage::EntryWiderThanTheInternalEntryFieldTable => {
            entry_width += 1;
            for entry in &mut entries {
                entry.push(0);
            }
        }
    }
    let damaged_root = build_index_node(
        root.tree,
        level,
        key_width,
        &root.smallest_key,
        &largest_key,
        root.birth_txg,
        &parameters().filesystem_identifier,
        root.instance,
        root.birth_sequence,
        u16::try_from(entry_width).expect("条目宽"),
        &entries,
    );
    write_unit_to_the_same_slot_on_both_devices(&mut image, root_unit.slot.0, &damaged_root);
    let units: Vec<(u64, usize)> = output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect();
    propagate_the_new_checksum(
        &mut image,
        &units,
        (
            root_unit.slot.0,
            crc32_castagnoli(&root_unit.bytes),
            crc32_castagnoli(&damaged_root),
        ),
    );
    image
}

/// ④ checker 逐节点核多层码 2 树（I-1.1：索引节点的身份是树 ID + 层级 + key 区间，都从块头读）：记账树与中央映射树的根各改坏一处，
/// 那一格就红，说明里点得出是哪一条；改动之外的单元全补好了校验和，别的判定不先挡住。
/// 判别力：`crates/singlefs-checker/src/walk.rs` 的 `walk_code_two_subtree` 里对应那一判改成恒真，对应那一份就不红——
/// `crates/mutations.tsv` 里以「树分裂 checker」开头的几行。
pub fn the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root(
    images: &mut PoolCheckerImages,
) {
    let pool = TreeSplitPool::with_the_first_file_version_under(capacities(
        SMALL_ACCOUNTING,
        SMALL_CENTRAL_MAPPING,
    ));
    images.hand_out(
        "干净的那一份全绿",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
    let cases: [(MultiLevelCodeTwoTree, RootDamage, &str, &str); 7] = [
        (
            MultiLevelCodeTwoTree::Accounting,
            RootDamage::LevelOneAboveItsChildren,
            "I-1.1",
            "父层级减一",
        ),
        (
            MultiLevelCodeTwoTree::Accounting,
            RootDamage::SeparatorAboveTheSmallestKeyOfItsChild,
            "I-1.1",
            "分隔 key 大于这个孩子头里的最小 key",
        ),
        (
            MultiLevelCodeTwoTree::Accounting,
            RootDamage::SeparatorNotAboveTheLargestKeyOfTheLeftNeighbour,
            "I-1.1",
            "不大于左邻孩子头里的最大 key",
        ),
        (
            MultiLevelCodeTwoTree::Accounting,
            RootDamage::HeaderRangeNotCoveringTheSubtree,
            "I-1.1",
            "不是子树覆盖区间",
        ),
        (
            MultiLevelCodeTwoTree::Accounting,
            RootDamage::EntryWiderThanTheInternalEntryFieldTable,
            "I-1.10",
            "这一层的条目字段表宽度 108",
        ),
        (
            MultiLevelCodeTwoTree::CentralMapping,
            RootDamage::LevelOneAboveItsChildren,
            "I-1.1",
            "父层级减一",
        ),
        (
            MultiLevelCodeTwoTree::CentralMapping,
            RootDamage::HeaderRangeNotCoveringTheSubtree,
            "I-1.1",
            "不是子树覆盖区间",
        ),
    ];
    for (tree, damage, invariant, detail_fragment) in cases {
        let image = image_with_the_root_damaged(&pool, tree, damage);
        // invariant 要红，说明要点出 detail_fragment
        images.hand_out(
            &format!("{tree:?} {damage:?}"),
            || image.clone(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_violated_with_a_detail_containing(invariant, detail_fragment),
        );
        // 冷走读同一套读者（`code_two_tree::read_code_two_tree`）：五份都走读失败。条目宽只多不少的那一份报条目宽与字段表不符
        // （C780，用户 2026-10-18 定「宽于字段表也拒」），其余四份按 I-1.1 拒。
        let recovered = recover(&image, JournalPolicy::Consult).outcome;
        match damage {
            RootDamage::EntryWiderThanTheInternalEntryFieldTable => assert!(
                matches!(
                    recovered,
                    RecoveryOutcome::Failed {
                        failure: RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                            what: "记账树内部条目",
                            entry_bytes: 109,
                            field_table_bytes: 108,
                        },
                        ..
                    }
                ),
                "{tree:?} {damage:?}：条目比字段表宽，冷走读拒、不照读：{recovered:?}"
            ),
            RootDamage::LevelOneAboveItsChildren
            | RootDamage::SeparatorAboveTheSmallestKeyOfItsChild
            | RootDamage::SeparatorNotAboveTheLargestKeyOfTheLeftNeighbour
            | RootDamage::HeaderRangeNotCoveringTheSubtree => assert!(
                matches!(
                    recovered,
                    RecoveryOutcome::Failed {
                        failure: RecoveryFailure::InvariantViolated {
                            invariant: "I-1.1",
                            ..
                        },
                        ..
                    }
                ),
                "{tree:?} {damage:?}：冷走读按 I-1.1 拒：{recovered:?}"
            ),
        }
    }
}
