//! inode 记录的截断字段 T 与 N_w（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2、R5 与第六节用户定案）每一种发布写成什么：
//! 截断意图那次放了单元才写（这次 txg, N）、整个替换，否则照抄上一版；各批与最后一批不碰记录；变长与 N = S 照抄；整份写（顺序写、覆盖写）清零；
//! unlink 的意图与各批照抄；建 inode 写 0；回退随 R_old，不另写。每条用例钉字段的绝对值（txg 按这条历史数：mkfs 第 0 代、暖机两次、
//! 新池新建文件 txg 3，之后每次发布加一），另经只读挂载从盘上读回来比一次。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{
    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
};
use common_deletion_in_progress::{
    content_of_three_data_units, pool_with_a_three_unit_file,
    publish_new_inodes_and_advance_the_pool, publish_the_intent,
    publish_the_next_step_in_batches_of_one, roll_back_to,
};
use common_truncation::{
    batches_of_one, change_the_size_of_the_first_file_and_advance, payload_capacity,
    publish_the_next_truncation_step_and_advance, walk_the_queues_and_advance,
};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::{InodeRecord, InodeRecordTruncationField};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};

/// 三个单元的第一个文件那条顺序写之后，截断意图那次发布的 checkpoint_txg（新池新建文件 3、顺序写 4、意图 5）。
const TXG_OF_THE_TRUNCATION_INTENT_AFTER_THE_THREE_UNIT_WRITE: CheckpointTxg = CheckpointTxg(5);
/// 截断的目标落在单元 0 中间。
const SIZE_INSIDE_UNIT_ZERO: u64 = 100;

fn written(txg: u64, size_in_bytes: u64) -> InodeRecordTruncationField {
    InodeRecordTruncationField::written_by_the_truncation_intent_of(
        CheckpointTxg(txg),
        size_in_bytes,
    )
}

/// 这一版里 `inode` 那条记录（发布路径交回的内存里那一份）。
fn record_of(version: &TransactionOutput, inode: u64) -> InodeRecord {
    version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode)
        .copied()
        .unwrap_or_else(|| panic!("这一版里有 inode {inode} 的记录"))
}

/// 池现在那一版从盘上读回来（只读挂载择根、扫环、打开挂载态）的第一个文件那条记录的截断字段。
fn truncation_field_of_the_first_file_read_back_from_the_disk(
    pool: &BuiltPool,
) -> InodeRecordTruncationField {
    let image = pool.memory_pool();
    let mounted = mount_read_only(&image).expect("只读挂载");
    mounted
        .mounted
        .inode_records()
        .iter()
        .find(|record| record.inode == FIRST_INODE_NUMBER)
        .expect("盘上那一版有第一个文件那条记录")
        .truncation_field
}

/// 三个单元（2.5 个净荷）截到单元 0 中间：意图那次放两个单元之前存在三个，多于要的一个 ⇒ 写成（5, 100）；
/// T 是这次发布自己的 checkpoint_txg，N_w 是目标 size。从盘上读回来是同一个值。
#[test]
fn the_truncation_intent_that_releases_data_units_writes_its_own_txg_and_the_target_size() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-intent-writes");
    assert_eq!(
        record_of(&pool.output, FIRST_INODE_NUMBER).truncation_field,
        InodeRecordTruncationField::NOT_WRITTEN,
        "前提：顺序写之后字段是 0"
    );
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    assert_eq!(
        intent.root.checkpoint_txg, TXG_OF_THE_TRUNCATION_INTENT_AFTER_THE_THREE_UNIT_WRITE,
        "前提：意图那次是 txg 5"
    );
    assert_eq!(
        record_of(&intent, FIRST_INODE_NUMBER).truncation_field,
        written(5, SIZE_INSIDE_UNIT_ZERO),
        "意图那次写（这次 txg 5, N 100）"
    );
    assert_eq!(
        truncation_field_of_the_first_file_read_back_from_the_disk(&pool),
        written(5, SIZE_INSIDE_UNIT_ZERO),
        "盘上读回来是（5, 100）"
    );
}

