//! m2-final-code-r4 云端攻方 Z19：空间准入接进发布与可写挂载之后，在小盘上跑随机历史（准入判着，产品路径），
//! 数每一种拒绝成员；找「准入放行、取号之后被落点拒」（`MountError::Publish(..)`、抬 F 那一串已落盘几次之后被拒）与池级 checker / 模型的新发现。
//! 种子基与库里那几条用例不同（库里用 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`），这里取 `R4_OPUS_Z19_SEED_BASE`（默认 7_040_000_000）。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    run_history_campaign, FindingShrinking, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, PerStepChecker,
};

fn threads() -> usize {
    std::env::var("SINGLEFS_THREAD_CAP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4)
}

#[test]
fn z19_small_pool_campaigns_with_the_space_admission_judged() {
    let base: u64 = std::env::var("R4_OPUS_Z19_SEED_BASE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7_040_000_000);
    let seeds: u64 = std::env::var("R4_OPUS_Z19_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(48);
    let widths = [
        ("240", HistoryDeviceWidth::UnitAreaOf240Slots),
        ("256", HistoryDeviceWidth::UnitAreaOf256Slots),
        ("384", HistoryDeviceWidth::UnitAreaOf384Slots),
    ];
    let weights = [
        ("broad", GenerationWeights::BROAD),
        ("unit_area_wall", GenerationWeights::TOWARD_THE_UNIT_AREA_WALL),
        ("reuse_after_raise", GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR),
        ("rollback_after_raise", GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR),
    ];
    let mut cell = 0u64;
    for (width_name, width) in widths {
        for (weights_name, weights) in &weights {
            let first_seed = base + cell * 1_000;
            cell += 1;
            let report = run_history_campaign(
                first_seed,
                seeds,
                120,
                weights,
                HistoryExecution {
                    per_step_checker: PerStepChecker::Run,
                    device_width: width,
                    space_admission: SpaceAdmission::JudgedByTheFormula,
                },
                threads(),
                FindingShrinking::ReportSeedsOnly,
            );
            println!(
                "Z19 width={width_name} weights={weights_name} first_seed={first_seed} seeds={seeds} histories={} completed={} new_findings={} known_red={}",
                report.tally.histories,
                report.tally.histories_completed,
                report.new_findings.len(),
                report.known_red_hits.len()
            );
            for (member, count) in &report.tally.refusals_by_member {
                println!("Z19   refusal width={width_name} weights={weights_name} {member} = {count}");
            }
            for (kind, tally) in &report.tally.operations_by_kind {
                println!(
                    "Z19   operation width={width_name} weights={weights_name} {kind:?} applied={} refused={} not_applicable={}",
                    tally.applied, tally.refused, tally.not_applicable
                );
            }
            if !report.new_findings.is_empty() || !report.known_red_hits.is_empty() {
                println!("Z19   RENDER width={width_name} weights={weights_name}\n{}", report.render());
            }
        }
    }
}

use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryOperation,
    HistorySeed, HistoryStartingPoint, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::SharedStream;

fn member(outcome: &StepOutcome) -> String {
    match outcome {
        StepOutcome::Applied(_) => "Applied".to_string(),
        StepOutcome::Refused { member } => member.clone(),
        StepOutcome::NotApplicable(missing) => format!("NotApplicable({missing:?})"),
    }
}

fn overwrite(selector: u64, fill_seed: u64) -> HistoryOperation {
    HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector },
        fill_seed,
    })
}

