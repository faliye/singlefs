//! 草稿探针二（不交付）。
mod common_admission;

use common_admission::{start_plain, PoolUnderTest, OVERWRITE_BYTES, content_of, checker_violations_on};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::admission::{admission_reading_before_a_publish, checkpoint_cost_of_the_version_to_build_on, SpaceAdmission};
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::{mount_writable_with_space_admission, MountSpaceAdmission, Mounted};
use singlefs_core::transaction::{
    publish_overwrite, publish_sequential_write, publish_version, FirstFile, InstanceTablePlan,
    PoolVersion, PoolWriter, PublishPlan, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepOutcome,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;

fn publish_empty(pool: &mut PoolUnderTest<SparseBlockDevice>) {
    let session = pool.session.as_mut().unwrap();
    let PoolVersion::WithFile(current) = &session.current else { panic!() };
    let current = current.clone();
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let published = publish_version(&mut writer, &mut session.allocator, PublishPlan {
        txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1), counter: current.record.counter + 1, transaction: 0,
        highest_transaction_number_before_this_publish: current.highest_transaction_number_in_this_instance,
        instance: current.root.instance, back_chain: back_chain_of(&current.record_bytes), file: None, new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current.root.instance_table), tree_birth_txg: current.tree_birth_txg(),
        tree_identifier_watermark: current.root.tree_identifier_watermark, rollback_floor: current.root.rollback_floor,
    }, Some(&current)).expect("空发布");
    session.current = PoolVersion::WithFile(published);
}

fn write_seq(pool: &mut PoolUnderTest<SparseBlockDevice>, units: usize) -> Result<TransactionOutput, singlefs_core::transaction::PublishError> {
    pool.write_time_seconds += 1;
    let content = content_of(units * data_unit_payload_capacity(), pool.write_time_seconds);
    let session = pool.session.as_mut().unwrap();
    let PoolVersion::WithFile(current) = &session.current else { panic!() };
    let current = current.clone();
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let outcome = publish_sequential_write(&mut writer, &mut session.allocator, &current, FirstFile { content: &content, write_time_seconds: pool.write_time_seconds }, session.instance);
    if let Ok(published) = &outcome { session.current = PoolVersion::WithFile(published.clone()); }
    outcome
}

#[test]
fn probe_eight_admission_384() {
    for empties in [140usize, 200] {
        let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
        pool.crash_and_mount_writable().expect("挂");
        for _ in 0..empties { publish_empty(&mut pool); }
        let mut last = None;
        let mut refused_at = 0;
        for units in 2..=12 {
            let allocator_before = pool.session().allocator.clone();
            match write_seq(&mut pool, units) {
                Ok(published) => last = Some((published, allocator_before)),
                Err(error) => {
                    let (published, allocator_before) = last.clone().unwrap();
                    let bound = published.rewritten.iter().map(|role| role.span_slots()).sum::<u64>() + 2 + checkpoint_cost_of_the_version_to_build_on(Some(&published), &pool.session().allocator).0;
                    let available: Vec<i128> = admission_reading_before_a_publish(&pool.session().allocator, Some(&published)).available_on_each_device().into_iter().map(|(_, a)| a.0 / i128::from(SLOT_BYTES)).collect();
                    let _ = allocator_before;
                    println!("PROBE8 empties={empties} refused_units={units} bound={bound} available={available:?} error={error:?}");
                    refused_at = units;
                    break;
                }
            }
        }
        // 关掉准入那一格
        let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
        pool.crash_and_mount_writable().expect("挂");
        pool.session.as_mut().unwrap().allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
        for _ in 0..empties { publish_empty(&mut pool); }
        for units in 2..refused_at.max(3) {
            match pool.sequential_write(units * data_unit_payload_capacity()) {
                Ok(published) => println!("PROBE8off empties={empties} units={units} ok raises={}", published.floor_raises.len()),
                Err(error) => println!("PROBE8off empties={empties} units={units} err {}", format!("{error:?}").chars().take(300).collect::<String>()),
            }
        }
        let floor_before = pool.rollback_floor();
        match pool.sequential_write(refused_at * data_unit_payload_capacity()) {
            Ok(published) => println!("PROBE8off empties={empties} units={refused_at} ok raises={} pushed={:?} floor {floor_before:?}->{:?} checker={:?}", published.floor_raises.len(), published.refusals_that_pushed_the_floor_raises, pool.rollback_floor(), checker_violations_on(&pool.image())),
            Err(error) => println!("PROBE8off empties={empties} units={refused_at} err {}", format!("{error:?}").chars().take(400).collect::<String>()),
        }
    }
}

