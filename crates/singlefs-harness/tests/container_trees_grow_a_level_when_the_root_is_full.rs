//! 墓碑树与谱系树根满长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」，用户定案 K-R3-3）：
//! 码 2 节点装满时末尾分裂，根满时长一层——新根与分出的右半各一个码 2，旧根一个字节不动、留在原位置当根之下的节点；
//! 只供测试的旋钮把容器与节点压小（一片 1 条、一个节点 2 条），旋钮不落盘；重建读层级 > 1 的根。池是两块 4 GiB 盘。
//! 池级 checker 从镜像外拿墓碑容器的旋钮值（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：
//! 这些镜像走带旋钮值的入口，I-9.22（测试镜像里墓碑容器不超旋钮值） 按旋钮值判；同一张镜像交给旧入口（当没装旋钮）I-9.21 红。
//! 谱系容器同样压到一片 1 条，带旋钮值的入口一并给谱系的旋钮值（I-9.37，第 53 批；不给的话回退两次写出的两片谱系容器按产品容量判 I-9.36 红）。
//! 第 56 批（G2）：三棵容器树都压到一片 1 条的一段固定种子随机历史（建号、unlink、回退），每一步交带旋钮值的入口一条不红（慢档）；
//! 第 57 批（H1）：同一段历史压到一片 2 条，旋钮下「不是最右的那片装满旋钮值」真判到多片（慢档）。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{build_pool, file_content, parameters, BuiltPool};
use common_deletion_in_progress::{
    pool_with_new_inodes, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use singlefs_checker::image::{
    InvariantVerdict, NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
};
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_container_capacity_knobs,
    check_pool_image_given_the_tombstone_container_capacity_knob, ContainerCapacityKnobs,
    LineageContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::{
    central_mapping_entries_of_one_lineage_container_cow,
    checkpoint_cost_of_one_lineage_container_cow, MetadataBlocks,
};
use singlefs_core::allocator::Placement;
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
    ContainerTreeNodePosition,
};
use singlefs_core::lineage_tree::{
    LineageContainerCapacity, LineageRecordsPerContainerCappedForTests,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::records::STATISTIC_COMMITTED_RESERVATION_BYTES;
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
    TransactionOutput, TransactionUnit,
};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};

/// 写这些镜像时墓碑容器旋钮压到的每片记录数（`capacities_of_one_record_and_two_entries` 那一个），交给池级 checker 带旋钮值的入口。
const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
/// 写这些镜像时谱系容器旋钮压到的每片记录数（`capacities_of_one_record_and_two_entries` 那一个），同样交给带旋钮值的入口。
const LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

/// 一片容器 1 条、一个码 2 节点 2 条：第 3 片容器根满长一层，第 5 片长到层级 3。
fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(
                LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
            )
            .expect("1..=1359"),
        ),
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        ),
    }
}

fn position(level: u8, index_in_level: u32) -> ContainerTreeNodePosition {
    ContainerTreeNodePosition {
        level,
        index_in_level,
    }
}

/// unlink 一个空 inode：意图那次（交回它），再走空（空 inode 没有数据单元，直接是最后一批）。写入口装 `capacities`。
fn unlink(
    pool: &mut BuiltPool,
    inode: u64,
    capacities: ContainerTreeCapacities,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let intent = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    let mut current = intent.clone();
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("最后一批");
    pool.output = current;
    intent
}

fn rebuilt_from_disk(pool: &BuiltPool) -> TransactionOutput {
    match rebuild_version(
        &pool.memory_pool(),
        &pool.output.root,
        Some(pool.output.record.clone()),
    )
    .expect("从盘上重建")
    {
        RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version) => version,
        RebuiltVersion::WithoutFile => panic!("有树的一版"),
    }
}

fn tombstone_container_capacity_knob(records_per_container: u64) -> TombstoneContainerCapacityKnob {
    TombstoneContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=815 里")
}

