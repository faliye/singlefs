//! 攻方原型（m3-unlink-code-r2，云端攻方）：窄盘上崩在删除中的一版，重开可写挂载，续做为空间推抬 F。
//! 一、N5：挂载录制流上按 crash_injection.rs 抄来的 `floor_raises_located_in_the_mount_writes` 取界，与「第一个带新 F 的系统配置槽写」比。
//! 二、N2 / N5（U9、U12）：同一份镜像上挂载第 n 个写报块设备错，n 扫遍整次挂载；看错的成员、错里的写账与设备一层落下的写、
//!     抄来的 `publishes_before_the_continuation_of_a_mount_that_handed_back_an_error` 划的续做段与真值。
//! 用户决定的那几步放开扫：单元区槽数、unlink 之前覆盖写几次、文件几个单元、崩在第几批之后、挂载用哪一档删除批。
//! 只放进冻结副本的拷贝里跑（harness 档，测试名不带 layer0）。

mod common_admission;
#[path = "attack_r2_support/copied.rs"]
#[allow(dead_code, clippy::all)]
mod copied;

use common_admission::*;
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
    MountError, Mounted, ShadowLedger,
};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::mount_transparency::FloorRaisesInTheMountWrites;
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::cell::Cell;
use std::rc::Rc;

fn batches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

#[derive(Clone, Copy, Debug)]
struct Scenario {
    slots: u64,
    overwrites: usize,
    units: usize,
    batches_before_crash: usize,
    product_batches_in_the_mount: bool,
}

/// 窄盘上造好文件、在这个进程里发意图与 `batches_before_crash` 批（批大小 1），进程退出。造不成的交 None（取样点落在墙外）。
fn crashed_image(s: Scenario) -> Option<(MemoryPool, MakeFs)> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            s.slots,
        );
    for _ in 0..s.overwrites {
        pool.overwrite(OVERWRITE_BYTES).ok()?;
    }
    if s.units > 1 {
        pool.sequential_write((s.units - 1) * data_unit_payload_capacity() + 100)
            .ok()?;
    }
    let PoolUnderTest {
        session,
        devices,
        parameters,
        ..
    } = &mut pool;
    let session = session.as_mut()?;
    let mut current = session.current.version_of_trees()?.clone();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    current = publish_unlink_intent(
        &mut writer,
        &mut session.allocator,
        &current,
        InodeNumber(FIRST_INODE_NUMBER),
        session.instance,
    )
    .ok()?;
    for _ in 0..s.batches_before_crash {
        current = publish_unlink_step(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            batches_of_one().batch_size,
            session.instance,
        )
        .ok()?;
    }
    if current.deleted_inodes.is_empty() {
        return None;
    }
    pool.session = None;
    Some((pool.image(), MakeFs(pool.parameters.clone())))
}

struct MakeFs(singlefs_core::make_filesystem::MakeFilesystemParameters);

/// 数整池的写调用（两块盘共用一个计数），第 `failing_write` 次报块设备错、不落盘；0 表示不注错。
struct FailingNth<Inner> {
    inner: Inner,
    attempts: Rc<Cell<u64>>,
    failing_write: u64,
}

impl<Inner: BlockDevice> BlockDevice for FailingNth<Inner> {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let this = self.attempts.get() + 1;
        self.attempts.set(this);
        if this == self.failing_write {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("注入")));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let this = self.attempts.get() + 1;
        self.attempts.set(this);
        if this == self.failing_write {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("注入")));
        }
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

type Dev = FailingNth<RecordingBlockDevice<SparseBlockDevice>>;

struct MountRun {
    result: Result<Mounted, MountError>,
    landed_writes: Vec<RetainedWrite>,
    attempts: u64,
    image_after: MemoryPool,
}

