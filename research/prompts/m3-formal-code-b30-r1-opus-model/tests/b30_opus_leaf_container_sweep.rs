//! b30 攻方腿 M1 探针：inode 树叶容器取样点放开扫（第 5 片、回退目标与回退后建几个 inode 由用户定的那几步放开）。
//! 每条历史从 `AfterFirstFile` 起：unlink 第一个文件，建 `base` 个 inode，可选回退到根环里第 `index` 新那条根、再建 `after` 个。
//! 断言：历史跑完（模型与实现每一步对得上）、回退那一步做成（有回退的）。每条一行 `SWEEP` 打出结局，供汇总。

use singlefs_core::admission::SpaceAdmission;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistoryOperation, HistoryRun, HistorySeed, HistoryStartingPoint,
    PerStepChecker, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::SharedStream;

fn run_one(name: &str, base: u64, rollback: Option<(u64, u64)>) {
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(std::iter::repeat_n(
        HistoryOperation::CreateInode,
        usize::try_from(base).expect("usize"),
    ));
    let mut rollback_step = None;
    if let Some((index, after)) = rollback {
        rollback_step = Some(operations.len());
        operations.push(HistoryOperation::RollBackWhileMounted(
            RollbackTargetChoice::RingRoot {
                index_from_newest: index,
            },
        ));
        operations.extend(std::iter::repeat_n(
            HistoryOperation::CreateInode,
            usize::try_from(after).expect("usize"),
        ));
    }
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let run: HistoryRun = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    let rolled_back = rollback_step.map(|step| {
        run.outcomes.get(step).is_some_and(|outcome| {
            matches!(
                outcome,
                StepOutcome::Applied(AppliedEffect::RolledBack { .. })
            )
        })
    });
    let completed = run.ending == HistoryEnding::Completed;
    let short = format!("{:?}", run.ending).chars().take(160).collect::<String>();
    println!(
        "SWEEP {name} base={base} rollback={rollback:?} steps={} completed={completed} rolled_back={rolled_back:?} ending={short}",
        run.outcomes.len()
    );
    assert!(completed, "{name}: 历史没跑完：{short}");
    if let Some(rolled_back) = rolled_back {
        assert!(rolled_back, "{name}: 回退没做成");
    }
}

const L: u64 = INODE_LEAF_RECORDS;

macro_rules! sweep {
    ($($name:ident: $base:expr, $rollback:expr;)*) => {
        $(#[test] fn $name() { run_one(stringify!($name), $base, $rollback); })*
    };
}

sweep! {
    // A：建到第 5 片（4 × 233 + 4）
    a_build_into_the_fifth: 4 * L + 4, None;
    // B：第三片被重写过的回退（2 × 233 + 5），回退目标与回退后建几个放开
    b_idx1_k0: 2 * L + 5, Some((1, 0));
    b_idx1_k3: 2 * L + 5, Some((1, 3));
    b_idx2_k0: 2 * L + 5, Some((2, 0));
    b_idx2_k1: 2 * L + 5, Some((2, 1));
    b_idx2_k8: 2 * L + 5, Some((2, 8));
    b_idx3_k0: 2 * L + 5, Some((3, 0));
    b_idx3_k3: 2 * L + 5, Some((3, 3));
    // C：没有第四片的那一版（3 × 233 + 2）
    c_idx1_k3: 3 * L + 2, Some((1, 3));
    c_idx2_k0: 3 * L + 2, Some((2, 0));
    c_idx2_k1: 3 * L + 2, Some((2, 1));
    c_idx2_k8: 3 * L + 2, Some((2, 8));
    c_idx3_k0: 3 * L + 2, Some((3, 0));
    c_idx3_k3: 3 * L + 2, Some((3, 3));
    // D：第四片被重写过的回退（3 × 233 + 5）
    d_idx2_k0: 3 * L + 5, Some((2, 0));
    d_idx2_k3: 3 * L + 5, Some((2, 3));
    d_idx3_k3: 3 * L + 5, Some((3, 3));
    // E：没有第五片的那一版（4 × 233 + 2）
    e_idx2_k0: 4 * L + 2, Some((2, 0));
    e_idx2_k3: 4 * L + 2, Some((2, 3));
    e_idx3_k3: 4 * L + 2, Some((3, 3));
}
