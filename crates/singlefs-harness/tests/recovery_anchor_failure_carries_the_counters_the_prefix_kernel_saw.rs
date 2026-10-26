//! 取前缀的核（`singlefs_replay_prefix::replay_core`）报「所选根那次发布带末条标志的多于一条」时，`replay_journal` 拿核带出的计数器
//! 直接拼 `RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided` 返回，不再调一份手抄的筛选去重算、
//! 也不在两份筛选对不上时 panic（代码轮判决 `research/prompts/m3-verus-land-code-r1-main-verification.md` 第二节 C4 那一行，G6）。
//!
//! 压着的条款：D23（journal 的角色与格式） 已定项 14 注 1（锚点 = 与所选根同实例、同 checkpoint_txg、带末条标志的那一条，读法乙）
//! 与「这一版的失败处置」（同一 (实例代号, checkpoint_txg) 两条带末条标志就是盘坏了，在任何写之前停下）。
//!
//! 记录表是手造的：一块盘都没有的池当读者（反向链判不了、不拿它断前缀；记录不点名单元，点名验证恒过），
//! 只看取前缀那一截怎么认锚点。同实例里所选根之前、之后两次发布各带一条末条标志：它们不属于所选根那次发布，
//! 不进报错的计数器清单，也不让恢复停下。

use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::{
    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{replay_journal, NamedUnitVerification, RecoveryFailure};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const CHOSEN_ROOT_CHECKPOINT_TXG: CheckpointTxg = CheckpointTxg(2);
/// 根的 fsid 每个字节都取这个值；记录带它的低 8 字节（`unit_filesystem_identifier`）。读者是空池，反向链重读不了，取前缀不拿它比。
const FILESYSTEM_IDENTIFIER_BYTE: u8 = 0x53;
const FILESYSTEM_IDENTIFIER: [u8; 16] = [FILESYSTEM_IDENTIFIER_BYTE; 16];

fn chosen_root() -> RootRecord {
    RootRecord {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: INSTANCE,
        checkpoint_txg: CHOSEN_ROOT_CHECKPOINT_TXG,
        tree_table: NodePointer::empty_root(),
        tree_identifier_watermark: 0,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer::empty_root(),
        allocation_record_tree_root: NodePointer::empty_root(),
        reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
        pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
    }
}

/// 一次只有一条记录的发布：序号 1、带末条标志与提交标记，事务号取计数器。
fn single_record_publish(counter: u64, checkpoint_txg: CheckpointTxg) -> JournalRecord {
    JournalRecord {
        instance: INSTANCE,
        counter,
        checkpoint_txg,
        transaction: counter,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish(1),
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: 0,
        filesystem_identifier: unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER),
        new_tree_table: NodePointer::empty_root(),
        new_mapping_root: NodePointer::empty_root(),
        new_tree_identifier_watermark: 0,
        new_rollback_floor: CheckpointTxg(0),
        new_instance_table: NodePointer::empty_root(),
        new_allocation_record_tree_root: NodePointer::empty_root(),
        new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        named: Vec::new(),
        new_reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
        new_pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
    }
}

fn table_of(records: Vec<JournalRecord>) -> BTreeMap<(InstanceGeneration, u64), JournalRecord> {
    records
        .into_iter()
        .map(|record| ((record.instance, record.counter), record))
        .collect()
}

fn reader_with_no_device() -> MemoryPool {
    MemoryPool::with_devices(&[], 0)
}

/// jsn 1（txg 1）、jsn 2 与 jsn 3（都是所选根的 txg 2）、jsn 4（txg 3）各带末条标志：所选根那次发布带标志的是 jsn 2、3 两条，
/// 锚点认哪一条条款没写 ⇒ 报错，计数器只列这两条（另外两条不在所选根那次发布里）；不 panic。
#[test]
fn two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs_counters(
) {
    let records = table_of(vec![
        single_record_publish(1, CheckpointTxg(1)),
        single_record_publish(2, CHOSEN_ROOT_CHECKPOINT_TXG),
        single_record_publish(3, CHOSEN_ROOT_CHECKPOINT_TXG),
        single_record_publish(4, CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1)),
    ]);
    let replayed = replay_journal(
        &reader_with_no_device(),
        &chosen_root(),
        JOURNAL_RING_DEFAULT_BYTES,
        &records,
        NamedUnitVerification::Verify,
    );
    assert_eq!(
        replayed,
        Err(
            RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                instance: INSTANCE,
                checkpoint_txg: CHOSEN_ROOT_CHECKPOINT_TXG,
                counters: vec![2, 3],
            }
        ),
        "报错带的是所选根那次发布里带末条标志的两条（jsn 2、3），txg 1 与 txg 3 那两条不算"
    );
}

