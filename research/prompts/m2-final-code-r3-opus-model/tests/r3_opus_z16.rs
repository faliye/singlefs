//! m2-final-code-r3 云端攻方 Z16：取号之前在分配器拷贝上预演的那一串，与真发起来的那一串。只在冻结副本的拷贝上跑。
//! 副本专用补丁只加两处环境变量开关（不设时与冻结副本同行为）：`SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS` 改 240 槽那一档的单元区宽，
//! `SINGLEFS_R3_OPUS_REPORT` 让 `establish_instance` 把预演与真发逐次比一遍打到 stderr（含被隔离那一格；不改行为）。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::address::DeviceIdentity;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultOccurrence, FaultPlacement, FaultSchedule,
    InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::{
    execute_history_with_faults, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, RollbackTargetChoice, StepPosition,
};
use singlefs_harness::SharedStream;

const EMPTY_CONTENT_OVERWRITE: HistoryOperation = HistoryOperation::PublishOverwrite(ContentChoice {
    length: ContentLength::Empty,
    fill_seed: 0,
});

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn width() -> HistoryDeviceWidth {
    match std::env::var("R3_OPUS_WIDTH").as_deref() {
        Ok("384") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok("256") => HistoryDeviceWidth::UnitAreaOf256Slots,
        Ok("4g") => HistoryDeviceWidth::FourGibibytes,
        // 其余一律走 240 那一档（副本上它的宽由 SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS 定）。
        _ => HistoryDeviceWidth::UnitAreaOf240Slots,
    }
}

/// 第一个文件 → 可写挂载 → 覆盖写 n 次 → 回退到环里第 `rollback_index` 新的根 → 覆盖写一次。
fn history(overwrites: usize, rollback_index: u64) -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, overwrites))
            .chain([
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                    index_from_newest: rollback_index,
                }),
                EMPTY_CONTENT_OVERWRITE,
            ])
            .collect(),
    }
}

fn ending_name(ending: &HistoryEnding) -> String {
    match ending {
        HistoryEnding::Completed => "Completed".to_string(),
        HistoryEnding::KnownRed { form, observation } => {
            format!("KnownRed#{form}@{:?}", observation.position)
        }
        HistoryEnding::NewFinding { signature, observation } => format!(
            "NewFinding[{}]@{:?}",
            format!("{signature:?}").chars().take(400).collect::<String>(),
            observation.position
        ),
    }
}

/// 跑一段：`fault` 为 Some((盘, 槽)) 时，在最后一次覆盖写之后（回退之前）把那块盘上那个槽起的读一律报错（之后每一次都报）。
fn run_one(overwrites: usize, rollback_index: u64, fault: Option<(u32, u64)>) -> String {
    let w = width();
    let execution = HistoryExecution {
        per_step_checker: if std::env::var("R3_OPUS_CHECKER").as_deref() == Ok("0") {
            PerStepChecker::Skipped
        } else {
            PerStepChecker::Run
        },
        device_width: w,
    };
    let plan = SharedFaultPlan::unarmed(w.fixed_geometry());
    let arm_after = StepPosition::Operation(overwrites);
    let plan_for_observer = plan.clone();
    let history = history(overwrites, rollback_index);
    let run = execute_history_with_faults(
        &history,
        execution,
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if let (Some((device, slot)), true) = (fault, observation.position == arm_after) {
                plan_for_observer.arm(FaultSchedule {
                    fault: InjectedFault::ReadFails,
                    device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(device)),
                    placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(slot * 16384)),
                    counting: FaultCounting::AcrossThePool,
                    occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(1),
                });
            }
        },
    );
    let rollback_step = overwrites + 1;
    format!(
        "fault={fault:?} fired={} ending={} rollback={:?} after={:?}",
        plan.fired_count(),
        ending_name(&run.ending),
        run.outcomes.get(rollback_step),
        run.outcomes.get(rollback_step + 1),
    )
}

