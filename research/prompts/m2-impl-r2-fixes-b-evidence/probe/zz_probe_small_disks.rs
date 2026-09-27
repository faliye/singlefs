//! 草稿探针（不交付）：三档小盘新几何下撞墙的步数与读数。
mod common_admission;

use common_admission::{start_plain, PoolUnderTest, OVERWRITE_BYTES, content_of};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::admission::{admission_reading_before_a_publish, SpaceAdmission};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::{mount_writable_with_space_admission, MountSpaceAdmission, Mounted};
use singlefs_core::transaction::{
    publish_overwrite, publish_sequential_write, publish_version, FirstFile, InstanceTablePlan,
    PoolVersion, PoolWriter, PublishPlan, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, ContentChoice, ContentLength,
    GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, StepOutcome,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;

fn pool_off(width: HistoryDeviceWidth, overwrites: usize) -> Option<PoolUnderTest<SparseBlockDevice>> {
    let mut pool = start_plain(width);
    pool.session.as_mut().unwrap().allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    for _ in 0..overwrites {
        if pool.overwrite(OVERWRITE_BYTES).is_err() { return None; }
    }
    Some(pool)
}

#[test]
fn probe_one_session_wall() {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf256Slots);
    pool.session.as_mut().unwrap().allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    for index in 1..=80 {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(published) => println!("PROBE1 overwrite {index} ok floor_raises={}", published.floor_raises.len()),
            Err(refusal) => { println!("PROBE1 overwrite {index} refused {refusal:?}"); break; }
        }
    }
}

#[test]
fn probe_two_mount_after_overwrites() {
    for overwrites in 10..=30 {
        let Some(mut pool) = pool_off(HistoryDeviceWidth::UnitAreaOf256Slots, overwrites) else { println!("PROBE2 {overwrites} overwrites fail"); break; };
        match pool.crash_and_mount_writable() {
            Ok(output) => {
                let text = format!("{:?}", output.space_admission);
                let cut: String = text.chars().take(600).collect();
                println!("PROBE2 {overwrites} mount ok instance={:?} admission={cut}", output.instance);
            }
            Err(error) => println!("PROBE2 {overwrites} mount err {}", format!("{error:?}").chars().take(400).collect::<String>()),
        }
    }
}

