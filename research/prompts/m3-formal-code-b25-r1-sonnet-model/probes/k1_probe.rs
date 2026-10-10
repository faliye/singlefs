//! K1 探针（正推腿自造，只在草稿副本里）：在新用例之外，把挂载、冷启动恢复插进建 inode 的序列里，看 O1 逐步比得对不对。
//! 每个探针打印：结局、每类操作的结局数、模型比过的计数。不替主 agent 判，只贴原样输出。

use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::SharedStream;

fn run(name: &str, pieces: &[(HistoryOperation, usize)]) {
    let mut operations = Vec::new();
    for (operation, times) in pieces {
        for _ in 0..*times {
            operations.push(operation.clone());
        }
    }
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &SharedStream::new(),
        &mut |_| {},
    );
    println!("PROBE {name} ending={:?}", run.ending);
    println!("PROBE {name} outcomes={}", run.outcomes.len());
    println!("PROBE {name} by_kind={:?}", run.tally.operations_by_kind);
    println!("PROBE {name} model_counts={:?}", run.tally.model_counts);
}

const L: usize = 233;

#[test]
fn probe_remount_before_the_split() {
    run(
        "remount_before_split",
        &[
            (HistoryOperation::Unlink, 1),
            (HistoryOperation::CreateInode, L),
            (HistoryOperation::CloseAndMountWritable, 1),
            (HistoryOperation::CreateInode, 3),
        ],
    );
}

#[test]
fn probe_remount_after_the_split() {
    run(
        "remount_after_split",
        &[
            (HistoryOperation::Unlink, 1),
            (HistoryOperation::CreateInode, L + 2),
            (HistoryOperation::CloseAndMountWritable, 1),
            (HistoryOperation::CreateInode, 3),
        ],
    );
}

#[test]
fn probe_cold_start_after_the_split() {
    run(
        "coldstart_after_split",
        &[
            (HistoryOperation::Unlink, 1),
            (HistoryOperation::CreateInode, L + 2),
            (HistoryOperation::ColdStartRecover, 1),
            (HistoryOperation::CloseAndMountWritable, 1),
            (HistoryOperation::CreateInode, 3),
        ],
    );
}

#[test]
fn probe_two_remounts_across_the_boundary() {
    run(
        "two_remounts_boundary",
        &[
            (HistoryOperation::Unlink, 1),
            (HistoryOperation::CreateInode, L - 1),
            (HistoryOperation::CloseAndMountWritable, 1),
            (HistoryOperation::CreateInode, 2),
            (HistoryOperation::CloseAndMountWritable, 1),
            (HistoryOperation::CreateInode, 1),
        ],
    );
}

#[test]
fn probe_unlink_again_after_the_split() {
    run(
        "unlink_after_split",
        &[
            (HistoryOperation::Unlink, 1),
            (HistoryOperation::CreateInode, L + 1),
            (HistoryOperation::Unlink, 1),
            (HistoryOperation::CreateInode, 1),
        ],
    );
}
