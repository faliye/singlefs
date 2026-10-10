//! 正推腿探针（第 23 批代码轮第一轮 K1）：在 harness 档复现 K3-a 用例的态，并在判官一层（`judge_writable_mount_refusal_of_a_crash_state`）
//! 对「所选那一版」两种接线各算一次，看 C670 那一条（`WitnessCriterionTrueOnACrashImageWithoutReadFaults`）在不在。
//! 不跑 checker 档的注入函数：调用点接线与 tally 的记数是读码推的。态的摆法逐句抄 `crash_injection.rs` 的 K3-a 用例。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
    ReadOnlyReadBackOfTheCrashImage,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn mount_recording(
    image: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    admission: SpaceAdmission,
) -> (Result<(), MountError>, SharedStream, MemoryPool) {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let result = mount_writable_with_space_admission(parameters, &mut devices, admission).map(|_| ());
    let pool_after = MemoryPool {
        devices: devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.image))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    (result, stream, pool_after)
}

fn run_case(label: &str, width: HistoryDeviceWidth, length: ContentLength, overwrites: usize) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (0..overwrites)
            .map(|_| {
                HistoryOperation::PublishOverwrite(ContentChoice {
                    length,
                    fill_seed: 1,
                })
            })
            .collect(),
    };
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let stream = SharedStream::retaining_contents();
    let mut after_start = None;
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        if observation.position == StepPosition::StartingPoint {
            after_start = Some(stream.operation_count());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "{label}: 历史跑完");
    let after_start = after_start.expect("起点登记过");
    let geometry = width.fixed_geometry();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
    let first_ow = stream_indexes
        .iter()
        .position(|index| *index >= after_start)
        .expect("覆盖写写过东西");
    let root_ow = (first_ow..writes.len())
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .unwrap();
    let root_ff = (0..first_ow)
        .rev()
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .unwrap();
    let root_before = (0..root_ff)
        .rev()
        .find(|w| writes[*w].kind == StepKind::RootRecordFua)
        .unwrap();
    let ff_records: Vec<usize> = (root_before + 1..root_ff)
        .filter(|w| writes[*w].kind == StepKind::JournalRecord)
        .collect();
    let ff_rotation: Vec<usize> = (root_ff + 1..first_ow)
        .filter(|w| writes[*w].kind == StepKind::SystemConfigurationSlot)
        .collect();
    println!(
        "[{label}] writes={} segments={} first_ow={first_ow} root_ow={root_ow} root_ff={root_ff} ff_records={ff_records:?} ff_rotation={ff_rotation:?}",
        writes.len(),
        segments.len()
    );
    if (ff_records.len(), ff_rotation.len()) != (2, 2) {
        println!("[{label}] 前提 (2,2) 不成立，用例会在这里 assert 失败：{}, {}", ff_records.len(), ff_rotation.len());
        return;
    }
    let seg_of = |segs: &Vec<Vec<usize>>, w: usize| segs.iter().position(|s| s.contains(&w)).unwrap();
    let first_segment = seg_of(&segments, ff_records[0]);
    let first_write_of_segment = segments[first_segment][0];
    println!(
        "[{label}] 第一次崩溃那一段 index={first_segment} 段长={} 段首写={first_write_of_segment}",
        segments[first_segment].len()
    );
    let withheld_first: Vec<usize> = ff_records
        .iter()
        .chain(std::iter::once(&ff_rotation[1]))
        .map(|w| w - first_write_of_segment)
        .collect();
    let count_first = root_ow + 1 - first_write_of_segment;
    let mut first_crash = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        width.device_bytes(),
    );
    first_crash.apply_writes(&writes[..first_write_of_segment]);
    let persisted: Vec<RetainedWrite> = (0..count_first)
        .filter(|k| !withheld_first.contains(k))
        .map(|k| writes[first_write_of_segment + k].clone())
        .collect();
    first_crash.apply_writes(&persisted);
    let parameters = width.parameters();
    let (first_mount, mount_stream, _) =
        mount_recording(&first_crash, &parameters, execution.space_admission);
    println!("[{label}] 第一次崩溃镜像上可写挂载：{:?}", first_mount.as_ref().map(|_| "Ok"));
    if first_mount.is_err() {
        return;
    }
    let (mount_writes, mount_segments, _) =
        writes_and_segments_with_stream_indexes(&mount_stream.retained_operations(), &geometry);
    let start = mount_writes
        .iter()
        .position(|w| w.kind != StepKind::SystemConfigurationSlot)
        .unwrap();
    let root_row = (start..mount_writes.len())
        .find(|w| mount_writes[*w].kind == StepKind::RootRecordFua)
        .unwrap();
    let row_records: Vec<usize> = (start..root_row)
        .filter(|w| mount_writes[*w].kind == StepKind::JournalRecord)
        .collect();
    let rotation: Vec<usize> = (root_row + 1..mount_writes.len())
        .take_while(|w| mount_writes[*w].kind == StepKind::SystemConfigurationSlot)
        .collect();
    println!(
        "[{label}] 挂载写表 {} 写 {} 段；写行记录={row_records:?} 根={root_row} 轮换={rotation:?}",
        mount_writes.len(),
        mount_segments.len()
    );
    if (row_records.len(), rotation.len()) != (2, 2) {
        println!("[{label}] 前提 (2,2) 不成立（写行）");
        return;
    }
    let second_segment = seg_of(&mount_segments, row_records[0]);
    let second_first = mount_segments[second_segment][0];
    let withheld_second: Vec<usize> = row_records
        .iter()
        .chain(std::iter::once(&rotation[0]))
        .map(|w| w - second_first)
        .collect();
    println!(
        "[{label}] 第二截段 index={second_segment} 段首写={second_first} withheld={withheld_second:?} count={}",
        rotation[0] + 1 - second_first
    );
    let mut second_crash = first_crash.clone();
    second_crash.apply_writes(&mount_writes[..second_first]);
    let persisted_second: Vec<RetainedWrite> = (second_first..=rotation[0])
        .filter(|w| !withheld_second.contains(&(w - second_first)))
        .map(|w| mount_writes[w].clone())
        .collect();
    second_crash.apply_writes(&persisted_second);
    let first_root = recover(&first_crash, JournalPolicy::Consult).effective_root;
    let second_root = recover(&second_crash, JournalPolicy::Consult).effective_root;
    println!("[{label}] 恢复落点：第一次崩溃 {first_root:?}，二次崩溃 {second_root:?}，两者不同={}", first_root != second_root);
    let (remount, _, pool_after) = mount_recording(&second_crash, &parameters, execution.space_admission);
    let Err(error) = remount else {
        println!("[{label}] 二次崩溃镜像上再挂做成了，没有拒");
        return;
    };
    println!("[{label}] 再挂拒：{error:?}");
    let disk = if pool_after == second_crash {
        DiskAfterTheRefusal::ByteForByteUnchanged
    } else {
        DiskAfterTheRefusal::Changed
    };
    for (wiring, selected) in [("接对(二次崩溃镜像落点)", second_root), ("接错(第一次崩溃落点)", first_root)] {
        let verdict = judge_writable_mount_refusal_of_a_crash_state(
            &second_crash,
            selected,
            &error,
            disk,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        );
        println!("[{label}] {wiring} selected={selected:?} => {verdict:?}");
    }
}

#[test]
fn b23_k3a_state_and_the_judge_on_both_wirings() {
    let one = ContentLength::InsideOneDataUnit { selector: 0 };
    run_case("384槽 1次覆盖写 (用例原样)", HistoryDeviceWidth::UnitAreaOf384Slots, one, 1);
    run_case("384槽 2次覆盖写", HistoryDeviceWidth::UnitAreaOf384Slots, one, 2);
    run_case("384槽 覆盖空内容", HistoryDeviceWidth::UnitAreaOf384Slots, ContentLength::Empty, 1);
    run_case("384槽 覆盖满载荷", HistoryDeviceWidth::UnitAreaOf384Slots, ContentLength::ExactlyDataUnitPayloadCapacity, 1);
    run_case("256槽 1次覆盖写", HistoryDeviceWidth::UnitAreaOf256Slots, one, 1);
    run_case("240槽 1次覆盖写", HistoryDeviceWidth::UnitAreaOf240Slots, one, 1);
    run_case("4GiB 1次覆盖写", HistoryDeviceWidth::FourGibibytes, one, 1);
}
