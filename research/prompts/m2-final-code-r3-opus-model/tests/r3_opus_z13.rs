//! m2-final-code-r3 云端攻方 Z13：分配记录树按绝对槽号按位置寻址。只在冻结副本的拷贝上跑，用它自己的历史执行器
//! （每一步之后池级 checker、理想模型对拍、panic 接住），种子取与门禁不同的一批。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

use singlefs_harness::history::{
    execute_history_with, run_history_campaign, ContentChoice, ContentLength, FindingShrinking,
    GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
    StepOutcome,
};
use singlefs_harness::SharedStream;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn width_of(name: &str) -> HistoryDeviceWidth {
    match name {
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    }
}

fn weights_of(name: &str) -> GenerationWeights {
    match name {
        "reuse" => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        "rollback" => GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        "wall" => GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        "unitwall" => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        _ => GenerationWeights::BROAD,
    }
}

/// 随机历史一段：比重、盘宽、种子起点、种子数、步数都走环境变量；每一步之后跑池级 checker。
#[test]
fn z13_campaign() {
    let weights_name = std::env::var("R3_OPUS_WEIGHTS").unwrap_or_else(|_| "broad".to_string());
    let width_name = std::env::var("R3_OPUS_WIDTH").unwrap_or_else(|_| "4g".to_string());
    let first_seed = env_u64("R3_OPUS_FIRST_SEED", 3_000_000_000);
    let seeds = env_u64("R3_OPUS_SEEDS", 32);
    let operations = env_u64("R3_OPUS_OPS", 40) as usize;
    let threads = env_u64("R3_OPUS_THREADS", 8) as usize;
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        first_seed,
        seeds,
        operations,
        &weights_of(&weights_name),
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width: width_of(&width_name),
        },
        threads,
        FindingShrinking::ReportSeedsOnly,
    );
    eprintln!(
        "R3OPUS-CAMPAIGN weights={weights_name} width={width_name} first_seed={first_seed} seeds={seeds} ops={operations} \
         new_findings={} known_red={} most_records={} secs={:.1}\n{}",
        report.new_findings.len(),
        report.known_red_hits.len(),
        report.tally.most_allocation_records_in_one_version,
        started.elapsed().as_secs_f64(),
        report.render()
    );
}

fn overwrite(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 2999 },
        fill_seed,
    })
}

/// 叶 62 缺席之后又长出来：第一个文件（4 GiB，叶 61 罩到槽 50343）→ 可写挂载 → 覆盖写 n1 次（提交内生块越过 50344，叶 62 出现）
/// → 回退到环里第 k 新的根（选在叶 62 出现之前的那几条上时，那一版里叶 62 缺席）→ 覆盖写 m 次（叶 62 按那一版的缺席重新长出来）
/// → 可写挂载 → 冷启动读回。用户决定的那几步（k、m）整段放开扫，只固定前缀（n1）。
#[test]
fn z13_leaf_absent_then_regrown_after_rollback() {
    let n1_from = env_u64("R3_OPUS_N1_FROM", 4);
    let n1_to = env_u64("R3_OPUS_N1_TO", 14);
    let m_to = env_u64("R3_OPUS_M_TO", 12);
    let shard = env_u64("R3_OPUS_SHARD", 0);
    let shards = env_u64("R3_OPUS_SHARDS", 1);
    let mut cells = 0u64;
    let mut endings: std::collections::BTreeMap<String, u64> = Default::default();
    let mut rollback_outcomes: std::collections::BTreeMap<String, u64> = Default::default();
    let mut index = 0u64;
    for n1 in n1_from..=n1_to {
        for k in 0..=23u64.min(n1 + 3) {
            for m in 0..=m_to {
                index += 1;
                if index % shards != shard {
                    continue;
                }
                let history = GeneratedHistory {
                    seed: HistorySeed(0),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
                        .chain((0..n1).map(overwrite))
                        .chain(std::iter::once(HistoryOperation::CloseAndMountRollback(
                            RollbackTargetChoice::RingRoot { index_from_newest: k },
                        )))
                        .chain((0..m).map(|i| overwrite(100 + i)))
                        .chain([HistoryOperation::CloseAndMountWritable, overwrite(999), HistoryOperation::ColdStartRecover])
                        .collect(),
                };
                let run = execute_history_with(
                    &history,
                    HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
                    &SharedStream::new(),
                    &mut |_| {},
                );
                cells += 1;
                let ending = match &run.ending {
                    HistoryEnding::Completed => "Completed".to_string(),
                    HistoryEnding::KnownRed { form, .. } => format!("KnownRed#{form}"),
                    HistoryEnding::NewFinding { signature, observation } => {
                        let text = format!(
                            "NewFinding {:?} @{:?}",
                            signature, observation.position
                        );
                        eprintln!("R3OPUS-Z13-FINDING n1={n1} k={k} m={m} {}", text.chars().take(800).collect::<String>());
                        format!("NewFinding {:?}", signature).chars().take(200).collect()
                    }
                };
                *endings.entry(ending).or_default() += 1;
                let rollback = run
                    .outcomes
                    .get(1 + n1 as usize)
                    .map(|outcome| match outcome {
                        StepOutcome::Applied(_) => "Applied".to_string(),
                        StepOutcome::Refused { member } => format!("Refused {member}"),
                        StepOutcome::NotApplicable(p) => format!("NotApplicable {p:?}"),
                    })
                    .unwrap_or_else(|| "none".to_string());
                *rollback_outcomes.entry(rollback).or_default() += 1;
            }
        }
    }
    eprintln!("R3OPUS-Z13-REGROW shard={shard}/{shards} cells={cells} endings={endings:?} rollback={rollback_outcomes:?}");
}

