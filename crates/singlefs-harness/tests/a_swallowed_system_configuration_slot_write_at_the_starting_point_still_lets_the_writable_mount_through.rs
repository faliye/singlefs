//! 起点段设备吞掉一次系统配置槽写、再可写挂载（里程碑三项五·6，C600（吞写之后可写挂载被拒而模型说该成））：mkfs 写盘 0 的系统配置槽 0
//! 那一次被说谎的设备吞掉（报成功、一个字节没落），盘 0 只剩槽 1 自证得过，盘 1 两槽都自证得过；之后可写挂载一次。
//! 见证读按 R2 判（用户 2026-10-03 定「改取 R2」，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_witness_is_undecidable`）：
//! 判不出 ⟺ 有槽设备报读错、且没有一块盘两槽都自证过。吞掉的写留下的是读得出、自证不过的一槽，设备没报错，判得出；c_见证 是 0，判据为假，
//! 挂载做成（取号 1、写行、暖机），模型也说该成。取甲（一槽读不出或自证不过就判不出）时这一形判不出、按真处置，重读一次仍判不出就
//! 以 `MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)` 拒，
//! 模型说该成——那正是 2026-09-29 大档撞出的那一条（种子 7463871032432355222；历史生成器后来变过，今天同一个种子重放的已不是这一段，
//! `records/2026-10-13-代码欠账批.md`「五·6 实测」那一条），所以这里不钉种子，显式造这一段。
//!
//! 判别力：R2 那一判改回甲（`crates/mutations.tsv` 钉着那一行）时挂载被拒、照报新发现，这里红；吞写那一步拿掉或挪到别的段时，
//! 注入点不再是起点段的系统配置槽、盘 0 槽 0 也不再是自证不过的那一形，前提那几条断言先红。

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::recovery::{
    recover, system_configuration_slots_of_one_device_read_once, JournalPolicy, RecoveryOutcome,
};
use singlefs_harness::fault_injection::{
    inject_one_fault_into_the_segment_handing_out_the_images, FaultedSegment, ImagesAfterAFault,
    InjectedFault, RecordedStreamOfAnImage,
};
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, StepPosition,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedOperationKind, SharedStream};

/// 盘宽与执行方式：两块 4 GiB 的盘，harness 不跑池级 checker，每一步的镜像与重开的盘交出去（这里只拿来重建镜像）。
const EXECUTION: HistoryExecution =
    HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;

/// 只做过 mkfs 的池，可写挂载一次。
fn mount_once_after_make_filesystem() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![HistoryOperation::CloseAndMountWritable],
    }
}

/// 不注入跑一遍，数起点段发出的写里第一次落在系统配置槽上的是这一段的第几次写（1 起，写零也算一次写）：
/// `inject_one_fault_into_the_segment` 按「这一段发出的第几次写」摆注入点。
fn ordinal_of_the_first_system_configuration_slot_write_at_the_starting_point(
    history: &GeneratedHistory,
) -> u64 {
    let stream = SharedStream::new();
    let mut stream_length_after_the_starting_point: Option<usize> = None;
    let run = execute_history_with(history, EXECUTION, &stream, &mut |observation| {
        if observation.position == StepPosition::StartingPoint {
            stream_length_after_the_starting_point = Some(stream.operation_count());
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "不注入时这段历史跑完");
    let starting_point_end =
        stream_length_after_the_starting_point.expect("观察者在起点之后调过一次");
    let geometry = EXECUTION.device_width.fixed_geometry();
    let index = stream.operations()[..starting_point_end]
        .iter()
        .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
        .position(|operation| geometry.classify(operation) == StepKind::SystemConfigurationSlot)
        .expect("mkfs 每块盘写两槽系统配置");
    u64::try_from(index + 1).expect("起点段至多几十次写")
}

fn image_of(stream: &RecordedStreamOfAnImage, operations_recorded: usize) -> MemoryPool {
    let mut image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        EXECUTION.device_width.device_bytes(),
    );
    image.apply(&stream.operations[..operations_recorded]);
    image
}

#[test]
fn a_writable_mount_goes_through_after_the_device_swallowed_a_system_configuration_slot_write_of_make_filesystem(
) {
    let history = mount_once_after_make_filesystem();
    let ordinal =
        ordinal_of_the_first_system_configuration_slot_write_at_the_starting_point(&history);
    let mut handed_out: Vec<ImagesAfterAFault> = Vec::new();
    let injection = inject_one_fault_into_the_segment_handing_out_the_images(
        &history,
        EXECUTION,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::TheStartingPoint,
        ordinal,
        &mut |images| handed_out.push(images),
    );
    let rendered = injection.tally.render();
    // 前提：注入的正是起点段的系统配置槽写，被吞之后盘 0 槽 0 读得出、自证不过，槽 1 与盘 1 两槽都自证得过（没有槽报读错）。
    assert_eq!(
        injection
            .tally
            .faults_by_injection_point
            .get("TheStartingPoint/system_configuration_slot"),
        Some(&1),
        "这一次注入要落在起点段的系统配置槽写上：\n{rendered}"
    );
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    let after_the_starting_point = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .find(|step_image| step_image.position == StepPosition::StartingPoint)
        .expect("起点之后那一张镜像交出去了");
    let image_before_the_mount = image_of(
        &images.image_after_the_fault,
        after_the_starting_point.operations_recorded,
    );
    let parameters = EXECUTION.device_width.parameters();
    let slot_spacing_in_bytes = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let slots_of_each_device: Vec<(usize, u64)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| {
            let slots = system_configuration_slots_of_one_device_read_once(
                &image_before_the_mount,
                device,
                slot_spacing_in_bytes,
                &parameters.filesystem_identifier,
            );
            (
                slots.self_verified.len(),
                slots.slots_reporting_a_read_error,
            )
        })
        .collect();
    assert_eq!(
        slots_of_each_device,
        vec![(1, 0), (2, 0)],
        "挂载之前：盘 0 只剩一槽自证得过、盘 1 两槽都自证得过，没有槽报读错（R2 判得出、甲判不出的那一形）"
    );
    // R2 下挂载做成、模型也说该成：一条新发现都没有，历史跑完；重开走到这次挂载取号 1 之后写的根。
    assert!(
        injection.new_findings.is_empty() && injection.known_red_hits.is_empty(),
        "吞掉一次系统配置槽写之后可写挂载该成（R2），不许有新发现或「已知红」：{:?}\n{rendered}",
        injection.new_findings
    );
    assert_eq!(
        injection.tally.faults_by_outcome.get("被容错、历史跑完"),
        Some(&1),
        "注入被容错、历史跑完（挂载那一步做成）：\n{rendered}"
    );
    let image_after_the_mount = image_of(
        &images.image_after_the_fault,
        images.image_after_the_fault.operations.len(),
    );
    let recovery = recover(&image_after_the_mount, JournalPolicy::Consult);
    assert!(
        matches!(
            &recovery.outcome,
            RecoveryOutcome::NoFile { root } if root.0 == InstanceGeneration(1)
        ),
        "重开走到这次挂载（实例 1）写的根：{:?}",
        recovery.outcome
    );
}