fn mount_on(image: &MemoryPool, params: &MakeFs, s: Scenario, failing_write: u64) -> MountRun {
    let stream = SharedStream::retaining_contents();
    let attempts = Rc::new(Cell::new(0));
    let mut devices: Vec<(DeviceIdentity, Dev)> = plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FailingNth {
                    inner: RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                    attempts: attempts.clone(),
                    failing_write,
                },
            )
        })
        .collect();
    let switches = if s.product_batches_in_the_mount {
        DeletionSwitches::PRODUCT
    } else {
        batches_of_one()
    };
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &params.0,
        &mut devices,
        if std::env::var("ATTACK_SKIP_ADMISSION").is_ok_and(|v| v == "1") {
            SpaceAdmission::SkippedByTheTestOnlySwitch
        } else {
            SpaceAdmission::JudgedByTheFormula
        },
        ShadowLedger::On,
        switches,
    );
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: params.0.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: params.0.geometry.journal_ring_bytes,
        root_ring_slots_per_region: params.0.geometry.root_ring_slots_per_region,
    };
    let (landed_writes, _segments, _indexes) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
    let image_after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    MountRun {
        result,
        landed_writes,
        attempts: attempts.get(),
        image_after,
    }
}

fn floor_in_system_configuration_write(write: &RetainedWrite) -> Option<u64> {
    let bytes = write.bytes()?;
    if bytes.len() < 489 {
        return None;
    }
    Some(u64::from_le_bytes(bytes[481..489].try_into().ok()?))
}

/// 真值：挂载写表里第一个带的 F 高于取号那一次系统配置写所带 F 的系统配置槽写。
fn first_write_carrying_a_raised_floor(writes: &[RetainedWrite]) -> Option<usize> {
    let floor_at_acquisition = writes
        .iter()
        .find(|write| write.kind == StepKind::SystemConfigurationSlot)
        .and_then(floor_in_system_configuration_write)?;
    writes.iter().position(|write| {
        write.kind == StepKind::SystemConfigurationSlot
            && floor_in_system_configuration_write(write).is_some_and(|f| f > floor_at_acquisition)
    })
}

fn root_writes(writes: &[RetainedWrite]) -> usize {
    writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count()
}

fn member(error: &MountError) -> String {
    let text = format!("{error:?}");
    text.split(|c: char| c == '(' || c == ' ' || c == '{')
        .next()
        .unwrap_or("")
        .to_string()
}

fn env_list(name: &str, default: &[u64]) -> Vec<u64> {
    std::env::var(name)
        .ok()
        .map(|text| text.split(',').map(|item| item.trim().parse().expect("数")).collect())
        .unwrap_or_else(|| default.to_vec())
}

