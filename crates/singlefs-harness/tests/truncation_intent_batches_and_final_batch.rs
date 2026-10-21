//! 截断的意图、各批与最后一批（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2、K3、K10、K11、K12）：
//! 意图那次写 size = N、签截断条目（logged_ops 种类 2）、截到单元 k 中间时只重写单元 k；之后每批只从尾部放单元、不碰 inode 记录，
//! 最后一批放完 size 之外剩下的（可以 0 个）并同一次发布删条目。读者在批间读到截后的 N，size 之外还挂着的单元不读。
//! 三个单元的第一个文件（2.5 个净荷）、只供测试的批大小 1，另有一个 150 个单元的文件截在第 0 片下段叶的末尾。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{build_pool, sequential_write_in_process_as_instance_one_and_advance_the_pool};
use common_deletion_in_progress::{
    content_of_three_data_units, pool_with_a_three_unit_file,
    publish_new_inodes_and_advance_the_pool, publish_the_intent_of,
};
use common_truncation::{
    assert_both_readers_read, batches_of_one, change_the_size_of_the_first_file_and_advance,
    payload_capacity, publish_size_change_of_and_advance,
    publish_the_next_truncation_step_and_advance, remount_with_batches_of_one, truncation_entry_of,
    walk_the_queues_and_advance, FIRST_FILE, WRITE_TIME_SECONDS_OF_THE_SIZE_CHANGE,
};
use singlefs_core::address::{DataUnitIndexInFile, InodeNumber};
use singlefs_core::logged_operations::{
    LoggedOperation, LoggedOperationEntry, TruncationIntent, LOGGED_OPERATION_KIND_TRUNCATION,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::transaction::{
    DeletionStepOfThePublish, PublishError, TransactionOutput, TransactionUnit, TruncationRefusal,
};
use singlefs_core::truncate::TailDataUnitRewrite;
use singlefs_core::unit::parse_data_unit;
use singlefs_core::unlink::UnlinkBatchSize;

/// 一版里第一个文件的 (size, 数据单元数, 有没有截断条目)。
fn size_units_and_entry(version: &TransactionOutput) -> (u64, usize, bool) {
    (
        version.inode_record.expect("第一个文件那条记录在").size,
        version.data_pointers.len(),
        truncation_entry_of(version).is_some(),
    )
}

/// 一版里第 `unit` 个数据单元头里的声明长度。
fn declared_length_of(version: &TransactionOutput, unit: u64) -> u16 {
    parse_data_unit(
        &version
            .unit(TransactionUnit::Data(DataUnitIndexInFile(unit)))
            .bytes,
    )
    .expect("数据单元解得开")
    .declared_length
}

fn tags_of(roles: &[TransactionUnit]) -> Vec<String> {
    roles.iter().map(|role| role.tag()).collect()
}

/// 截到单元 0 中间（N = 100），批大小 1：意图（size 100、三个单元都挂着、截断条目在）、放到两个、最后一批放到一个并删条目，版本序列钉绝对值。
/// 意图那次只重写单元 0（声明长度 100），换下的数据单元只有单元 0（K3：只重写 Data(0) 的发布换下清单只有 Data(0)）；
/// 各批不碰 inode 记录（与意图那一版的记录逐字段相同）。意图那一版与之后每一版两条读路径都读出截后的 100 字节。
#[test]
fn truncating_a_three_unit_file_into_unit_zero_in_batches_of_one_publishes_the_intent_two_batches_and_replaces_only_unit_zero_at_the_intent(
) {
    let mut pool = pool_with_a_three_unit_file("truncation-into-unit-zero");
    let content = content_of_three_data_units();
    let before = pool.output.clone();
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    assert_eq!(
        declared_length_of(&intent, 0),
        100,
        "单元 0 重写成声明长度 100"
    );
    let rewritten_data_roles: Vec<String> = tags_of(&intent.rewritten)
        .into_iter()
        .filter(|tag| tag.starts_with("t1"))
        .collect();
    assert_eq!(rewritten_data_roles, ["t1"], "意图那次只重写单元 0");
    let data_unit_slot = |version: &TransactionOutput, unit: u64| {
        version
            .unit(TransactionUnit::Data(DataUnitIndexInFile(unit)))
            .slot
    };
    let released_slots: Vec<_> = intent
        .released
        .iter()
        .map(|placement| placement.slot)
        .collect();
    assert!(
        released_slots.contains(&data_unit_slot(&before, 0)),
        "单元 0 的旧落点经映射换下"
    );
    assert!(
        !released_slots.contains(&data_unit_slot(&before, 1))
            && !released_slots.contains(&data_unit_slot(&before, 2)),
        "单元 1、2 还挂着，意图那次不换下：{released_slots:?}"
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &intent.root,
        &content[..100],
        "意图那一版",
    );
    let batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert_both_readers_read(
        &pool.memory_pool(),
        &batch.root,
        &content[..100],
        "一批之后",
    );
    let final_batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert_both_readers_read(
        &pool.memory_pool(),
        &final_batch.root,
        &content[..100],
        "最后一批之后",
    );
    assert_eq!(
        [&intent, &batch, &final_batch].map(size_units_and_entry),
        [(100, 3, true), (100, 2, true), (100, 1, false)],
        "意图、放到两个、最后一批放到一个并删截断条目"
    );
    assert!(
        batch.inode_record == intent.inode_record
            && final_batch.inode_record == intent.inode_record,
        "各批不碰 inode 记录（K2）"
    );
}

/// 截到单元 1 中间（N = 净荷 + 7）：意图那次只重写单元 1，声明长度 7；产品批大小下一次最后一批放掉单元 2；读回前 净荷 + 7 字节。
#[test]
fn truncating_into_the_middle_of_unit_one_rewrites_unit_one_to_the_bytes_beyond_its_start() {
    let mut pool = pool_with_a_three_unit_file("truncation-into-unit-one");
    let content = content_of_three_data_units();
    let new_size = payload_capacity() + 7;
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, new_size);
    let rewritten_data_roles: Vec<String> = tags_of(&intent.rewritten)
        .into_iter()
        .filter(|tag| tag.starts_with("t1"))
        .collect();
    assert_eq!(rewritten_data_roles, ["t1+1"], "意图那次只重写单元 1");
    assert_eq!(declared_length_of(&intent, 1), 7);
    let steps = walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    assert_eq!(
        steps.iter().map(size_units_and_entry).collect::<Vec<_>>(),
        [(new_size, 2, false)],
        "产品批大小：一次最后一批"
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &pool.output.root,
        &content[..usize::try_from(new_size).expect("长度")],
        "截后",
    );
}

