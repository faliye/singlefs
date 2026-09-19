//! 攻方腿（Opus）m2-supp3-item1-code-r1 · Z2 的机理核对：只放进仓副本的 `crates/singlefs-harness/tests/`。
//! 对每段以「已知红第 1 条」收尾的历史，取抬 F 那一步之前最后一份镜像（观察者交出的），按实现的读法（`singlefs_core::recovery`）读根环与最新根的实例表，
//! 问登记的机理成不成立：新 F 那个 txg 上的根是被抛弃的（F 落在回退留下的空档里），还是有效的（不在空档里——那就不是第 43 行那一类）。
//! 每段一行 `GAP seed … F … root_at_F …`，末尾 `GAP-SUMMARY`。环境变量同 opus_probe：OPUS_FIRST、OPUS_SEEDS、OPUS_OPS、OPUS_THREADS。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use singlefs_core::recovery::{choose_root, choose_superblock, instance_table_of_root, readable_roots};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::history::{
    execute_history_observing, generate_history, AppliedEffect, HistoryEnding, HistoryOperation,
    HistorySeed, StepOutcome, StepPosition,
};
use singlefs_harness::SharedStream;

fn number(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |text| text.parse().expect("非负整数"))
}

/// 这份镜像上 txg = `floor` 的根：("valid" / "abandoned" / "none")，按最新根指着的实例表判被抛弃（行 (i, T) 且 txg > T）。
fn root_at(image: &MemoryPool, floor: u64) -> String {
    let Ok(superblock) = choose_superblock(image) else {
        return "no-superblock".to_string();
    };
    let roots = readable_roots(
        image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let Some(newest) = choose_root(image, &superblock) else {
        return "no-root".to_string();
    };
    let Some(table) = instance_table_of_root(image, &newest) else {
        return "no-table".to_string();
    };
    let mut kinds: Vec<String> = roots
        .iter()
        .filter(|root| root.checkpoint_txg.0 == floor)
        .map(|root| {
            let abandoned = table.rows.iter().any(|row| {
                row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg
            });
            format!(
                "{}(instance {})",
                if abandoned { "abandoned" } else { "valid" },
                root.instance.0
            )
        })
        .collect();
    kinds.sort();
    kinds.dedup();
    if kinds.is_empty() {
        "none".to_string()
    } else {
        kinds.join("+")
    }
}

#[test]
#[ignore = "探针：只在副本上按环境变量跑"]
fn opus_gap_probe_known_red_second_form_mechanism() {
    let first = number("OPUS_FIRST", 0);
    let seeds = number("OPUS_SEEDS", 96);
    let operations = usize::try_from(number("OPUS_OPS", 30)).expect("步数");
    let threads = number("OPUS_THREADS", 16);
    let next = AtomicU64::new(0);
    let lines: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let offset = next.fetch_add(1, Ordering::Relaxed);
                if offset >= seeds {
                    break;
                }
                let seed = first + offset;
                let history = generate_history(HistorySeed(seed), operations);
                let mut last_image: Option<MemoryPool> = None;
                let run = execute_history_observing(&history, &SharedStream::new(), &mut |observation| {
                    last_image = Some(observation.image.clone());
                });
                let HistoryEnding::KnownRed { form: 1, observation } = &run.ending else {
                    continue;
                };
                let StepPosition::Operation(step) = observation.position else {
                    continue;
                };
                let floor = match run.outcomes.get(step) {
                    Some(StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. })) => new_floor.0,
                    other => {
                        lines.lock().expect("锁").push((seed, format!("GAP seed {seed} step {step} outcome {other:?}")));
                        continue;
                    }
                };
                let rollbacks_ok = run
                    .outcomes
                    .iter()
                    .zip(&history.operations)
                    .filter(|(outcome, operation)| {
                        matches!(operation, HistoryOperation::CloseAndMountRollback(_))
                            && matches!(outcome, StepOutcome::Applied(_))
                    })
                    .count();
                let verdict = last_image.as_ref().map_or("no-image".to_string(), |image| root_at(image, floor));
                lines.lock().expect("锁").push((
                    seed,
                    format!(
                        "GAP seed {seed} step {step} F {floor} rollbacks_ok {rollbacks_ok} root_at_F {verdict} violations {:?}",
                        observation.violations
                    ),
                ));
            });
        }
    });
    let mut lines = lines.into_inner().expect("锁");
    lines.sort_by_key(|(seed, _)| *seed);
    let (mut in_gap, mut not_in_gap) = (0, 0);
    for (_, line) in &lines {
        println!("{line}");
        if line.contains("root_at_F abandoned") && !line.contains("valid") {
            in_gap += 1;
        } else {
            not_in_gap += 1;
        }
    }
    println!(
        "GAP-SUMMARY first {first} seeds {seeds} ops {operations}: form1 {} in_gap {in_gap} not_in_gap {not_in_gap}",
        lines.len()
    );
}