/// 池级 checker 带旋钮值的入口判出来的违例：`records_per_container` 是写这张镜像时墓碑容器旋钮压到的每片记录数；谱系的旋钮值一并给
/// （[`LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB`]），退役容器照产品。
fn violations_given_the_knob(pool: &BuiltPool, records_per_container: u64) -> Vec<String> {
    check_pool_image_given_the_container_capacity_knobs(
        &pool.memory_pool(),
        ContainerCapacityKnobs {
            tombstone: Some(tombstone_container_capacity_knob(records_per_container)),
            lineage: Some(
                LineageContainerCapacityKnob::new(LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
                    .expect("旋钮值在 1..=1359 里"),
            ),
            retirement: None,
        },
    )
    .into_iter()
    .filter_map(|(invariant, verdict)| match verdict {
        InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
        InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_) => None,
    })
    .collect()
}

/// 池级 checker 旧入口（当没装旋钮）判出来的违例。
fn violations_without_the_knob(pool: &BuiltPool) -> Vec<String> {
    check_pool_image(&pool.memory_pool())
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn tags_of_the_tombstone_tree(roles: &[TransactionUnit]) -> Vec<String> {
    roles
        .iter()
        .filter(|role| {
            matches!(
                role,
                TransactionUnit::TombstoneLeafContainer(_)
                    | TransactionUnit::TombstoneRoot
                    | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            )
        })
        .map(|role| role.tag())
        .collect()
}

fn committed_reservation_of(version: &TransactionOutput) -> u64 {
    version
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == STATISTIC_COMMITTED_RESERVATION_BYTES)
        .expect("带树的一版记着标签 6")
        .value
}

/// 一片 1 条、一个节点 2 条：第 3 次 unlink 的意图那次墓碑树根满长一层——重写新开的容器 tt@2、右半 (1, 1)、新根，旧根不重写、
/// 不释放（它的落点在这一版里成了 (1, 0) 那个节点、照旧已分配）；墓碑预付按 H = 1 记（副本数 × ((2 + 1 + 1) × 16384 + 41 × 3 个活对象)）；
/// 从盘上重建读回层级 2 的根与它下面的两个节点；冷启动恢复读回第一个文件；池级 checker 全绿。
#[test]
fn the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place() {
    let mut pool = pool_with_new_inodes("tombstone-tree-grows-a-level", 5);
    let capacities = capacities_of_one_record_and_two_entries();
    unlink(&mut pool, 2, capacities);
    unlink(&mut pool, 3, capacities);
    let before_the_growth = pool.output.clone();
    let old_root = before_the_growth
        .unit(TransactionUnit::TombstoneRoot)
        .clone();
    let growth = unlink(&mut pool, 4, capacities);
    assert_eq!(
        tags_of_the_tombstone_tree(&growth.rewritten),
        ["tt@2", "tt^1.1", "tt"],
        "长高那次重写新开的容器、右半与新根，旧根不重写"
    );
    assert_eq!(growth.tombstone_tree_index_nodes.shape.root_level(), 2);
    let carried_down = growth.unit(TransactionUnit::TombstoneNodeBelowTheRoot(position(1, 0)));
    assert_eq!(
        (carried_down.slot, &carried_down.bytes),
        (old_root.slot, &old_root.bytes),
        "旧根一个字节不动、留在原位置当 (1, 0) 那个节点"
    );
    let old_root_placement = Placement {
        slot: old_root.slot,
        span: TransactionUnit::TombstoneRoot.span_slots(),
    };
    assert!(
        !growth.released.contains(&old_root_placement),
        "旧根不被这次换下：{:?}",
        growth.released
    );
    let retirement_tree_internal_levels = 0;
    assert_eq!(
        committed_reservation_of(&growth),
        2 * ((2 + 1 + 1 + 2 + retirement_tree_internal_levels + 1) * 16_384 + 41 * 3 + 35 * 4),
        "标签 6 按长高之后的 H_墓碑 = 1：inode 1、5、6 三个活对象（2、3 删完，4 删除中），四条记录（4 的 nlink = 0 也算）；退役树零根，H_退役 = 0"
    );
    let rebuilt = rebuilt_from_disk(&pool);
    assert_eq!(
        (
            &rebuilt.tombstone_containers,
            &rebuilt.tombstone_tree_index_nodes
        ),
        (
            &pool.output.tombstone_containers,
            &pool.output.tombstone_tree_index_nodes
        ),
        "从盘上重建读回层级 2 的墓碑树：每片容器、形状与根之下节点的指针逐项相等"
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (pool.output.root.instance, pool.output.root.checkpoint_txg),
            content: file_content(),
        },
        "冷启动恢复（映射条目数把墓碑树每个节点都算上）读回第一个文件"
    );
    assert_eq!(
        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
        Vec::<String>::new(),
        "池级 checker（带旋钮值）全绿"
    );
}

