//! 攻方 m3-formal-code-b20-r1-opus 的副本探针（harness 档，只放进攻方自己拷出的副本里跑）：
//! checker 档三条靶向用例（F1 第二截、F2 第三截、发布臂固定历史）摆的盘面，用 harness 档的公开入口逐步复做，
//! 打出每条断言在正确接线与点名的几种接错下各自取到的值。checker 档的胶水（调用点、计数）照 crash_injection.rs 抄写，不调 checker 档。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::model_comparison::refusal_reason_of_publish_error;
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, DiskAfterTheRefusal,
    ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet,
    RefusalReasonOnTheD23List, WitnessedCounterBranchThatHolds, WritableMountRefusalVerdict,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn inode_numbers(output: &singlefs_core::transaction::TransactionOutput) -> Vec<(u64, u32)> {
    output
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| (record.inode, record.nlink))
        .collect()
}

const EXECUTION: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

type Selected = Option<(InstanceGeneration, CheckpointTxg)>;

fn pool_of(writes: &[RetainedWrite]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        EXECUTION.device_width.device_bytes(),
    );
    pool.apply_writes(writes);
    pool
}

fn apply_masked(base: &MemoryPool, window: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in window.iter().zip(persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

fn recording_devices(
    image: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sectors.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect()
}

fn image_after(devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>, size: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.image))
            .collect(),
        device_size_in_bytes: size,
    }
}

/// 只挂一次（崩溃注入第二截与第三截再挂的同一种调法）：交回结局、挂之后的盘、挂载那一段录制流切出的写表与段。
fn mount_alone(image: &MemoryPool) -> (Result<(), MountError>, MemoryPool, Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recording_devices(image, &stream);
    let mounted = mount_writable_with_space_admission(
        &EXECUTION.device_width.parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .map(|_| ());
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &EXECUTION.device_width.fixed_geometry(),
    );
    (mounted, image_after(devices, image.device_size_in_bytes), writes, segments)
}

fn verdict_on(image: &MemoryPool, selected: Selected, faults: ReadFaultsInTheCrashState) -> Option<WritableMountRefusalVerdict> {
    let (mounted, after, _, _) = mount_alone(image);
    let error = mounted.err()?;
    let disk = if after == *image { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed };
    Some(judge_writable_mount_refusal_of_a_crash_state(
        image,
        selected,
        &error,
        disk,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        faults,
    ))
}

/// F1 用例两条新断言照原样判（Debug 文本里找名字）；交回（含 C670 那一条、不含「复算为假」）。
fn f1_assertions(conditions: &str) -> (bool, bool) {
    let witness = format!(
        "{:?}",
        RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
            WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
        )
    )
    .split('(')
    .next()
    .expect("一段")
    .to_string();
    let criterion_false = format!("{:?}", RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage);
    (conditions.contains(&witness), !conditions.contains(&criterion_false))
}

/// 第二截报红那一句（crash_injection.rs 拼的格式）与 F1 用例取「拒得不合法（…）：」那一段的取法。
fn conditions_parsed_the_way_the_f1_case_parses(verdict: &WritableMountRefusalVerdict) -> Option<String> {
    let WritableMountRefusalVerdict::Illegal { reason, conditions_not_met } = verdict else {
        return None;
    };
    let member = "NewerPublishStillUnreadable { .. }（假的成员文本，带全角）：";
    let answer = format!(
        "崩溃后镜像上的可写挂载拒了，理由 {reason:?} 在 D23 已定项 14 那张单子上，拒得不合法（{conditions_not_met:?}）：{member}"
    );
    answer
        .split_once("拒得不合法（")
        .and_then(|(_, after)| after.split_once("）："))
        .map(|(conditions, _)| conditions.to_string())
}

fn first_file_and_one_overwrite() -> (Vec<RetainedWrite>, Vec<Vec<usize>>, usize) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 0 },
            fill_seed: 1,
        })],
    };
    let stream = SharedStream::retaining_contents();
    let mut after_start = None;
    let run = execute_history_with(&history, EXECUTION, &stream, &mut |observation| {
        if observation.position == StepPosition::StartingPoint {
            after_start = Some(stream.operation_count());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed);
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &EXECUTION.device_width.fixed_geometry(),
    );
    let after_start = after_start.expect("起点");
    let first = stream_indexes.iter().position(|index| *index >= after_start).expect("覆盖写写过");
    (writes, segments, first)
}