/// 批大小 1：意图之后一批（放到两个）、最后一批（放到一个、删截断条目）都不碰记录，字段留着意图那次写的（5, 100）。
#[test]
fn the_truncation_batches_and_the_final_batch_leave_the_field_the_intent_wrote() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-batches-leave");
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    let batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    let final_batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert_eq!(
        [&batch, &final_batch].map(|version| version.data_pointers.len()),
        [2, 1],
        "前提：一批放到两个、最后一批放到一个"
    );
    assert_eq!(
        [&batch, &final_batch]
            .map(|version| record_of(version, FIRST_INODE_NUMBER).truncation_field),
        [
            written(5, SIZE_INSIDE_UNIT_ZERO),
            written(5, SIZE_INSIDE_UNIT_ZERO)
        ],
        "各批与最后一批不碰记录"
    );
    assert_eq!(
        truncation_field_of_the_first_file_read_back_from_the_disk(&pool),
        written(5, SIZE_INSIDE_UNIT_ZERO)
    );
}

/// 三个单元截到两个净荷整（放一个，写成（5, 2 × 净荷）），走空之后再截到两个净荷少 10 字节：截之前两个单元、要两个，不放单元 ⇒
/// 照抄上一版的（5, 2 × 净荷），不写这次的（7, …）。
#[test]
fn a_truncation_within_the_last_data_unit_releases_nothing_and_carries_the_field_of_the_version_before(
) {
    let mut pool = pool_with_a_three_unit_file("truncation-field-within-the-last-unit");
    let two_payloads = 2 * payload_capacity();
    change_the_size_of_the_first_file_and_advance(&mut pool, two_payloads);
    let walked = walk_the_queues_and_advance(&mut pool, batches_of_one());
    assert_eq!(walked.len(), 1, "前提：一个单元一批，最后一批放掉它");
    assert_eq!(pool.output.data_pointers.len(), 2, "前提：走空之后两个单元");
    let second_intent = change_the_size_of_the_first_file_and_advance(&mut pool, two_payloads - 10);
    assert_eq!(
        second_intent.root.checkpoint_txg,
        CheckpointTxg(7),
        "前提：第二次截断是 txg 7"
    );
    assert_eq!(
        record_of(&second_intent, FIRST_INODE_NUMBER).truncation_field,
        written(5, two_payloads),
        "不放单元：照抄上一版"
    );
}

/// 截到 100、走空（txg 5 意图、6 最后一批），变长到三个净荷（txg 7，尾部是洞，只剩一个单元），再截到两个净荷（txg 8）：
/// 截之前一个单元、要两个 ⇒ 截到洞里、不放单元，照抄（5, 100）。
#[test]
fn truncating_into_the_hole_after_an_extension_carries_the_field_of_the_version_before() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-into-the-hole");
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    walk_the_queues_and_advance(
        &mut pool,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
    );
    change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity());
    assert_eq!(
        pool.output.data_pointers.len(),
        1,
        "前提：变长之后只有单元 0，之后是洞"
    );
    let into_the_hole =
        change_the_size_of_the_first_file_and_advance(&mut pool, 2 * payload_capacity());
    assert_eq!(
        into_the_hole.root.checkpoint_txg,
        CheckpointTxg(8),
        "前提：截到洞里那次是 txg 8"
    );
    assert_eq!(
        record_of(&into_the_hole, FIRST_INODE_NUMBER).truncation_field,
        written(5, SIZE_INSIDE_UNIT_ZERO),
        "截到洞里不放单元：照抄（5, 100）"
    );
    assert_eq!(
        truncation_field_of_the_first_file_read_back_from_the_disk(&pool),
        written(5, SIZE_INSIDE_UNIT_ZERO)
    );
}

/// 截到 100、走空之后变长到 200（txg 7）再改成同样的 200（N = S，txg 8）：两次都照抄（5, 100）。
#[test]
fn an_extension_and_a_size_change_to_the_same_size_carry_the_field() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-extension-carries");
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    walk_the_queues_and_advance(
        &mut pool,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let extension = change_the_size_of_the_first_file_and_advance(&mut pool, 200);
    let unchanged = change_the_size_of_the_first_file_and_advance(&mut pool, 200);
    assert_eq!(
        [&extension, &unchanged].map(|version| version.root.checkpoint_txg),
        [CheckpointTxg(7), CheckpointTxg(8)],
        "前提：变长 txg 7、N = S txg 8"
    );
    assert_eq!(
        [&extension, &unchanged]
            .map(|version| record_of(version, FIRST_INODE_NUMBER).truncation_field),
        [
            written(5, SIZE_INSIDE_UNIT_ZERO),
            written(5, SIZE_INSIDE_UNIT_ZERO)
        ],
        "变长与 N = S 都照抄"
    );
}