/// 对照：不注入。
#[test]
fn z16_control() {
    let overwrites = env_u64("R3_OPUS_OVERWRITES", 24) as usize;
    let rollback_index = env_u64("R3_OPUS_ROLLBACK_INDEX", overwrites as u64 - 1);
    eprintln!(
        "R3OPUS-RUN width={:?} unit_area={:?} overwrites={overwrites} rollback_index={rollback_index} {}",
        width(),
        std::env::var("SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS").ok(),
        run_one(overwrites, rollback_index, None)
    );
}

/// 扫：盘 R3_OPUS_DEVICE（默认 1）上单元区里 [R3_OPUS_SLOT_FROM, R3_OPUS_SLOT_TO) 每一个槽各跑一段。
#[test]
fn z16_sweep_read_fault_on_one_slot() {
    let overwrites = env_u64("R3_OPUS_OVERWRITES", 24) as usize;
    let rollback_index = env_u64("R3_OPUS_ROLLBACK_INDEX", overwrites as u64 - 1);
    let device = env_u64("R3_OPUS_DEVICE", 1) as u32;
    let from = env_u64("R3_OPUS_SLOT_FROM", 50176);
    let to = env_u64("R3_OPUS_SLOT_TO", 50176 + 64);
    for slot in from..to {
        eprintln!(
            "R3OPUS-RUN width={:?} unit_area={:?} overwrites={overwrites} rollback_index={rollback_index} {}",
            width(),
            std::env::var("SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS").ok(),
            run_one(overwrites, rollback_index, Some((device, slot)))
        );
    }
}

/// 冻结着一次失败之后挂载：第一个文件 → 可写挂载 → 覆盖写 n0 次 → 第 n0 + 1 次覆盖写落盘时第 k 次写报错（这次发布冻结在那个进程的分配器上）
/// → 关掉、可写挂载（冻结的那一次只住内存，重开走恢复）→ 覆盖写一次 → 冷启动。扫 k 从 1 到 R3_OPUS_K_TO。
#[test]
fn z16_mount_after_a_frozen_publish() {
    let n0 = env_u64("R3_OPUS_OVERWRITES", 3) as usize;
    let k_to = env_u64("R3_OPUS_K_TO", 40);
    let fault_kind = std::env::var("R3_OPUS_FAULT").unwrap_or_else(|_| "write".to_string());
    for k in 1..=k_to {
        let w = width();
        let execution = HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width: w,
        };
        let plan = SharedFaultPlan::unarmed(w.fixed_geometry());
        let plan_for_observer = plan.clone();
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
                .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, n0 + 1))
                .chain([
                    HistoryOperation::CloseAndMountWritable,
                    EMPTY_CONTENT_OVERWRITE,
                    HistoryOperation::ColdStartRecover,
                ])
                .collect(),
        };
        let arm_after = StepPosition::Operation(n0);
        let disarm_after = StepPosition::Operation(n0 + 1);
        let run = execute_history_with_faults(
            &history,
            execution,
            &SharedStream::new(),
            &plan,
            &mut |observation| {
                if observation.position == arm_after {
                    let fault = match fault_kind.as_str() {
                        "barrier" => InjectedFault::BarrierFails,
                        _ => InjectedFault::WriteFails,
                    };
                    plan_for_observer.arm(FaultSchedule::the_nth_call_across_the_pool(fault, k));
                }
                if observation.position == disarm_after {
                    plan_for_observer.disarm();
                }
            },
        );
        eprintln!(
            "R3OPUS-FROZEN width={w:?} n0={n0} fault={fault_kind} k={k} fired={} ending={} failing_step={:?} mount={:?} after={:?}",
            plan.fired_count(),
            ending_name(&run.ending),
            run.outcomes.get(n0 + 1),
            run.outcomes.get(n0 + 2),
            run.outcomes.get(n0 + 3),
        );
    }
}