/// 第 `publishes` 个根槽写（与它后面紧跟的系统配置槽写）之后的第一个写的下标：续做从这里起（真值）。
fn first_write_after_root_writes(writes: &[RetainedWrite], publishes: usize) -> Option<usize> {
    let mut seen = 0;
    let mut index = 0;
    while index < writes.len() {
        if writes[index].kind == StepKind::RootRecordFua {
            seen += 1;
            if seen == publishes {
                index += 1;
                while index < writes.len() && writes[index].kind == StepKind::SystemConfigurationSlot {
                    index += 1;
                }
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn acquisition_writes(writes: &[RetainedWrite]) -> usize {
    writes
        .iter()
        .take_while(|write| write.kind == StepKind::SystemConfigurationSlot)
        .count()
}

#[test]
fn attack_r2_mount_continuation_probe() {
    let slots = env_list("ATTACK_SLOTS", &[192, 224, 256, 288, 320]);
    let overwrites = env_list("ATTACK_OVERWRITES", &[0, 1, 2, 3, 4, 6, 8]);
    let units = env_list("ATTACK_UNITS", &[1, 2, 4, 8, 14]);
    let product = env_list("ATTACK_PRODUCT", &[0, 1]);
    let sweep = std::env::var("ATTACK_SWEEP").map(|v| v == "1").unwrap_or(true);
    let mut n5_lines = 0u64;
    let mut n5_hits = 0u64;
    let mut sweep_lines = 0u64;
    for &slots in &slots {
        for &overwrites in &overwrites {
            for &units in &units {
                let units = usize::try_from(units).expect("单元数");
                let mut batch_choices: Vec<usize> = vec![0, 1, units / 2, units.saturating_sub(1)];
                batch_choices.retain(|b| *b < units);
                batch_choices.sort_unstable();
                batch_choices.dedup();
                for &batches_before_crash in &batch_choices {
                    for &product_flag in &product {
                        let s = Scenario {
                            slots,
                            overwrites: usize::try_from(overwrites).expect("次数"),
                            units,
                            batches_before_crash,
                            product_batches_in_the_mount: product_flag == 1,
                        };
                        let Some((image, params)) = crashed_image(s) else {
                            println!("SCENARIO {s:?} build=refused");
                            continue;
                        };
                        let pre_txg = singlefs_core::mounted_read::mount_read_only(&image)
                            .expect("崩溃镜像上只读挂载")
                            .effective_root
                            .checkpoint_txg
                            .0;
                        let run = mount_on(&image, &params, s, 0);
                        let mounted = match &run.result {
                            Ok(mounted) => mounted,
                            Err(error) => {
                                println!("SCENARIO {s:?} mount=Err({})", member(error));
                                continue;
                            }
                        };
                        let output = &mounted.output;
                        let admission_raises = output.space_admission.floor_raises().len();
                        let admission_raise_publishes: usize = output
                            .space_admission
                            .floor_raises()
                            .iter()
                            .map(|raised| raised.publishes.len())
                            .sum();
                        let continuation_raises = output.floor_raises_pushed_by_the_continuation.len();
                        let steps = output.deletion_queue_continuation.len();
                        let first_step_txg = output.deletion_queue_continuation.first().map(|step| step.root.checkpoint_txg);
                        let continuation_raises_before_the_first_step = output
                            .floor_raises_pushed_by_the_continuation
                            .iter()
                            .filter(|raised| first_step_txg.is_none_or(|first| raised.publishes[0].root.checkpoint_txg < first))
                            .count();
                        let outcome = match &output.deletion_queue_continuation_outcome {
                            DeletionQueueContinuationOutcome::WalkedToEmpty => "WalkedToEmpty".to_string(),
                            DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { stop, .. } => {
                                format!("StillShort[{}]", format!("{stop:?}").split(|c: char| c == ' ' || c == '{' || c == '(').next().unwrap_or(""))
                            }
                        };
                        let located = copied::located_counting_the_continuation(output, &run.landed_writes);
                        let truth = first_write_carrying_a_raised_floor(&run.landed_writes);
                        let verdict = match (located, truth) {
                            (FloorRaisesInTheMountWrites::NotPushed, None) => "ok-not-pushed",
                            (FloorRaisesInTheMountWrites::FirstWriteAt(k), Some(t)) if k == t => "ok-located",
                            (FloorRaisesInTheMountWrites::FirstWriteAt(k), Some(t)) if k < t => "HIT-boundary-early",
                            (FloorRaisesInTheMountWrites::FirstWriteAt(_), Some(_)) => "HIT-boundary-late",
                            (FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites, _) => "not-located",
                            (FloorRaisesInTheMountWrites::NotPushed, Some(_)) => "HIT-missed",
                            (FloorRaisesInTheMountWrites::FirstWriteAt(_), None) => "HIT-no-truth",
                        };
                        n5_lines += 1;
                        if verdict.starts_with("HIT") {
                            n5_hits += 1;
                        }
                        let attempts_equal_landed =
                            run.attempts == u64::try_from(run.landed_writes.len()).expect("写数");
                        println!(
                            "N5 {s:?} outcome={outcome} admission_raises={admission_raises} continuation_raises={continuation_raises} steps={steps} cont_raises_before_first_step={continuation_raises_before_the_first_step} located={located:?} truth={truth:?} verdict={verdict} writes={} attempts_equal_landed={attempts_equal_landed}",
                            run.landed_writes.len()
                        );
                        if !sweep || (steps == 0 && continuation_raises == 0) {
                            continue;
                        }
                        let truth_publishes_before_the_continuation =
                            1 + output.warm_up_publishes.len() + admission_raise_publishes;
                        let continuation_first_write = first_write_after_root_writes(
                            &run.landed_writes,
                            truth_publishes_before_the_continuation,
                        )
                        .expect("续做之前那几次的根槽写都在");
                        for failing_write in 1..=run.attempts {
                            let failed_run = mount_on(&image, &params, s, failing_write);
                            let failing_index = usize::try_from(failing_write - 1).expect("下标");
                            let failure_in_the_continuation = failing_index >= continuation_first_write;
                            let truth_partition =
                                failure_in_the_continuation.then_some(truth_publishes_before_the_continuation);
                            let landed_roots = root_writes(&failed_run.landed_writes);
                            let landed = failed_run.landed_writes.len();
                            let acquisition = acquisition_writes(&failed_run.landed_writes);
                            let ci_partition = match &failed_run.result {
                                Err(MountError::Publish(failed)) => copied::partition_of_a_mount_that_handed_back_an_error(
                                    &failed.writes_of_persisted_publishes,
                                    &failed.writes_of_failed_publishes,
                                ),
                                _ => None,
                            };
                            let phases_ci = copied::phases(&failed_run.landed_writes, ci_partition);
                            let phases_truth = copied::phases(&failed_run.landed_writes, truth_partition);
                            let mislabeled_continuation_writes = phases_ci
                                .iter()
                                .zip(&phases_truth)
                                .filter(|(a, b)| a != b && **b == copied::WritableMountPhase::Continuation)
                                .count();
                            let disk_publishes = singlefs_core::mounted_read::mount_read_only(&failed_run.image_after)
                                .map(|m| m.effective_root.checkpoint_txg.0 - pre_txg)
                                .ok();
                            let line = match &failed_run.result {
                                Ok(_) => format!("Ok"),
                                Err(MountError::Publish(failed)) => {
                                    let ledger: u64 = failed.writes_before_the_first_publish.total().write_calls
                                        + failed.writes_of_persisted_publishes.iter().map(|w| w.total().write_calls).sum::<u64>()
                                        + failed.writes_of_failed_publishes.iter().map(|w| w.total().write_calls).sum::<u64>();
                                    let partition = copied::partition_of_a_mount_that_handed_back_an_error(
                                        &failed.writes_of_persisted_publishes,
                                        &failed.writes_of_failed_publishes,
                                    );
                                    format!(
                                        "Publish persisted={} landed_roots={landed_roots} ledger={ledger} landed_minus_acq={} partition={partition:?} truth_partition={truth_partition:?}",
                                        failed.writes_of_persisted_publishes.len(),
                                        landed - acquisition
                                    )
                                }
                                Err(MountError::RaiseFloorSequencePublishFailed(failed)) => {
                                    let ledger: u64 = failed.writes_before_the_first_publish.total().write_calls
                                        + failed.writes_of_persisted_publishes.iter().map(|w| w.total().write_calls).sum::<u64>()
                                        + failed.writes_of_failed_publishes.iter().map(|w| w.total().write_calls).sum::<u64>();
                                    format!(
                                        "RaiseFloorSequencePublishFailed persisted={} landed_roots={landed_roots} ledger={ledger} landed_minus_acq={} partition=None(crash_injection Err(_)) truth_partition={truth_partition:?}",
                                        failed.writes_of_persisted_publishes.len(),
                                        landed - acquisition
                                    )
                                }
                                Err(error) => format!(
                                    "{} landed_roots={landed_roots} landed_minus_acq={} partition=None(crash_injection Err(_)) truth_partition={truth_partition:?}",
                                    member(error),
                                    landed - acquisition
                                ),
                            };
                            sweep_lines += 1;
                            println!("SWEEP {s:?} n={failing_write} in_continuation={failure_in_the_continuation} {line} disk_publishes={disk_publishes:?} mislabeled_continuation_writes={mislabeled_continuation_writes}");
                            drop(failed_run.image_after);
                        }
                    }
                }
            }
        }
    }
    println!("SUMMARY n5_lines={n5_lines} n5_hits={n5_hits} sweep_lines={sweep_lines}");
}
