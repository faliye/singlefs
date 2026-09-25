//! m2-safety-r1 云端攻方（Opus）S1：C542 那一格（取号之后发布失败、回退目标离环）在冻结副本的拷贝上量各候选。
//! 候选由副本专用补丁的环境变量开关选（不设时与冻结副本同行为）：
//! S_OPUS_S1_DRY_READS（甲：预演读盘核）、S_OPUS_S1_PROTECT（乙：这次挂载的根不落在目标的根环槽上）、
//! S_OPUS_S1_EQUIV（丁：目标已离环而最新根就是同一条回退写下的，改成回退到最新根）、S_OPUS_S1_DRY_PESSIMISTIC（戊：预演把换下的每一份都当作会被隔离）。
//! 单元区宽由 S_OPUS_UNIT_AREA_SLOTS 定（240 那一档的副本开关）。每段一行 `S1ROW`，制表符分列。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_rollback_with_space_admission, mount_writable_with_space_admission, MountError, Mounted,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_of_root, readable_roots, recover,
    rollback_witness_of_the_pool, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{instance_generation_to_acquire, PoolWriter};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots,
    RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_harness::history::{
    execute_history_with_faults, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    StepPosition,
};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Dev = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// S_OPUS_ADMISSION=skip：前缀与各次挂载都关掉空间准入（只供测试的开关；重现第三轮 Z16 那一格用）。不设时按产品路径判。
fn admission() -> SpaceAdmission {
    if std::env::var("S_OPUS_ADMISSION").as_deref() == Ok("skip") {
        SpaceAdmission::SkippedByTheTestOnlySwitch
    } else {
        SpaceAdmission::JudgedByTheFormula
    }
}

fn width() -> HistoryDeviceWidth {
    match std::env::var("S_OPUS_WIDTH").as_deref() {
        Ok("384") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok("256") => HistoryDeviceWidth::UnitAreaOf256Slots,
        Ok("4g") => HistoryDeviceWidth::FourGibibytes,
        _ => HistoryDeviceWidth::UnitAreaOf240Slots,
    }
}

const EMPTY_OVERWRITE: HistoryOperation = HistoryOperation::PublishOverwrite(ContentChoice {
    length: ContentLength::Empty,
    fill_seed: 0,
});

/// 第一个文件 → 可写挂载 → 空内容覆盖写 n 次之后的镜像（历史执行器，不跑每步 checker）。
fn prefix(n: usize) -> Option<MemoryPool> {
    let w = width();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(EMPTY_OVERWRITE, n))
            .collect(),
    };
    let mut image = None;
    let plan = SharedFaultPlan::unarmed(w.fixed_geometry());
    execute_history_with_faults(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: w, space_admission: admission() },
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if observation.position == StepPosition::Operation(n) {
                image = Some(observation.image.clone());
            }
        },
    );
    image
}

fn devices_of(image: &MemoryPool, plan: &SharedFaultPlan, stream: &SharedStream) -> Vec<(DeviceIdentity, Dev)> {
    let bytes = width().device_bytes();
    image
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
                    plan.clone(),
                ),
            )
        })
        .collect()
}

fn image_of(devices: &[(DeviceIdentity, Dev)]) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.inner().inner().image.clone())).collect(),
        device_size_in_bytes: width().device_bytes(),
    }
}