/// 打中那一格的写死复现：单元区 R3_OPUS_UNIT_AREA 槽（副本开关），第一个文件 → 可写挂载 → 覆盖写 24 次 → 盘 1 上槽 R3_OPUS_BAD_SLOT
/// 起的读一律报错（持久坏扇区，覆盖写之后、回退之前开）→ 回退到环里最旧的根，连试 R3_OPUS_ATTEMPTS 次 → 可写挂载两次。
/// 每一步之前与之后在镜像上读两块盘择到的系统配置里的实例代号（`instance_generation_to_acquire` 同一个读法：最高实例 + 1）。
#[test]
fn z16_rollback_refused_after_acquisition_burns_instance_generations() {
    use singlefs_core::recovery::highest_root_instance;
    let overwrites = env_u64("R3_OPUS_OVERWRITES", 24) as usize;
    let bad_slot = env_u64("R3_OPUS_BAD_SLOT", 50245);
    let attempts = env_u64("R3_OPUS_ATTEMPTS", 3) as usize;
    let checker = std::env::var("R3_OPUS_CHECKER").as_deref() != Ok("0");
    let w = width();
    let plan = SharedFaultPlan::unarmed(w.fixed_geometry());
    let plan_for_observer = plan.clone();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, overwrites))
            .chain(std::iter::repeat_n(
                HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                    index_from_newest: overwrites as u64 - 1,
                }),
                attempts,
            ))
            .chain([HistoryOperation::CloseAndMountWritable, HistoryOperation::CloseAndMountWritable])
            .collect(),
    };
    let arm_after = StepPosition::Operation(overwrites);
    let mut instance_seen: Vec<(StepPosition, String)> = Vec::new();
    let run = execute_history_with_faults(
        &history,
        HistoryExecution {
            per_step_checker: if checker { PerStepChecker::Run } else { PerStepChecker::Skipped },
            device_width: w,
        },
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            let sc = singlefs_core::recovery::choose_system_configuration(observation.image);
            let generations = format!(
                "{:?}",
                sc.map(|sc| sc.quantities.journal_instance)
            );
            instance_seen.push((observation.position, generations));
            if observation.position == arm_after {
                plan_for_observer.arm(FaultSchedule {
                    fault: InjectedFault::ReadFails,
                    device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
                    placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(bad_slot * 16384)),
                    counting: FaultCounting::AcrossThePool,
                    occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(1),
                });
            }
        },
    );
    eprintln!("R3OPUS-BURN width={w:?} unit_area={:?} bad_slot={bad_slot} fired={} ending={}", std::env::var("SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS").ok(), plan.fired_count(), ending_name(&run.ending));
    for (index, outcome) in run.outcomes.iter().enumerate().skip(overwrites) {
        eprintln!("R3OPUS-BURN step {index}: {outcome:?}");
    }
    for (position, generation) in instance_seen.iter().skip(overwrites) {
        eprintln!("R3OPUS-BURN after {position:?}: system configuration instance generation {generation}");
    }
}