/// Z19-B：准入判着（产品路径）。第一个文件之后可写挂载，覆盖写 k 次（k 由用户定，放开扫 0..=K），关掉再可写挂载一次。
/// 先找「这一次会话里覆盖写一直被准入放行到第几次」，再对每个 k 看下一次可写挂载做不做得成；
/// 做不成的那几格，接着试：再挂一次、回退到根环里每一条根、冷启动恢复——看池还回不回得到可写。每一步之后跑池级 checker（历史合法）。
#[test]
fn z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount() {
    let widths = [
        ("240", HistoryDeviceWidth::UnitAreaOf240Slots),
        ("256", HistoryDeviceWidth::UnitAreaOf256Slots),
        ("384", HistoryDeviceWidth::UnitAreaOf384Slots),
    ];
    for (width_name, width) in widths {
        for selector in [2999u64, 99] {
            // 先量：一次会话里连着覆盖写，准入放行到第几次。
            let probe = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
                    .chain((0..80u64).map(|i| overwrite(selector, i + 1)))
                    .collect(),
            };
            let run = execute_history_with(
                &probe,
                HistoryExecution {
                    per_step_checker: PerStepChecker::Skipped,
                    device_width: width,
                    space_admission: SpaceAdmission::JudgedByTheFormula,
                },
                &SharedStream::new(),
                &mut |_| {},
            );
            let members: Vec<String> = run.outcomes.iter().map(member).collect();
            let admitted = members.iter().skip(1).take_while(|m| *m == "Applied").count();
            println!(
                "Z19-B width={width_name} selector={selector} first_mount={} overwrites_admitted_in_one_session={admitted} next_step={:?}",
                members[0],
                members.get(1 + admitted)
            );
            for k in 0..=admitted {
                let mut operations = vec![HistoryOperation::CloseAndMountWritable];
                operations.extend((0..k as u64).map(|i| overwrite(selector, i + 1)));
                operations.push(HistoryOperation::CloseAndMountWritable);
                operations.push(HistoryOperation::CloseAndMountWritable);
                for index in 0..24u64 {
                    operations.push(HistoryOperation::CloseAndMountRollback(
                        RollbackTargetChoice::RingRoot { index_from_newest: index },
                    ));
                }
                operations.push(HistoryOperation::ColdStartRecover);
                operations.push(HistoryOperation::CloseAndMountWritable);
                let history = GeneratedHistory {
                    seed: HistorySeed(0),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations,
                };
                // R4_OPUS_Z19_SKIP_AFTER_THE_PROBE=1：量「放行到第几次」照判，扫 k 的那几段历史关掉准入（改前的行为：挂载与发布都不判准入），
                // 看同一批盘面上下一次挂载在改前做不做得成。
                let admission_of_the_sweep = if std::env::var("R4_OPUS_Z19_SKIP_AFTER_THE_PROBE").as_deref() == Ok("1") {
                    SpaceAdmission::SkippedByTheTestOnlySwitch
                } else {
                    SpaceAdmission::JudgedByTheFormula
                };
                let run = execute_history_with(
                    &history,
                    HistoryExecution {
                        per_step_checker: PerStepChecker::Run,
                        device_width: width,
                        space_admission: admission_of_the_sweep,
                    },
                    &SharedStream::new(),
                    &mut |_| {},
                );
                let members: Vec<String> = run.outcomes.iter().map(member).collect();
                let session_ok = members.iter().skip(1).take(k).all(|m| m == "Applied");
                let next_mount = members.get(1 + k).cloned().unwrap_or_default();
                let again = members.get(2 + k).cloned().unwrap_or_default();
                let rollbacks = &members[(3 + k).min(members.len())..(27 + k).min(members.len())];
                let rollbacks_ok = rollbacks.iter().filter(|m| *m == "Applied").count();
                let mut rollback_members: std::collections::BTreeMap<&str, usize> = Default::default();
                for m in rollbacks {
                    *rollback_members.entry(m.as_str()).or_default() += 1;
                }
                println!(
                    "Z19-B width={width_name} selector={selector} k={k} session_all_admitted={session_ok} next_writable_mount={next_mount} again={again} rollbacks_applied={rollbacks_ok}/24 rollback_members={rollback_members:?} tail={:?} ending={:?} checker_runs={}",
                    &members[(27 + k).min(members.len())..],
                    run.ending,
                    run.tally.checker_runs
                );
            }
        }
    }
}

/// Z19-C：Z19-B 里「下一次可写挂载被准入拒」那几格的逐步结局：覆盖写 k 次之后，关掉重挂 3 次；再逐个回退到根环里第 0..23 新的根
/// （第 0 新就是最新那条：回退到它不丢一版数据）；每一步打出成员与这一步之后镜像上最新根的 (实例, txg)。
#[test]
fn z19_c_step_by_step_after_the_last_admitted_overwrite() {
    let cells = [
        ("240", HistoryDeviceWidth::UnitAreaOf240Slots, 4usize),
        ("240", HistoryDeviceWidth::UnitAreaOf240Slots, 5),
        ("256", HistoryDeviceWidth::UnitAreaOf256Slots, 6),
        ("384", HistoryDeviceWidth::UnitAreaOf384Slots, 0),
    ];
    for (width_name, width, k) in cells {
        let mut operations = vec![HistoryOperation::CloseAndMountWritable];
        operations.extend((0..k as u64).map(|i| overwrite(2999, i + 1)));
        operations.extend(std::iter::repeat_n(HistoryOperation::CloseAndMountWritable, 3));
        for index in 0..24u64 {
            operations.push(HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
                index_from_newest: index,
            }));
        }
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: operations.clone(),
        };
        let mut newest_after_each_step: Vec<String> = Vec::new();
        let run = execute_history_with(
            &history,
            HistoryExecution {
                per_step_checker: PerStepChecker::Run,
                device_width: width,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
            &SharedStream::new(),
            &mut |observation| {
                let image: &singlefs_harness::crash::MemoryPool = observation.image;
                let newest = singlefs_core::recovery::choose_system_configuration(image)
                    .ok()
                    .and_then(|sc| singlefs_core::recovery::choose_root(image, &sc))
                    .map(|root| format!("({},{})", root.instance.0, root.checkpoint_txg.0))
                    .unwrap_or_default();
                newest_after_each_step.push(newest);
            },
        );
        for (step, outcome) in run.outcomes.iter().enumerate() {
            println!(
                "Z19-C width={width_name} k={k} step={step} op={:?} -> {} newest_root_after={}",
                operations[step],
                member(outcome),
                newest_after_each_step.get(step + 1).cloned().unwrap_or_default()
            );
        }
        println!("Z19-C width={width_name} k={k} ending={:?} checker_runs={}", run.ending, run.tally.checker_runs);
    }
}
