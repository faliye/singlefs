//! 攻方腿原型（m4-segment-code-r1）：把攻方混合比重下打中的一段随机历史缩短（逐块删操作，删了仍打中就留下删后的）。
//! 打中 = 某一步镜像上池级 checker 红了 ATTACK_INVARIANTS 里的一条，或结局不是 Completed。
//! ATTACK_SEED_INDEX 选种子（与 attack_segment_random_histories 同一个区间），ATTACK_KEEP 给起点（留哪几步，逗号隔开），
//! 挂钟超过 ATTACK_BUDGET_SECONDS 就停下打出当前的 KEEP，下一次接着缩。
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GeneratedHistory, GenerationWeights,
    HistoryEnding, HistoryExecution, HistoryOperationKind as K, HistorySeed,
    HistoryStartingPoint, NamedGenerationWeights, OverwriteContentDraw, RollbackTargetDraw,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

include!("attack_common/weights.rs");

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// 诊断：这张镜像根环里每一条读得出的根，重建那一版，打出落在 `dump_slot`（或它前一槽）上的角色与树表各条的根指针落点。
fn dump_every_root(image: &singlefs_harness::memory_pool::MemoryPool, dump_slot: u64) {
    use singlefs_core::recovery::{choose_system_configuration, readable_roots, rebuild_version_with_its_snapshot_entries, RebuiltVersion};
    let Ok(configuration) = choose_system_configuration(image) else {
        eprintln!("DUMP 系统配置择不出");
        return;
    };
    let roots = readable_roots(
        image,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    );
    for root in &roots {
        match rebuild_version_with_its_snapshot_entries(image, root, None) {
            Ok(rebuilt) => {
                let output = match &rebuilt.version {
                    RebuiltVersion::WithFile(output) | RebuiltVersion::WithTreesAndNoFile(output) => Some(output),
                    RebuiltVersion::WithoutFile => None,
                };
                if let Some(output) = output {
                    let at: Vec<String> = output
                        .units
                        .iter()
                        .filter(|unit| unit.slot.0 == dump_slot || unit.slot.0 + 1 == dump_slot)
                        .map(|unit| format!("{:?}@{}", unit.identity, unit.slot.0))
                        .collect();
                    let entries: Vec<String> = output
                        .tree_table_entries
                        .iter()
                        .map(|entry| format!("k{}:t{}:{}", entry.kind, entry.tree.0, entry.root.locations[0].slot.0))
                        .collect();
                    eprintln!("DUMP root txg={} inst={} at_slot={at:?} tree_table={entries:?}", root.checkpoint_txg.0, root.instance.0);
                }
            }
            Err(failure) => eprintln!("DUMP root txg={} inst={} 重建失败 {failure:?}", root.checkpoint_txg.0, root.instance.0),
        }
    }
}

fn hit(history: &GeneratedHistory, invariants: &[String], verbose: bool) -> bool {
    let mut red = false;
    let run = execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            if verbose {
                eprintln!(
                    "STEP {:?} op={:?} outcome={}",
                    observation.position,
                    observation.operation,
                    observation
                        .outcome
                        .map(|o| format!("{o:?}").chars().take(600).collect::<String>())
                        .unwrap_or_default()
                );
            }
            if red && !verbose {
                return;
            }
            if let Ok(dump_slot) = std::env::var("ATTACK_DUMP_SLOT") {
                let dump_slot: u64 = dump_slot.parse().expect("槽号");
                dump_every_root(observation.image, dump_slot);
            }
            for (name, verdict) in check_pool_image(observation.image) {
                if let InvariantVerdict::Violated(why) = verdict {
                    if invariants.iter().any(|wanted| wanted == name) {
                        red = true;
                    }
                    if verbose {
                        eprintln!("RED {:?} {name}: {}", observation.position, why.chars().take(2500).collect::<String>());
                    }
                }
            }
        },
    );
    if verbose {
        eprintln!("ENDING {}", format!("{:?}", run.ending).chars().take(3000).collect::<String>());
    }
    let ending_hit = env_u64("ATTACK_ENDING_COUNTS", 1) == 1 && run.ending != HistoryEnding::Completed;
    red || ending_hit
}

#[test]
fn minimize_a_hit() {
    let index = env_u64("ATTACK_SEED_INDEX", 126);
    let ops = usize::try_from(env_u64("ATTACK_OPS", 40)).expect("步数");
    let budget = env_u64("ATTACK_BUDGET_SECONDS", 200);
    let verbose = env_u64("ATTACK_VERBOSE", 0) == 1;
    let invariants: Vec<String> = std::env::var("ATTACK_INVARIANTS")
        .unwrap_or_else(|_| "I-3.9".to_string())
        .split(',')
        .map(str::to_string)
        .collect();
    let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + ATTACK_SEED_OFFSET + index;
    let full = generate_history_with_weights(HistorySeed(seed), ops, attack_weights());
    let mut keep: Vec<usize> = match std::env::var("ATTACK_KEEP") {
        Ok(text) if !text.is_empty() => text.split(',').map(|v| v.parse().expect("下标")).collect(),
        _ => (0..full.operations.len()).collect(),
    };
    let history_of = |keep: &[usize]| GeneratedHistory {
        seed: full.seed,
        starting_point: full.starting_point,
        operations: keep.iter().map(|i| full.operations[*i]).collect(),
    };
    eprintln!("START starting_point={:?}", full.starting_point);
    if verbose {
        let h = history_of(&keep);
        eprintln!("OPS {:?}", h.operations);
        let result = hit(&h, &invariants, true);
        eprintln!("HIT {result}");
        return;
    }
    assert!(hit(&history_of(&keep), &invariants, false), "起点就没打中");
    let started = Instant::now();
    let mut chunk = (keep.len() / 2).max(1);
    'outer: loop {
        let mut position = 0;
        let mut removed_any = false;
        while position < keep.len() {
            if started.elapsed().as_secs() > budget {
                eprintln!("BUDGET_SPENT");
                break 'outer;
            }
            let end = (position + chunk).min(keep.len());
            let candidate: Vec<usize> = keep[..position].iter().chain(keep[end..].iter()).copied().collect();
            if !candidate.is_empty() && hit(&history_of(&candidate), &invariants, false) {
                keep = candidate;
                removed_any = true;
                eprintln!("KEEP {}", keep.iter().map(ToString::to_string).collect::<Vec<_>>().join(","));
            } else {
                position = end;
            }
        }
        if !removed_any {
            if chunk == 1 {
                eprintln!("MINIMAL");
                break;
            }
            chunk = (chunk / 2).max(1);
        }
    }
    eprintln!("FINAL_KEEP {}", keep.iter().map(ToString::to_string).collect::<Vec<_>>().join(","));
    eprintln!("FINAL_OPS {:?}", history_of(&keep).operations);
}