/// 同一格，历史执行器停在第一次模型对不上之后接着手动走：拿回退之前那一刻的镜像（观察者取），换上带「盘 1 槽 R3_OPUS_BAD_SLOT 读一律报错」
/// 的注入层，连着调 `mount_rollback`（回退到环里最旧的根）R3_OPUS_ATTEMPTS 次、再 `mount_writable` 两次；每次之前与之后读
/// `instance_generation_to_acquire`、两块盘四个系统配置槽的字节是否变了、录制流步数。
#[test]
fn z16_rollback_refused_after_acquisition_burns_instance_generations_by_hand() {
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::mount::{mount_rollback, mount_writable, RollbackTarget, ShadowLedger};
    use singlefs_core::recovery::{choose_system_configuration, readable_roots};
    use singlefs_core::transaction::{instance_generation_to_acquire, PoolWriter};
    use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
    use singlefs_harness::fault_injection::FaultInjectingBlockDevice;
    use singlefs_harness::RecordingBlockDevice;
    let overwrites = env_u64("R3_OPUS_OVERWRITES", 24) as usize;
    let bad_slot = env_u64("R3_OPUS_BAD_SLOT", 50245);
    let attempts = env_u64("R3_OPUS_ATTEMPTS", 3) as usize;
    let w = width();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, overwrites))
            .collect(),
    };
    let mut before: Option<MemoryPool> = None;
    let plan = SharedFaultPlan::unarmed(w.fixed_geometry());
    let run = execute_history_with_faults(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: w },
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if observation.position == StepPosition::Operation(overwrites) {
                before = Some(observation.image.clone());
            }
        },
    );
    eprintln!("R3OPUS-HAND prefix ending={} last={:?}", ending_name(&run.ending), run.outcomes.last());
    let image = before.expect("前缀跑完");
    let parameters = w.parameters();
    let bytes = w.device_bytes();
    let stream = SharedStream::new();
    let fault = SharedFaultPlan::armed(
        w.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::ReadFails,
            device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
            placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(bad_slot * 16384)),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(1),
        },
    );
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (
                *identity,
                FaultInjectingBlockDevice::new(
                    *identity,
                    RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
                    fault.clone(),
                ),
            )
        })
        .collect();
    let oldest = {
        let sc = choose_system_configuration(&image).expect("系统配置");
        let mut roots: Vec<(u64, u32)> = readable_roots(&image, &sc.immutable.region_devices, &sc.immutable.sizes, &sc.immutable.filesystem_identifier)
            .iter()
            .map(|root| (root.checkpoint_txg.0, root.instance.0))
            .collect();
        roots.sort();
        roots[0]
    };
    let to_acquire = |devices: &mut Vec<_>| instance_generation_to_acquire(&PoolWriter::new(&parameters, devices.as_mut_slice()));
    for attempt in 0..attempts {
        let expected = to_acquire(&mut devices);
        let ops_before = stream.operation_count();
        let result = mount_rollback(
            &parameters,
            &mut devices,
            RollbackTarget { instance: InstanceGeneration(oldest.1), checkpoint_txg: CheckpointTxg(oldest.0) },
            ShadowLedger::On,
        );
        eprintln!(
            "R3OPUS-HAND rollback attempt {attempt} to (instance {}, txg {}): to_acquire before={expected:?} after={:?} writes+barriers issued={} result={}",
            oldest.1,
            oldest.0,
            to_acquire(&mut devices),
            stream.operation_count() - ops_before,
            match &result {
                Ok(mounted) => format!("Ok(instance {:?})", mounted.output.instance),
                Err(error) => format!("{error:?}").chars().take(300).collect(),
            }
        );
    }
    for attempt in 0..2 {
        let expected = to_acquire(&mut devices);
        let ops_before = stream.operation_count();
        let result = mount_writable(&parameters, &mut devices);
        eprintln!(
            "R3OPUS-HAND writable attempt {attempt}: to_acquire before={expected:?} after={:?} writes+barriers issued={} result={}",
            to_acquire(&mut devices),
            stream.operation_count() - ops_before,
            match &result {
                Ok(mounted) => format!("Ok(instance {:?})", mounted.output.instance),
                Err(error) => format!("{error:?}").chars().take(300).collect(),
            }
        );
    }
}

