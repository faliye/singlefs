//! unlink 单文件的三种发布（D8（核心索引结构） 已定项 16 第 1、2、4、6、7 条；D18（块里携带什么信息） 已定项 24）：
//! 意图那次、不是最后一批的一批、最后一批。三个单元的文件按只供测试的批大小 1 删：意图 → 留 2 → 留 1 → 最后一批。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use common_deletion_in_progress::{
    batches_of_one_data_unit, pool_with_a_three_unit_file, publish_the_next_step_in_batches_of_one,
    FIRST_FILE, INSTANCE_OF_THE_PUBLISHES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::CheckpointTxg;
use singlefs_core::allocator::Placement;
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    TombstoneRecord, NLINK_OF_AN_INODE_BEING_DELETED, TREE_KIND_DELETED_INODES, TREE_KIND_TOMBSTONE,
};
use singlefs_core::recovery::{
    rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_overwrite, publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty,
    FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, PACKED_TYPE_TOMBSTONE};
use singlefs_harness::memory_pool::MemoryPool;

/// 意图那次被拒要交回错（已经删完的那一格），`common_deletion_in_progress::publish_the_intent` 被拒就 panic：这一份照旧留在这里。
fn publish_the_intent(pool: &mut BuiltPool) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    // 第一个文件那条记录现有的时间（已经删完的那一格没有记录，意图在任何写之前被拒，给的时间不落盘）。
    let write_time_seconds = previous
        .inode_record
        .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds);
    let output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FIRST_FILE,
        write_time_seconds,
        INSTANCE_OF_THE_PUBLISHES,
    )?;
    pool.output = output.clone();
    Ok(output)
}

fn tags_of(roles: &[TransactionUnit]) -> Vec<String> {
    roles.iter().map(|role| role.tag()).collect()
}

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn root_pointer_of_the_tree_kind(version: &TransactionOutput, kind: u16) -> NodePointer {
    version
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == kind)
        .expect("树表十一条，每种一条")
        .root
}

fn placement_of(version: &TransactionOutput, role: TransactionUnit) -> Placement {
    let unit = version.unit(role);
    Placement {
        slot: unit.slot,
        span: role.span_slots(),
    }
}

#[test]
fn intent_rewrites_the_inode_leaf_and_root_the_deleted_inodes_root_and_the_tombstone_container_and_root(
) {
    let mut pool = pool_with_a_three_unit_file("unlink-intent-roles");
    let before = pool.output.clone();
    let intent = publish_the_intent(&mut pool).expect("意图");
    assert_eq!(
        tags_of(&intent.rewritten),
        [
            "t3", "t4", "t5@0.0.61", "t5@0.1.61", "t5@1.0.0", "t5@1.1.0", "t5", "t6", "td",
            "tt@0", "tt", "t7", "t8"
        ],
        "意图那次只重写 inode 叶与根、分配记录树、记账树、deleted_inodes 根、墓碑容器与根、映射树、树表：数据单元与 extent 树照抄"
    );
    let record_before = before.inode_record.expect("意图之前有第一个文件那条记录");
    let record = intent.inode_record.expect("意图之后那条记录还在树里");
    assert_eq!(
        record.nlink, NLINK_OF_AN_INODE_BEING_DELETED,
        "nlink 写成 0"
    );
    assert_eq!(record.size, record_before.size, "size 不变");
    assert_eq!(
        intent.deleted_inodes,
        [FIRST_FILE],
        "deleted_inodes 加 key = 1"
    );
    assert_eq!(intent.data_pointers, before.data_pointers, "数据单元照抄");
    let [container] = intent.tombstone_containers.as_slice() else {
        panic!("墓碑树一片容器：{:?}", intent.tombstone_containers);
    };
    assert_eq!(
        (
            container.contents.identity.birth_tree,
            container.contents.identity.record_type,
            container.contents.identity.container,
            container.contents.identity.container_birth,
        ),
        (
            intent.tree_identifiers.tombstone,
            PACKED_TYPE_TOMBSTONE,
            1,
            intent.root.checkpoint_txg,
        ),
        "容器号从 1 起，出生代是这次发布的 txg"
    );
    assert_eq!(
        container.contents.records,
        [TombstoneRecord::of_the_whole_object(
            FIRST_FILE,
            record_before.object_birth,
            intent.root.checkpoint_txg
        )],
        "一条整对象墓碑，死亡 txg = 意图那次发布的 txg"
    );
    assert_ne!(
        root_pointer_of_the_tree_kind(&intent, TREE_KIND_DELETED_INODES),
        NodePointer::empty_root(),
        "树表里 deleted_inodes 的根指针换成这一版的根"
    );
    assert_ne!(
        root_pointer_of_the_tree_kind(&intent, TREE_KIND_TOMBSTONE),
        NodePointer::empty_root(),
        "树表里墓碑树的根指针换成这一版的根"
    );
}

