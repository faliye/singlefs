//! 正推腿副本探针（m3-formal-code-b24-r1 sonnet）：B1-c 的新前提与「建 0 个」变异在盘面参数网格上是不是同真同假。
//! 只在副本的 crates/singlefs-harness/tests/ 下跑（harness 档）。盘面造法抄 checker 档用例
//! `crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs` 的 `crash_image_between_the_deletion_batches`，
//! 但覆盖写次数、单元数、批数与单元区槽数是参数；发布臂抄 `crash_injection.rs` 的 `mount_and_publish_once_on_the_crash_image`
//! （RecordingBlockDevice、建池那份参数的物理块大小、`first_file_is_a_live_record` 与建几个）；前提块抄 B1-c 加的那一段（plain_devices_on）。
#[path = "common_admission/mod.rs"]
mod common_admission;

use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable_with_space_admission, DeletionQueueContinuationOutcome};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, publish_unlink_intent, publish_unlink_step, FirstFile,
    PoolVersion, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::{HistoryDeviceWidth, HistoryExecution, PerStepChecker};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
use singlefs_harness::model_comparison::refusal_reason_of_publish_error;
use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn crash_image(slots: u64, overwrites: usize, units: usize, batches: usize) -> MemoryPool {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            slots,
        );
    for _ in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    pool.sequential_write((units - 1) * data_unit_payload_capacity() + 100).expect("顺序写");
    let parameters = pool.parameters.clone();
    let write_time_seconds = pool.write_time_seconds;
    let session = pool.session.as_mut().expect("会话还开着");
    let mut current = session.current.version_of_trees().expect("有树").clone();
    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
    current = publish_unlink_intent(
        &mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER),
        write_time_seconds, session.instance,
    )
    .expect("意图");
    for _ in 0..batches {
        current = publish_unlink_step(
            &mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER),
            UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
            session.instance,
        )
        .expect("一批");
    }
    drop(writer);
    pool.session = None;
    pool.image()
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Arm {
    MountRefused,
    Published,
    Refused(ObservedRefusalReason, String),
}

/// 发布臂（抄 `mount_and_publish_once_on_the_crash_image`）：`only_some` 为真时判法只看 inode_record 是不是 Some，`count` 是非活记录那一支建几个。
fn arm(image: &MemoryPool, execution: HistoryExecution, count: u64, only_some: bool) -> (Arm, bool, Option<u32>) {
    let parameters = execution.device_width.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect();
    let mounted = mount_writable_with_space_admission(&parameters, &mut devices, execution.space_admission);
    let Ok(mounted) = mounted else {
        return (Arm::MountRefused, false, None);
    };
    let still_short = matches!(
        mounted.output.deletion_queue_continuation_outcome,
        DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
    );
    let mut allocator = mounted.allocator;
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let content: Vec<u8> = (0..97).map(|index: usize| u8::try_from(index % 251).expect("小于 251")).collect();
    let file = FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS };
    let mut nlink = None;
    let published = match &mounted.current {
        PoolVersion::WithFile(current) => {
            nlink = current.inode_record.as_ref().map(|record| record.nlink);
            let live = if only_some {
                current.inode_record.is_some()
            } else {
                current.inode_record.as_ref().is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
            };
            if live {
                publish_overwrite(&mut writer, &mut allocator, current, file, mounted.output.instance).map(|_| ())
            } else {
                publish_new_inodes(&mut writer, &mut allocator, current, count, FIXED_WRITE_TIME_SECONDS, mounted.output.instance)
                    .map(|_| ())
            }
        }
        _ => return (Arm::MountRefused, still_short, None),
    };
    let result = match published {
        Ok(()) => Arm::Published,
        Err(error) => {
            let kind: String = format!("{error:?}").chars().take_while(|c| c.is_alphanumeric()).collect();
            Arm::Refused(refusal_reason_of_publish_error(&error), kind)
        }
    };
    (result, still_short, nlink)
}