/// 冻结着一次失败之后挂载（手动走）：历史前缀（第一个文件 → 可写挂载 → 覆盖写 n0 次）之后的镜像上，由这里自己的进程接着发一次覆盖写，
/// 第 k 次写（或屏障）报错 → 那次发布冻结在这个进程的分配器上；同一个进程里再试一次覆盖写（冻结着，应在任何读写之前拒）；
/// 然后放手（进程退出：冻结的那一次只住内存）→ 同一批盘上 `mount_writable`（副本开关打出预演与真发逐次比）→ 覆盖写一次 → 池级 checker。
#[test]
fn z16_mount_after_a_frozen_publish_by_hand() {
    use singlefs_checker::image::InvariantVerdict;
    use singlefs_checker::walk::check_pool_image;
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::mount::mount_writable;
    use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
    use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
    use singlefs_harness::fault_injection::FaultInjectingBlockDevice;
    use singlefs_harness::RecordingBlockDevice;
    let n0 = env_u64("R3_OPUS_OVERWRITES", 3) as usize;
    let k_to = env_u64("R3_OPUS_K_TO", 40);
    let fault_kind = std::env::var("R3_OPUS_FAULT").unwrap_or_else(|_| "write".to_string());
    let w = width();
    let parameters = w.parameters();
    let bytes = w.device_bytes();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_CONTENT_OVERWRITE, n0))
            .chain([HistoryOperation::CloseAndMountWritable])
            .collect(),
    };
    let mut before: Option<MemoryPool> = None;
    let plan = SharedFaultPlan::unarmed(w.fixed_geometry());
    execute_history_with_faults(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: w },
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if observation.position == StepPosition::Operation(n0) {
                before = Some(observation.image.clone());
            }
        },
    );
    let image = before.expect("前缀跑完");
    for k in 1..=k_to {
        let stream = SharedStream::new();
        let fault = SharedFaultPlan::unarmed(w.fixed_geometry());
        let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>)> = image
            .devices
            .iter()
            .map(|(identity, sparse)| {
                let mut device = SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512));
                device.image = sparse.clone();
                (*identity, FaultInjectingBlockDevice::new(*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()), fault.clone()))
            })
            .collect();
        // 这个进程：可写挂载（不注入），接着覆盖写一次、注入第 k 次写 / 屏障。
        let mut mounted = mount_writable(&parameters, &mut devices).expect("前缀之后可写挂载");
        let PoolVersion::WithFile(current) = mounted.current.clone() else { panic!("带文件") };
        fault.arm(FaultSchedule::the_nth_call_across_the_pool(
            if fault_kind == "barrier" { InjectedFault::BarrierFails } else { InjectedFault::WriteFails },
            k,
        ));
        let content = vec![7u8; 100];
        let first = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_overwrite(&mut writer, &mut mounted.allocator, &current, FirstFile { content: &content, write_time_seconds: 1_788_000_100 }, mounted.output.instance)
        };
        fault.disarm();
        let second = {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_overwrite(&mut writer, &mut mounted.allocator, &current, FirstFile { content: &content, write_time_seconds: 1_788_000_200 }, mounted.output.instance)
        };
        drop(mounted);
        // 进程退出；同一批盘重开可写挂载。
        let remount = mount_writable(&parameters, &mut devices);
        let remount_text = match &remount {
            Ok(m) => format!("Ok(instance {:?}, publishes {})", m.output.instance, 1 + m.output.warm_up_publishes.len()),
            Err(e) => format!("{e:?}").chars().take(200).collect(),
        };
        let after = remount.ok().map(|mut m| {
            let PoolVersion::WithFile(current) = m.current.clone() else { return "no-file".to_string() };
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            match publish_overwrite(&mut writer, &mut m.allocator, &current, FirstFile { content: &content, write_time_seconds: 1_788_000_300 }, m.output.instance) {
                Ok(_) => "Ok".to_string(),
                Err(e) => format!("{e:?}").chars().take(160).collect(),
            }
        });
        let final_image = MemoryPool {
            devices: devices.iter().map(|(identity, device)| (*identity, device.inner().inner().image.clone())).collect(),
            device_size_in_bytes: bytes,
        };
        let violated: Vec<String> = check_pool_image(&final_image)
            .into_iter()
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {}", detail.chars().take(120).collect::<String>())),
                _ => None,
            })
            .collect();
        eprintln!(
            "R3OPUS-FROZEN2 width={w:?} fault={fault_kind} k={k} fired={} first={} second={} remount={remount_text} after={after:?} checker={violated:?}",
            fault.fired_count(),
            match &first { Ok(_) => "Ok".to_string(), Err(e) => format!("{e:?}").chars().take(80).collect() },
            match &second { Ok(_) => "Ok".to_string(), Err(e) => format!("{e:?}").chars().take(120).collect() },
        );
    }
}