fn small_image(width: HistoryDeviceWidth, overwrites: usize) -> Option<MemoryPool> {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 2999 }, fill_seed: 1 });
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations: std::iter::once(HistoryOperation::CloseAndMountWritable).chain(std::iter::repeat_n(overwrite, overwrites)).collect() };
    let mut last = None;
    let run = execute_history_with(&history, HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch }, &SharedStream::new(), &mut |observation| last = Some(observation.image.clone()));
    if run.ending != HistoryEnding::Completed || !run.outcomes.iter().all(|o| matches!(o, StepOutcome::Applied(_))) { return None; }
    last
}

fn devices_on(image: &MemoryPool) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    let stream = SharedStream::new();
    image.devices.iter().map(|(identity, sparse)| { let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512)); device.image = sparse.clone(); (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone())) }).collect()
}

#[test]
fn probe_seven_direct_overwrites() {
    for (width, range) in [(HistoryDeviceWidth::UnitAreaOf240Slots, 1..=7usize), (HistoryDeviceWidth::UnitAreaOf256Slots, 1..=8usize), (HistoryDeviceWidth::UnitAreaOf384Slots, 1..=12usize)] {
        for overwrites in range {
            let Some(image) = small_image(width, overwrites) else { break; };
            let mut devices = devices_on(&image);
            let Ok(Mounted { output, mut allocator, current }) = mount_writable_with_space_admission(&width.parameters(), &mut devices, SpaceAdmission::JudgedByTheFormula) else { println!("PROBE7 {width:?} {overwrites} mount err"); continue; };
            if !matches!(output.space_admission, MountSpaceAdmission::AdmittedBeforeAcquisition) { println!("PROBE7 {width:?} {overwrites} not admitted before"); continue; }
            let mut current = current.into_file_version().unwrap();
            let width_parameters = width.parameters();
            for direct in 1..=4 {
                let content = content_of(3000, direct);
                let mut writer = PoolWriter::new(&width_parameters, devices.as_mut_slice());
                match publish_overwrite(&mut writer, &mut allocator, &current, FirstFile { content: &content, write_time_seconds: 1_788_000_600 + direct }, output.instance) {
                    Ok(published) => { println!("PROBE7 {width:?} overwrites={overwrites} direct={direct} ok"); current = published; }
                    Err(singlefs_core::transaction::PublishError::SpaceAdmissionRefused(refusal)) => { println!("PROBE7 {width:?} overwrites={overwrites} direct={direct} refused {:?}", refusal.short_devices.iter().map(|s| (s.available.0 / i128::from(SLOT_BYTES), s.demand.0 / 16384)).collect::<Vec<_>>()); break; }
                    Err(error) => { println!("PROBE7 {width:?} overwrites={overwrites} direct={direct} other {error:?}"); break; }
                }
            }
        }
    }
}

#[test]
fn probe_nine_mount_stop() {
    for overwrites in 16..=18 {
        let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf256Slots);
        pool.session.as_mut().unwrap().allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
        for _ in 0..overwrites { pool.overwrite(OVERWRITE_BYTES).unwrap(); }
        match pool.crash_and_mount_writable() {
            Ok(output) => match &output.space_admission {
                MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, stop, .. } => println!("PROBE9 {overwrites} instance={:?} still_short raises={} stop={stop:?}", output.instance, floor_raises.len()),
                other => println!("PROBE9 {overwrites} other {}", format!("{other:?}").chars().take(200).collect::<String>()),
            },
            Err(error) => println!("PROBE9 {overwrites} err {}", format!("{error:?}").chars().take(300).collect::<String>()),
        }
    }
}

const EMPTY_CONTENT_OVERWRITE: HistoryOperation = HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::Empty, fill_seed: 0 });

#[test]
fn probe_ten_direct_mount_after_empty_overwrites() {
    let overwrites = 18;
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations: std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, overwrites).chain(std::iter::once(HistoryOperation::CloseAndMountWritable)).collect() };
    let mut images = Vec::new();
    let run = execute_history_with(&history, HistoryExecution { per_step_checker: PerStepChecker::Run, device_width: HistoryDeviceWidth::UnitAreaOf240Slots, space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch }, &SharedStream::new(), &mut |observation| images.push(observation.image.clone()));
    println!("PROBE10 ending={:?} last={:?}", run.ending, run.outcomes.last());
    let image = &images[overwrites];
    let mut devices = devices_on(image);
    let refused = mount_writable_with_space_admission(&HistoryDeviceWidth::UnitAreaOf240Slots.parameters(), &mut devices, SpaceAdmission::SkippedByTheTestOnlySwitch);
    println!("PROBE10 direct {:?}", refused.as_ref().err());
}