#[test]
fn a_batch_of_one_unit_rewrites_the_cut_lower_leaf_and_the_extent_root_and_releases_the_tail_unit()
{
    let mut pool = pool_with_a_three_unit_file("unlink-first-batch");
    let intent = publish_the_intent(&mut pool).expect("意图");
    let batch = publish_the_next_step_in_batches_of_one(&mut pool);
    assert_eq!(
        tags_of(&batch.rewritten),
        [
            "t2下@0.0",
            "t2",
            "t3",
            "t4",
            "t5@0.0.61",
            "t5@0.1.61",
            "t5@1.0.0",
            "t5@1.1.0",
            "t5",
            "t6",
            "t7",
            "t8"
        ],
        "一批只重写被截掉那一片下段叶与它的祖先、inode 叶与根（size 改了）与固定点单元"
    );
    assert_eq!(
        batch.data_pointers,
        intent.data_pointers[..2],
        "留下前两个数据单元"
    );
    let record = batch.inode_record.expect("一批之后记录还在");
    assert_eq!(record.nlink, NLINK_OF_AN_INODE_BEING_DELETED, "nlink 仍 0");
    assert_eq!(
        record.size,
        2 * u64::try_from(data_unit_payload_capacity()).expect("净荷"),
        "size = 剩下单元数 × 净荷容量"
    );
    assert_eq!(batch.deleted_inodes, [FIRST_FILE], "key 留着");
    assert_eq!(
        batch.tombstone_containers, intent.tombstone_containers,
        "墓碑树照抄"
    );
    assert!(
        batch.released.contains(&placement_of(
            &intent,
            TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile(2))
        )),
        "截掉的第三个数据单元经映射释放：{:?}",
        batch.released
    );
}

#[test]
fn the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_the_same_publish() {
    let mut pool = pool_with_a_three_unit_file("unlink-final-batch");
    publish_the_intent(&mut pool).expect("意图");
    publish_the_next_step_in_batches_of_one(&mut pool);
    let last_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    assert_eq!(last_batch.data_pointers.len(), 1, "最后一批之前剩一个单元");
    assert!(
        last_batch.inode_record.is_some() && last_batch.deleted_inodes == [FIRST_FILE],
        "最后一批之前记录与 key 都在"
    );
    let final_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    assert_eq!(
        tags_of(&final_batch.rewritten),
        [
            "t2",
            "t4",
            "t5@0.0.61",
            "t5@0.1.61",
            "t5@1.0.0",
            "t5@1.1.0",
            "t5",
            "t6",
            "tr@0",
            "tr",
            "t7",
            "t8"
        ],
        "最后一批重写 extent 零条目根、inode 零条目根与固定点单元；deleted_inodes 删空取根指针置零、不写它的节点；唯一那片叶容器身份消失，\
         退役树开第一片容器、写根（判决 m3-retire-r1 第三节第 3 条）"
    );
    assert_eq!(final_batch.inode_record, None, "记录删掉");
    assert!(final_batch.deleted_inodes.is_empty(), "key 删掉");
    assert!(final_batch.data_pointers.is_empty(), "剩下的单元释放");
    assert!(
        final_batch.inode_leaf_containers.is_empty(),
        "树里只有第一个文件：删完一片叶都不剩"
    );
    assert!(
        final_batch.extent_tree.lower_nodes.is_empty()
            && final_batch.extent_tree.upper_nodes.len() == 1,
        "extent 树只剩上段零条目根：{:?}",
        final_batch.extent_tree
    );
    assert_eq!(
        root_pointer_of_the_tree_kind(&final_batch, TREE_KIND_DELETED_INODES),
        NodePointer::empty_root(),
        "队列空时树表里 deleted_inodes 的根指针置零"
    );
    assert_eq!(
        final_batch.root.checkpoint_txg,
        CheckpointTxg(last_batch.root.checkpoint_txg.0 + 1),
        "删记录、删 key、释放剩下的单元是同一次发布"
    );
    for (role, placement) in [
        TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile(0)),
        TransactionUnit::DeletedInodesRoot,
    ]
    .map(|role| (role, placement_of(&last_batch, role)))
    {
        assert!(
            final_batch.released.contains(&placement),
            "{role:?} 在最后一批里经映射释放：{:?}",
            final_batch.released
        );
    }
}

#[test]
fn the_pool_checker_finds_no_violation_after_the_intent_and_after_every_batch() {
    let mut pool = pool_with_a_three_unit_file("unlink-checker-between-batches");
    publish_the_intent(&mut pool).expect("意图");
    let mut versions_judged = vec![("意图".to_string(), violations(&pool.memory_pool()))];
    for batch in 1..=3 {
        publish_the_next_step_in_batches_of_one(&mut pool);
        versions_judged.push((format!("第 {batch} 批"), violations(&pool.memory_pool())));
    }
    assert!(pool.output.deleted_inodes.is_empty(), "三批之后删完");
    assert!(
        versions_judged
            .iter()
            .all(|(_, violated)| violated.is_empty()),
        "每一版池级 checker 全绿：{versions_judged:?}"
    );
}

