//! checker 档模块：crash_point_replay_progress、crash_injection、crash_given_states、crash_amplification
//! 记录核对器第二条判据在掉电那道接缝处断开（`memory_pool::RecordStreamSeam::AfterAPowerLoss`）。留孤儿那段历史
//! （`history::history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`）最后一步开头摆一个崩溃状态
//! （更早的写全落、最后一步一个都没落，崩溃注入 `inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans`
//! 第一截的同一种摆法），只读恢复落到最后一次发布。掉电那次覆盖写落了盘、没有根的单元写接成一条来分时归进掉电之后写行那次发布 (2, 4)，
//! 第 1 步覆盖写合法复用了它们的落点，回收谓词却按 (2, 4) 判那次复用证得出不合法，于是判 (2, 4) 缺了单元（假红）；
//! 在接缝处断开，那几个写不归任何一次发布，不红。阳性对照：断开之后把最后一次发布的一个单元两份都扣下，照样红。
//! 每个崩溃状态作一张给定状态表交崩溃点重放的引擎判（两段：表先写进判定库、读回再判，配了 GPU 走 GPU；用户 2026-10-13 定「能走两段的都走两段」）。
//! 分发布那一半（接缝记在哪、断开之后那几个写归不归发布）在 harness 档
//! （`crates/singlefs-harness/tests/publishes_of_a_history_broken_at_the_power_loss_seam.rs`）。

use singlefs_checker_tier::crash_amplification::{GivenStatesEngine, GivenStatesItem};
use singlefs_checker_tier::crash_given_states::GivenStatesConclusionsSource;
use singlefs_checker_tier::crash_injection::{
    GivenStatesConclusionsOfATable, GivenStatesJudge, GivenStatesOfATable,
    HistoryAfterTheMakeFilesystem,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::history::{
    execute_history_with,
    history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::{
    root_identity_of_write, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool,
    RecordCheck, RecordStreamSeam, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 历史本身不跑每一步的 checker，两块 4 GiB 的盘（与崩溃注入快档同一种跑法）。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 给定状态表交崩溃点重放的引擎判：表与结论进家目录下给定状态表那一类的判定库（判过的复用），配了 GPU 且编进了 `gpu` 特性上卡、否则 CPU
/// （用户 2026-10-13 定「能走两段的都走两段」：表先写进库、读回再判）。
fn given_states_engine() -> GivenStatesEngine {
    GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: "record-checker-at-the-power-loss-seam::history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring".to_string(),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
}

/// 最后一步开头那个崩溃状态：整条写表与它的切段、录制流下标、mkfs 写完时的录制流下标，持久集合（最后一步第一个写之前全落、之后一个都没落），
/// 只读恢复落到的那一版、掉电接缝在写表里的下标、最后一步之前最后一条根槽写的下标。
struct CrashAtTheStartOfTheLastStep {
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    stream_indexes: Vec<usize>,
    operations_written_by_make_filesystem: usize,
    persisted: Vec<bool>,
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
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
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
        run.earliest_stream_index_after_each_power_loss.len(),
        1,
        "这段历史掉了一次电"
    );
    let first_write_after_the_power_loss = stream_indexes
        .iter()
        .position(|stream_index| {
            *stream_index >= run.earliest_stream_index_after_each_power_loss[0]
        })
        .expect("掉电之后那次可写挂载写了东西");
    let last_root_write_before_the_crash = (0..first_write_of_the_last_step)
        .rev()
        .find(|write_index| writes[*write_index].kind == StepKind::RootRecordFua)
        .expect("最后一步之前写过根");
    let mut persisted = vec![false; writes.len()];
    persisted[..first_write_of_the_last_step].fill(true);
    let effective_root =
        recover(&crash_image(&writes, &persisted), JournalPolicy::Consult).effective_root;
    CrashAtTheStartOfTheLastStep {
        writes,
        segments,
        stream_indexes,
        operations_written_by_make_filesystem: run.operations_written_by_make_filesystem,
        persisted,
        effective_root,
        first_write_after_the_power_loss,
        last_root_write_before_the_crash,
    }
}

/// 两块空盘上叠写表里持久了的写（`persisted` 与写表逐条对应）。
fn crash_image<'image>(writes: &'image [RetainedWrite], persisted: &[bool]) -> CrashImage<'image> {
    static EMPTY_POOL: std::sync::OnceLock<MemoryPool> = std::sync::OnceLock::new();
    CrashImage {
        base: EMPTY_POOL.get_or_init(|| {
            MemoryPool::with_devices(
                &[DeviceIdentity(0), DeviceIdentity(1)],
                UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
                    .device_width
                    .device_bytes(),
            )
        }),
        writes,
        persisted: persisted.to_vec(),
    }
}

/// 记录核对器拿到的写表在哪几处断开：掉电那道接缝处断开，或接成一条录制流来分。
#[derive(Clone, Copy, Debug)]
enum RecordStreamAsHandedToTheRecordChecker {
    BrokenAtThePowerLossSeam,
    AsOneRecording,
}