/// B1-c 加的前提块（抄原文）：同一份盘面新挂一次、发一次建 0 个的空发布，返回它做成没有（错误的名字）。
fn premise_block(image: &MemoryPool, execution: HistoryExecution) -> Result<(), String> {
    let parameters = execution.device_width.parameters();
    let mut devices = plain_devices_on(image);
    let mut mounted = mount_writable_with_space_admission(&parameters, &mut devices, execution.space_admission)
        .map_err(|e| format!("mount:{e:?}"))?;
    let PoolVersion::WithFile(current) = &mounted.current else {
        return Err("not-with-file".into());
    };
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_new_inodes(
        &mut writer, &mut mounted.allocator, current, 0, FIXED_WRITE_TIME_SECONDS, mounted.output.instance,
    )
    .map(|_| ())
    .map_err(|e| format!("{e:?}").chars().take_while(|c| c.is_alphanumeric()).collect())
}

const WALL: Arm = Arm::Refused(ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall), String::new());

fn is_wall(arm: &Arm) -> bool {
    matches!(arm, Arm::Refused(ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall), _))
}

#[test]
fn the_new_premise_and_the_arm_builds_zero_mutation_agree_over_a_grid_of_disk_parameters() {
    let _ = WALL;
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let mode = std::env::var("B24_GRID").unwrap_or_else(|_| "base".into());
    let mut cells: Vec<(u64, usize, usize, usize)> = Vec::new();
    if mode == "base" {
        for slots in 110..=420u64 {
            cells.push((slots, 3, 14, 1));
        }
    } else {
        for ow in 1..=6usize {
            for units in [10usize, 12, 14, 16, 18] {
                for batches in 1..=2usize {
                    for slots in (100..=330u64).step_by(2) {
                        cells.push((slots, ow, units, batches));
                    }
                }
            }
        }
    }
    let (mut built, mut build_failed, mut old_green, mut new_green, mut some_only, mut both) = (0, 0, 0, 0, 0, 0);
    let mut disagreements = 0;
    let mut premise_wrong_variant = std::collections::BTreeMap::<String, usize>::new();
    for (slots, ow, units, batches) in cells {
        let image = match std::panic::catch_unwind(|| crash_image(slots, ow, units, batches)) {
            Ok(image) => image,
            Err(_) => {
                build_failed += 1;
                continue;
            }
        };
        built += 1;
        let (correct, still_short, nlink) = arm(&image, execution, 1, false);
        let precondition_1 = still_short && nlink == Some(NLINK_OF_AN_INODE_BEING_DELETED);
        let old_test_green = precondition_1 && is_wall(&correct);
        if !old_test_green {
            continue;
        }
        old_green += 1;
        let (zero, _, _) = arm(&image, execution, 0, false);
        let (only_some_arm, _, _) = arm(&image, execution, 1, true);
        let mutation_zero_red = !is_wall(&zero);
        let mutation_some_red = !is_wall(&only_some_arm);
        let premise = premise_block(&image, execution);
        let new_test_green = premise.is_ok();
        if new_test_green {
            new_green += 1;
        }
        // 同真同假：用例（旧）绿的盘面上，新前提成立 ⟺ 「建 0 个」变异红
        if premise.is_ok() != mutation_zero_red {
            disagreements += 1;
            println!("DISAGREE slots={slots} ow={ow} units={units} batches={batches} premise={premise:?} zero_arm={zero:?}");
        }
        if let Err(kind) = &premise {
            *premise_wrong_variant.entry(kind.clone()).or_insert(0) += 1;
        }
        if mutation_zero_red && mutation_some_red {
            both += 1;
        }
        if !mutation_zero_red && mutation_some_red {
            some_only += 1;
        }
        if mode == "base" && (126..=140).contains(&slots) {
            println!(
                "cell slots={slots} ow={ow} units={units} batches={batches} old_test=green new_premise_ok={} mutation_zero_red={mutation_zero_red} (zero-arm: {zero:?}) mutation_only_some_red={mutation_some_red}",
                premise.is_ok()
            );
        }
    }
    println!(
        "b24-b1 mode={mode} cells_built={built} build_failed={build_failed} old_test_green={old_green} new_test_green_(premise_ok)={new_green} \
         green_cells_where_zero_mutation_red_and_only_some_red={both} green_cells_where_only_some_red_zero_not_red={some_only} \
         disagreements_between_premise_and_zero_mutation={disagreements} premise_failure_kinds={premise_wrong_variant:?}"
    );
    assert_eq!(disagreements, 0);
}
