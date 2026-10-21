//! 攻方腿原型（m4-segment-code-r3，Q2 H5）：抛弃最新根的那次恢复之后，RefFS 取的快照集合（照 U2 / H5：允许集合里每个状态「之前那一版」的）。
//! RefFS 的输入只有操作序列与读出的内容，看不见被抛弃的是哪一条根。一版在核心层不止一条根（可写挂载的写行与暖机、整理的各批、为空间推的抬 F
//! 都映到现行那一版，不进操作序列；删快照一次调用 = 意图 + 全部批），被抛弃的是同一版的后一条根时，恢复落到的那条根仍是这一版。
//! 下面两格的操作序列与 RefFS 自己的单测逐字相同或只多一个删快照，各打印 RefFS 交回的快照集合与读快照的判定。
use singlefs_reference_filesystem::{
    FileContent, Operation, OperationOutcome, ReferenceFileSystem, SnapshotName, SubmittedOperation,
    VisibleState,
};

fn file_with(text: &str) -> VisibleState {
    VisibleState::OneFile(FileContent(text.as_bytes().to_vec()))
}

fn succeed(model: &mut ReferenceFileSystem, operation: Operation) {
    match model.submit(operation, OperationOutcome::Succeeded) {
        Ok(SubmittedOperation::Entered(_)) => {}
        other => panic!("前提：操作进序列：{other:?}"),
    }
}

/// 写 A、发布、建快照 7；核心层随后可写挂载一次（写行与暖机两三条根，都映到建快照那一版，不进操作序列）；
/// 进程退出，抛弃最新根（暖机那一条）的那次恢复落到它前一条根——仍是建快照那一版，快照 7 在、读出 A。
#[test]
fn the_abandoned_root_is_a_later_root_of_the_snapshot_version() {
    let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
    succeed(&mut model, Operation::CreateTheFirstFile(FileContent(b"A".to_vec())));
    succeed(&mut model, Operation::ExplicitPublish);
    succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
    let crash = model.crash_after_every_operation_returned();
    model
        .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file_with("A"))
        .expect("抛弃最新根的那一次恢复不判读出的内容");
    eprintln!("CASE mount-roots allowed_snapshot_sets={:?}", model.allowed_snapshot_sets());
    eprintln!("CASE mount-roots allowed_reads_of_7={:?}", model.allowed_reads_of_the_snapshot(SnapshotName(7)));
    let verdict = model.judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(SnapshotName(7), &file_with("A"));
    eprintln!("CASE mount-roots read_7_as_A={verdict:?}");
}

/// 写 A、发布、建快照 7、删快照 7（核心层：意图那次 + 批，至少两条根）；抛弃最新根（最后一批）的那次恢复落到意图那一条根：
/// 快照 7 带销毁中记号，读照读（读出 A），下一次用户改动先走空、把它删完。
#[test]
fn the_abandoned_root_is_the_last_batch_of_a_snapshot_destruction() {
    let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
    succeed(&mut model, Operation::CreateTheFirstFile(FileContent(b"A".to_vec())));
    succeed(&mut model, Operation::ExplicitPublish);
    succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
    succeed(&mut model, Operation::DestroySnapshot(SnapshotName(7)));
    let crash = model.crash_after_every_operation_returned();
    model
        .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file_with("A"))
        .expect("抛弃最新根的那一次恢复不判读出的内容");
    eprintln!("CASE destroy-batches allowed_snapshot_sets={:?}", model.allowed_snapshot_sets());
    eprintln!("CASE destroy-batches allowed_reads_of_7={:?}", model.allowed_reads_of_the_snapshot(SnapshotName(7)));
    let mut after_the_walk = model.clone();
    // 核心层走空之后快照 7 没了：再建同名 7 在核心层照收；RefFS 这边先看它收不收。
    let recreate = after_the_walk.submit(Operation::CreateSnapshot(SnapshotName(7)), OperationOutcome::Succeeded);
    eprintln!("CASE destroy-batches recreate_7_after_the_walk={recreate:?}");
}
