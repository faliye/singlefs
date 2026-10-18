//! 掉电那道接缝（`memory_pool::RecordStreamSeam::AfterAPowerLoss`）。留孤儿那段历史
//! （`history::history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`：第一个文件之后一次覆盖写掉电在它的第一个
//! journal 记录写之前、进程崩掉再可写挂载成实例 2，之后覆盖写到实例 1 的根全轮出根环）上，执行器记下掉电之后录制流里第一步的下标；
//! 分发布时在那里断开，实例 1 那次改动落了盘、没有根的单元写不归掉电之后可写挂载写行那次发布。
//! 接成一条来分时它们归进写行那次发布 (2, 4)，第 1 步覆盖写复用了它们的落点，记录核对器第二条判据就判 (2, 4) 缺了单元；
//! 记录核对器用这道接缝判的那一半在 checker 档（`crates/singlefs-checker-tier/tests/record_checker_breaks_the_history_at_the_power_loss_seam.rs`）。

use singlefs_core::address::InstanceGeneration;
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with,
    history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    publishes_in, root_identity_of_write, writes_and_segments_with_stream_indexes, PublishWrites,
    RecordStreamContinuity, RecordStreamSeam, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 历史本身不跑每一步的 checker（这段历史每一步都做得成、checker 与模型都不红，由 `history.rs` 里的用例钉），两块 4 GiB 的盘。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 留孤儿那段历史跑完之后的写表，与执行器记下的那几道掉电接缝换成的写表下标。
struct HistoryLeavingOrphans {
    writes: Vec<RetainedWrite>,
    first_write_after_each_power_loss: Vec<usize>,
}

fn record_the_history_leaving_orphans() -> HistoryLeavingOrphans {
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(
        &history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(),
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &stream,
        &mut |_| {},
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "这段历史每一步都做得成"
    );
    let (writes, _segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
            .device_width
            .fixed_geometry(),
    );
    let first_write_after_each_power_loss = run
        .earliest_stream_index_after_each_power_loss
        .iter()
        .map(|earliest_stream_index_after_the_power_loss| {
            stream_indexes
                .iter()
                .position(|stream_index| {
                    *stream_index >= *earliest_stream_index_after_the_power_loss
                })
                .expect("掉电之后那次可写挂载与之后的覆盖写都写了东西")
        })
        .collect();
    HistoryLeavingOrphans {
        writes,
        first_write_after_each_power_loss,
    }
}

/// 掉电之前最后一条根槽写在写表里的下标（实例 1 的最后一次发布）。
fn last_root_write_before(
    writes: &[RetainedWrite],
    first_write_after_the_power_loss: usize,
) -> usize {
    (0..first_write_after_the_power_loss)
        .rev()
        .find(|write_index| writes[*write_index].kind == StepKind::RootRecordFua)
        .expect("从第一个文件起：掉电之前写表里有实例 1 的根")
}

/// 掉电之前最后一条根槽写之后、掉电接缝之前的单元写：那次掉电的改动落了盘、没有根的那几个。
fn unrooted_unit_writes_before_the_power_loss(
    writes: &[RetainedWrite],
    first_write_after_the_power_loss: usize,
) -> Vec<usize> {
    (last_root_write_before(writes, first_write_after_the_power_loss) + 1
        ..first_write_after_the_power_loss)
        .filter(|write_index| writes[*write_index].kind == StepKind::UnitWrite)
        .collect()
}