/// 这几个崩溃状态（每个持久集合与整条写表逐条对应）作一张给定状态表交引擎判：基线是 mkfs 之后的池，写表从 mkfs 第一个不是清零的写起
/// （`crash_injection::HistoryAfterTheMakeFilesystem`，与崩溃注入第一截同一种建法）；记录核对器照 `record_stream` 断开。
fn judged_through_the_given_states_engine(
    crash: &CrashAtTheStartOfTheLastStep,
    persisted_sets_over_the_whole_stream: &[Vec<bool>],
    record_stream: RecordStreamAsHandedToTheRecordChecker,
    table_name: &str,
) -> GivenStatesConclusionsOfATable {
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &crash.writes,
        &crash.segments,
        &crash.stream_indexes,
        crash.operations_written_by_make_filesystem,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
    );
    let seams: Vec<RecordStreamSeam> = match record_stream {
        RecordStreamAsHandedToTheRecordChecker::BrokenAtThePowerLossSeam => {
            let first_write_after_the_power_loss_in_the_table = history_after_the_make_filesystem
                .index_in_the_table(crash.first_write_after_the_power_loss);
            vec![RecordStreamSeam::AfterAPowerLoss {
                first_write_after_the_power_loss: first_write_after_the_power_loss_in_the_table,
            }]
        }
        RecordStreamAsHandedToTheRecordChecker::AsOneRecording => Vec::new(),
    };
    let conclusions = given_states_engine()
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: format!("{table_name}/{record_stream:?}"),
            base: history_after_the_make_filesystem.base_pool(),
            writes: history_after_the_make_filesystem.writes(),
            segments: history_after_the_make_filesystem.segments(),
            seams: &seams,
            persisted_sets: persisted_sets_over_the_whole_stream
                .iter()
                .map(|persisted| {
                    history_after_the_make_filesystem.persisted_set_of_the_table(persisted)
                })
                .collect(),
        })
        .unwrap_or_else(|reason| panic!("引擎判得了这张表：{reason}"));
    assert_eq!(
        conclusions.states.len(),
        persisted_sets_over_the_whole_stream.len(),
        "判器逐状态交回结论"
    );
    conclusions
}

/// 掉电接缝处断开与接成一条两种交法各一张表（同一个崩溃状态），交引擎判（两段，配了 GPU 走 GPU）。
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
    let as_one_recording = judged_through_the_given_states_engine(
        &crash,
        std::slice::from_ref(&crash.persisted),
        RecordStreamAsHandedToTheRecordChecker::AsOneRecording,
        "crash-at-the-start-of-the-last-step",
    );
    if as_one_recording.source == GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly {
        // 配置开了 GPU 而这份二进制没编进 gpu 特性：引擎只录不判、交占位结论（开跑时已打本次未跑那一行），判归带特性那一趟
        return;
    }
    assert!(
        as_one_recording.states[0]
            .record_check()
            .claimed_state_missing_unit,
        "前提：接成一条来分时，掉电那次改动没有根的单元写归进写行那次发布，判它缺了单元"
    );
    let broken_at_the_power_loss_seam = judged_through_the_given_states_engine(
        &crash,
        std::slice::from_ref(&crash.persisted),
        RecordStreamAsHandedToTheRecordChecker::BrokenAtThePowerLossSeam,
        "crash-at-the-start-of-the-last-step",
    );
    assert_eq!(
        broken_at_the_power_loss_seam.states[0].record_check(),
        RecordCheck::default(),
        "在掉电接缝处断开：那几个写不归任何一次发布，记录核对器两条判据都不红"
    );
}

/// 阳性对照：同一个崩溃状态上把最后一步之前那次发布的第一个单元两份（两块盘各一份）都扣下，在掉电接缝处断开交引擎判，照样判缺单元。
#[test]
fn a_unit_of_the_newest_publish_missing_from_the_crash_state_is_still_red_once_broken_at_the_power_loss_seam(
) {
    let crash = crash_at_the_start_of_the_last_step();
    let first_unit_of_the_newest_publish = (0..crash.last_root_write_before_the_crash)
        .rev()
        .take_while(|write_index| crash.writes[*write_index].kind != StepKind::RootRecordFua)
        .filter(|write_index| crash.writes[*write_index].kind == StepKind::UnitWrite)
        .min()
        .expect("最后一步之前那次发布（覆盖写）写了单元");
    let unit = &crash.writes[first_unit_of_the_newest_publish];
    let mut persisted_without_the_unit = crash.persisted.clone();
    let mut copies_withheld = 0usize;
    for (write_index, write) in crash.writes[..crash.last_root_write_before_the_crash]
        .iter()
        .enumerate()
        .rev()
        .take_while(|(_, write)| write.kind != StepKind::RootRecordFua)
    {
        if write.kind == StepKind::UnitWrite && write.offset == unit.offset {
            persisted_without_the_unit[write_index] = false;
            copies_withheld += 1;
        }
    }
    assert_eq!(copies_withheld, 2, "前提：那个单元两块盘各一份副本，都扣下");
    assert_eq!(
        recover(
            &crash_image(&crash.writes, &persisted_without_the_unit),
            JournalPolicy::Consult
        )
        .effective_root,
        crash.effective_root,
        "前提：扣下那个单元之后只读恢复仍落到最后一步之前那次发布"
    );
    let conclusions = judged_through_the_given_states_engine(
        &crash,
        &[persisted_without_the_unit],
        RecordStreamAsHandedToTheRecordChecker::BrokenAtThePowerLossSeam,
        "crash-at-the-start-of-the-last-step/first-unit-of-the-newest-publish-withheld",
    );
    if conclusions.source == GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly {
        // 同上一条用例：占位结论不比，判归带 gpu 特性那一趟
        return;
    }
    assert!(
        conclusions.states[0]
            .record_check()
            .claimed_state_missing_unit,
        "断开之后最后一次发布的单元两份都没落：照样判缺单元"
    );
}