/// K11 单一路径：截到单元 2 中间（N = 2 × 净荷 + 100）不放任何单元，照样是意图（重写单元 2）+ 一个放 0 个单元的最后一批（删条目）。
#[test]
fn a_truncation_releasing_no_data_unit_still_publishes_an_intent_and_a_final_batch() {
    let mut pool = pool_with_a_three_unit_file("truncation-releasing-nothing");
    let new_size = 2 * payload_capacity() + 100;
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, new_size);
    assert_eq!(declared_length_of(&intent, 2), 100);
    let steps = walk_the_queues_and_advance(&mut pool, batches_of_one());
    assert_eq!(
        steps.iter().map(size_units_and_entry).collect::<Vec<_>>(),
        [(new_size, 3, false)],
        "只有一个最后一批：三个单元都留着、删截断条目"
    );
    assert!(
        !tags_of(&steps[0].rewritten)
            .iter()
            .any(|tag| tag.starts_with("t1")),
        "最后一批不写数据单元"
    );
}

/// K1：意图那一版从盘上重建出来的 logged_ops 就是发布交回的那一条截断条目：key = 种类 2 + 签发 txg（意图那次的 txg），
/// value = inode 1、第一个文件的对象出生代、目标 size；条目 44 字节逐字节。
#[test]
fn the_truncation_entry_rebuilt_from_disk_is_the_kind_two_entry_the_intent_issued() {
    let mut pool = pool_with_a_three_unit_file("truncation-entry-on-disk");
    let object_birth = pool.output.inode_record.expect("第一个文件").object_birth;
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    let expected = LoggedOperationEntry {
        issued_txg: intent.root.checkpoint_txg,
        operation: LoggedOperation::Truncation(TruncationIntent {
            inode: FIRST_FILE,
            object_birth,
            target_size_in_bytes: 100,
        }),
    };
    assert_eq!(intent.logged_operations, [expected]);
    let RebuiltVersion::WithFile(rebuilt) = rebuild_version(
        &pool.memory_pool(),
        &intent.root,
        Some(intent.record.clone()),
    )
    .expect("从盘上重建") else {
        panic!("意图那一版带文件");
    };
    assert_eq!(rebuilt.logged_operations, [expected], "盘上读回同一条");
    let mut expected_bytes = LOGGED_OPERATION_KIND_TRUNCATION.to_le_bytes().to_vec();
    expected_bytes.extend_from_slice(&intent.root.checkpoint_txg.0.to_le_bytes());
    expected_bytes.extend_from_slice(&1u64.to_le_bytes());
    expected_bytes.extend_from_slice(&object_birth.0.to_le_bytes());
    expected_bytes.extend_from_slice(&100u64.to_le_bytes());
    expected_bytes.extend_from_slice(&[0; 10]);
    assert_eq!(expected.to_bytes(), expected_bytes, "条目 44 字节");
}

