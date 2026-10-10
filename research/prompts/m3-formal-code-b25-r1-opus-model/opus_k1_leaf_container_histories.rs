//! 攻方腿 m3-formal-code-b25-r1-opus 的 K1 原型（只在攻方副本里编、跑；不入库装置）。
//! 从 `K1_HISTORIES` 指的文件逐行读历史（`名字<TAB>操作串`），每段从起点 `AfterFirstFile` 跑一遍，
//! 每段打一行：名字、结局、各步结局计数、模型计数。不断言，判读在报告里做。
//! 操作串：U=unlink，C 或 C*n=建 inode，Rn=回退到根环第 n 新，Bn=回退到环外，L=回退到 F 那条，
//! M=关会话再可写挂载，K=冷启动恢复，X=崩溃恢复抛弃最新根，Fn=抬 F，W=零单元发布。
//! On=覆盖写一个数据单元以内的内容（选择子与填充种子都取 n）。
//! 设了 K1_CHECKER 时每一步跑池级 checker（`PerStepChecker::Run`），不设时不跑。

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::SharedStream;

fn parse(ops: &str) -> Vec<HistoryOperation> {
    let mut out = Vec::new();
    for token in ops.split_whitespace() {
        let (head, rest) = token.split_at(1);
        let (arg, times) = match rest.split_once('*') {
            Some((a, n)) => (a, n.parse::<usize>().expect("次数")),
            None => (rest, 1),
        };
        let num = || arg.parse::<u64>().unwrap_or(0);
        let op = match head {
            "U" => HistoryOperation::Unlink,
            "C" => HistoryOperation::CreateInode,
            "R" => HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: num(),
            }),
            "B" => HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot {
                txg_beyond_newest: num(),
            }),
            "L" => HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRootAtTheNewestFloor),
            "M" => HistoryOperation::CloseAndMountWritable,
            "K" => HistoryOperation::ColdStartRecover,
            "X" => HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            "F" => HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: num(),
            }),
            "W" => HistoryOperation::PublishWithoutUnits,
            "O" => HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: num() },
                fill_seed: num(),
            }),
            other => panic!("认不出的操作 {other}"),
        };
        for _ in 0..times {
            out.push(op.clone());
        }
    }
    out
}

fn short(outcome: &StepOutcome) -> String {
    match outcome {
        StepOutcome::Applied(_) => "A".to_string(),
        StepOutcome::Refused { member } => format!("R({member})"),
        StepOutcome::NotApplicable(m) => format!("N({m:?})"),
    }
}

#[test]
fn opus_k1_histories() {
    let path = std::env::var("K1_HISTORIES").expect("K1_HISTORIES 指历史清单");
    let text = std::fs::read_to_string(&path).expect("读清单");
    for line in text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')) {
        let (name, ops) = line.split_once('\t').expect("名字<TAB>操作串");
        let operations = parse(ops);
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations,
        };
        let started = std::time::Instant::now();
        let run = execute_history_with(
            &history,
            HistoryExecution {
                per_step_checker: if std::env::var("K1_CHECKER").is_ok() {
                    PerStepChecker::Run
                } else {
                    PerStepChecker::Skipped
                },
                device_width: HistoryDeviceWidth::FourGibibytes,
                space_admission: SpaceAdmission::JudgedByTheFormula,
            },
            &SharedStream::new(),
            &mut |_| {},
        );
        let applied = run.outcomes.iter().filter(|o| matches!(o, StepOutcome::Applied(_))).count();
        let tail: Vec<String> = run.outcomes.iter().rev().take(4).rev().map(short).collect();
        let ending = match &run.ending {
            HistoryEnding::Completed => "Completed".to_string(),
            HistoryEnding::KnownRed { form, observation } => {
                format!("KnownRed form={form} at={:?} panic={:?} violations={:?}", observation.position, observation.panic, observation.violations)
            }
            HistoryEnding::NewFinding { signature, observation } => format!(
                "NewFinding sig={signature:?} at={:?} panic={:?} violations={:?}",
                observation.position, observation.panic, observation.violations
            ),
        };
        let c = &run.tally.model_counts;
        println!(
            "K1ROW name={name} steps={} applied={applied} tail={tail:?} matched={} alloc_compared={} roots={} ms={} ending={ending}",
            run.outcomes.len(),
            c.successes_matched,
            c.allocation_records_compared,
            c.roots_compared,
            started.elapsed().as_millis()
        );
    }
}