/// 再 unlink 两个：第 5 片时层级 1 与根都满，长到层级 3（新开 (1, 2)、(2, 1)、新根 (3, 0)）；重建读回层级 3 的根，池级 checker 全绿。
#[test]
fn the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it() {
    let mut pool = pool_with_new_inodes("tombstone-tree-grows-to-three-levels", 5);
    let capacities = capacities_of_one_record_and_two_entries();
    for inode in 2..=5 {
        unlink(&mut pool, inode, capacities);
    }
    let growth = unlink(&mut pool, 6, capacities);
    assert_eq!(
        tags_of_the_tombstone_tree(&growth.rewritten),
        ["tt@4", "tt^1.2", "tt^2.1", "tt"],
        "层级 1 与根都满：新开的容器、(1, 2)、(2, 1)、新根"
    );
    assert_eq!(growth.tombstone_tree_index_nodes.shape.root_level(), 3);
    let rebuilt = rebuilt_from_disk(&pool);
    assert_eq!(
        (
            &rebuilt.tombstone_containers,
            &rebuilt.tombstone_tree_index_nodes
        ),
        (
            &pool.output.tombstone_containers,
            &pool.output.tombstone_tree_index_nodes
        ),
        "从盘上重建读回层级 3 的墓碑树"
    );
    assert_eq!(
        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
        Vec::<String>::new(),
        "池级 checker（带旋钮值）全绿"
    );
}