/// K1：不在树里的 inode 改 size 在任何写之前拒成类型化错误，一个写都不发。
#[test]
fn a_size_change_of_an_inode_that_is_not_in_the_tree_is_refused_before_any_write() {
    let mut pool = pool_with_a_three_unit_file("truncation-of-a-missing-inode");
    let operations_before = pool.retained_operations().len();
    let refused = publish_size_change_of_and_advance(&mut pool, InodeNumber(99), 0)
        .expect_err("inode 99 不在树里");
    assert!(
        matches!(
            refused,
            PublishError::TruncationRefused(TruncationRefusal::InodeIsNotALiveRecord { inode: 99 })
        ),
        "{refused:?}"
    );
    assert_eq!(
        pool.retained_operations().len(),
        operations_before,
        "一个写都不发"
    );
}

/// K1：删除中（nlink = 0）的 inode 改 size 在任何写之前拒成类型化错误，一个写都不发。
#[test]
fn a_size_change_of_an_inode_being_deleted_is_refused_before_any_write() {
    let mut pool = build_pool("truncation-of-an-inode-being-deleted");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    publish_the_intent_of(&mut pool, 2);
    let operations_before = pool.retained_operations().len();
    let refused = publish_size_change_of_and_advance(&mut pool, InodeNumber(2), 0)
        .expect_err("inode 2 在删除中");
    assert!(
        matches!(
            refused,
            PublishError::TruncationRefused(TruncationRefusal::InodeIsNotALiveRecord { inode: 2 })
        ),
        "{refused:?}"
    );
    assert_eq!(
        pool.retained_operations().len(),
        operations_before,
        "一个写都不发"
    );
}

/// 第一版只有第一个文件在 extent 树里有条目：别的 inode 改成别的 size 条款没写，拒成类型化错误；改成原来的 size（只改记录）照做。
#[test]
fn a_size_change_of_an_inode_other_than_the_first_file_is_refused_unless_the_size_stays() {
    let mut pool = build_pool("size-change-of-a-second-inode");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let operations_before = pool.retained_operations().len();
    let refused = publish_size_change_of_and_advance(&mut pool, InodeNumber(2), 10)
        .expect_err("inode 2 变长条款没写");
    assert!(
        matches!(
            refused,
            PublishError::TruncationRefused(
                TruncationRefusal::SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses {
                    inode: 2
                }
            )
        ),
        "{refused:?}"
    );
    assert_eq!(
        pool.retained_operations().len(),
        operations_before,
        "一个写都不发"
    );
    let unchanged = publish_size_change_of_and_advance(&mut pool, InodeNumber(2), 0)
        .expect("size 不变只改记录");
    assert!(truncation_entry_of(&unchanged).is_none());
}

