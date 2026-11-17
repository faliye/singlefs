//! m4-close-fixes-code-r3 攻方腿模型（副本用）：映射 key 集合那一判有没有拒合法的盘。随机历史的十六组比重（留存条目、删快照、整理、回退、截断、按 inode 写都在里面）、
//! 两种盘宽，每一步之后把盘上根环里读得出的每条根都重建一遍（同一条根只判一次），数拒在 key 集合那一判的根；历史自己的可写挂载要是被这一判拒，历史报失败，同样打印。
//! 段数按先跑一批估的时长定：估出全量超过 40 分钟就停，打印估时。
//! 不枚举崩溃状态：key 集合是一条根下面映射树与各树自己的内容，封在这条根的 Merkle 链下，崩溃只决定落到哪一条根，发过的根这里都判到（环轮出去之前）。

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, rebuild_version, RebuildVersionFailure,
    RecoveryFailure, CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS,
};
use singlefs_verification::history::{
    execute_history_with, generate_history_with_weights, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistorySeed, NamedGenerationWeights, PerStepChecker,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_verification::SharedStream;

const OPERATIONS_PER_HISTORY: usize = 60;
const WEIGHTS: [(NamedGenerationWeights, &str); 16] = [
    (NamedGenerationWeights::Broad, "broad"),
    (NamedGenerationWeights::ReuseAfterRaisingTheFloor, "reuse"),
    (NamedGenerationWeights::RollbackAfterRaisingTheFloor, "rollback"),
    (NamedGenerationWeights::TowardTheAllocationRecordWall, "record_wall"),
    (NamedGenerationWeights::TowardTheUnitAreaWall, "unit_area_wall"),
    (NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves, "deleting"),
    (NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep, "orphans"),
    (NamedGenerationWeights::NewestSystemConfigurationSlotReadingZeros, "sysconf_zeros"),
    (NamedGenerationWeights::SnapshotOperations, "snapshots"),
    (NamedGenerationWeights::CompactionWhileMounted, "compaction"),
    (NamedGenerationWeights::CompactionWhileMountedWithRollback, "compaction_rollback"),
    (NamedGenerationWeights::BroadWithCompactionWhileMounted, "broad_compaction"),
    (NamedGenerationWeights::Truncation, "truncation"),
    (NamedGenerationWeights::WritesByInodeNumber, "by_inode"),
    (NamedGenerationWeights::SizeChangesOfInodesOtherThanTheFirst, "size_changes"),
    (NamedGenerationWeights::PartialWrites, "partial_writes"),
];
const SEEDS_PER_WEIGHT_AT_MOST: u64 = 24;
const WALL_CLOCK_BUDGET_SECONDS: f64 = 40.0 * 60.0;

#[derive(Default)]
struct Tally {
    histories: AtomicU64,
    roots_judged: AtomicU64,
    key_set_refusals: AtomicU64,
    other_refusals: AtomicU64,
    endings_not_completed: AtomicU64,
    endings_naming_the_key_set: AtomicU64,
}

fn run_one_history(weight: usize, seed: u64, width: HistoryDeviceWidth, tally: &Tally) {
    let (named, name) = WEIGHTS[weight];
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(seed)),
        OPERATIONS_PER_HISTORY,
        &named.weights(),
    );
    let mut judged: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &SharedStream::new(),
        &mut |observation| {
            let Some(record) = observation
                .current_version_of_trees
                .map(|version| version.record.clone())
            else {
                return;
            };
            let Ok(system_configuration) = choose_system_configuration(observation.image) else {
                return;
            };
            for root in readable_roots(
                observation.image,
                &system_configuration.immutable.region_devices,
                &system_configuration.immutable.sizes,
                &system_configuration.immutable.filesystem_identifier,
            ) {
                let identity = (
                    root.instance.0,
                    root.checkpoint_txg.0,
                    root.tree_table.locations[0].unit_checksum,
                );
                if !judged.insert(identity) {
                    continue;
                }
                tally.roots_judged.fetch_add(1, Ordering::SeqCst);
                match rebuild_version(observation.image, &root, Some(record.clone())) {
                    Ok(_) => {}
                    Err(RebuildVersionFailure::Walk(RecoveryFailure::InvariantViolated {
                        detail,
                        ..
                    })) if detail == CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS => {
                        tally.key_set_refusals.fetch_add(1, Ordering::SeqCst);
                        println!(
                            "attack-fixes-r3: legal_disks KEY_SET_REFUSAL weight={name} seed={seed} width={} instance={} txg={} operation={:?}",
                            width.name(),
                            root.instance.0,
                            root.checkpoint_txg.0,
                            observation.operation
                        );
                    }
                    Err(_) => {
                        tally.other_refusals.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        },
    );
    tally.histories.fetch_add(1, Ordering::SeqCst);
    if !matches!(run.ending, HistoryEnding::Completed) {
        tally.endings_not_completed.fetch_add(1, Ordering::SeqCst);
        let ending = format!("{:?}", run.ending);
        if ending.contains("key 集合") || ending.contains(CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS) {
            tally.endings_naming_the_key_set.fetch_add(1, Ordering::SeqCst);
        }
        println!(
            "attack-fixes-r3: legal_disks ending weight={name} seed={seed} width={} ending={}",
            width.name(),
            ending.chars().take(400).collect::<String>()
        );
    }
}

/// 每组比重、两种盘宽各跑 `seeds` 段，起满线程分着跑。
fn run_the_batch(first_seed: u64, seeds: u64, tally: &Tally) {
    let mut jobs: Vec<(usize, u64, HistoryDeviceWidth)> = Vec::new();
    for weight in 0..WEIGHTS.len() {
        for seed in first_seed..first_seed + seeds {
            jobs.push((weight, seed, HistoryDeviceWidth::FourGibibytes));
            jobs.push((weight, seed, HistoryDeviceWidth::UnitAreaOf384Slots));
        }
    }
    let threads = std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get).min(16);
    let next = AtomicU64::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = usize::try_from(next.fetch_add(1, Ordering::SeqCst)).expect("下标");
                let Some((weight, seed, width)) = jobs.get(index).copied() else {
                    break;
                };
                run_one_history(weight, seed, width, tally);
            });
        }
    });
}