/// 旋钮不落盘：压小扇出写出来的两层树，换回产品扇出（135）接着写，照盘上的形状往最右那个层级 1 节点里挂（它只有 1 个孩子），不再长高。
#[test]
fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout() {
    let mut pool = pool_with_new_inodes("tombstone-tree-capped-then-product-fanout", 5);
    let capped = capacities_of_one_record_and_two_entries();
    for inode in 2..=4 {
        unlink(&mut pool, inode, capped);
    }
    let product_fanout = ContainerTreeCapacities {
        tombstone_container: capped.tombstone_container,
        lineage_container: capped.lineage_container,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    let next = unlink(&mut pool, 5, product_fanout);
    assert_eq!(
        tags_of_the_tombstone_tree(&next.rewritten),
        ["tt@3", "tt^1.1", "tt"],
        "产品扇出下 (1, 1) 还装得下：挂进去、重写它与根，根层级仍 2"
    );
    assert_eq!(next.tombstone_tree_index_nodes.shape.root_level(), 2);
    assert_eq!(
        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
        Vec::<String>::new(),
        "池级 checker（带旋钮值）全绿"
    );
}

/// 谱系树同一条（第 5 条「谱系树根满同样长高」）：一片 1 条、一个节点 2 条，回退三次之后谱系树两层；从盘上重建读回层级 2 的根；
/// ckpt_cost 谱系那一项按内部 1 层算 4 块、中央映射树那一项的删插多算 3 把（容器、根、内部 1 层）；池级 checker 全绿。
#[test]
fn the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_internal_level() {
    let mut pool = build_pool("lineage-tree-grows-a-level");
    let first_file_version = pool.output.clone();
    let capacities = capacities_of_one_record_and_two_entries();
    pool.allocator.set_deletion_switches(DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: capacities.tombstone_container,
        lineage_container_capacity: capacities.lineage_container,
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: capacities.node_entries,
    });
    for _ in 0..3 {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &publish_parameters,
            devices,
            &mut pool.allocator,
            &mut pool.output,
            RollbackTarget {
                instance: first_file_version.root.instance,
                checkpoint_txg: first_file_version.root.checkpoint_txg,
            },
        )
        .expect("回退到第一个文件那一版");
    }
    assert_eq!(
        pool.output
            .rewritten
            .iter()
            .filter(|role| matches!(
                role,
                TransactionUnit::LineageLeafContainer(_)
                    | TransactionUnit::LineageRoot
                    | TransactionUnit::LineageNodeBelowTheRoot(_)
            ))
            .map(|role| role.tag())
            .collect::<Vec<_>>(),
        ["tl@2", "tl^1.1", "tl"],
        "第三次回退：谱系树根满长一层"
    );
    assert_eq!(pool.output.lineage_tree_internal_levels(), 1);
    assert_eq!(
        (
            checkpoint_cost_of_one_lineage_container_cow(
                pool.output.lineage_tree_internal_levels()
            ),
            central_mapping_entries_of_one_lineage_container_cow(
                pool.output.lineage_tree_internal_levels()
            ),
        ),
        (MetadataBlocks(4), 3),
        "谱系那一项：容器 2 + 根 1 + 内部 1 层；映射里容器、根、内部节点各一把"
    );
    let rebuilt = rebuilt_from_disk(&pool);
    assert_eq!(
        (
            &rebuilt.lineage_containers,
            &rebuilt.lineage_tree_index_nodes
        ),
        (
            &pool.output.lineage_containers,
            &pool.output.lineage_tree_index_nodes
        ),
        "从盘上重建读回层级 2 的谱系树"
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (pool.output.root.instance, pool.output.root.checkpoint_txg),
            content: file_content(),
        },
        "冷启动恢复读回第一个文件"
    );
    assert_eq!(
        violations_given_the_knob(&pool, TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
        Vec::<String>::new(),
        "池级 checker（带旋钮值）全绿"
    );
}

/// I-9.21（产品镜像里非最右墓碑容器装满） 只红自己的那一形（判决第二节 T5 那一格：旋钮压小的合法镜像上按产品容量判就红）：
/// 墓碑容器压到一片 1 条，unlink 2、3、4 号各占一片容器；这张镜像交给旧入口（当没装旋钮），不是最右的两片各只装 1 条、不是 815，
/// I-9.21 红、别的一条不红（I-9.22 报不适用）；交给带旋钮值 1 的入口则全绿，I-9.22 真被评估过且成立、I-9.21 报不适用。
#[test]
fn a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant() {
    let mut pool = pool_with_new_inodes("tombstone-tree-judged-as-a-product-image", 5);
    let capacities = capacities_of_one_record_and_two_entries();
    for inode in 2..=4 {
        unlink(&mut pool, inode, capacities);
    }
    let verdicts_without_the_knob = check_pool_image(&pool.memory_pool());
    let violated_without_the_knob: Vec<&str> = verdicts_without_the_knob
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(
        violated_without_the_knob,
        [NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "旧入口当没装旋钮：只红 I-9.21"
    );
    assert!(
        violations_without_the_knob(&pool)
            .iter()
            .any(|violation| violation.contains("装了 1 条，不是 815")),
        "违例说明写出那片容器装了几条：{:?}",
        violations_without_the_knob(&pool)
    );
    let verdicts_given_the_knob = check_pool_image_given_the_tombstone_container_capacity_knob(
        &pool.memory_pool(),
        Some(tombstone_container_capacity_knob(
            TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
        )),
    );
    for (invariant, verdict) in &verdicts_given_the_knob {
        let expected_holds =
            *invariant == TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE;
        let expected_not_applicable =
            *invariant == NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE;
        if expected_holds {
            assert_eq!(
                *verdict,
                InvariantVerdict::Holds,
                "带旋钮值：I-9.22 真被评估过且成立"
            );
        }
        if expected_not_applicable {
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "带旋钮值：I-9.21 报不适用，实际 {verdict:?}"
            );
        }
        assert!(
            !matches!(verdict, InvariantVerdict::Violated(_)),
            "带旋钮值：{invariant} 不红，实际 {verdict:?}"
        );
    }
}