mod without_file {
    use singlefs_checker::image::InvariantVerdict;
    use singlefs_checker::walk::check_pool_image;
    use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
    use singlefs_core::mount::{mount_rollback, mount_writable, RollbackTarget, ShadowLedger};
    use singlefs_core::recovery::{choose_system_configuration, readable_roots};
    use singlefs_core::transaction::{acquire_instance, PoolWriter};
    use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
    use singlefs_harness::history::HistoryDeviceWidth;
    use singlefs_harness::{RecordingBlockDevice, SharedStream};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

    fn devices(bytes: u64, stream: &SharedStream) -> Devices {
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect()
    }

    fn image(devices: &Devices, bytes: u64) -> MemoryPool {
        MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.inner().image.clone()))
                .collect(),
            device_size_in_bytes: bytes,
        }
    }

    fn violations(devices: &Devices, bytes: u64) -> Vec<String> {
        check_pool_image(&image(devices, bytes))
            .into_iter()
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => {
                    Some(format!("{invariant}: {}", detail.chars().take(160).collect::<String>()))
                }
                _ => None,
            })
            .collect()
    }

    fn ring_roots_newest_first(devices: &Devices, bytes: u64) -> Vec<(u64, u32)> {
        let img = image(devices, bytes);
        let sc = choose_system_configuration(&img).expect("系统配置");
        let mut roots: Vec<(u64, u32)> = readable_roots(
            &img,
            &sc.immutable.region_devices,
            &sc.immutable.sizes,
            &sc.immutable.filesystem_identifier,
        )
        .iter()
        .map(|root| (root.checkpoint_txg.0, root.instance.0))
        .collect();
        roots.sort();
        roots.reverse();
        roots
    }

    /// 树表 0 条的一版：mkfs → 可写挂载（实例 1）→ 取号之后崩溃 a 次（实例表变长）→ 可写挂载 m1 次 → 回退到环里第 j 新的根
    /// → 可写挂载 m2 次。每一步之后池级 checker；报第一次违例、第一次 panic 与挂载被拒。
    fn one(width: HistoryDeviceWidth, a: u64, m1: u64, j: usize, m2: u64) -> String {
        let bytes = width.device_bytes();
        let parameters: MakeFilesystemParameters = width.parameters();
        let stream = SharedStream::new();
        let mut devs = devices(bytes, &stream);
        let outcome = catch_unwind(AssertUnwindSafe(|| -> String {
            make_filesystem(&parameters, &mut devs).expect("mkfs");
            if let Err(e) = mount_writable(&parameters, &mut devs) {
                return format!("first-mount-refused {:?}", format!("{e:?}").chars().take(160).collect::<String>());
            }
            {
                let mut writer = PoolWriter::new(&parameters, devs.as_mut_slice());
                for _ in 0..a {
                    acquire_instance(&mut writer).expect("取号之后崩溃");
                }
            }
            for i in 0..m1 {
                if let Err(e) = mount_writable(&parameters, &mut devs) {
                    return format!("m1[{i}]-refused {}", format!("{e:?}").chars().take(200).collect::<String>());
                }
                let v = violations(&devs, bytes);
                if !v.is_empty() {
                    return format!("m1[{i}]-checker {v:?}");
                }
            }
            let roots = ring_roots_newest_first(&devs, bytes);
            let Some((txg, instance)) = roots.get(j).copied() else {
                return "no-such-ring-root".to_string();
            };
            match mount_rollback(
                &parameters,
                &mut devs,
                RollbackTarget {
                    instance: InstanceGeneration(instance),
                    checkpoint_txg: singlefs_core::address::CheckpointTxg(txg),
                },
                ShadowLedger::On,
            ) {
                Ok(_) => {}
                Err(e) => {
                    return format!("rollback({instance},{txg})-refused {}", format!("{e:?}").chars().take(200).collect::<String>())
                }
            }
            let v = violations(&devs, bytes);
            if !v.is_empty() {
                return format!("rollback({instance},{txg})-checker {v:?}");
            }
            for i in 0..m2 {
                if let Err(e) = mount_writable(&parameters, &mut devs) {
                    return format!("m2[{i}]-refused {}", format!("{e:?}").chars().take(200).collect::<String>());
                }
                let v = violations(&devs, bytes);
                if !v.is_empty() {
                    return format!("m2[{i}]-checker {v:?}");
                }
            }
            "ok".to_string()
        }));
        match outcome {
            Ok(text) => text,
            Err(payload) => format!(
                "PANIC {}",
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default()
                    .chars()
                    .take(300)
                    .collect::<String>()
            ),
        }
    }

    #[test]
    fn z13_without_file_versions_rollback_and_regrow() {
        let width = match std::env::var("R3_OPUS_WIDTH").as_deref() {
            Ok("384") => HistoryDeviceWidth::UnitAreaOf384Slots,
            _ => HistoryDeviceWidth::FourGibibytes,
        };
        let shard: u64 = std::env::var("R3_OPUS_SHARD").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        let shards: u64 = std::env::var("R3_OPUS_SHARDS").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
        let mut index = 0u64;
        let mut tally: std::collections::BTreeMap<String, u64> = Default::default();
        let a_list: Vec<u64> = std::env::var("R3_OPUS_A_LIST")
            .ok()
            .map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect())
            .unwrap_or_else(|| vec![0, 368, 369, 738, 1200]);
        for a in a_list {
            for m1 in [1u64, 3, 6] {
                for j in 0..8usize {
                    for m2 in [0u64, 1, 3, 6] {
                        index += 1;
                        if index % shards != shard {
                            continue;
                        }
                        let text = one(width, a, m1, j, m2);
                        let class: String = text
                            .split(|c: char| c == ' ' || c == '(' || c == '[')
                            .next()
                            .unwrap_or("")
                            .to_string()
                            + if text.contains("refused") { " refused" } else if text.contains("checker") { " checker" } else { "" };
                        if !text.starts_with("ok") && !text.starts_with("no-such") {
                            eprintln!("R3OPUS-Z13WF a={a} m1={m1} j={j} m2={m2} {text}");
                        }
                        *tally.entry(class).or_default() += 1;
                    }
                }
            }
        }
        eprintln!("R3OPUS-Z13WF-TALLY width={width:?} shard={shard}/{shards} {tally:?}");
    }
}

