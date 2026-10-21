//! 攻方腿原型（m4-segment-code-r3，Q2 H4）：修复批三的计划补满（跳过这次发布本来就重写的节点、试算被拒再计划、只剩本来就重写的节点时全带上并删意图）。
//! 随机历史（比重取修复批的 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK` 或快照那一组，种子 = 本轮基数 + 9_000_000 + 下标，前两轮攻方用 +7_000_000、+8_000_000）
//! 跑到底，取最后一步的镜像，在它上面重新可写挂载，按 K = 1、2、3 与候选段逐段强制整理，一次只放一次发布（B_gc 压到 1），每一次发布之后核：
//! 照抄意图的一批至少真搬一个（H4 的承诺）；搬的个数不超过 K（删意图那一批另记，实现员交下来的「可能超过 K」）；池级 checker 不红
//! （「钉最旧 txg、记账多」那一形单列）。每一格从同一张镜像重开。
use std::collections::BTreeSet;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionBatchSize, CompactionBatchUnits, CompactionSwitches};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};
use singlefs_format::CLUSTER_SEGMENT_SLOTS;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::SharedStream;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image.devices.iter().map(|(identity, sectors)| {
        let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
        device.image = sectors.clone();
        (*identity, device)
    }).collect()
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)], size: u64) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: size }
}

fn reds_of(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image).into_iter().filter_map(|(name, verdict)| match verdict {
        InvariantVerdict::Violated(why) => {
            let known = name == "I-3.1" && why.contains("记账的已分配") && !why.contains("环里自证过的根槽 24 个");
            Some(format!("{}{name}: {}", if known { "已知形?" } else { "" }, why.chars().take(260).collect::<String>()))
        }
        _ => None,
    }).collect()
}

fn width_of(name: &str) -> HistoryDeviceWidth {
    match name {
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    }
}

