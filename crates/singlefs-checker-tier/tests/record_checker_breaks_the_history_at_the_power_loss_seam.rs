//! checker 档模块：crash
//! 记录核对器第二条判据在掉电那道接缝处断开（`memory_pool::RecordStreamSeam::AfterAPowerLoss`）。留孤儿那段历史
//! （`history::history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`）最后一步开头摆一个崩溃状态
//! （更早的写全落、最后一步一个都没落，崩溃注入 `inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans`
//! 第一截的同一种摆法），只读恢复落到最后一次发布。掉电那次覆盖写落了盘、没有根的单元写接成一条来分时归进掉电之后写行那次发布 (2, 4)，
//! 第 1 步覆盖写合法复用了它们的落点，回收谓词却按 (2, 4) 判那次复用证得出不合法，于是判 (2, 4) 缺了单元（假红）；
//! 在接缝处断开，那几个写不归任何一次发布，不红。阳性对照：断开之后把最后一次发布的一个单元在镜像上盖掉，照样红。
//! 分发布那一半（接缝记在哪、断开之后那几个写归不归发布）在 harness 档
//! （`crates/singlefs-harness/tests/publishes_of_a_history_broken_at_the_power_loss_seam.rs`）。

use singlefs_checker_tier::crash::check_records_against;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::history::{
    execute_history_with,
    history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::{
    root_identity_of_write, writes_and_segments_with_stream_indexes, MemoryPool, RecordCheck,
    RecordStreamContinuity, RecordStreamSeam, RetainedWrite, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 历史本身不跑每一步的 checker，两块 4 GiB 的盘（与崩溃注入快档同一种跑法）。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 最后一步开头那个崩溃状态：写表、持久集合（最后一步第一个写之前全落、之后一个都没落）、崩溃镜像（基线加落了的写）、
/// 只读恢复落到的那一版、掉电接缝在写表里的下标、最后一步之前最后一条根槽写的下标。
struct CrashAtTheStartOfTheLastStep {
    writes: Vec<RetainedWrite>,
    persisted: Vec<bool>,
    first_write_of_the_last_step: usize,
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
    first_write_after_the_power_loss: usize,
    last_root_write_before_the_crash: usize,
}

fn crash_at_the_start_of_the_last_step() -> CrashAtTheStartOfTheLastStep {
    let history = history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring();
    let step_before_the_last = StepPosition::Operation(history.operations.len() - 2);
    let stream = SharedStream::retaining_contents();
    let mut operations_before_the_last_step = None;
    let run = execute_history_with(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &stream,
        &mut |observation| {
            if observation.position == step_before_the_last {
                operations_before_the_last_step = Some(stream.operation_count());
            }
        },
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "这段历史每一步都做得成"
    );
    let operations_before_the_last_step =
        operations_before_the_last_step.expect("倒数第二步之后观察者被调过");
    let (writes, _segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
            .device_width
            .fixed_geometry(),
    );
    let first_write_of_the_last_step = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= operations_before_the_last_step)
        .expect("最后一步（覆盖写）写过东西");
    assert_eq!(
        run.first_stream_index_after_each_power_loss.len(),
        1,
        "这段历史掉了一次电"
    );
    let first_write_after_the_power_loss = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= run.first_stream_index_after_each_power_loss[0])
        .expect("掉电之后那次可写挂载写了东西");
    let last_root_write_before_the_crash = (0..first_write_of_the_last_step)
        .rev()
        .find(|write_index| writes[*write_index].kind == StepKind::RootRecordFua)
        .expect("最后一步之前写过根");
    let mut persisted = vec![false; writes.len()];
    persisted[..first_write_of_the_last_step].fill(true);
    let effective_root = recover(
        &crash_image(&writes, first_write_of_the_last_step, &[]),
        JournalPolicy::Consult,
    )
    .effective_root;
    CrashAtTheStartOfTheLastStep {
        writes,
        persisted,
        first_write_of_the_last_step,
        effective_root,
        first_write_after_the_power_loss,
        last_root_write_before_the_crash,
    }
}

/// 写表前 `first_write_withheld` 个写落在两块空盘上，再叠上 `damage`。
fn crash_image(
    writes: &[RetainedWrite],
    first_write_withheld: usize,
    damage: &[RetainedWrite],
) -> MemoryPool {
    let mut image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
            .device_width
            .device_bytes(),
    );
    image.apply_writes(&writes[..first_write_withheld]);
    image.apply_writes(damage);
    image
}

fn power_loss_seam_of(crash: &CrashAtTheStartOfTheLastStep) -> [RecordStreamSeam; 1] {
    [RecordStreamSeam::AfterAPowerLoss {
        first_write_after_the_power_loss: crash.first_write_after_the_power_loss,
    }]
}

#[test]
fn the_crash_at_the_start_of_the_last_step_of_the_history_leaving_orphans_misses_no_unit_once_broken_at_the_power_loss_seam(
) {
    let crash = crash_at_the_start_of_the_last_step();
    assert_eq!(
        crash.effective_root,
        Some(root_identity_of_write(
            &crash.writes[crash.last_root_write_before_the_crash]
        )),
        "前提：只读恢复落到最后一步之前那次发布"
    );
    let image = crash_image(&crash.writes, crash.first_write_of_the_last_step, &[]);
    let as_one_recording = check_records_against(
        &image,
        &image,
        &crash.writes,
        &crash.persisted,
        RecordStreamContinuity::OneRecording,
        crash.effective_root,
    );
    assert!(
        as_one_recording.claimed_state_missing_unit,
        "前提：接成一条来分时，掉电那次改动没有根的单元写归进写行那次发布，判它缺了单元"
    );
    assert_eq!(
        check_records_against(
            &image,
            &image,
            &crash.writes,
            &crash.persisted,
            RecordStreamContinuity::BrokenAtSeams(&power_loss_seam_of(&crash)),
            crash.effective_root,
        ),
        RecordCheck::default(),
        "在掉电接缝处断开：那几个写不归任何一次发布，记录核对器两条判据都不红"
    );
}

#[test]
fn a_unit_of_the_newest_publish_damaged_on_the_crash_image_is_still_missing_once_broken_at_the_power_loss_seam(
) {
    let crash = crash_at_the_start_of_the_last_step();
    let first_unit_of_the_newest_publish = (0..crash.last_root_write_before_the_crash)
        .rev()
        .take_while(|write_index| crash.writes[*write_index].kind != StepKind::RootRecordFua)
        .filter(|write_index| crash.writes[*write_index].kind == StepKind::UnitWrite)
        .min()
        .expect("最后一步之前那次发布（覆盖写）写了单元");
    let unit = &crash.writes[first_unit_of_the_newest_publish];
    let unit_bytes = usize::try_from(unit.length_in_bytes()).expect("写长装得进 usize");
    let damage_on_both_devices: Vec<RetainedWrite> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: unit.offset,
            contents: WrittenContents::Bytes(vec![0xA5; unit_bytes]),
        })
        .collect();
    let damaged_image = crash_image(
        &crash.writes,
        crash.first_write_of_the_last_step,
        &damage_on_both_devices,
    );
    assert!(
        check_records_against(
            &damaged_image,
            &damaged_image,
            &crash.writes,
            &crash.persisted,
            RecordStreamContinuity::BrokenAtSeams(&power_loss_seam_of(&crash)),
            crash.effective_root,
        )
        .claimed_state_missing_unit,
        "断开之后最后一次发布的单元两份都盖掉了：照样判缺单元"
    );
}
