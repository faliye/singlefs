//! harness 测试文件 `crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::lineage_tree::{LineageContainer, LineageContainerIndexInTree};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::records::LineageRecord;
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_new_inodes, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{PackedIdentity, PACKED_TYPE_LINEAGE};
use singlefs_core::write_accounting::{WriteCallsAndBytes, WrittenStructureKind};
use singlefs_format::TREE_IDENTIFIER_LINEAGE;

/// 第一个文件那一版之后覆盖写一次，交回第一个文件那一版（回退目标）。
pub fn pool_with_one_overwrite_after_the_first_file(tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"overwritten content");
    (pool, first_file_version)
}

pub fn roll_back_to(pool: &mut BuiltPool, target: &TransactionOutput) {
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
    .expect("回退目标在候选集里");
}

pub fn rebuilt_from_disk(pool: &BuiltPool) -> TransactionOutput {
    match rebuild_version(
        &pool.memory_pool(),
        &pool.output.root,
        Some(pool.output.record.clone()),
    )
    .expect("回退之后那一版从盘上重建")
    {
        RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version) => version,
        RebuiltVersion::WithoutFile => panic!("回退之后那一版有树"),
    }
}

/// 交 checker 档：这一刻的镜像上池级 checker 全绿。
pub fn hand_out_expecting_every_invariant_green(
    images: &mut PoolCheckerImages,
    label: &str,
    pool: &BuiltPool,
) {
    images.hand_out(
        label,
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

pub fn lineage_records_of(version: &TransactionOutput) -> Vec<LineageContainer> {
    version
        .lineage_containers
        .iter()
        .map(|container| container.contents.clone())
        .collect()
}

/// 第一次回退（零根上）：谱系树开容器号 1、出生代 = 回退那次的 txg、类型 5，装一条 (T_rb = 回退那次的 txg, T_old = R_old 的 txg,
/// H_cur = cur 的 inode 号水位 2)；
/// 这次重写谱系容器与谱系树根各一个，写账记在 `LineageTreeNode` 那一种（两块盘各一份：4 次写、2 × (32768 + 16384) 字节）；
/// 从盘上重建读回的谱系容器逐字段相等；冷启动恢复的走读（映射条目数把谱系树的容器与根也算上）读回第一个文件；池级 checker 全绿。
pub fn a_rollback_appends_one_lineage_record_that_reads_back_field_by_field(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, first_file_version) =
        pool_with_one_overwrite_after_the_first_file("rollback-appends-a-lineage-record");
    assert!(
        pool.output.lineage_containers.is_empty(),
        "回退之前谱系树零根"
    );
    roll_back_to(&mut pool, &first_file_version);
    let rollback_txg = pool.output.root.checkpoint_txg;
    let expected = vec![LineageContainer {
        identity: PackedIdentity {
            birth_tree: singlefs_core::address::TreeIdentifier(TREE_IDENTIFIER_LINEAGE),
            record_type: PACKED_TYPE_LINEAGE,
            container: 1,
            container_birth: rollback_txg,
        },
        records: vec![LineageRecord {
            rollback_txg,
            rollback_target_txg: first_file_version.root.checkpoint_txg,
            inode_number_watermark_of_the_version_rolled_back_from: 2,
        }],
    }];
    assert_eq!(
        (rollback_txg, lineage_records_of(&pool.output),),
        (
            CheckpointTxg(first_file_version.root.checkpoint_txg.0 + 2),
            expected.clone()
        ),
        "第一个文件那一版之后覆盖写一次、再回退：T_rb 比 T_old 大 2，容器号 1"
    );
    assert_eq!(
        pool.output
            .rewritten
            .iter()
            .filter(|role| matches!(
                role,
                TransactionUnit::LineageLeafContainer(_) | TransactionUnit::LineageRoot
            ))
            .copied()
            .collect::<Vec<_>>(),
        vec![
            TransactionUnit::LineageLeafContainer(LineageContainerIndexInTree(0)),
            TransactionUnit::LineageRoot,
        ],
        "回退那次重写谱系容器与谱系树根，先叶后根"
    );
    assert_eq!(
        pool.output.writes.of(WrittenStructureKind::LineageTreeNode),
        WriteCallsAndBytes {
            write_calls: 4,
            written_bytes: 2 * (32_768 + 16_384),
        },
        "谱系树那一种：两个单元各落两块盘"
    );
    assert_eq!(
        lineage_records_of(&rebuilt_from_disk(&pool)),
        expected,
        "从盘上重建读回的谱系容器与写出的逐字段相等"
    );
    assert_eq!(
        recover(&pool.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (pool.output.root.instance, rollback_txg),
            content: file_content(),
        },
        "冷启动恢复走到回退那一版，读回第一个文件"
    );
    hand_out_expecting_every_invariant_green(images, "回退写下谱系记录之后", &pool);
}

/// 第二次回退追加进同一片开放容器（一片装 1359 条）：容器号仍 1、出生代仍是第一次回退那次，两条记录按先后排；之间的发布照抄谱系树。
pub fn a_second_rollback_appends_to_the_same_open_lineage_container(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, first_file_version) =
        pool_with_one_overwrite_after_the_first_file("second-rollback-appends-to-the-container");
    roll_back_to(&mut pool, &first_file_version);
    let first_rollback = pool.output.clone();
    let after_the_overwrite =
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"again");
    assert_eq!(
        lineage_records_of(&after_the_overwrite),
        lineage_records_of(&first_rollback),
        "不回退的发布照抄谱系树"
    );
    roll_back_to(&mut pool, &first_rollback);
    let second_rollback_txg = pool.output.root.checkpoint_txg;
    let containers = lineage_records_of(&rebuilt_from_disk(&pool));
    assert_eq!(
        (
            containers.len(),
            containers
                .first()
                .map(|container| container.identity.container),
            containers
                .first()
                .map(|container| container.identity.container_birth),
            containers
                .first()
                .map(|container| container.records.clone())
                .unwrap_or_default(),
        ),
        (
            1,
            Some(1),
            Some(first_rollback.root.checkpoint_txg),
            vec![
                LineageRecord {
                    rollback_txg: first_rollback.root.checkpoint_txg,
                    rollback_target_txg: first_file_version.root.checkpoint_txg,
                    inode_number_watermark_of_the_version_rolled_back_from: 2,
                },
                LineageRecord {
                    rollback_txg: second_rollback_txg,
                    rollback_target_txg: first_rollback.root.checkpoint_txg,
                    inode_number_watermark_of_the_version_rolled_back_from: 2,
                },
            ],
        ),
        "一片容器两条记录：(第一次回退, 第一个文件)、(第二次回退, 第一次回退)"
    );
    hand_out_expecting_every_invariant_green(images, "回退写下谱系记录之后", &pool);
}