#[test]
fn b20_opus_k1_f1_conditions_named_with_and_without_the_selected_version() {
    let (writes, _segments, first) = first_file_and_one_overwrite();
    let window = &writes[first..];
    let persisted: Vec<bool> = window.iter().map(|write| write.kind == StepKind::SystemConfigurationSlot).collect();
    let image = apply_masked(&pool_of(&writes[..first]), window, &persisted);
    let effective_root = recover(&image, JournalPolicy::Consult).effective_root;
    println!("F1 effective_root={effective_root:?} rotation_writes={}", persisted.iter().filter(|p| **p).count());
    for (label, selected) in [("正确接线 report.effective_root", effective_root), ("变异 None", None)] {
        let verdict = verdict_on(&image, selected, ReadFaultsInTheCrashState::NoneInjected).expect("第二截那次挂载被拒");
        let parsed = conditions_parsed_the_way_the_f1_case_parses(&verdict);
        let (has_witness, lacks_false) = parsed.as_deref().map_or((false, false), f1_assertions);
        println!("F1 {label}: verdict={verdict:?}\n   parsed={parsed:?} 断言一(含 C670)={has_witness} 断言二(不含复算为假)={lacks_false}");
    }
}

#[test]
fn b20_opus_k1_f2_third_stage_state_and_the_named_miswirings() {
    let (writes, segments, first) = first_file_and_one_overwrite();
    let segment_of_the_overwrite = segments.iter().position(|segment| segment[0] == first).expect("覆盖写从段首起");
    println!("F2 first crash: overwrite segment {segment_of_the_overwrite} writes {:?}", segments[segment_of_the_overwrite].len());
    let first_crash_image = pool_of(&writes[..first]);
    let (mounted, _after, mount_writes, mount_segments) = mount_alone(&first_crash_image);
    assert!(mounted.is_ok(), "第一次崩溃镜像上可写挂载做成");
    let root = mount_writes.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("写行的根槽");
    let rotation: Vec<usize> = (root + 1..mount_writes.len())
        .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
        .collect();
    let segment_of_the_root = mount_segments.iter().position(|segment| segment.contains(&root)).expect("段");
    let segment_start = mount_segments[segment_of_the_root][0];
    let last_rotation = *rotation.last().expect("轮换");
    println!(
        "F2 mount writes={} root={root} rotation={rotation:?} root segment={segment_of_the_root} {:?} next segment {:?} kinds in window {:?}",
        mount_writes.len(),
        mount_segments[segment_of_the_root],
        mount_segments.get(segment_of_the_root + 1),
        mount_writes[segment_start..=last_rotation].iter().map(|write| write.kind).collect::<Vec<_>>()
    );
    // exactly_one_second_crash_point 那一摆：从段首起数到最后一个轮换，扣下根槽。
    let persisted: Vec<bool> = (segment_start..=last_rotation).map(|write| write != root).collect();
    let second_base = apply_masked(&first_crash_image, &mount_writes[..segment_start], &vec![true; segment_start]);
    let second_image = apply_masked(&second_base, &mount_writes[segment_start..=last_rotation], &persisted);
    let effective_root = recover(&second_image, JournalPolicy::Consult).effective_root;
    let first_crash_root = recover(&first_crash_image, JournalPolicy::Consult).effective_root;
    println!("F2 second crash effective_root={effective_root:?} (first crash {first_crash_root:?})");
    let counts = |verdict: Option<WritableMountRefusalVerdict>| -> (u32, u32) {
        match verdict {
            Some(WritableMountRefusalVerdict::Legal(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)) => (1, 0),
            Some(WritableMountRefusalVerdict::Illegal { reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration, conditions_not_met }) => {
                let witness = conditions_not_met.iter().any(|condition| matches!(condition, RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)));
                (1, u32::from(witness))
            }
            _ => (0, 0),
        }
    };
    for (label, image, selected, faults) in [
        ("正确接线", &second_image, effective_root, ReadFaultsInTheCrashState::NoneInjected),
        ("变异 读故障恒交 Injected", &second_image, effective_root, ReadFaultsInTheCrashState::Injected),
        ("变异 再挂的盘交第一次崩溃镜像", &first_crash_image, effective_root, ReadFaultsInTheCrashState::NoneInjected),
        ("变异 再挂的盘交 second_base", &second_base, effective_root, ReadFaultsInTheCrashState::NoneInjected),
        ("未点名 所选那一版交 None", &second_image, None, ReadFaultsInTheCrashState::NoneInjected),
    ] {
        let verdict = verdict_on(image, selected, faults);
        println!("F2 {label}: (判过, 不合法)={:?} verdict={verdict:?}", counts(verdict.clone()));
    }
}

