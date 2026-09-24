//! C513（复用豁免不判那次复用合不合法）：记录核对器第二条判据（恢复自称的 txg ≥ 某次发布、那次发布的某个单元两份都不在）的复用豁免
//! （`crash::check_records_against` 里的 `written_over_later`）此前只问「更晚那次同槽的写落没落盘」，不问那次复用合不合法。
//! 另接的判据：那次复用照 D16（发布语义） 已定项 1 的回收谓词（可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)）
//! 证得出过不了时不开脱。
//!
//! 造的状态：复用窗口置 0（只供测试的开关，C22（刚释放的块立即重分配） 那一格）下 mkfs → A（txg 3）→ 覆盖写 txg 4 → 覆盖写 txg 5，
//! txg 5 的数据单元落回 txg 4 的数据单元那一对槽 50178——释放代 5，而 F 为 0、环里最旧的根 txg ≤ 1，回收谓词过不了。
//! 崩在 txg 5 的单元写全部落盘、记录与根槽都没落盘的那一刻：恢复走 txg 4，而 txg 4 的数据单元两份都被那次违规的复用盖掉了
//! （oracle 判红：读不出 txg 4 的内容）。收严之前记录核对器在这里开脱（更晚那次写已落盘）、判绿；接上回收谓词之后判红。
//! 合法复用那一侧（抬 F 之后落回 A 的数据单元那一对槽）照样开脱，由 `second_transaction_step_zero_layer0.rs` 的
//! `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state` 钉着（记录核对器两类计数都为 0）。

mod common;

use common::{build_pool, file_content, geometry, publish_overwrite_in_process};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::ReuseWindow;
use singlefs_core::recovery::{PoolReader, RecoveryOutcome};
use singlefs_harness::crash::{
    evaluate_state_for_versions, writes_and_segments, CrashImage, Layer0Tally, PublishedVersion,
};
use singlefs_harness::segments::StepKind;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 13 + seed) % 241).expect("小于 256"))
        .collect()
}

#[test]
fn an_illegal_reuse_whose_later_write_landed_no_longer_excuses_the_missing_unit_in_the_record_checker(
) {
    let mut pool = build_pool("record-checker-illegal-reuse");
    pool.allocator.set_reuse_window(ReuseWindow::ForcedToZero);
    let fourth_content = content_of(3100, 3);
    let fifth_content = content_of(2900, 7);
    let first = pool.output.clone();
    let fourth = publish_overwrite_in_process(
        &mut pool,
        &first,
        &fourth_content,
        common::FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 txg 4");
    let fifth = publish_overwrite_in_process(
        &mut pool,
        &fourth,
        &fifth_content,
        common::FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("覆盖写 txg 5");
    assert_eq!(
        (fourth.root.checkpoint_txg, fifth.root.checkpoint_txg),
        (CheckpointTxg(4), CheckpointTxg(5))
    );
    let reused_slot = SlotNumber(50178);
    assert_eq!(
        (
            fourth.data_pointers[0].locations[0].slot,
            fifth.data_pointers[0].locations[0].slot
        ),
        (reused_slot, reused_slot),
        "复用窗口置 0：txg 5 的数据单元落回 txg 4 的数据单元那一对槽（释放代 5，F = 0，回收谓词过不了）"
    );

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, _segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let (fourth_root_index, fifth_root_index) = match root_indexes.as_slice() {
        [.., fourth_root_index, fifth_root_index] => (*fourth_root_index, *fifth_root_index),
        too_few => panic!("写表里至少有暖机、A、txg 4、txg 5 的根槽写：{too_few:?}"),
    };
    let first_record_of_the_fifth_publish = (fourth_root_index + 1..fifth_root_index)
        .find(|index| writes[*index].kind == StepKind::JournalRecord)
        .expect("txg 5 写了 journal 记录");
    // 崩在 txg 5 的单元写全部落盘、它的记录与根槽都没落盘的那一刻。
    let persisted: Vec<bool> = (0..writes.len())
        .map(|index| index < first_record_of_the_fifth_publish)
        .collect();

    let copies_at_the_reused_slot: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite && write.offset == reused_slot.to_device_offset()
        })
        .map(|(index, _)| index)
        .collect();
    let (fourth_copies, fifth_copies): (Vec<usize>, Vec<usize>) = copies_at_the_reused_slot
        .iter()
        .partition(|index| **index < fourth_root_index);
    assert_eq!(
        (fourth_copies.len(), fifth_copies.len()),
        (2, 2),
        "50178 上两次写各两盘一份：txg 4 的数据单元与 txg 5 复用它的那一次"
    );
    let image = CrashImage {
        base: &base,
        writes: &writes,
        persisted: persisted.clone(),
    };
    for copy in &copies_at_the_reused_slot {
        let write = &writes[*copy];
        let length = usize::try_from(write.length_in_bytes()).expect("单元写的长度装得进 usize");
        let on_disk = PoolReader::read(&image, write.device, write.offset, length)
            .expect("50178 在两盘上都读得回来");
        assert_eq!(
            write.contents.still_on_disk(&on_disk),
            fifth_copies.contains(copy),
            "更晚那次写（txg 5）已落盘、txg 4 那一份被它盖掉了（写表下标 {copy}）"
        );
    }

    let versions = [
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: fourth_content,
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(5),
            content: fifth_content,
        },
    ];
    let mut tally = Layer0Tally::default();
    let report = evaluate_state_for_versions(
        &base,
        &writes,
        persisted,
        fifth_root_index,
        &versions,
        &mut tally,
    );
    println!(
        "ILLEGAL_REUSE effective_root={:?} failed={} violations={} record=({}, {})",
        report.effective_root,
        matches!(report.outcome, RecoveryOutcome::Failed { .. }),
        tally.violations,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit
    );
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "txg 5 的记录与根槽都没落盘，恢复走 txg 4"
    );
    assert_eq!(
        tally.violations, 1,
        "oracle 判红：恢复走 txg 4 而它的数据单元两份都被违规复用盖掉了（{:?}）",
        tally.first_violation
    );
    assert_eq!(
        tally.record_claimed_state_missing_unit, 1,
        "记录核对器第二条判据判红：更晚那次写已落盘，但那次复用过不了回收谓词，不开脱（C513）"
    );
    assert_eq!(
        tally.record_root_without_record, 0,
        "记录核对器第一条判据与这一格无关：落了盘的根槽写各自的记录都在"
    );
}