/// I-9.22（测试镜像里墓碑容器不超旋钮值） 只红自己的那一形：墓碑容器压到一片 2 条写出来的镜像（unlink 2、3 号同在一片容器里），
/// 交给带旋钮值的入口时报旋钮值 1——那片容器装了 2 条、多于旋钮值，I-9.22 红、别的一条不红（I-9.21 报不适用）。同一张镜像报旋钮值 2 全绿。
#[test]
fn a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
) {
    let mut pool = pool_with_new_inodes("tombstone-container-above-the-knob", 5);
    let capacities_of_two_records = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(2).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    for inode in 2..=3 {
        unlink(&mut pool, inode, capacities_of_two_records);
    }
    assert_eq!(
        violations_given_the_knob(&pool, 2),
        Vec::<String>::new(),
        "报写的时候那个旋钮值 2：全绿"
    );
    let verdicts = check_pool_image_given_the_tombstone_container_capacity_knob(
        &pool.memory_pool(),
        Some(tombstone_container_capacity_knob(1)),
    );
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(
        violated,
        [TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE],
        "报旋钮值 1：只红 I-9.22"
    );
    assert!(
        violations_given_the_knob(&pool, 1)
            .iter()
            .any(|violation| violation.contains("装了 2 条，多于旋钮值 1")),
        "违例说明写出那片容器装了几条与旋钮值：{:?}",
        violations_given_the_knob(&pool, 1)
    );
}

/// 这一段随机历史的种子与步数（固定：同一个种子逐位复现同一段历史；种子 1–8 在实现员副本上各跑一遍，取三棵树都写出至少三片容器、
/// 退役树也长高了的 7）。
const RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB: u64 = 7;
const RANDOM_HISTORY_OPERATIONS_UNDER_EVERY_CONTAINER_KNOB: u64 = 60;
/// 回退的目标从最近这么多版里挑：每步至多三次发布，挑到的那一版留在根环一圈里（回退候选集）。
const ROLLBACK_TARGETS_KEPT: usize = 8;

/// 三棵容器树都压到一片 `records_per_container` 条、一个码 2 节点 2 条（第 3 片容器根满长一层）。
fn capacities_of_every_container_tree_at_and_two_entries(
    records_per_container: u64,
) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(records_per_container)
                .expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(records_per_container).expect("1..=1359"),
        ),
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
                    records_per_container,
                )
                .expect("1..=959"),
            ),
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        ),
    }
}

/// 同一组旋钮值交给池级 checker 的新入口（三棵都是一片 `records_per_container` 条）。
fn knobs_of_every_container_tree_at(records_per_container: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: Some(tombstone_container_capacity_knob(records_per_container)),
        lineage: Some(
            LineageContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=1359 里"),
        ),
        retirement: Some(
            singlefs_checker::walk::RetirementContainerCapacityKnob::new(records_per_container)
                .expect("旋钮值在 1..=959 里"),
        ),
    }
}

