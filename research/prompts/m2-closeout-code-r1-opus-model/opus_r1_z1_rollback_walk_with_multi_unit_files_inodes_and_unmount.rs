//! 代码轮第一轮云端攻方腿（m2-closeout-code-r1）Z1 那一格：随机历史生成器（`history.rs`）不走的三样——多单元文件（顺序写）、
//! 建 inode、正常卸载——与挂着时回退交错。每一步之后：池级 checker（违例数）；冷恢复（`recovery::recover`，看 journal）读回的内容
//! 必须是走到的那条根那一版的内容；回退做成的那一步另核：新根那一版读回的是目标那一版的内容。
//! 种子与步数从环境变量取（`OPUS_Z1_SEEDS`、`OPUS_Z1_STEPS`、`OPUS_Z1_FIRST_SEED`、`OPUS_Z1_WIDTH`：4g / 384 / 256）。
mod common;
mod common_admission;

use std::collections::BTreeMap;

use common_admission::{checker_violations_on, content_of, start_plain, PoolUnderTest};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{
    raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, unmount,
    RollbackError, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::recovery::{choose_system_configuration, readable_roots, recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::history::HistoryDeviceWidth;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|text| text.parse().ok()).unwrap_or(default)
}

fn width() -> HistoryDeviceWidth {
    match std::env::var("OPUS_Z1_WIDTH").as_deref() {
        Ok("384") => HistoryDeviceWidth::UnitAreaOf384Slots,
        Ok("256") => HistoryDeviceWidth::UnitAreaOf256Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    }
}

#[derive(Default, Debug)]
struct Tally {
    steps: u64,
    outcomes: BTreeMap<String, u64>,
    checker_red_steps: u64,
    first_checker_red: Option<String>,
    read_back_wrong: u64,
    first_read_back_wrong: Option<String>,
    rollbacks_done: u64,
    rollbacks_with_multi_unit_target: u64,
}

fn note(tally: &mut Tally, what: String) {
    *tally.outcomes.entry(what).or_insert(0) += 1;
}

fn short(debug: &str) -> String {
    debug.split(|c: char| c == ' ' || c == '(' || c == '{').next().unwrap_or("").to_string()
}

fn judge_after_step(pool: &PoolUnderTest<SparseBlockDevice>, tally: &mut Tally, seed: u64, step: u64, op: &str) {
    let image = pool.image();
    let violations = checker_violations_on(&image);
    if !violations.is_empty() {
        tally.checker_red_steps += 1;
        if tally.first_checker_red.is_none() {
            tally.first_checker_red = Some(format!("seed={seed} step={step} op={op} {:?}", violations));
        }
    }
    let report = recover(&image, JournalPolicy::Consult);
    let wrong = match (&report.outcome, report.effective_root) {
        (RecoveryOutcome::FileRead { content, .. }, Some(root)) => match pool.content_of_each_root.get(&root) {
            Some(expected) if expected == content => None,
            Some(expected) => Some(format!("root=({},{}) read {} bytes expected {} bytes", root.0 .0, root.1 .0, content.len(), expected.len())),
            None => Some(format!("root=({},{}) not a noted version", root.0 .0, root.1 .0)),
        },
        (other, root) => Some(format!("outcome={other:?} root={root:?}")),
    };
    if let Some(wrong) = wrong {
        tally.read_back_wrong += 1;
        if tally.first_read_back_wrong.is_none() {
            tally.first_read_back_wrong = Some(format!("seed={seed} step={step} op={op} {wrong}"));
        }
    }
}

