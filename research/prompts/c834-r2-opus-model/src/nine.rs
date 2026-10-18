//! 第一轮正推的九段历史（造法照它的模型 `tests/c834_order_of_devices_decides_the_newer_root_judgement.rs` 的 `histories`，逐段同写法），
//! 每段两种交盘次序、每个候选各挂一次（不走强制开关，候选直接判）。
use crate::common_unwitnessed_publish::{self, fixed_geometry_of_the_384_slot_small_devices, recorded_devices_reading_back_all_zero_on, RangesReadingBackAllZero, RecordedPublish};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{c834_force, c834_set_candidate, mount_writable};
use singlefs_core::pointer::NodePointer;
use singlefs_core::transaction::{acquire_instance, AcquisitionRollback, InstanceAcquisitionFailed, PoolWriter};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn unit_writes_of_r_at<'recorded>(recorded: &'recorded RecordedPublish, pointer: &NodePointer) -> Vec<&'recorded RetainedWrite> {
    let writes = recorded.unit_writes_at(pointer);
    assert_eq!(writes.len(), 2);
    writes
}

fn ranges_that_hide_the_account_of_r(recorded: &RecordedPublish) -> RangesReadingBackAllZero {
    let root_of_r = recorded.root();
    RangesReadingBackAllZero::default()
        .with_the_writes(&crate::every_record_of_r(recorded))
        .with_the_writes(&unit_writes_of_r_at(recorded, &root_of_r.mapping_root))
        .with_the_writes(&unit_writes_of_r_at(recorded, &root_of_r.tree_table))
}

fn writes_of_the_mount_that_takes_a_new_instance(recorded: &RecordedPublish, ranges: &RangesReadingBackAllZero) -> Vec<RetainedWrite> {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, ranges, &stream);
    c834_force(None);
    c834_set_candidate("today");
    mount_writable(&recorded.pool.parameters, &mut devices).expect("取新实例的那次挂载做得成（这一形的前提）");
    writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices()).0
}

struct History {
    name: &'static str,
    image: MemoryPool,
    ranges: RangesReadingBackAllZero,
}

fn image_after_a_rolled_back_acquisition(recorded: &RecordedPublish, base: &MemoryPool, schedule: FaultSchedule) -> MemoryPool {
    let plan = SharedFaultPlan::unarmed(fixed_geometry_of_the_384_slot_small_devices());
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        common_unwitnessed_publish::plain_devices_on(base)
            .into_iter()
            .map(|(identity, inner)| (identity, FaultInjectingBlockDevice::new(identity, inner, plan.clone())))
            .collect();
    plan.arm(schedule);
    let parameters = recorded.pool.parameters.clone();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    match acquire_instance(&mut writer).expect_err("取号必须失败") {
        InstanceAcquisitionFailed::Acquisition(failure) => {
            assert!(matches!(failure.rollback, AcquisitionRollback::RolledBack), "{failure:?}")
        }
        other => panic!("{other:?}"),
    }
    drop(writer);
    let _ = PhysicalBlockSizeInBytes(512);
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: recorded.pool.device_bytes,
    }
}