fn roots(image: &MemoryPool) -> Vec<(u64, u32)> {
    let sc = choose_system_configuration(image).expect("系统配置");
    let mut roots: Vec<(u64, u32)> = readable_roots(image, &sc.immutable.region_devices, &sc.immutable.sizes, &sc.immutable.filesystem_identifier)
        .iter()
        .map(|root| (root.checkpoint_txg.0, root.instance.0))
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

fn classify(result: &Result<Mounted, MountError>) -> String {
    match result {
        Ok(m) => format!("ok#{}", m.output.instance.0),
        Err(MountError::RollbackTargetNotACandidate { exclusion, .. }) => format!("notcand:{exclusion:?}"),
        Err(MountError::Publish(failed)) => {
            let cause = format!("{:?}", failed.cause);
            format!("AFTER:{}", cause.chars().take(90).collect::<String>())
        }
        Err(other) => {
            let text = format!("{other:?}");
            let head: String = text.chars().take_while(|c| *c != '{' && *c != '(' && *c != ' ').collect();
            let detail = if head.starts_with("Placement") || head.starts_with("WarmUp") || head.starts_with("RowPublish") {
                let unit = text.find("unit:").map(|i| text[i..].chars().take(40).collect::<String>()).unwrap_or_default();
                format!("[{unit}]")
            } else {
                String::new()
            };
            format!("{head}{detail}")
        }
    }
}

struct Attempt {
    class: String,
    burned: bool,
    image: MemoryPool,
    instance_acquired: Option<u32>,
}

/// 在镜像上跑一次操作（回退到 target 或普通可写挂载），fault 给就装上。交回结局、号涨没涨、之后的镜像。
fn attempt(image: &MemoryPool, target: Option<(u64, u32)>, fault: Option<FaultSchedule>) -> Attempt {
    let w = width();
    let parameters = w.parameters();
    let plan = match fault {
        Some(schedule) => SharedFaultPlan::armed(w.fixed_geometry(), schedule),
        None => SharedFaultPlan::unarmed(w.fixed_geometry()),
    };
    let stream = SharedStream::new();
    let mut devices = devices_of(image, &plan, &stream);
    let before = instance_generation_to_acquire(&PoolWriter::new(&parameters, devices.as_mut_slice()));
    let result = match target {
        Some((txg, instance)) => mount_rollback_with_space_admission(
            &parameters,
            &mut devices,
            RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) },
            ShadowLedger::On,
            admission(),
        ),
        None => mount_writable_with_space_admission(&parameters, &mut devices, admission()),
    };
    plan.disarm();
    let after = instance_generation_to_acquire(&PoolWriter::new(&parameters, devices.as_mut_slice()));
    let class = classify(&result);
    Attempt {
        class,
        burned: after != before,
        instance_acquired: if after != before { Some(before.0) } else { None },
        image: image_of(&devices),
    }
}

fn read_fault(device: u32, slot: u64, from: u64) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(device)),
        placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(slot * 16384)),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(from),
    }
}

/// 回退失败之后：让失败那次取到的实例 N 的根全读不出，恢复择到哪一条；落在 (txg, 实例) 大于目标、实例小于 N 的根上就是回退被撤销。
fn probe_with_failed_instance_unreadable(image: &MemoryPool, target: (u64, u32), failed_instance: u32) -> String {
    let parameters = width().parameters();
    let txgs: Vec<u64> = roots(image).into_iter().filter(|(_, i)| *i >= failed_instance).map(|(t, _)| t).collect();
    let slots: Vec<_> = txgs
        .iter()
        .map(|txg| target_for_publish(CheckpointTxg(*txg), parameters.geometry.root_ring_slots_per_region))
        .collect();
    let reader = PoolReaderWithUnreadableRootRingSlots::new(
        image,
        RootRingSlotTarget {
            named_slots: NamedRootRingSlots::naming(&slots),
            region_devices: parameters.region_devices,
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        },
    );
    let report = recover(&reader, JournalPolicy::Consult);
    let root = match &report.outcome {
        RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => Some(*root),
        RecoveryOutcome::Failed { root, .. } => *root,
    };
    let verdict = match root {
        // 失败那次一条根都没落：回退没生效，恢复择到哪条都与没发起回退时相同。
        _ if txgs.is_empty() => "noeffect",
        Some((i, t)) if (t.0, i.0) > target && i.0 < failed_instance => "ABANDONED",
        Some((i, t)) if (t.0, i.0) == target => "target",
        Some(_) => "other",
        None => "none",
    };
    format!("{verdict}:{:?}:unreadable={}", root.map(|(i, t)| (i.0, t.0)), txgs.len())
}