#[test]
fn the_power_loss_step_records_one_seam_between_the_unrooted_unit_writes_of_instance_one_and_the_mount_of_instance_two(
) {
    let history = record_the_history_leaving_orphans();
    assert_eq!(
        history.first_write_after_each_power_loss.len(),
        1,
        "这段历史掉了一次电，执行器记下一道接缝"
    );
    let seam = history.first_write_after_each_power_loss[0];
    let last_root_before_the_power_loss = last_root_write_before(&history.writes, seam);
    assert_eq!(
        root_identity_of_write(&history.writes[last_root_before_the_power_loss]).0,
        InstanceGeneration(1),
        "接缝之前最后一条根是实例 1 的"
    );
    let writes_between_the_last_root_and_the_seam =
        &history.writes[last_root_before_the_power_loss + 1..seam];
    assert!(
        writes_between_the_last_root_and_the_seam
            .iter()
            .all(|write| write.kind != StepKind::JournalRecord
                && write.kind != StepKind::RootRecordFua),
        "掉电截在第一个 journal 记录写上：接缝之前那次改动一条记录、一条根都没写下"
    );
    assert!(
        !unrooted_unit_writes_before_the_power_loss(&history.writes, seam).is_empty(),
        "接缝之前那次改动的单元写落了盘"
    );
    let first_root_after_the_seam = (seam..history.writes.len())
        .find(|write_index| history.writes[*write_index].kind == StepKind::RootRecordFua)
        .expect("掉电之后的可写挂载写了根");
    assert_eq!(
        root_identity_of_write(&history.writes[first_root_after_the_seam]).0,
        InstanceGeneration(2),
        "接缝之后第一条根是掉电之后可写挂载另起的实例 2 写的"
    );
}

#[test]
fn publishes_broken_at_the_power_loss_seam_put_the_unrooted_unit_writes_of_instance_one_in_no_publish(
) {
    let history = record_the_history_leaving_orphans();
    let seam = history.first_write_after_each_power_loss[0];
    let unrooted_unit_writes = unrooted_unit_writes_before_the_power_loss(&history.writes, seam);
    let every_write_persisted = vec![true; history.writes.len()];
    let first_publish_rooted_after = |publishes: &[PublishWrites]| {
        publishes
            .iter()
            .position(|publish| publish.root >= seam)
            .expect("掉电之后的可写挂载发过布")
    };

    let as_one_recording = publishes_in(
        &history.writes,
        &every_write_persisted,
        RecordStreamContinuity::OneRecording,
    );
    let row_publish_as_one_recording =
        &as_one_recording[first_publish_rooted_after(&as_one_recording)];
    assert!(
        unrooted_unit_writes
            .iter()
            .all(|unit| row_publish_as_one_recording.units.contains(unit)),
        "前提：接成一条来分，实例 1 那几个没有根的单元写归进掉电之后的第一次发布 ({}, {})",
        row_publish_as_one_recording.instance,
        row_publish_as_one_recording.checkpoint_txg
    );

    let broken_at_the_seam = publishes_in(
        &history.writes,
        &every_write_persisted,
        RecordStreamContinuity::BrokenAtSeams(&[RecordStreamSeam::AfterAPowerLoss {
            first_write_after_the_power_loss: seam,
        }]),
    );
    for publish in &broken_at_the_seam {
        assert!(
            unrooted_unit_writes
                .iter()
                .all(|unit| !publish.units.contains(unit)),
            "在掉电接缝处断开：实例 1 那几个没有根的单元写不归任何一次发布，却归进了 ({}, {})",
            publish.instance,
            publish.checkpoint_txg
        );
    }
    let row_publish = &broken_at_the_seam[first_publish_rooted_after(&broken_at_the_seam)];
    assert_eq!(
        row_publish.instance, 2,
        "掉电之后的第一次发布是实例 2 写行那次"
    );
    assert!(
        row_publish.units.iter().all(|unit| *unit >= seam),
        "掉电之后的第一次发布只有接缝之后的单元写：{:?}",
        row_publish.units
    );
    assert_eq!(
        broken_at_the_seam.len(),
        as_one_recording.len(),
        "断开只改单元写归哪次发布，发布还是那几次"
    );
}

#[test]
#[should_panic(expected = "掉电那道接缝")]
fn a_power_loss_seam_whose_earlier_recording_has_a_write_withheld_by_the_persisted_set_is_refused()
{
    let history = record_the_history_leaving_orphans();
    let seam = history.first_write_after_each_power_loss[0];
    let mut persisted = vec![true; history.writes.len()];
    persisted[seam - 1] = false;
    let _publishes = publishes_in(
        &history.writes,
        &persisted,
        RecordStreamContinuity::BrokenAtSeams(&[RecordStreamSeam::AfterAPowerLoss {
            first_write_after_the_power_loss: seam,
        }]),
    );
}