/// 截到 100、走空之后覆盖写一版（一个单元的内容）：整份写清零。
#[test]
fn a_whole_file_overwrite_after_a_truncation_clears_the_field() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-overwrite-clears");
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    walk_the_queues_and_advance(
        &mut pool,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
    );
    assert_eq!(
        record_of(&pool.output, FIRST_INODE_NUMBER).truncation_field,
        written(5, SIZE_INSIDE_UNIT_ZERO),
        "前提：覆盖写之前字段是（5, 100）"
    );
    let overwritten =
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"after");
    assert_eq!(
        record_of(&overwritten, FIRST_INODE_NUMBER).truncation_field,
        InodeRecordTruncationField::NOT_WRITTEN,
        "覆盖写清零"
    );
    assert_eq!(
        truncation_field_of_the_first_file_read_back_from_the_disk(&pool),
        InodeRecordTruncationField::NOT_WRITTEN
    );
}

/// 截到 100、走空之后顺序写回三个单元：整份写清零。
#[test]
fn a_whole_file_sequential_write_after_a_truncation_clears_the_field() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-sequential-write-clears");
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    walk_the_queues_and_advance(
        &mut pool,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let rewritten = sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    assert_eq!(rewritten.data_pointers.len(), 3, "前提：顺序写回三个单元");
    assert_eq!(
        record_of(&rewritten, FIRST_INODE_NUMBER).truncation_field,
        InodeRecordTruncationField::NOT_WRITTEN,
        "顺序写清零"
    );
}

/// 三个单元截到两个净荷整（写成（5, 2 × 净荷））、走空之后 unlink 第一个文件：意图（nlink 0）与之后那一批都照抄字段。
#[test]
fn the_unlink_intent_and_its_batches_carry_the_field() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-unlink-carries");
    let two_payloads = 2 * payload_capacity();
    change_the_size_of_the_first_file_and_advance(&mut pool, two_payloads);
    walk_the_queues_and_advance(
        &mut pool,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let unlink_intent = publish_the_intent(&mut pool);
    let unlink_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    assert_eq!(
        [&unlink_intent, &unlink_batch].map(|version| {
            let record = record_of(version, FIRST_INODE_NUMBER);
            (record.nlink, record.truncation_field)
        }),
        [(0, written(5, two_payloads)), (0, written(5, two_payloads))],
        "unlink 的意图与那一批（不是最后一批）都照抄"
    );
}

/// 回退随 R_old（判决 m4-trunc-r2 R5）：截到 100（写成（5, 100））、走空之后回退到截断意图之前那一版（顺序写那一版，字段 0）：
/// 回退那一版的字段是 0，不把现行那一版的（5, 100）并回来。
#[test]
fn rolling_back_to_the_version_before_the_truncation_intent_restores_a_zero_field() {
    let mut pool = pool_with_a_three_unit_file("truncation-field-rollback");
    let before_the_intent = pool.output.clone();
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    walk_the_queues_and_advance(
        &mut pool,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
    );
    assert_eq!(
        record_of(&pool.output, FIRST_INODE_NUMBER).truncation_field,
        written(5, SIZE_INSIDE_UNIT_ZERO),
        "前提：回退之前字段是（5, 100）"
    );
    let rolled_back = roll_back_to(&mut pool, &before_the_intent);
    let record = record_of(&rolled_back, FIRST_INODE_NUMBER);
    assert_eq!(
        (record.size, record.truncation_field),
        (
            u64::try_from(content_of_three_data_units().len()).expect("长度"),
            InodeRecordTruncationField::NOT_WRITTEN
        ),
        "回退到写字段之前那一版：size 回到 2.5 个净荷、字段 0"
    );
    assert_eq!(
        truncation_field_of_the_first_file_read_back_from_the_disk(&pool),
        InodeRecordTruncationField::NOT_WRITTEN
    );
}

/// 截到 100（写成（5, 100））、走空之后建两个 inode：新建的两条字段是 0，第一个文件那条照抄（5, 100）。
#[test]
fn new_inodes_are_created_with_a_zero_field_beside_a_truncated_first_file() {
    let mut pool = build_pool("truncation-field-new-inodes");
    let content = content_of_three_data_units();
    sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    walk_the_queues_and_advance(
        &mut pool,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
    );
    let created = publish_new_inodes_and_advance_the_pool(&mut pool, 2);
    assert_eq!(
        [FIRST_INODE_NUMBER, 2, 3].map(|inode| record_of(&created, inode).truncation_field),
        [
            written(5, SIZE_INSIDE_UNIT_ZERO),
            InodeRecordTruncationField::NOT_WRITTEN,
            InodeRecordTruncationField::NOT_WRITTEN
        ],
        "建 inode 写 0、第一个文件照抄"
    );
}
