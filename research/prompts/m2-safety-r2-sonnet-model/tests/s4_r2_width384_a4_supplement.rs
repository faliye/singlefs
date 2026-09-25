//! S4 二轮（`research/prompts/m2-safety-r2-body.md` 第 37-42 行）云端正推腿的补测：`s4_r2_candidates.rs` 的
//! `s4_r2_b_admitted_session_then_reopen_across_candidates` 在 width=384 上因 A3 的 k 循环（0..=80）耗时超出
//! 40 分钟预算被本腿主动停止（k 跑到 66，见 `m2-safety-r2-sonnet-output.md` 第三节「跑前登记」一段），
//! width=384 的 A4 数据没跑到。这份补测只跑 width=384、candidate=A4 这一格，独立文件、不改原测试。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

use singlefs_core::admission::S4Candidate;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    StepOutcome,
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

#[test]
fn s4_r2_b_width384_a4_supplement() {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let candidate = S4Candidate::A4;
    let selector = 2999u64;
    let probe = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain((0..80u64).map(|i| overwrite(selector, i + 1)))
            .collect(),
    };
    let run = execute_history_with(&probe, execution(width, candidate), &SharedStream::new(), &mut |_| {});
    let members: Vec<String> = run.outcomes.iter().map(member).collect();
    let admitted = members.iter().skip(1).take_while(|m| *m == "Applied").count();
    println!(
        "S4R2-B width=384 candidate=a4 first_mount={} overwrites_admitted_in_one_session={admitted} next_step={:?}",
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
        let run = execute_history_with(&history, execution(width, candidate), &SharedStream::new(), &mut |_| {});
        let members: Vec<String> = run.outcomes.iter().map(member).collect();
        let session_ok = members.iter().skip(1).take(k).all(|m| m == "Applied");
        let next_mount = members.get(1 + k).cloned().unwrap_or_default();
        let again = members.get(2 + k).cloned().unwrap_or_default();
        println!(
            "S4R2-B width=384 candidate=a4 k={k} session_all_admitted={session_ok} next_writable_mount={next_mount} again={again} ending={:?} checker_runs={}",
            run.ending,
            run.tally.checker_runs
        );
    }
}