#[test]
fn compaction_batches_on_random_prefixes() {
    let width = width_of(&std::env::var("ATTACK_WIDTH").unwrap_or_else(|_| "4g".into()));
    let weights = match std::env::var("ATTACK_WEIGHTS").as_deref() {
        Ok("snapshots") => GenerationWeights::SNAPSHOT_OPERATIONS,
        Ok("compaction") => GenerationWeights::COMPACTION_WHILE_MOUNTED,
        _ => GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
    };
    let start = env_u64("ATTACK_SEED_START", 0);
    let seeds = env_u64("ATTACK_SEEDS", 4);
    let steps = usize::try_from(env_u64("ATTACK_HISTORY_STEPS", 20)).expect("小");
    let publishes = env_u64("ATTACK_PUBLISHES", 30);
    let regions = usize::try_from(env_u64("ATTACK_REGIONS", 3)).expect("小");
    let mut totals = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    for index in start..start + seeds {
        let seed = HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(9_000_000 + index));
        let history = generate_history_with_weights(seed, steps, &weights);
        let mut last: Option<MemoryPool> = None;
        let check_every_step = env_u64("ATTACK_CHECK_EVERY_STEP", 0) == 1;
        let mut step_reds: Vec<String> = Vec::new();
        let run = execute_history_with(
            &history,
            HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: SpaceAdmission::JudgedByTheFormula },
            &SharedStream::new(),
            &mut |observation| {
                if check_every_step {
                    for red in reds_of(observation.image) {
                        step_reds.push(format!("{:?} {red}", observation.position));
                    }
                }
                last = Some(observation.image.clone());
            },
        );
        let ending = format!("{:?}", run.ending).chars().take(400).collect::<String>();
        let kinds: std::collections::BTreeMap<String, usize> = history.operations.iter().fold(std::collections::BTreeMap::new(), |mut m, o| { *m.entry(format!("{:?}", o.kind())).or_default() += 1; m });
        eprintln!("HISTORY seed={index} ending={ending} step_reds={} kinds={kinds:?}", step_reds.len());
        for red in step_reds.iter().take(3) {
            eprintln!("STEPRED seed={index} {red}");
        }
        if env_u64("ATTACK_HISTORY_ONLY", 0) == 1 {
            continue;
        }
        let Some(image) = last else { continue };
        let parameters = width.parameters();
        // 候选段：现行那一版装着未释放记录、不是这次挂载开着的段（与执行器的强制选段同一个取法）。
        let mut probe = devices_of(&image);
        let Ok(mounted) = mount_writable_with_space_admission(&parameters, &mut probe, SpaceAdmission::JudgedByTheFormula) else {
            eprintln!("SEED {index} mount-refused ending={ending}");
            continue;
        };
        let (_o, probe_session) = MountedSession::of_the_mount(mounted);
        let Some(version) = probe_session.current.version_of_trees() else { continue };
        let candidates: Vec<CompactionRegion> = version.allocation_records.iter()
            .filter(|r| r.device == DeviceIdentity(0) && !r.is_released)
            .map(|r| r.slot.0 - r.slot.0 % CLUSTER_SEGMENT_SLOTS)
            .filter(|s| !probe_session.allocator.cluster_segments().contains(&SlotNumber(*s)))
            .collect::<BTreeSet<u64>>()
            .into_iter()
            .filter_map(|s| CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(s)).ok())
            .collect();
        let live_on_disk = probe_session.allocator.compaction().live_intent().map(|l| l.intent.region);
        let mut regions_to_try: Vec<Option<CompactionRegion>> = candidates.iter().take(regions).map(|r| Some(*r)).collect();
        if live_on_disk.is_some() {
            regions_to_try = vec![live_on_disk];
        }
        for k in 1..=3u64 {
            for region in &regions_to_try {
                let mut devices = devices_of(&image);
                let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula).expect("同一张镜像挂得上");
                let (_o, mut session) = MountedSession::of_the_mount(mounted);
                let mut line = format!("SEED {index} k={k} region={:?}", region.map(|r| r.start_slot().0));
                let mut published = 0u64;
                for _ in 0..publishes {
                    let live_before = session.allocator.compaction().live_intent();
                    let mut switches = CompactionSwitches::PRODUCT;
                    switches.batch_size = CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(usize::try_from(k).expect("小")).expect("至少 1"));
                    switches.forced_region = live_before.map(|l| l.intent.region).or(*region);
                    session.allocator.set_compaction_switches(switches);
                    session.compaction.publishes_since_the_last_user_change = 19;
                    let run = match session.run_compaction(&mut devices) {
                        Ok(run) => run,
                        Err(refused) => { line.push_str(&format!(" refused={refused:?}")); break; }
                    };
                    let live_after = session.allocator.compaction().live_intent();
                    let stop = match &run.outcome {
                        CompactionRunOutcome::Stopped(stop) => format!("{stop:?}"),
                        CompactionRunOutcome::Failed(failure) => format!("FAILED {failure:?}"),
                    };
                    if run.versions_published.is_empty() {
                        line.push_str(&format!(" stop={}", stop.chars().take(60).collect::<String>()));
                        break;
                    }
                    for version in &run.versions_published {
                        published += 1;
                        totals.0 += 1;
                        let relocated = u64::try_from(version.units_relocated_by_this_publish.len()).expect("小");
                        let txg = version.root.checkpoint_txg.0;
                        // 照抄意图的一批：发之前、发之后都有活意图，且不是签发（签发那一次发之前没有）。
                        if live_before.is_some() && live_after.is_some() {
                            totals.1 += 1;
                            if relocated == 0 {
                                totals.3 += 1;
                                eprintln!("PROBLEM seed={index} k={k} txg={txg} 照抄意图的一批一个没搬");
                            }
                        }
                        if live_before.is_some() && live_after.is_none() {
                            totals.2 += 1;
                            if relocated > k {
                                totals.4 += 1;
                                eprintln!("NOTE seed={index} k={k} txg={txg} 删意图那一批搬了 {relocated} 个，多于 K");
                            }
                        }
                        if relocated > k && live_after.is_some() {
                            totals.4 += 1;
                            eprintln!("PROBLEM seed={index} k={k} txg={txg} 照抄意图的一批搬了 {relocated} 个，多于 K");
                        }
                    }
                    for red in reds_of(&image_of(&devices, image.device_size_in_bytes)) {
                        totals.5 += 1;
                        eprintln!("RED seed={index} k={k} after-publish={published} {red}");
                    }
                    if stop.starts_with("FAILED") {
                        eprintln!("PROBLEM seed={index} k={k} {}", stop.chars().take(300).collect::<String>());
                        break;
                    }
                }
                line.push_str(&format!(" published={published}"));
                eprintln!("{line}");
            }
        }
    }
    eprintln!("ATTACK_SUMMARY publishes={} carry_batches={} completing={} carry_zero={} over_k={} reds={}", totals.0, totals.1, totals.2, totals.3, totals.4, totals.5);
}