/// 谱系记录的 H_cur 从 cur（回退之前的现行那一版）读，不取回退目标的（D18（块里携带什么信息） 已定项 24 第 5 条，W3 判决
/// `research/prompts/m3-w3-r2-main-verification.md` 第二节攻方 K3）：第一个文件那一版（txg 3，水位 2）之后建两个空 inode（txg 4，水位 4）、
/// 再建一个（txg 5，水位 5），挂着的时候回退到建两个的那一版（水位 4）：谱系记录 (6, 4, 5)——H_cur 是 cur 的 5，不是回退目标的 4。
/// 从盘上重建读回的同一条；池级 checker 一条违例都没有（I-9.30 谱系那一判拿它比台阶）。
pub fn a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("rollback-lineage-record-watermark-of-the-current-version");
    let mut create_inodes = |count: u64| {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let output = publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            count,
            FIXED_WRITE_TIME_SECONDS + 60,
            InstanceGeneration(1),
        )
        .expect("建空 inode");
        pool.output = output.clone();
        output
    };
    let target = create_inodes(2);
    let current = create_inodes(1);
    assert_eq!(
        (
            target.root.checkpoint_txg,
            target.inode_number_watermark(),
            current.root.checkpoint_txg,
            current.inode_number_watermark()
        ),
        (CheckpointTxg(4), 4, CheckpointTxg(5), 5),
        "回退目标 txg 4、水位 4；cur txg 5、水位 5"
    );
    roll_back_to(&mut pool, &target);
    let expected = vec![LineageRecord {
        rollback_txg: CheckpointTxg(6),
        rollback_target_txg: CheckpointTxg(4),
        inode_number_watermark_of_the_version_rolled_back_from: 5,
    }];
    let records_of = |version: &TransactionOutput| -> Vec<LineageRecord> {
        lineage_records_of(version)
            .into_iter()
            .flat_map(|container| container.records)
            .collect()
    };
    assert_eq!(
        (records_of(&pool.output), records_of(&rebuilt_from_disk(&pool))),
        (expected.clone(), expected),
        "谱系记录 (T_rb 6, T_old 4, H_cur 5)：H_cur 是 cur 的水位，不是回退目标的 4；从盘上重建读回同一条"
    );
    hand_out_expecting_every_invariant_green(images, "回退写下谱系记录之后", &pool);
}