/// 一段：前缀之后回退到环里最旧的根。控制组不注入；做成的，取它这一串换下的每个槽，各注入一段。
/// S_OPUS_MODE：persistent（那一槽起的读一直报错，回退两次、清掉故障再回退一次、普通可写挂载）
/// | transient（那一槽第 k 次起报错，k = 1..=3，这一次之后故障消失，再回退一次）。
#[test]
fn s1_scan_one_width() {
    let n_from = env_u64("S_OPUS_N_FROM", 18) as usize;
    let n_to = env_u64("S_OPUS_N_TO", 35) as usize;
    let device = env_u64("S_OPUS_FAULT_DEVICE", 1) as u32;
    let mode = std::env::var("S_OPUS_MODE").unwrap_or_else(|_| "persistent".to_string());
    let slots_label = std::env::var("S_OPUS_UNIT_AREA_SLOTS").unwrap_or_else(|_| format!("{:?}", width()));
    for n in n_from..=n_to {
        let Some(image) = prefix(n) else {
            println!("S1ROW\t{slots_label}\t{n}\t-\tno-prefix");
            continue;
        };
        let all = roots(&image);
        let target = all[0];
        let control = attempt(&image, Some(target), None);
        let control_ring = roots(&control.image).contains(&target);
        println!(
            "S1ROW\t{slots_label}\t{n}\tcontrol\t-\t-\t{}\tburned={}\ttarget_in_ring={control_ring}",
            control.class, control.burned
        );
        if !control.class.starts_with("ok") {
            continue;
        }
        // 控制组那一串换下的槽：重跑一次拿 Mounted（attempt 不交回它）。
        let released: Vec<u64> = {
            let w = width();
            let plan = SharedFaultPlan::unarmed(w.fixed_geometry());
            let stream = SharedStream::new();
            let mut devices = devices_of(&image, &plan, &stream);
            let mounted = mount_rollback_with_space_admission(
                &w.parameters(),
                &mut devices,
                RollbackTarget { instance: InstanceGeneration(target.1), checkpoint_txg: CheckpointTxg(target.0) },
                ShadowLedger::On,
                admission(),
            )
            .expect("控制组做成");
            let mut slots: Vec<u64> = std::iter::once(&mounted.output.row_publish)
                .chain(&mounted.output.warm_up_publishes)
                .filter_map(|version| version.file_version())
                .flat_map(|output| output.released.iter().map(|placement| placement.slot.0))
                .collect();
            slots.sort();
            slots.dedup();
            slots
        };
        for slot in released {
            match mode.as_str() {
                "transient" => {
                    for k in 1..=3u64 {
                        let first = attempt(&image, Some(target), Some(read_fault(device, slot, k)));
                        let ring = roots(&first.image).contains(&target);
                        let retry = attempt(&first.image, Some(target), None);
                        let probe = match first.instance_acquired {
                            Some(n_failed) if !first.class.starts_with("ok") => {
                                probe_with_failed_instance_unreadable(&first.image, target, n_failed)
                            }
                            _ => "-".to_string(),
                        };
                        println!(
                            "S1ROW\t{slots_label}\t{n}\ttransient\t{slot}\tk={k}\t{}\tburned={}\ttarget_in_ring={ring}\tretry_cleared={}\tprobe={probe}",
                            first.class, first.burned, retry.class
                        );
                    }
                }
                _ => {
                    let schedule = read_fault(device, slot, 1);
                    let first = attempt(&image, Some(target), Some(schedule));
                    let ring = roots(&first.image).contains(&target);
                    let again = attempt(&first.image, Some(target), Some(schedule));
                    let cleared = attempt(&first.image, Some(target), None);
                    let writable_faulted = attempt(&first.image, None, Some(schedule));
                    let writable_cleared = attempt(&first.image, None, None);
                    let probe = match first.instance_acquired {
                        Some(n_failed) if !first.class.starts_with("ok") => {
                            probe_with_failed_instance_unreadable(&first.image, target, n_failed)
                        }
                        _ => "-".to_string(),
                    };
                    println!(
                        "S1ROW\t{slots_label}\t{n}\tpersistent\t{slot}\t-\t{}\tburned={}\ttarget_in_ring={ring}\tretry_faulted={}\tretry_cleared={}\twritable_faulted={}\twritable_cleared={}\tprobe={probe}",
                        first.class, first.burned, again.class, cleared.class, writable_faulted.class, writable_cleared.class
                    );
                }
            }
        }
    }
}

/// 池级 checker 在镜像上判红的不变量（空 = 全绿）。
fn checker_red(image: &MemoryPool) -> String {
    let red: Vec<&'static str> = singlefs_checker::walk::check_pool_image(image)
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(_) => Some(*invariant),
            _ => None,
        })
        .collect();
    format!("{red:?}")
}