#[test]
fn no_published_root_of_the_random_histories_is_refused_by_the_key_set_judgement() {
    let tally = Tally::default();
    let started = Instant::now();
    run_the_batch(0, 1, &tally);
    let seconds_of_one_seed = started.elapsed().as_secs_f64();
    let seeds_left = (((WALL_CLOCK_BUDGET_SECONDS - seconds_of_one_seed) / seconds_of_one_seed.max(0.001)).floor() as u64)
        .min(SEEDS_PER_WEIGHT_AT_MOST - 1);
    println!(
        "attack-fixes-r3: legal_disks estimate seconds_of_one_seed_per_weight={seconds_of_one_seed:.1} seeds_at_most={SEEDS_PER_WEIGHT_AT_MOST} seeds_run={} estimated_total_seconds={:.0}",
        seeds_left + 1,
        seconds_of_one_seed * (seeds_left + 1) as f64
    );
    run_the_batch(1, seeds_left, &tally);
    println!(
        "attack-fixes-r3: legal_disks histories={} roots_judged={} key_set_refusals={} other_refusals={} endings_not_completed={} endings_naming_the_key_set={} seconds={:.0}",
        tally.histories.load(Ordering::SeqCst),
        tally.roots_judged.load(Ordering::SeqCst),
        tally.key_set_refusals.load(Ordering::SeqCst),
        tally.other_refusals.load(Ordering::SeqCst),
        tally.endings_not_completed.load(Ordering::SeqCst),
        tally.endings_naming_the_key_set.load(Ordering::SeqCst),
        started.elapsed().as_secs_f64()
    );
    assert!(tally.roots_judged.load(Ordering::SeqCst) > 0);
    assert_eq!(tally.key_set_refusals.load(Ordering::SeqCst), 0);
    assert_eq!(tally.endings_naming_the_key_set.load(Ordering::SeqCst), 0);
}