fn run_one(seed: u64, steps: u64, tally: &mut Tally) {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut pool = start_plain(width());
    let unit_payload = 32_000_usize;
    for step in 0..steps {
        tally.steps += 1;
        let pick = rng.below(100);
        let op: String;
        if pool.session.is_none() {
            op = "mount".into();
            match pool.crash_and_mount_writable() {
                Ok(_) => note(tally, "mount ok".into()),
                Err(error) => {
                    note(tally, format!("mount err {}", short(&format!("{error:?}"))));
                    judge_after_step(&pool, tally, seed, step, &op);
                    return;
                }
            }
        } else if pick < 25 {
            op = "overwrite".into();
            let length = 1 + usize::try_from(rng.below(2999)).expect("小");
            let outcome = pool.overwrite(length);
            note(tally, format!("overwrite {}", outcome.as_ref().map_or_else(|e| short(&format!("{e:?}")), |_| "ok".into())));
        } else if pick < 42 {
            op = "sequential_write".into();
            let units = 1 + usize::try_from(rng.below(5)).expect("小");
            let length = units * unit_payload - usize::try_from(rng.below(3000)).expect("小");
            let outcome = pool.sequential_write(length);
            note(tally, format!("sequential_write {}", outcome.as_ref().map_or_else(|e| short(&format!("{e:?}")), |_| "ok".into())));
        } else if pick < 52 {
            op = "new_inode".into();
            let outcome = pool.create_one_inode();
            note(tally, format!("new_inode {}", outcome.as_ref().map_or_else(|e| short(&format!("{e:?}")), |_| "ok".into())));
        } else if pick < 77 {
            op = "rollback".into();
            let image = pool.image();
            let system_configuration = choose_system_configuration(&image).expect("系统配置");
            let mut roots = readable_roots(&image, &system_configuration.immutable.region_devices, &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier);
            roots.sort_by_key(|root| std::cmp::Reverse((root.checkpoint_txg, root.instance)));
            if roots.is_empty() {
                note(tally, "rollback no-roots".into());
            } else {
                let depth = usize::try_from(rng.below(u64::try_from(roots.len().min(12)).expect("小"))).expect("小");
                let target_root = roots[depth];
                let target = RollbackTarget { instance: target_root.instance, checkpoint_txg: target_root.checkpoint_txg };
                let session = pool.session.as_mut().expect("会话");
                let PoolVersion::WithFile(current) = &mut session.current else { unreachable!("第一个文件之后都带文件") };
                match roll_back_by_a_forward_publish(&pool.parameters, &mut pool.devices, &mut session.allocator, current, target) {
                    Ok(_) => {
                        tally.rollbacks_done += 1;
                        let new_root = current.root;
                        let content = pool.content_of_each_root.get(&(target.instance, target.checkpoint_txg)).cloned();
                        match content {
                            Some(content) => {
                                if content.len() > unit_payload {
                                    tally.rollbacks_with_multi_unit_target += 1;
                                }
                                pool.content_of_the_current_version = content;
                                pool.note_root_of_the_current_content(&new_root);
                            }
                            None => note(tally, "rollback target content unknown".into()),
                        }
                        note(tally, "rollback ok".into());
                    }
                    Err(RollbackError::TargetNotACandidate { exclusion, .. }) => note(tally, format!("rollback not-candidate {exclusion:?}")),
                    Err(error) => note(tally, format!("rollback err {}", short(&format!("{error:?}")))),
                }
            }
        } else if pick < 85 {
            op = "raise_to_ceiling".into();
            let session = pool.session.as_mut().expect("会话");
            let PoolVersion::WithFile(current) = &mut session.current else { unreachable!() };
            match raise_rollback_floor_to_the_admission_ceiling(&pool.parameters, &mut pool.devices, &mut session.allocator, current, ShadowLedger::On) {
                Ok(singlefs_core::mount::RaiseToTheAdmissionCeiling::Raised(raised)) => {
                    pool.note_floor_raises(&[raised]);
                    note(tally, "raise ok".into());
                }
                Ok(_) => note(tally, "raise at-ceiling".into()),
                Err(error) => note(tally, format!("raise err {}", short(&format!("{error:?}")))),
            }
        } else if pick < 93 {
            op = "unmount".into();
            let mut session = pool.session.take().expect("会话");
            match unmount(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut session.current, ShadowLedger::On) {
                Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
                    for publish in &raised.publishes {
                        pool.note_root_of_the_current_content(&publish.root);
                    }
                    note(tally, "unmount ok".into());
                }
                Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => note(tally, "unmount nothing".into()),
                Err(error) => note(tally, format!("unmount err {}", short(&format!("{error:?}")))),
            }
        } else {
            op = "process_exit".into();
            pool.session = None;
            note(tally, "process_exit".into());
        }
        judge_after_step(&pool, tally, seed, step, &op);
    }
    let _ = (content_of, CheckpointTxg(0), InstanceGeneration(0));
}

#[test]
fn rollback_walk_with_multi_unit_files_inodes_and_unmount() {
    let first = env_u64("OPUS_Z1_FIRST_SEED", 1);
    let seeds = env_u64("OPUS_Z1_SEEDS", 4);
    let steps = env_u64("OPUS_Z1_STEPS", 30);
    let started = std::time::Instant::now();
    let mut tally = Tally::default();
    for seed in first..first + seeds {
        run_one(seed, steps, &mut tally);
    }
    println!("Z1W width={:?} seeds={first}..{} steps={steps} elapsed_ms={}", width(), first + seeds, started.elapsed().as_millis());
    println!("Z1W steps_total={} rollbacks_done={} multi_unit_rollbacks={} checker_red_steps={} read_back_wrong={}", tally.steps, tally.rollbacks_done, tally.rollbacks_with_multi_unit_target, tally.checker_red_steps, tally.read_back_wrong);
    println!("Z1W first_checker_red={:?}", tally.first_checker_red);
    println!("Z1W first_read_back_wrong={:?}", tally.first_read_back_wrong);
    for (outcome, count) in &tally.outcomes {
        println!("Z1W outcome {outcome} = {count}");
    }
}
