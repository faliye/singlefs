//! 回锁复核用不含「现行那一版没换」的判据（只比活意图：选段签发那一支锁外前后都是「没有活意图」，照样相等）时，锁外那一批建在第一段克隆的
//! 那一版上发出去。这里不拆核心层，照拆开之后第三段会做的事摆：取输入那一刻克隆会话的现行那一版（今天 `advance_compaction_by_one_publish_computing_on`
//! 进门就是克隆 `current`），前台发 k 次改动（每次部分写一个字节、各自发成、各自答了成功），再把会话的现行那一版换回克隆的那一版，走显式整理入口
//! （`MountedSession::run_compaction`：与后台入口推一批是同一个 `advance_compaction_by_one_publish`，`previous` 取的是 `current`）。
//! 三臂加一条判法自证臂 planted_rollback（由用户决定的几步放开扫：前台发几次 k、整理之后再发几次 m、前台写哪个单元）：
//!   stale_base_live_intent_check：现行那一版换回克隆的（弱判据放行）；
//!   current_base：在现行那一版上整理（锁外的结果作废、回锁重算，或回锁之后照现行那一版重做）；
//!   txg_guard：克隆的那一版与现行那一版 checkpoint_txg 不等就不整理（作废），相等才整理。
//! 每段历史末尾丢会话（等价于断电，写都落了盘），只读挂载读回两个字节：前台 k 次里最后一次写的那个单元、整理之后 m 次写的那个单元。
//! 没取崩溃状态：要看的是全部写都落盘之后的那一态，丢的是答了成功的发布，不靠撕裂写。
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::MountedSession;

use crate::pool::{byte_after_a_read_only_mount, copied, parameters, start_pool, write_one_byte, Devices};

const FOREGROUND_PUBLISHES: [usize; 6] = [0, 1, 2, 3, 5, 8];
const PUBLISHES_AFTER_THE_COMPACTION: [usize; 3] = [0, 1, 3];
const UNITS_WRITTEN_BY_THE_FOREGROUND: [usize; 3] = [1, 2, 39];
const UNIT_WRITTEN_AFTER_THE_COMPACTION: usize = 5;
const ESTIMATE_LIMIT_SECONDS: f64 = 2400.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    StaleBaseLiveIntentCheck,
    CurrentBase,
    TxgGuard,
    /// 判法自证：前台发完之后把盘换回发之前那一份（不整理、之后不再发），前台那几次一定读不回；`lost_foreground` 该是 k > 0 的每一段。
    PlantedRollback,
}

impl Arm {
    const ALL: [Arm; 4] = [Arm::StaleBaseLiveIntentCheck, Arm::CurrentBase, Arm::TxgGuard, Arm::PlantedRollback];
    fn name(self) -> &'static str {
        match self {
            Arm::StaleBaseLiveIntentCheck => "stale_base_live_intent_check",
            Arm::CurrentBase => "current_base",
            Arm::TxgGuard => "txg_guard",
            Arm::PlantedRollback => "planted_rollback",
        }
    }
}

#[derive(Clone, Copy)]
struct History {
    arm: Arm,
    foreground_publishes: usize,
    publishes_after_the_compaction: usize,
    unit: usize,
}

#[derive(Default)]
struct Tally {
    histories: u64,
    lost_foreground: u64,
    lost_after: u64,
    panicked: u64,
    compaction_refused: u64,
    writable_remount_failed: u64,
    compacted_on_a_moved_version: u64,
}

fn short(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join("_").chars().take(100).collect()
}

struct Outcome {
    line: String,
    lost_foreground: bool,
    lost_after: bool,
    compaction_refused: bool,
    writable_remount_failed: bool,
    compacted_on_a_moved_version: bool,
}

