//! 攻方腿原型（m4-segment-code-r2，R6）：RefFS 改过的快照允许集合（F6）。
//! 一、直接在 RefFS 上走：恢复读出的内容对不上允许集合里任何一个状态时（抛弃最新根的那次恢复不判读出的内容），快照集合照取 S_0 那一刻的——
//!    S_0 之后建过的快照在盘上还在，RefFS 记成没有，之后读它判红。
//! 二、同一形在历史里走不走得到：起点第一个文件 → 建快照 → 覆盖写 → 崩溃恢复抛弃根 → 读快照，经 RefFS 翻译，打出每一步实现的结局与翻译的判定。
use std::collections::BTreeMap;

use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, SnapshotChoice,
};
use singlefs_harness::model::ObservedReadBack;
use singlefs_harness::reference_filesystem_translation::{
    ReferenceFileSystemCrashMoment, ReferenceFileSystemTranslation,
};
use singlefs_harness::SharedStream;
use singlefs_reference_filesystem::{
    FileContent, Operation, OperationOutcome, ReferenceFileSystem, SnapshotName, VisibleState,
};

fn file(text: &str) -> VisibleState {
    VisibleState::OneFile(FileContent(text.as_bytes().to_vec()))
}

fn succeed(model: &mut ReferenceFileSystem, operation: Operation) {
    model.submit(operation, OperationOutcome::Succeeded).expect("收下");
}

#[test]
fn a_restart_after_an_abandoned_newest_root_forgets_a_snapshot_created_since_the_last_recovery() {
    let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
    succeed(&mut model, Operation::CreateTheFirstFile(FileContent(b"A".to_vec())));
    succeed(&mut model, Operation::ExplicitPublish);
    succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
    succeed(&mut model, Operation::OverwriteTheFile(FileContent(b"B".to_vec())));
    succeed(&mut model, Operation::ExplicitPublish);
    eprintln!("BEFORE allowed_snapshot_sets={:?}", model.allowed_snapshot_sets());
    let crash = model.crash_after_every_operation_returned();
    // 盘上最新那条根（覆盖写 B 那次）被抛弃，恢复落到它前一条（建快照那次）：读出 A，快照 7 在那一版的树表里。
    model
        .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file("A"))
        .expect("只核 hi");
    eprintln!("AFTER allowed_snapshot_sets={:?}", model.allowed_snapshot_sets());
    let read = model.judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(SnapshotName(7), &file("A"));
    eprintln!("READ_SNAPSHOT_7_AS_A {read:?}");
    let expected: Vec<BTreeMap<SnapshotName, VisibleState>> = vec![BTreeMap::from([(SnapshotName(7), file("A"))])];
    eprintln!("CONTAINS_THE_SET_ON_DISK {}", model.allowed_snapshot_sets().iter().any(|set| expected.contains(set)));
}

#[test]
fn the_same_shape_through_a_history_and_the_translation() {
    let overwrite = ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 200 }, fill_seed: 7 };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CreateSnapshot,
            HistoryOperation::PublishOverwrite(overwrite),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperation::ReadSnapshot(SnapshotChoice { index_among_the_snapshots: 0 }),
        ],
    };
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let mut translation = ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        eprintln!(
            "STEP {:?} outcome={}",
            observation.position,
            observation.outcome.map(|o| format!("{o:?}").chars().take(200).collect::<String>()).unwrap_or_default()
        );
        translation.note_an_observed_step(observation, stream.operation_count());
    });
    eprintln!("ENDING {}", format!("{:?}", run.ending).chars().take(600).collect::<String>());
    let translated = translation.translate(&stream.retained_operations(), &execution.device_width.fixed_geometry());
    let judgement = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        &ObservedReadBack::FileRead {
            root: singlefs_harness::model_comparison::model_root_key(
                singlefs_core::address::InstanceGeneration(1),
                singlefs_core::address::CheckpointTxg(0),
            ),
            content: overwrite.bytes(),
        },
    );
    eprintln!("VOTE {:?}", judgement.vote);
}