/// K12：N = S 发一次只改记录的发布：记录的改动计数 = 这次的 txg、时间是给的，size、数据单元、extent 树都不动，不签截断条目；
/// 之后走空一个写都不发。
#[test]
fn a_size_change_to_the_current_size_publishes_one_version_changing_only_the_record() {
    let mut pool = pool_with_a_three_unit_file("size-change-to-the-current-size");
    let before = pool.output.clone();
    let size = before.inode_record.expect("第一个文件").size;
    let changed = change_the_size_of_the_first_file_and_advance(&mut pool, size);
    let record = changed.inode_record.expect("第一个文件");
    assert_eq!(
        (record.size, record.change_count, record.write_time_seconds),
        (
            size,
            changed.root.checkpoint_txg.0,
            WRITE_TIME_SECONDS_OF_THE_SIZE_CHANGE
        )
    );
    assert_eq!(changed.data_pointers, before.data_pointers, "数据单元不动");
    assert!(
        !tags_of(&changed.rewritten)
            .iter()
            .any(|tag| tag.starts_with("t1") || tag.starts_with("t2")),
        "不写数据单元、不写 extent 树：{:?}",
        tags_of(&changed.rewritten)
    );
    assert!(truncation_entry_of(&changed).is_none(), "不签截断条目");
    assert!(
        walk_the_queues_and_advance(&mut pool, batches_of_one()).is_empty(),
        "之后走空一个写都不发"
    );
}

/// K12：⌈N ÷ 净荷⌉ × 净荷溢出 u64 的 N 在任何写之前拒成类型化错误，一个写都不发。
#[test]
fn a_target_size_whose_whole_data_units_overflow_is_refused_before_any_write() {
    let mut pool = pool_with_a_three_unit_file("truncation-target-overflow");
    let operations_before = pool.retained_operations().len();
    let refused = publish_size_change_of_and_advance(&mut pool, FIRST_FILE, u64::MAX)
        .expect_err("u64::MAX 向上取整到整单元溢出");
    assert!(
        matches!(
            refused,
            PublishError::TruncationRefused(
                TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {
                    target_size_in_bytes: u64::MAX
                }
            )
        ),
        "{refused:?}"
    );
    assert_eq!(
        pool.retained_operations().len(),
        operations_before,
        "一个写都不发"
    );
}

/// K10：截断的意图、各批、最后一批是删除类（准入不判式子那一判），变长与只改记录是普通写。
#[test]
fn the_truncation_publishes_are_deletion_class_and_the_size_change_without_an_intent_is_not() {
    let record = singlefs_core::records::InodeRecord {
        inode: 1,
        object_birth: singlefs_core::address::CheckpointTxg(3),
        nlink: 1,
        size: 100,
        change_count: 9,
        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_SIZE_CHANGE,
        truncation_field:
            singlefs_core::records::InodeRecordTruncationField::written_by_the_truncation_intent_of(
                singlefs_core::address::CheckpointTxg(9),
                100,
            ),
    };
    let tail_data_unit_rewrite = Some(TailDataUnitRewrite {
        unit: DataUnitIndexInFile(0),
        declared_length_in_bytes: 100,
    });
    assert!(DeletionStepOfThePublish::TruncationIntent {
        record_after_the_intent: record,
        tail_data_unit_rewrite,
    }
    .is_a_deletion_class_publish());
    assert!(
        DeletionStepOfThePublish::TruncationBatch { data_units_kept: 2 }
            .is_a_deletion_class_publish()
    );
    assert!(
        DeletionStepOfThePublish::TruncationFinalBatch { data_units_kept: 1 }
            .is_a_deletion_class_publish()
    );
    assert!(!DeletionStepOfThePublish::SizeChangeWithoutAnIntent {
        record_after_the_change: record,
        tail_data_unit_rewrite,
    }
    .is_a_deletion_class_publish());
}