fn one_history(start: &Devices, baseline: &[(usize, u8)], history: History) -> Result<Outcome, String> {
    let baseline_of = |unit: usize| {
        baseline.iter().find(|(known, _)| *known == unit).map(|(_, byte)| *byte).expect("起点读过")
    };
    let parameters = parameters();
    let mut devices = copied(start);
    let devices_before_the_foreground = copied(&devices);
    let mounted = mount_writable(&parameters, &mut devices).map_err(|e| short(&format!("mount:{e:?}")))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let snapshot = session.current.clone();
    let snapshot_txg = snapshot.root().checkpoint_txg.0;
    let mut expected_unit = baseline_of(history.unit);
    let mut acknowledged = Vec::new();
    for index in 0..history.foreground_publishes {
        let marker = 0xA0 + u8::try_from(index).expect("小于 256");
        let txg = write_one_byte(&mut session, &mut devices, history.unit, marker)
            .map_err(|refused| format!("foreground_refused:{}", short(&refused)))?;
        expected_unit = marker;
        acknowledged.push(txg);
    }
    let txg_before_the_compaction = session.current.root().checkpoint_txg.0;
    let compacted_on_a_moved_version =
        history.arm == Arm::StaleBaseLiveIntentCheck && snapshot_txg != txg_before_the_compaction;
    let compaction = match history.arm {
        Arm::StaleBaseLiveIntentCheck => {
            session.current = snapshot;
            Some(session.run_compaction(&mut devices))
        }
        Arm::CurrentBase => Some(session.run_compaction(&mut devices)),
        Arm::TxgGuard if snapshot_txg == txg_before_the_compaction => {
            Some(session.run_compaction(&mut devices))
        }
        Arm::TxgGuard | Arm::PlantedRollback => None,
    };
    if history.arm == Arm::PlantedRollback {
        devices = copied(&devices_before_the_foreground);
    }
    let (compaction_text, compaction_refused) = match &compaction {
        None => ("discarded".to_string(), false),
        Some(Ok(run)) => {
            let txgs: Vec<String> = run
                .versions_published
                .iter()
                .map(|version| version.root.checkpoint_txg.0.to_string())
                .collect();
            let outcome = short(&format!("{:?}", run.outcome));
            let outcome_name: String = outcome.chars().take_while(|c| *c != '{').collect();
            (
                format!(
                    "published={} batches={} txgs={} outcome={}",
                    run.versions_published.len(),
                    run.batches_published,
                    if txgs.is_empty() { "-".to_string() } else { txgs.join(",") },
                    outcome_name
                ),
                false,
            )
        }
        Some(Err(refused)) => (format!("refused:{}", short(&format!("{refused:?}"))), true),
    };
    let mut expected_after = baseline_of(UNIT_WRITTEN_AFTER_THE_COMPACTION);
    let mut after_refused = 0_u32;
    let mut after_txgs = Vec::new();
    let publishes_after = if history.arm == Arm::PlantedRollback { 0 } else { history.publishes_after_the_compaction };
    for index in 0..publishes_after {
        let marker = 0xC0 + u8::try_from(index).expect("小于 256");
        match write_one_byte(&mut session, &mut devices, UNIT_WRITTEN_AFTER_THE_COMPACTION, marker) {
            Ok(txg) => {
                expected_after = marker;
                after_txgs.push(txg.to_string());
            }
            Err(_refused) => after_refused += 1,
        }
    }
    drop(session);
    let recovered_unit = byte_after_a_read_only_mount(&devices, history.unit);
    let recovered_after = byte_after_a_read_only_mount(&devices, UNIT_WRITTEN_AFTER_THE_COMPACTION);
    let mut remount_devices = copied(&devices);
    let writable_remount = match mount_writable(&parameters, &mut remount_devices) {
        Ok(_) => "ok".to_string(),
        Err(error) => short(&format!("{error:?}")).chars().take(60).collect(),
    };
    let lost_foreground = history.foreground_publishes > 0
        && recovered_unit.as_ref().map(|(byte, _)| *byte) != Ok(expected_unit);
    let lost_after = !after_txgs.is_empty()
        && recovered_after.as_ref().map(|(byte, _)| *byte) != Ok(expected_after);
    let shown = |read: &Result<(u8, u64), String>| match read {
        Ok((byte, txg)) => format!("0x{byte:02X}@txg{txg}"),
        Err(failure) => short(failure).chars().take(60).collect(),
    };
    let line = format!(
        "HISTORY name=stale_base arm={} k={} m={} unit={} snapshot_txg={} acknowledged_txgs={} txg_before_compaction={} compaction=[{}] after_txgs={} after_refused={} expected_unit=0x{:02X} recovered_unit={} expected_after=0x{:02X} recovered_after={} writable_remount={} lost_foreground={} lost_after={}",
        history.arm.name(), history.foreground_publishes, history.publishes_after_the_compaction, history.unit,
        snapshot_txg,
        if acknowledged.is_empty() { "-".to_string() } else { acknowledged.iter().map(u64::to_string).collect::<Vec<_>>().join(",") },
        txg_before_the_compaction, compaction_text,
        if after_txgs.is_empty() { "-".to_string() } else { after_txgs.join(",") },
        after_refused, expected_unit, shown(&recovered_unit), expected_after, shown(&recovered_after),
        writable_remount, u8::from(lost_foreground), u8::from(lost_after),
    );
    Ok(Outcome {
        line,
        lost_foreground,
        lost_after,
        compaction_refused,
        writable_remount_failed: writable_remount != "ok",
        compacted_on_a_moved_version,
    })
}