#[test]
fn from_the_intent_on_the_read_only_mount_and_the_cold_walk_answer_no_file() {
    let mut pool = pool_with_a_three_unit_file("unlink-reads-no-file");
    publish_the_intent(&mut pool).expect("意图");
    let after_the_intent = pool.memory_pool();
    let mounted = mount_read_only(&after_the_intent).expect("只读挂载");
    assert!(
        matches!(
            mounted.mounted.open_file(&after_the_intent, FIRST_FILE),
            Err(OpenFileFailure::NoSuchInode { inode }) if inode == FIRST_FILE
        ),
        "意图之后只读挂载打开第一个文件答「没有文件」"
    );
    let report = recover(&after_the_intent, JournalPolicy::Consult);
    assert!(
        matches!(report.outcome, RecoveryOutcome::NoFile { .. }),
        "意图之后冷走读答「没有文件」：{:?}",
        report.outcome
    );
    for _ in 0..3 {
        publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let after_the_final_batch = pool.memory_pool();
    let report_after_the_final_batch = recover(&after_the_final_batch, JournalPolicy::Consult);
    assert!(
        matches!(
            report_after_the_final_batch.outcome,
            RecoveryOutcome::NoFile { .. }
        ),
        "删完之后冷走读答「没有文件」：{:?}",
        report_after_the_final_batch.outcome
    );
}

#[test]
fn every_deletion_version_rebuilt_from_disk_equals_the_published_one() {
    let mut pool = pool_with_a_three_unit_file("unlink-rebuild");
    let mut published = vec![publish_the_intent(&mut pool).expect("意图")];
    for _ in 0..3 {
        published.push(publish_the_next_step_in_batches_of_one(&mut pool));
    }
    let image = pool.memory_pool();
    for version in &published {
        let (RebuiltVersion::WithFile(rebuilt) | RebuiltVersion::WithTreesAndNoFile(rebuilt)) =
            rebuild_version(&image, &version.root, Some(version.record.clone())).unwrap_or_else(
                |failure| {
                    panic!(
                        "txg {:?} 从盘上重建：{failure:?}",
                        version.root.checkpoint_txg
                    )
                },
            )
        else {
            panic!("这一版有树");
        };
        assert_eq!(
            (
                &rebuilt.inode_record,
                &rebuilt.deleted_inodes,
                &rebuilt.tombstone_containers,
                &rebuilt.data_pointers,
                &rebuilt.extent_tree,
                &rebuilt.mapped_units,
            ),
            (
                &version.inode_record,
                &version.deleted_inodes,
                &version.tombstone_containers,
                &version.data_pointers,
                &version.extent_tree,
                &version.mapped_units,
            ),
            "txg {:?}：重建出来的记录、队列、墓碑、数据指针、extent 树与映射 key 与发布的那一版相同",
            version.root.checkpoint_txg
        );
    }
}

#[test]
fn continuing_after_the_queue_is_empty_publishes_nothing() {
    let mut pool = pool_with_a_three_unit_file("unlink-continue-after-empty");
    publish_the_intent(&mut pool).expect("意图");
    for _ in 0..3 {
        publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let operations_before = pool.retained_operations().len();
    let publish_parameters = parameters();
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let publishes = publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        batches_of_one_data_unit(),
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("续做");
    assert_eq!(publishes, 0, "删完再续做一次都不发");
    assert_eq!(current, pool.output, "现行那一版不变");
    assert_eq!(
        pool.retained_operations().len(),
        operations_before,
        "一个写都不发"
    );
}

#[test]
fn unlink_of_an_inode_that_is_not_a_live_record_is_refused_before_any_write() {
    let mut pool = pool_with_a_three_unit_file("unlink-not-a-live-record");
    publish_the_intent(&mut pool).expect("意图");
    for _ in 0..3 {
        publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let operations_before = pool.retained_operations().len();
    let refusal = publish_the_intent(&mut pool).expect_err("第一个文件已经删完");
    assert!(
        matches!(
            refusal,
            PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { inode } if inode == FIRST_INODE_NUMBER
        ),
        "{refusal:?}"
    );
    assert_eq!(
        pool.retained_operations().len(),
        operations_before,
        "拒在任何写之前"
    );
}

#[test]
fn overwriting_the_first_file_after_it_is_deleted_is_refused_as_not_in_the_tree() {
    let mut pool = pool_with_a_three_unit_file("unlink-overwrite-after-delete");
    publish_the_intent(&mut pool).expect("意图");
    for _ in 0..3 {
        publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let refusal = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: b"after",
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
        },
        INSTANCE_OF_THE_PUBLISHES,
    )
    .expect_err("删完之后不再建带数据的文件（C618）");
    assert!(
        matches!(
            refusal,
            PublishError::WriteToAnInodeThatIsNotInTheTree { inode } if inode == FIRST_INODE_NUMBER
        ),
        "{refusal:?}"
    );
}