/// 合法镜像不红（第 56 批 G2 验收，判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节；攻方
/// `opus_probe_k2_random_knob_histories` 那一形）：三棵容器树都压到一片 1 条、一个节点 2 条，建满一片叶再多一个号（最右那片叶独占它），
/// 之后固定种子随机建一个号、unlink（多半删最大的活号：最右那片叶删空就退役；也从记录最少的叶里删、随便删一个）、挂着回退到最近几版之一；
/// 每一步之后带同一组旋钮值交池级 checker 的新入口，一条都不许红。历史走完三棵树各至少三片容器（容器号从 1 起逐片加一、每片至少一条那三判
/// 真判到了多片），I-9.22、I-9.37 真被评估过且成立。
#[test]
#[ignore = "harness 慢档用例：60 步随机历史、每步一次池级 checker，debug 下约两分钟"]
fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step() {
    assert_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
        1,
        "container-trees-random-history-under-every-knob",
        3,
    );
}

/// 第 57 批 H1（判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节；攻方 `k2_random_histories` 那一类）：同一段固定种子
/// 随机历史，三棵容器树都压到一片 2 条（写侧装满 2 条才关，D18（块里携带什么信息） 已定项 24 第 3 条），每一步之后带同一组旋钮值 2 交新入口，
/// 一条都不许红；历史走完三棵树各至少两片容器，「不是最右的那片装满旋钮值」真判到了。写侧早关一片（攻方 `w-close-one-early`：装 1 条就开新片）
/// 写出的镜像在这里红。
#[test]
#[ignore = "harness 慢档用例：60 步随机历史、每步一次池级 checker，debug 下约两分钟"]
fn a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step() {
    assert_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
        2,
        "container-trees-random-history-under-every-knob-of-two-records",
        2,
    );
}