pub fn run() {
    let started = Instant::now();
    let start = start_pool();
    let mut baseline = Vec::new();
    for unit in UNITS_WRITTEN_BY_THE_FOREGROUND.into_iter().chain([UNIT_WRITTEN_AFTER_THE_COMPACTION]) {
        let (byte, _txg) = byte_after_a_read_only_mount(&start, unit).expect("起点池读得回");
        baseline.push((unit, byte));
    }
    println!("SETUP name=stale_base start_pool_seconds={:.2} baseline={:?}", started.elapsed().as_secs_f64(), baseline);
    let mut histories = Vec::new();
    for foreground_publishes in FOREGROUND_PUBLISHES {
        for publishes_after_the_compaction in PUBLISHES_AFTER_THE_COMPACTION {
            for unit in UNITS_WRITTEN_BY_THE_FOREGROUND {
                for arm in Arm::ALL {
                    histories.push(History { arm, foreground_publishes, publishes_after_the_compaction, unit });
                }
            }
        }
    }
    // 先跑的那一段取 k 最大、m 最大的那几段（最费时的形），估全量。
    histories.sort_by_key(|history| std::cmp::Reverse((history.foreground_publishes, history.publishes_after_the_compaction)));
    let total = histories.len();
    let mut tallies: Vec<(Arm, Tally)> = Arm::ALL.into_iter().map(|arm| (arm, Tally::default())).collect();
    let mut panicked_lines = 0_u64;
    let loop_started = Instant::now();
    for (index, history) in histories.into_iter().enumerate() {
        let outcome = catch_unwind(AssertUnwindSafe(|| one_history(&start, &baseline, history)));
        let tally = &mut tallies.iter_mut().find(|(arm, _)| *arm == history.arm).expect("每一臂都有一份").1;
        tally.histories += 1;
        match outcome {
            Ok(Ok(outcome)) => {
                println!("{}", outcome.line);
                tally.lost_foreground += u64::from(outcome.lost_foreground);
                tally.lost_after += u64::from(outcome.lost_after);
                tally.compaction_refused += u64::from(outcome.compaction_refused);
                tally.writable_remount_failed += u64::from(outcome.writable_remount_failed);
                tally.compacted_on_a_moved_version += u64::from(outcome.compacted_on_a_moved_version);
            }
            Ok(Err(failure)) => println!(
                "HISTORY name=stale_base arm={} k={} m={} unit={} setup_failed={}",
                history.arm.name(), history.foreground_publishes, history.publishes_after_the_compaction, history.unit, failure
            ),
            Err(payload) => {
                tally.panicked += 1;
                panicked_lines += 1;
                let text = payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
                    .unwrap_or_else(|| "panic 带的不是字符串".to_string());
                println!(
                    "HISTORY name=stale_base arm={} k={} m={} unit={} panicked={}",
                    history.arm.name(), history.foreground_publishes, history.publishes_after_the_compaction, history.unit, short(&text)
                );
            }
        }
        if index == 2 {
            let per_history = loop_started.elapsed().as_secs_f64() / 3.0;
            let estimate = per_history * total as f64;
            println!("ESTIMATE name=stale_base histories={total} seconds_per_history_of_the_first_three={per_history:.2} full_seconds={estimate:.0} limit_seconds={ESTIMATE_LIMIT_SECONDS:.0}");
            if estimate > ESTIMATE_LIMIT_SECONDS {
                println!("ESTIMATE name=stale_base over_the_limit=1：全量不跑，缩 FOREGROUND_PUBLISHES 与 UNITS_WRITTEN_BY_THE_FOREGROUND 的取样再跑");
                std::process::exit(3);
            }
        }
    }
    for (arm, tally) in &tallies {
        println!(
            "RESULT name=stale_base arm={} histories={} compacted_on_a_moved_version={} lost_foreground={} lost_after={} panicked={} compaction_refused={} writable_remount_failed={}",
            arm.name(), tally.histories, tally.compacted_on_a_moved_version, tally.lost_foreground, tally.lost_after,
            tally.panicked, tally.compaction_refused, tally.writable_remount_failed
        );
    }
    println!("WALL_SECONDS name=stale_base seconds={:.1} panicked_lines={panicked_lines}", started.elapsed().as_secs_f64());
}