/// K10：截断意图那次每盘新占的槽（分配记录树之外：数据单元 2 槽，extent 树、inode 树、logged_ops、记账、映射、树表各节点）钉绝对值，
/// 并核它不超过 W 里那一项 24（`admission::DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST`）。
#[test]
fn the_truncation_intent_newly_takes_eleven_slots_per_device_outside_the_allocation_record_tree() {
    let mut pool = pool_with_a_three_unit_file("truncation-intent-slots");
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    let outside_the_allocation_record_tree: Vec<TransactionUnit> = intent
        .rewritten
        .iter()
        .copied()
        .filter(|role| {
            !matches!(
                role,
                TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                    | TransactionUnit::AllocationTree
            )
        })
        .collect();
    let slots: u64 = outside_the_allocation_record_tree
        .iter()
        .map(|role| role.span_slots())
        .sum();
    assert_eq!(
        slots,
        11,
        "意图那次分配记录树之外每盘新占的槽：{:?}",
        tags_of(&outside_the_allocation_record_tree)
    );
    assert!(
        slots <= singlefs_core::admission::DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST
    );
}

/// 会话里一次 `UserChange::Truncate` = 先走空 + 意图 + 全部批：批大小 1 时交回意图、放到两个、最后一批三版，按 txg 连着。
#[test]
fn a_session_truncate_publishes_the_intent_and_every_batch_in_one_call() {
    let mut pool = pool_with_a_three_unit_file("session-truncate");
    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 100,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_SIZE_CHANGE,
            },
        )
        .expect("会话截断");
    assert_eq!(
        published
            .versions_published
            .iter()
            .map(size_units_and_entry)
            .collect::<Vec<_>>(),
        [(100, 3, true), (100, 2, true), (100, 1, false)]
    );
    let txgs: Vec<u64> = published
        .versions_published
        .iter()
        .map(|version| version.root.checkpoint_txg.0)
        .collect();
    assert_eq!(txgs, [txgs[0], txgs[0] + 1, txgs[0] + 2], "按 txg 连着");
}

/// K3：150 个单元的文件截到单元 143 中间（留下 144 个，单元 k = 143 在第 0 片下段叶的末尾、叶边界上）：意图那次重写单元 143
/// 与罩着它的第 0 片叶到根；之后那一批截掉第 1 片叶、第 0 片叶照抄成下段根。读回前 143 × 净荷 + 9 字节。
#[test]
fn keeping_one_hundred_and_forty_four_data_units_rewrites_the_leaf_holding_the_cut_unit_and_reads_back(
) {
    let mut pool = build_pool("truncation-keeping-one-hundred-and-forty-four");
    let content: Vec<u8> = (0..150 * payload_capacity())
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    let new_size = 143 * payload_capacity() + 9;
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, new_size);
    let intent_tags = tags_of(&intent.rewritten);
    assert!(
        intent_tags.iter().any(|tag| tag == "t1+143")
            && intent_tags.iter().any(|tag| tag == "t2下@0.0")
            && intent_tags.iter().any(|tag| tag == "t2下@1.0")
            && intent_tags.iter().any(|tag| tag == "t2"),
        "单元 143、第 0 片叶、下段根、上段根都重写：{intent_tags:?}"
    );
    walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    assert_eq!(size_units_and_entry(&pool.output), (new_size, 144, false));
    assert_both_readers_read(
        &pool.memory_pool(),
        &pool.output.root,
        &content[..usize::try_from(new_size).expect("长度")],
        "截后",
    );
}