fn publish_empty(pool: &mut PoolUnderTest<SparseBlockDevice>) {
    let session = pool.session.as_mut().unwrap();
    let PoolVersion::WithFile(current) = &session.current else { panic!() };
    let current = current.clone();
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let published = publish_version(
        &mut writer,
        &mut session.allocator,
        PublishPlan {
            txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
            counter: current.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: current.highest_transaction_number_in_this_instance,
            instance: current.root.instance,
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(current.root.instance_table),
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        },
        Some(&current),
    ).expect("空发布");
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

fn pairs(allocator: &PoolAllocator) -> Vec<(u64, u64)> {
    allocator.devices.iter().map(|device_map| {
        let end = device_map.unit_area_start().slot().0 + device_map.unit_area_slots();
        let (mut outside, mut inside) = (0, 0);
        let mut first = device_map.unit_area_start().slot().0;
        while first + 1 < end {
            if device_map.is_free(SlotNumber(first)) && device_map.is_free(SlotNumber(first + 1)) {
                if allocator.cluster_segments().iter().any(|s| first >= s.0 && first < s.0 + CLUSTER_SEGMENT_SLOTS) { inside += 1 } else { outside += 1 }
            }
            first += 2;
        }
        (outside, inside)
    }).collect()
}

#[test]
fn probe_three_cluster_segment_pairs() {
    for (width, empties_list) in [(HistoryDeviceWidth::UnitAreaOf384Slots, vec![60usize, 80, 100, 120, 140, 160, 200]), (HistoryDeviceWidth::UnitAreaOf256Slots, vec![20, 30, 40, 50, 60, 80])] {
        for empties in empties_list {
            let mut pool = start_plain(width);
            pool.crash_and_mount_writable().expect("挂");
            for _ in 0..empties { publish_empty(&mut pool); }
            let current = pool.current_file_version().clone();
            let reading: Vec<i128> = admission_reading_before_a_publish(&pool.session().allocator, Some(&current)).available_on_each_device().into_iter().map(|(_, a)| a.0 / i128::from(SLOT_BYTES)).collect();
            println!("PROBE3 width={width:?} empties={empties} pairs_after_empties={:?} available={reading:?}", pairs(&pool.session().allocator));
            for units in 2..=12 {
                let before = pairs(&pool.session().allocator);
                let current = pool.current_file_version().clone();
                let reading: Vec<i128> = admission_reading_before_a_publish(&pool.session().allocator, Some(&current)).available_on_each_device().into_iter().map(|(_, a)| a.0 / i128::from(SLOT_BYTES)).collect();
                match write_seq(&mut pool, units) {
                    Ok(_) => println!("PROBE3 width={width:?} empties={empties} units={units} ok pairs_before={before:?} available_before={reading:?}"),
                    Err(error) => { println!("PROBE3 width={width:?} empties={empties} units={units} refused pairs_before={before:?} available_before={reading:?} {}", format!("{error:?}").chars().take(400).collect::<String>()); break; }
                }
            }
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
fn probe_four_formula_mounts() {
    for (width, range) in [(HistoryDeviceWidth::UnitAreaOf256Slots, 6..=20usize), (HistoryDeviceWidth::UnitAreaOf240Slots, 1..=14usize)] {
        for overwrites in range {
            let Some(image) = small_image(width, overwrites) else { println!("PROBE4 width={width:?} {overwrites} image fail"); break; };
            let mut devices = devices_on(&image);
            match mount_writable_with_space_admission(&width.parameters(), &mut devices, SpaceAdmission::JudgedByTheFormula) {
                Ok(Mounted { output, mut allocator, current }) => {
                    let summary = match &output.space_admission {
                        MountSpaceAdmission::AdmittedAfterTheFloorRaises { refusal_before_acquisition, floor_raises } => format!("after_raises refusal={:?} raises={} ceilings={:?} publishes={:?}", refusal_before_acquisition.short_devices.iter().map(|s| (s.available.0 / i128::from(SLOT_BYTES), s.demand.0)).collect::<Vec<_>>(), floor_raises.len(), floor_raises.iter().map(|r| r.ceiling).collect::<Vec<_>>(), floor_raises.iter().map(|r| r.publishes.len()).collect::<Vec<_>>()),
                        other => format!("{}", format!("{other:?}").chars().take(300).collect::<String>()),
                    };
                    let current = current.into_file_version().unwrap();
                    let content = content_of(3000, 1);
                    let width_parameters = width.parameters();
                    let mut writer = PoolWriter::new(&width_parameters, devices.as_mut_slice());
                    let outcome = publish_overwrite(&mut writer, &mut allocator, &current, FirstFile { content: &content, write_time_seconds: 1_788_000_600 }, output.instance);
                    let outcome_text = match &outcome { Ok(_) => "overwrite ok".to_string(), Err(error) => format!("overwrite err {}", format!("{error:?}").chars().take(400).collect::<String>()) };
                    println!("PROBE4 width={width:?} overwrites={overwrites} instance={:?} {summary} | {outcome_text}", output.instance);
                }
                Err(error) => println!("PROBE4 width={width:?} overwrites={overwrites} mount err {}", format!("{error:?}").chars().take(300).collect::<String>()),
            }
        }
    }
}

const EMPTY_CONTENT_OVERWRITE: HistoryOperation = HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::Empty, fill_seed: 0 });

#[test]
fn probe_five_mount_after_empty_overwrites() {
    for overwrites in 8..=30 {
        let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations: std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, overwrites).chain(std::iter::once(HistoryOperation::CloseAndMountWritable)).collect() };
        let run = execute_history_with(&history, HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::UnitAreaOf240Slots, space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch }, &SharedStream::new(), &mut |_| {});
        let all_applied = run.outcomes[..overwrites.min(run.outcomes.len())].iter().all(|o| matches!(o, StepOutcome::Applied(_)));
        println!("PROBE5 overwrites={overwrites} ending={:?} all_overwrites_applied={all_applied} last={:?}", run.ending, run.outcomes.last());
    }
}

#[test]
fn probe_six_seeds() {
    let seeds: Vec<u64> = (4_000_000_000..4_000_000_400).collect();
    let threads = 10;
    let chunk = seeds.len().div_ceil(threads);
    std::thread::scope(|scope| {
        for slice in seeds.chunks(chunk) {
            scope.spawn(move || {
                for seed in slice {
                    let history = generate_history_with_weights(HistorySeed(*seed), 60, &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR);
                    let run = execute_history_with(&history, HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::UnitAreaOf240Slots, space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch }, &SharedStream::new(), &mut |_| {});
                    for (step, outcome) in run.outcomes.iter().enumerate() {
                        if let StepOutcome::Refused { member } = outcome {
                            if member.starts_with("MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite(publish 2 of") {
                                println!("PROBE6 seed={seed} step={step} member={member} ending={:?} len={}", run.ending, run.outcomes.len());
                            }
                        }
                    }
                }
            });
        }
    });
}