/// 同实例里所选根之前（jsn 1，txg 1）、之后（jsn 3，txg 3）的发布各带末条标志，所选根那次发布只有 jsn 2 带：
/// 锚点是 jsn 2，链首接 jsn 3，txg 3 那次发布整次施加。别的发布的末条标志不让恢复停下。
#[test]
fn last_record_flags_of_other_publishes_of_the_same_instance_leave_the_anchor_alone_and_the_next_publish_applies(
) {
    let next_publish_checkpoint_txg = CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1);
    let records = table_of(vec![
        single_record_publish(1, CheckpointTxg(1)),
        single_record_publish(2, CHOSEN_ROOT_CHECKPOINT_TXG),
        single_record_publish(3, next_publish_checkpoint_txg),
    ]);
    let (report, rebuilt_root) = replay_journal(
        &reader_with_no_device(),
        &chosen_root(),
        JOURNAL_RING_DEFAULT_BYTES,
        &records,
        NamedUnitVerification::Verify,
    )
    .expect("所选根那次发布只有 jsn 2 带末条标志：锚点认得出");
    assert_eq!(
        (report.above_water, report.prefix_applied),
        (1, 1),
        "水位之上只有 jsn 3，它接在锚点 jsn 2 之后、整次施加"
    );
    assert_eq!(
        (rebuilt_root.instance, rebuilt_root.checkpoint_txg),
        (INSTANCE, next_publish_checkpoint_txg),
        "重建出来的根是 txg 3 那一版"
    );
}

/// 核按切片次序收记录、前置条件是「按 (实例代号, 计数器) 严格升序」；胶水按记录表的键序交给它，所以键与记录自己的
/// (实例代号, 计数器) 不等时，胶水在调核之前断言失败，不带着次序不对的切片往下取前缀（判决同上 C1 那一行，G2）。
/// 造法：jsn 2、jsn 3 两条的键对调（今天没有调用方这样建表）。
#[test]
#[should_panic(expected = "记录表的键要等于记录自己的 (实例代号, 计数器)")]
fn a_table_whose_keys_disagree_with_its_records_trips_the_glue_assertion_before_the_kernel_runs() {
    let mut records = table_of(vec![single_record_publish(1, CHOSEN_ROOT_CHECKPOINT_TXG)]);
    let next_publish_checkpoint_txg = CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1);
    records.insert(
        (INSTANCE, 3),
        single_record_publish(2, next_publish_checkpoint_txg),
    );
    records.insert(
        (INSTANCE, 2),
        single_record_publish(3, next_publish_checkpoint_txg),
    );
    let replayed = replay_journal(
        &reader_with_no_device(),
        &chosen_root(),
        JOURNAL_RING_DEFAULT_BYTES,
        &records,
        NamedUnitVerification::Verify,
    );
    panic!("键与记录不等的表没被胶水的断言拦下，取前缀交回了 {replayed:?}");
}

/// 核要给链上下一号加一，前置条件是计数器都不是 u64 的最大值；盘上的计数器是 6 字节，产品路径到不了，
/// 直接造表的调用方给了这个值时，胶水在调核之前断言失败。
#[test]
#[should_panic(expected = "计数器是盘上 6 字节读来的")]
fn a_record_whose_counter_is_the_largest_value_trips_the_glue_assertion_before_the_kernel_runs() {
    let records = table_of(vec![single_record_publish(
        u64::MAX,
        CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1),
    )]);
    let replayed = replay_journal(
        &reader_with_no_device(),
        &chosen_root(),
        JOURNAL_RING_DEFAULT_BYTES,
        &records,
        NamedUnitVerification::Verify,
    );
    panic!("计数器是 u64 最大值的记录没被胶水的断言拦下，取前缀交回了 {replayed:?}");
}
