//! S4 二轮（`research/prompts/m2-safety-r2-body.md` 第 37-42 行）云端正推腿：A1–A4 与 Baseline 在同一段历史上的对拍。
//! 复用 Z19-B 的历史形状（`research/prompts/m2-final-code-r4-opus-model/tests/r4_opus_z19.rs`
//! 的 `z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount`）：可写挂载之后连续覆盖写到准入放行的
//! 上限，关掉再挂载（`CloseAndMountWritable` 是真正的 `establish_instance` 重跑，不是复用同一个进程的分配器）。
//! selector 只取 2999（selector=99 在 Baseline 上产出与 2999 逐字相同，见 `m2-safety-r2-sonnet-output.md` 附录）。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

use singlefs_core::admission::S4Candidate;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    RollbackTargetChoice, StepOutcome,
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

fn execution(width: HistoryDeviceWidth, candidate: S4Candidate) -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Run,
        device_width: width,
        space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
        s4_candidate: candidate,
    }
}

const CANDIDATES: [S4Candidate; 5] = [
    S4Candidate::Baseline,
    S4Candidate::A1,
    S4Candidate::A2,
    S4Candidate::A3,
    S4Candidate::A4,
];

fn candidate_name(candidate: S4Candidate) -> &'static str {
    match candidate {
        S4Candidate::Baseline => "baseline",
        S4Candidate::A1 => "a1",
        S4Candidate::A2 => "a2",
        S4Candidate::A3 => "a3",
        S4Candidate::A4 => "a4",
    }
}

/// S4-R2-B：Z19-B 的历史形状，五个候选各跑一遍，逐格打印。这一条只看「一次会话覆盖写到放行上限，
/// 关掉再挂载」这一步能不能过——不看回退与冷启动那一段（那一段在 `s4_r2_c_cold_start_and_rollback_recovery` 里）。
#[test]
fn s4_r2_b_admitted_session_then_reopen_across_candidates() {
    let widths = [
        ("240", HistoryDeviceWidth::UnitAreaOf240Slots),
        ("256", HistoryDeviceWidth::UnitAreaOf256Slots),
        ("384", HistoryDeviceWidth::UnitAreaOf384Slots),
    ];
    let selector = 2999u64;
    for (width_name, width) in widths {
        for candidate in CANDIDATES {
            let probe = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
                    .chain((0..80u64).map(|i| overwrite(selector, i + 1)))
                    .collect(),
            };
            let run = execute_history_with(
                &probe,
                execution(width, candidate),
                &SharedStream::new(),
                &mut |_| {},
            );
            let members: Vec<String> = run.outcomes.iter().map(member).collect();
            let admitted = members.iter().skip(1).take_while(|m| *m == "Applied").count();
            println!(
                "S4R2-B width={width_name} candidate={} first_mount={} overwrites_admitted_in_one_session={admitted} next_step={:?}",
                candidate_name(candidate),
                members[0],
                members.get(1 + admitted)
            );
            for k in 0..=admitted {
                let mut operations = vec![HistoryOperation::CloseAndMountWritable];
                operations.extend((0..k as u64).map(|i| overwrite(selector, i + 1)));
                operations.push(HistoryOperation::CloseAndMountWritable);
                operations.push(HistoryOperation::CloseAndMountWritable);
                let history = GeneratedHistory {
                    seed: HistorySeed(0),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations,
                };
                let run = execute_history_with(
                    &history,
                    execution(width, candidate),
                    &SharedStream::new(),
                    &mut |_| {},
                );
                let members: Vec<String> = run.outcomes.iter().map(member).collect();
                let session_ok = members.iter().skip(1).take(k).all(|m| m == "Applied");
                let next_mount = members.get(1 + k).cloned().unwrap_or_default();
                let again = members.get(2 + k).cloned().unwrap_or_default();
                println!(
                    "S4R2-B width={width_name} candidate={} k={k} session_all_admitted={session_ok} next_writable_mount={next_mount} again={again} ending={:?} checker_runs={}",
                    candidate_name(candidate),
                    run.ending,
                    run.tally.checker_runs
                );
            }
        }
    }
}

/// S4-R2-D3I9：D3（空间分配） 已定项 9 第 2 条的字面——删掉一个 s 字节的对象之后，同样大小的写必须在有界步数（3 次改变
/// 用户可见状态的发布）内成功。用「先填满、再删一个数据单元、立刻覆盖写同样大小」的历史，逐候选数「立刻覆盖写」是不是当场成功；
/// 不成功就数还要再发生几次改变用户可见状态的发布（用空发布抬 F 不算这 3 次）才成功——这里先看当场（0 次界内）。
#[test]
fn s4_r2_d3_item9_delete_then_write_same_size_across_candidates() {
    let widths = [
        ("240", HistoryDeviceWidth::UnitAreaOf240Slots),
        ("256", HistoryDeviceWidth::UnitAreaOf256Slots),
    ];
    let selector = 2999u64;
    for (width_name, width) in widths {
        for candidate in CANDIDATES {
            // 先填到准入放行的上限（同 S4-R2-B 的探针），再做「删一个、写一个同样大小」。
            let probe = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
                    .chain((0..80u64).map(|i| overwrite(selector, i + 1)))
                    .collect(),
            };
            let run = execute_history_with(
                &probe,
                execution(width, candidate),
                &SharedStream::new(),
                &mut |_| {},
            );
            let members: Vec<String> = run.outcomes.iter().map(member).collect();
            let admitted = members.iter().skip(1).take_while(|m| *m == "Applied").count();
            // 填到放行上限（k 次覆盖写，覆盖写不改变占用——同一个数据单元反复写），此刻已经是「小盘上准入把它卡住」那一格；
            // 再一次覆盖写就是「删掉这次自己换下的旧内容、写等大的新内容」——D3 已定项 9 第 2 条问的正是这一步当场成不成功。
            let mut operations = vec![HistoryOperation::CloseAndMountWritable];
            operations.extend((0..admitted as u64).map(|i| overwrite(selector, i + 1)));
            operations.push(overwrite(selector, admitted as u64 + 1));
            let history = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations,
            };
            let run = execute_history_with(
                &history,
                execution(width, candidate),
                &SharedStream::new(),
                &mut |_| {},
            );
            let members: Vec<String> = run.outcomes.iter().map(member).collect();
            let one_more_overwrite_after_the_limit = members.last().cloned().unwrap_or_default();
            println!(
                "S4R2-D3I9 width={width_name} candidate={} admitted_before_limit={admitted} one_more_overwrite_of_the_same_size={one_more_overwrite_after_the_limit}",
                candidate_name(candidate)
            );
        }
    }
}