/// 取样核对：regrow 那一段扫描里「叶 62 先有、回退后缺席、再长出来」真的走到了。对 n1、k 各取几格，
/// 每一步之后读镜像上择到的根下的分配记录（`recovery::allocation_records_under_root`，从盘上读），报盘 0 上最大的槽与叶号集合。
#[test]
fn z13_regrow_coverage_probe() {
    use singlefs_core::recovery::{allocation_records_under_root, choose_root, choose_system_configuration};
    use singlefs_harness::history::StepPosition;
    for (n1, k, m) in [(4u64, 3u64, 6u64), (6, 5, 6), (10, 9, 8), (14, 13, 12), (8, 2, 4)] {
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
                .chain((0..n1).map(overwrite))
                .chain(std::iter::once(HistoryOperation::CloseAndMountRollback(
                    RollbackTargetChoice::RingRoot { index_from_newest: k },
                )))
                .chain((0..m).map(|i| overwrite(100 + i)))
                .collect(),
        };
        let mut leaves_by_step: Vec<String> = Vec::new();
        let run = execute_history_with(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |observation| {
                let sc = choose_system_configuration(observation.image).expect("系统配置");
                let root = choose_root(observation.image, &sc).expect("根");
                let records = allocation_records_under_root(observation.image, &root).unwrap_or_default();
                let leaves: std::collections::BTreeSet<u64> = records
                    .iter()
                    .filter(|record| record.device.0 == 0)
                    .map(|record| record.slot.0 / 812)
                    .collect();
                let step = match observation.position {
                    StepPosition::StartingPoint => "S".to_string(),
                    StepPosition::Operation(i) => i.to_string(),
                };
                leaves_by_step.push(format!("{step}:{leaves:?}"));
            },
        );
        eprintln!("R3OPUS-Z13-PROBE n1={n1} k={k} m={m} ending={:?} leaves={}", matches!(run.ending, HistoryEnding::Completed), leaves_by_step.join(" "));
    }
}

/// 单个种子复看：打出生成的历史、每一步的结局与结局。
#[test]
fn z13_one_seed() {
    use singlefs_harness::history::generate_history_with_weights;
    let seed = env_u64("R3_OPUS_SEED", 4_000_000_045);
    let operations = env_u64("R3_OPUS_OPS", 60) as usize;
    let weights_name = std::env::var("R3_OPUS_WEIGHTS").unwrap_or_else(|_| "reuse".to_string());
    let width_name = std::env::var("R3_OPUS_WIDTH").unwrap_or_else(|_| "240".to_string());
    let history = generate_history_with_weights(HistorySeed(seed), operations, &weights_of(&weights_name));
    let run = execute_history_with(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Run, device_width: width_of(&width_name) },
        &SharedStream::new(),
        &mut |_| {},
    );
    eprintln!("R3OPUS-SEED seed={seed} start={:?}", history.starting_point);
    for (index, (operation, outcome)) in history.operations.iter().zip(run.outcomes.iter().map(Some).chain(std::iter::repeat(None))).enumerate() {
        eprintln!("R3OPUS-SEED step {index}: {operation:?} => {outcome:?}");
        if outcome.is_none() {
            break;
        }
    }
    eprintln!("R3OPUS-SEED ending={:?}", run.ending);
}