#[test]
fn b20_opus_k2_fixed_history_unlink_then_create_inode() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::Unlink, HistoryOperation::CreateInode],
    };
    let stream = SharedStream::retaining_contents();
    let mut after_unlink = None;
    let run = execute_history_with(&history, EXECUTION, &stream, &mut |observation| {
        if observation.position == StepPosition::Operation(0) {
            after_unlink = Some(stream.operation_count());
        }
    });
    println!("K2 fixed ending={:?} outcomes={:?}", run.ending, run.outcomes);
    let after_unlink = after_unlink.expect("unlink 跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &EXECUTION.device_width.fixed_geometry(),
    );
    let root = (0..writes.len())
        .rev()
        .find(|write| stream_indexes[*write] >= after_unlink && writes[*write].kind == StepKind::RootRecordFua)
        .expect("建 inode 的根槽");
    let segment_of_the_root = segments.iter().position(|segment| segment.contains(&root)).expect("段");
    let start = segments[segment_of_the_root][0];
    println!(
        "K2 fixed root={root} root segment={segment_of_the_root} {:?} next {:?} kinds after root {:?} writes total {}",
        segments[segment_of_the_root],
        segments.get(segment_of_the_root + 1),
        writes[root..].iter().map(|write| write.kind).collect::<Vec<_>>(),
        writes.len()
    );
    // 用例的摆法：从根槽所在段的段首起数到 root + 1，扣下 root + 1（第一个轮换槽），root + 2 起一个都不落。
    let window_len = root + 2 - start;
    let persisted: Vec<bool> = (0..window_len).map(|within| within != root + 1 - start).collect();
    let image = apply_masked(&pool_of(&writes[..start]), &writes[start..start + window_len], &persisted);
    let same_as_prefix = image == pool_of(&writes[..=root]);
    let report = recover(&image, JournalPolicy::Consult);
    println!("K2 fixed state == 写表前缀到根槽为止：{same_as_prefix}; effective_root={:?}", report.effective_root);
    for arm in ["20-1", "一律覆盖写", "建 0 个 inode"] {
        let parameters = EXECUTION.device_width.parameters();
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
                device.image = sectors.clone();
                (*identity, device)
            })
            .collect();
        let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
            .expect("可写挂载做成");
        let PoolVersion::WithFile(current) = &mounted.current else {
            println!("K2 fixed arm {arm}: 现行那一版不是带文件的一版");
            continue;
        };
        println!(
            "K2 fixed arm {arm}: current inode_record={:?} deleted_inodes={:?} trees_and_no_file={}",
            current.inode_record,
            current.deleted_inodes,
            current.is_a_version_with_trees_and_no_file()
        );
        let instance = mounted.output.instance;
        let mut allocator = mounted.allocator;
        let content = vec![1_u8; 100];
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let published = if arm == "20-1" {
            publish_new_inodes(&mut writer, &mut allocator, current, 1, 1_788_000_100, instance)
                .map(|output| println!("K2 fixed arm {arm}: 发成之后 inode 号 {:?}", inode_numbers(&output)))
        } else if arm == "建 0 个 inode" {
            publish_new_inodes(&mut writer, &mut allocator, current, 0, 1_788_000_100, instance)
                .map(|output| println!("K2 fixed arm {arm}: 发成之后 inode 号 {:?}", inode_numbers(&output)))
        } else {
            publish_overwrite(&mut writer, &mut allocator, current, FirstFile { content: &content, write_time_seconds: 1_788_000_100 }, instance)
                .map(|_| ())
        };
        drop(writer);
        let after = MemoryPool {
            devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
            device_size_in_bytes: image.device_size_in_bytes,
        };
        let violations: Vec<String> = singlefs_checker::walk::check_pool_image(&after)
            .into_iter()
            .filter_map(|(invariant, verdict)| match verdict {
                singlefs_checker::image::InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
                _ => None,
            })
            .collect();
        println!(
            "K2 fixed arm {arm}: {} | checker {violations:?}",
            match published {
                Ok(()) => "发成".to_string(),
                Err(error) => format!("拒 {:?} {error:?}", refusal_reason_of_publish_error(&error)),
            }
        );
    }
}

/// F5 那一格的射程：崩溃注入跑历史时，「崩溃恢复抛弃根」那一步往录制流里发没发过东西（发过的才让之后的崩溃态算注入过读故障）。
/// 几组比重、几档盘宽上各跑一批随机历史，数那一步发了几步录制流。
#[test]
fn b20_opus_k1_f5_does_the_abandoning_recovery_step_send_anything() {
    use singlefs_harness::history::{generate_history_with_weights, GenerationWeights, HistoryOperationKind};
    let mut steps = 0u64;
    let mut steps_that_sent = 0u64;
    for weights in [
        GenerationWeights::BROAD,
        GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
        GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
    ] {
        for width in [HistoryDeviceWidth::UnitAreaOf384Slots, HistoryDeviceWidth::UnitAreaOf256Slots] {
            for seed in 0..80 {
                let history = generate_history_with_weights(HistorySeed(seed), 24, &weights);
                let execution = HistoryExecution { device_width: width, ..EXECUTION };
                let stream = SharedStream::retaining_contents();
                let mut marks: Vec<(StepPosition, usize)> = Vec::new();
                let _run = execute_history_with(&history, execution, &stream, &mut |observation| {
                    marks.push((observation.position, stream.operation_count()));
                });
                let mut before = 0usize;
                for (position, after) in marks {
                    if let StepPosition::Operation(index) = position {
                        if history.operations[index].kind() == HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot {
                            steps += 1;
                            if after > before {
                                steps_that_sent += 1;
                            }
                        }
                    }
                    before = after;
                }
            }
        }
    }
    println!("F5 abandoning-recovery steps observed={steps} steps_that_sent_something={steps_that_sent}");
}