/// 上面两条的本体：三棵容器树都压到一片 `records_per_container` 条、一个节点 2 条，跑 [`RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB`] 那一段
/// 随机历史，每一步之后带同一组旋钮值交新入口一条都不红；历史走完三棵树各至少 `fewest_containers_of_each_tree_at_the_end` 片容器，
/// I-9.22、I-9.37 真被评估过且成立。
fn assert_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
    records_per_container: u64,
    tag: &str,
    fewest_containers_of_each_tree_at_the_end: usize,
) {
    let mut random = singlefs_harness::history::SeededRandomSource::from_seed(
        RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB,
    );
    let capacities = capacities_of_every_container_tree_at_and_two_entries(records_per_container);
    let mut pool = pool_with_new_inodes(tag, singlefs_format::INODE_LEAF_RECORDS);
    pool.allocator.set_deletion_switches(DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: capacities.tombstone_container,
        lineage_container_capacity: capacities.lineage_container,
        retirement_container_capacity: capacities.retirement_container,
        container_tree_node_entries: capacities.node_entries,
    });
    let mut recent_versions: Vec<TransactionOutput> = vec![pool.output.clone()];
    let mut knob_invariants_held: Vec<&str> = Vec::new();
    // 迭代上界：常量 60 步。跨步携带的状态：池与最近几版（至多 ROLLBACK_TARGETS_KEPT 版，最旧的先丢）。
    for step in 0..RANDOM_HISTORY_OPERATIONS_UNDER_EVERY_CONTAINER_KNOB {
        let choice = random.below(100);
        let live_inodes: Vec<u64> = pool
            .output
            .inode_leaf_containers
            .iter()
            .flat_map(|leaf| leaf.contents.records.iter().map(|record| record.inode))
            .filter(|inode| *inode != singlefs_core::transaction::FIRST_INODE_NUMBER)
            .collect();
        let rollback_targets: Vec<&TransactionOutput> = recent_versions
            .iter()
            .filter(|version| version.root.checkpoint_txg != pool.output.root.checkpoint_txg)
            .collect();
        if choice < 20 || live_inodes.is_empty() {
            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
        } else if choice < 85 || rollback_targets.is_empty() {
            let inode = if choice < 25 {
                let fewest_records = pool
                    .output
                    .inode_leaf_containers
                    .iter()
                    .filter(|leaf| {
                        leaf.contents.records.iter().any(|record| {
                            record.inode != singlefs_core::transaction::FIRST_INODE_NUMBER
                        })
                    })
                    .min_by_key(|leaf| leaf.contents.records.len())
                    .expect("有活着的号就有这样一片叶");
                let candidates: Vec<u64> = fewest_records
                    .contents
                    .records
                    .iter()
                    .map(|record| record.inode)
                    .filter(|inode| *inode != singlefs_core::transaction::FIRST_INODE_NUMBER)
                    .collect();
                candidates[usize::try_from(
                    random.below(u64::try_from(candidates.len()).expect("装得进 u64")),
                )
                .expect("装得进 usize")]
            } else if choice < 75 {
                *live_inodes.iter().max().expect("非空")
            } else {
                live_inodes[usize::try_from(
                    random.below(u64::try_from(live_inodes.len()).expect("装得进 u64")),
                )
                .expect("装得进 usize")]
            };
            unlink(&mut pool, inode, capacities);
        } else {
            let target = rollback_targets[usize::try_from(
                random.below(u64::try_from(rollback_targets.len()).expect("装得进 u64")),
            )
            .expect("装得进 usize")]
            .clone();
            let publish_parameters = parameters();
            let devices = pool.devices.as_mut().expect("镜像还开着");
            roll_back_by_a_forward_publish(
                &publish_parameters,
                devices,
                &mut pool.allocator,
                &mut pool.output,
                RollbackTarget {
                    instance: target.root.instance,
                    checkpoint_txg: target.root.checkpoint_txg,
                },
            )
            .unwrap_or_else(|refusal| {
                panic!(
                    "第 {step} 步回退到 txg {}：{refusal:?}",
                    target.root.checkpoint_txg.0
                )
            });
        }
        recent_versions.push(pool.output.clone());
        if recent_versions.len() > ROLLBACK_TARGETS_KEPT {
            recent_versions.remove(0);
        }
        let verdicts = check_pool_image_given_the_container_capacity_knobs(
            &pool.memory_pool(),
            knobs_of_every_container_tree_at(records_per_container),
        );
        let violated: Vec<String> = verdicts
            .iter()
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
                InvariantVerdict::Holds
                | InvariantVerdict::NotApplicable(_)
                | InvariantVerdict::PartiallyJudged(_) => None,
            })
            .collect();
        assert_eq!(
            violated,
            Vec::<String>::new(),
            "第 {step} 步之后的合法镜像（带同一组旋钮值 {records_per_container}）一条都不红"
        );
        knob_invariants_held.extend(
            verdicts
                .iter()
                .filter(|(_, verdict)| *verdict == InvariantVerdict::Holds)
                .map(|(invariant, _)| *invariant)
                .filter(|invariant| {
                    *invariant == TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
                        || *invariant
                            == singlefs_checker::image::LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
                }),
        );
    }
    let container_counts = (
        pool.output.tombstone_containers.len(),
        pool.output.lineage_containers.len(),
        pool.output.retirement_containers.len(),
    );
    eprintln!(
        "CONTAINER_TREES_RANDOM_HISTORY seed={RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB} records_per_container={records_per_container} containers(tombstone, lineage, retirement)={container_counts:?} levels(tombstone, lineage, retirement)=({}, {}, {})",
        pool.output.tombstone_tree_index_nodes.shape.root_level(),
        pool.output.lineage_tree_index_nodes.shape.root_level(),
        pool.output.retirement_tree_index_nodes.shape.root_level()
    );
    assert!(
        container_counts.0 >= fewest_containers_of_each_tree_at_the_end
            && container_counts.1 >= fewest_containers_of_each_tree_at_the_end
            && container_counts.2 >= fewest_containers_of_each_tree_at_the_end,
        "历史走完三棵树各至少 {fewest_containers_of_each_tree_at_the_end} 片容器：{container_counts:?}"
    );
    for invariant in [
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
        singlefs_checker::image::LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    ] {
        assert!(
            knob_invariants_held.contains(&invariant),
            "{invariant} 真被评估过且成立"
        );
    }
}
