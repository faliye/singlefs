//! 可写挂载退回见证到的那一版之后、按它当水位再判缺记录（`F1_AFTER_F2`）之前先重扫一遍 journal 环，用新扫出的记录判
//! （C754（退回之后再判缺记录不重扫环），用户 2026-10-20 定「重扫」；两处「重读一次」都是环与点名单元一起重读）。
//!
//! 盘面与 `writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault.rs` 同一个（`tests/common_unwitnessed_publish/mod.rs`）：
//! 384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的顺序写 R，崩在 R 的根槽落盘之后、系统配置轮换之前；R 的树表两份一直读回全零，
//! R 被择、走读失败，F2 退回 P。不同的是 R 的第 1 条记录只在每块盘上头一次读回全零（读阶段扫环那一次），之后读得出。
mod common;
mod common_unwitnessed_publish;

use common::{
    with_unreadable_ranges, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
    UnreadableRangeReadBack,
};
use common_unwitnessed_publish::{
    plain_devices_on, record_one_publish_after_overwrites, PublishRecordedAsR,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ShadowLedger,
};
use singlefs_harness::memory_pool::RetainedWrite;

const OVERWRITES_BEFORE_R: u64 = 5;

fn unreadable_ranges_of(
    writes: &[&RetainedWrite],
    failing_reads: FailingReadsOfARange,
) -> Vec<UnreadableRange> {
    writes
        .iter()
        .map(|write| UnreadableRange {
            device: write.device,
            offset_in_bytes: write.offset.0,
            length_in_bytes: write.length_in_bytes(),
            failing_reads,
        })
        .collect()
}

/// 读阶段扫环时 R 的第 1 条记录在两块盘上都读回全零，退回 P 之后重扫读得出：R 的三条记录齐（序号 1..3、带末条），
/// 按 P 当水位再判时没有缺记录的发布，「重读一次」那一支不走（钩子一次都没调），可写挂载建在 P 上做成。
/// 判别力：再判用读阶段扫出的那一份记录（变异「C754 退回之后再判缺记录用读阶段扫出的记录、不重扫环」）时 R 缺第 1 条、点名的树表验不过，
/// 走「重读一次」、钩子调了一次，这一条红。
#[test]
fn a_record_read_back_as_zero_once_while_scanning_the_ring_is_read_again_before_rejudging_and_leaves_no_gap(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let tree_table_writes = recorded.unit_writes_at(&recorded.root().tree_table);
    assert_eq!(tree_table_writes.len(), 2, "R 自己写了树表，两块盘各一份");
    let mut ranges = unreadable_ranges_of(
        &recorded.record_writes_of_the_record(1),
        FailingReadsOfARange::OnlyTheFirst,
    );
    ranges.extend(unreadable_ranges_of(
        &tree_table_writes,
        FailingReadsOfARange::Every,
    ));
    let shared_ranges = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::Zeros);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = with_unreadable_ranges(plain_devices_on(&crash_image), &shared_ranges);
    let mut hook_calls = 0u32;
    let mut count_the_hook_calls = || {
        hook_calls += 1;
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut count_the_hook_calls),
    )
    .expect("退回 P 之后重扫读得出 R 的第 1 条记录，R 的记录齐：不按缺记录拒，可写挂载做成");
    assert_eq!(
        hook_calls, 0,
        "再判用的是重扫出的记录，没有缺记录的发布，「重读一次」那一支没走"
    );
    assert!(
        shared_ranges.reads_of_range(0) >= 2 && shared_ranges.reads_of_range(1) >= 2,
        "R 的第 1 条记录每块盘至少读了两次：读阶段扫环一次、退回之后重扫一次（{}、{}）",
        shared_ranges.reads_of_range(0),
        shared_ranges.reads_of_range(1)
    );
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "F2 退回 P，可写挂载建在 P 上"
    );
}