fn histories(recorded: &RecordedPublish) -> Vec<History> {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let hide_account = ranges_that_hide_the_account_of_r(recorded);
    let mut histories = vec![History { name: "baseline", image: crash_image.clone(), ranges: hide_account.clone() }];
    let m0 = writes_of_the_mount_that_takes_a_new_instance(recorded, &RangesReadingBackAllZero::default());
    let system_configuration_writes: Vec<usize> = m0
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::SystemConfigurationSlot)
        .map(|(index, _)| index)
        .collect();
    let mut partial = crash_image.clone();
    partial.apply_writes(&m0[..=system_configuration_writes[0]]);
    histories.push(History { name: "partial", image: partial, ranges: hide_account.clone() });
    let mut complete = crash_image.clone();
    complete.apply_writes(&m0[..=system_configuration_writes[1]]);
    histories.push(History { name: "complete", image: complete, ranges: hide_account.clone() });
    let m1_ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&recorded.record_writes_of_the_record(1));
    let m1 = writes_of_the_mount_that_takes_a_new_instance(recorded, &m1_ranges);
    let first_record = m1.iter().position(|write| write.kind == StepKind::JournalRecord).expect("M1 写行发布有记录写");
    let row_root = m1.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("M1 写行发布有根槽写");
    let mut units_only = crash_image.clone();
    units_only.apply_writes(&m1[..first_record]);
    histories.push(History {
        name: "units_only",
        image: units_only,
        ranges: RangesReadingBackAllZero::default().with_the_writes(&crate::every_record_of_r(recorded)),
    });
    let mut urj = crash_image.clone();
    urj.apply_writes(&m1[..row_root]);
    histories.push(History { name: "urj", image: urj.clone(), ranges: RangesReadingBackAllZero::default() });
    let row_record_writes: Vec<&RetainedWrite> =
        m1[..row_root].iter().filter(|write| write.kind == StepKind::JournalRecord).collect();
    histories.push(History {
        name: "urj_row_record_also_hidden",
        image: urj,
        ranges: RangesReadingBackAllZero::default().with_the_writes(&row_record_writes),
    });
    let first_root = m0.iter().position(|write| write.kind == StepKind::RootRecordFua).expect("M0 写行发布有根槽写");
    let mut later_records = crash_image.clone();
    later_records.apply_writes(&m0[..first_root]);
    histories.push(History { name: "later_mount_records_landed_r_units_intact", image: later_records, ranges: hide_account.clone() });
    let rolled_back_one = image_after_a_rolled_back_acquisition(
        recorded,
        &crash_image,
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
            placement: FaultPlacement::OffsetBelow(
                singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                    * u64::from(recorded.pool.parameters.geometry.fixed_structure_slot_spacing),
            ),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        },
    );
    histories.push(History { name: "acquisition_failed_on_device_1_rolled_back", image: rolled_back_one, ranges: hide_account.clone() });
    let rolled_back_both = image_after_a_rolled_back_acquisition(
        recorded,
        &crash_image,
        FaultSchedule {
            fault: InjectedFault::BarrierFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(3),
        },
    );
    histories.push(History { name: "acquisition_barrier_failed_rolled_back_on_both", image: rolled_back_both, ranges: hide_account });
    histories
}

pub fn run(recorded: &RecordedPublish) {
    println!("name=c834_nine r=({},{}) p=({},{})", recorded.key.0 .0, recorded.key.1 .0, recorded.previous_key.0 .0, recorded.previous_key.1 .0);
    for history in histories(recorded) {
        let per_device: Vec<String> = crate::instances_of_each_device(&history.image)
            .iter()
            .map(|(device, slots)| format!("{device}:{}", slots.iter().map(u32::to_string).collect::<Vec<_>>().join("/")))
            .collect();
        let red = crate::red_invariants(&history.image);
        let in_image = crate::units_of_r_overwritten_on(recorded, &history.image).len();
        for order in ["forward", "reversed"] {
            for candidate in crate::CANDIDATES {
                c834_force(None);
                c834_set_candidate(candidate);
                let (judge, outcome) = crate::judged_mount(recorded, &history.image, order, &history.ranges);
                let verdicts: Vec<&str> = judge.iter().map(|line| crate::field(line, "verdict")).collect();
                let slots = judge.first().map_or("-", |line| crate::field(line, "slots"));
                println!(
                    "name=c834_nine_row history={} per_device={} image_red={} r_units_overwritten_in_image={in_image} order={order} candidate={candidate} judge={} slots={slots} outcome={outcome}",
                    history.name,
                    per_device.join(","),
                    if red.is_empty() { "-".to_string() } else { red.join("+") },
                    if verdicts.is_empty() { "not_reached".to_string() } else { verdicts.join("+") }
                );
            }
        }
    }
    c834_set_candidate("today");
    let _ = InstanceGeneration(0);
}