/// 写 / 屏障失败：前缀之后回退到环里第 S_OPUS_TARGET_INDEX 旧的根（默认最旧），这次挂载的第 k 次写（或屏障）报错一次，k = 1..=K；
/// 之后故障消失：目标还在不在环里、同一条回退再试一次、普通可写挂载一次、失败实例的根全读不出时恢复落在哪。
#[test]
fn s1_write_fault_sweep() {
    let n = env_u64("S_OPUS_N", 24) as usize;
    let k_to = env_u64("S_OPUS_K_TO", 60);
    let index = env_u64("S_OPUS_TARGET_INDEX", 0) as usize;
    let kind = std::env::var("S_OPUS_FAULT").unwrap_or_else(|_| "write".to_string());
    let slots_label = std::env::var("S_OPUS_UNIT_AREA_SLOTS").unwrap_or_else(|_| format!("{:?}", width()));
    let Some(image) = prefix(n) else {
        println!("S1W\t{slots_label}\t{n}\tno-prefix");
        return;
    };
    let all = roots(&image);
    let target = all[index.min(all.len() - 1)];
    let control = attempt(&image, Some(target), None);
    println!("S1W\t{slots_label}\t{n}\ttarget={target:?}\tcontrol\t{}\ttarget_in_ring={}", control.class, roots(&control.image).contains(&target));
    for k in 1..=k_to {
        let fault = match kind.as_str() {
            "barrier" => InjectedFault::BarrierFails,
            _ => InjectedFault::WriteFails,
        };
        let first = attempt(&image, Some(target), Some(FaultSchedule::the_nth_call_across_the_pool(fault, k)));
        let ring = roots(&first.image).contains(&target);
        let retry = attempt(&first.image, Some(target), None);
        let writable = attempt(&first.image, None, None);
        let probe = match first.instance_acquired {
            Some(n_failed) if !first.class.starts_with("ok") => probe_with_failed_instance_unreadable(&first.image, target, n_failed),
            _ => "-".to_string(),
        };
        let checker = if std::env::var("S_OPUS_CHECKER").is_ok() {
            format!("\tchecker_first={}\tchecker_retry={}\tchecker_writable={}", checker_red(&first.image), checker_red(&retry.image), checker_red(&writable.image))
        } else {
            String::new()
        };
        println!(
            "S1W\t{slots_label}\t{n}\t{kind}\tk={k}\t{}\tburned={}\ttarget_in_ring={ring}\tretry={}\twritable={}\tprobe={probe}{checker}",
            first.class, first.burned, retry.class, writable.class
        );
    }
}

/// 候选多拒的那一格用户手里还有什么：前缀之后（不注入）依次试普通可写挂载、回退到环里每一条根（从最旧起），各在前缀那份镜像上独立试。
/// 再对「回退到最旧」做成的那一份，接着试普通可写挂载、回退到新的最新根。
#[test]
fn s1_exits() {
    let n = env_u64("S_OPUS_N", 24) as usize;
    let slots_label = std::env::var("S_OPUS_UNIT_AREA_SLOTS").unwrap_or_else(|_| format!("{:?}", width()));
    let Some(image) = prefix(n) else {
        println!("S1X\t{slots_label}\t{n}\tno-prefix");
        return;
    };
    let all = roots(&image);
    let writable = attempt(&image, None, None);
    println!("S1X\t{slots_label}\t{n}\twritable\t{}", writable.class);
    for (index, root) in all.iter().enumerate() {
        let tried = attempt(&image, Some(*root), None);
        println!("S1X\t{slots_label}\t{n}\trollback#{index}\t{root:?}\t{}", tried.class);
        if index == 0 && tried.class.starts_with("ok") {
            let after = attempt(&tried.image, None, None);
            let newest = *roots(&tried.image).last().expect("有根");
            let empty_rollback = attempt(&tried.image, Some(newest), None);
            println!("S1X\t{slots_label}\t{n}\tafter-oldest\twritable={}\trollback-to-newest={}", after.class, empty_rollback.class);
        }
    }
}

/// 新失败面：仓里自己的随机历史执行器（每一步之后池级 checker、理想模型对拍），候选开关由环境变量给。
/// S_OPUS_WEIGHTS（broad | rollback | reuse | unitwall）、S_OPUS_WIDTH、S_OPUS_FIRST_SEED、S_OPUS_SEEDS、S_OPUS_OPS、S_OPUS_THREADS。
#[test]
fn s1_campaign() {
    use singlefs_harness::history::{run_history_campaign, FindingShrinking, GenerationWeights};
    let weights = match std::env::var("S_OPUS_WEIGHTS").as_deref() {
        Ok("reuse") => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        Ok("rollback") => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        Ok("unitwall") => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        _ => GenerationWeights::BROAD,
    };
    let first_seed = env_u64("S_OPUS_FIRST_SEED", 5_100_000_000);
    let seeds = env_u64("S_OPUS_SEEDS", 64);
    let operations = env_u64("S_OPUS_OPS", 40) as usize;
    let threads = env_u64("S_OPUS_THREADS", 8) as usize;
    let report = run_history_campaign(
        first_seed,
        seeds,
        operations,
        &weights,
        HistoryExecution { per_step_checker: PerStepChecker::Run, device_width: width(), space_admission: admission() },
        threads,
        FindingShrinking::ReportSeedsOnly,
    );
    println!(
        "S1CAMPAIGN weights={:?} width={:?} admission={:?} first_seed={first_seed} seeds={seeds} ops={operations} new_findings={} known_red={}\n{}",
        std::env::var("S_OPUS_WEIGHTS").ok(),
        width(),
        admission(),
        report.new_findings.len(),
        report.known_red_hits.len(),
        report.render()
    );
}
