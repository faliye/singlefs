//! 正推腿探针（T11）：RefFS 在「建快照那次发布崩在中途」「删快照崩在两批之间」上的口径。
//! 写了之后报错的建快照（做没做成两可）之后崩溃，恢复读出的文件内容与建快照之前、之后的状态相同：快照集合定不下来，
//! RefFS 取下标最大的那个状态的快照集合（`restart_from_the_recovered_state`），核心层则是「那次发布落没落盘」二选一。
use singlefs_reference_filesystem::{
    FileContent, Operation, OperationOutcome, ReferenceFileSystem, SnapshotName, VisibleState,
};

fn content(text: &str) -> FileContent {
    FileContent(text.as_bytes().to_vec())
}

#[test]
fn probe_forward_reference_filesystem_picks_the_latest_snapshot_set_when_the_recovered_file_cannot_tell() {
    // 建快照写了之后报错（做没做成两可），之后崩溃：恢复读出 A。
    let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
    model.submit(Operation::CreateTheFirstFile(content("A")), OperationOutcome::Succeeded).unwrap();
    model.submit(Operation::CreateSnapshot(SnapshotName(7)), OperationOutcome::FailedAfterWriting).unwrap();
    let crash = model.crash_after_every_operation_returned();
    let allowed = model.crash_and_recover(crash, VisibleState::OneFile(content("A"))).expect("A 在允许集合里");
    println!(
        "name=probe_forward_reffs creation_failed_after_writing allowed_states={} snapshots_after_the_recovery={:?}",
        allowed.allowed.len(),
        model.snapshots().keys().collect::<Vec<_>>()
    );
    // 对照：同一个输入里建快照是成功返回的，崩溃之后快照接着活着（原有用例钉了）。
    let mut returned = ReferenceFileSystem::on_a_freshly_formatted_pool();
    returned.submit(Operation::CreateTheFirstFile(content("A")), OperationOutcome::Succeeded).unwrap();
    returned.submit(Operation::CreateSnapshot(SnapshotName(7)), OperationOutcome::Succeeded).unwrap();
    let crash = returned.crash_after_every_operation_returned();
    returned.crash_and_recover(crash, VisibleState::OneFile(content("A"))).expect("A 在允许集合里");
    println!("name=probe_forward_reffs creation_returned snapshots_after_the_recovery={:?}", returned.snapshots().keys().collect::<Vec<_>>());
    // 删快照写了之后报错（意图落了、批没落完），之后崩溃，恢复读出 A（活头文件不变）。
    let mut destroying = ReferenceFileSystem::on_a_freshly_formatted_pool();
    destroying.submit(Operation::CreateTheFirstFile(content("A")), OperationOutcome::Succeeded).unwrap();
    destroying.submit(Operation::CreateSnapshot(SnapshotName(7)), OperationOutcome::Succeeded).unwrap();
    destroying.submit(Operation::DestroySnapshot(SnapshotName(7)), OperationOutcome::FailedAfterWriting).unwrap();
    let crash = destroying.crash_after_every_operation_returned();
    destroying.crash_and_recover(crash, VisibleState::OneFile(content("A"))).expect("A 在允许集合里");
    println!("name=probe_forward_reffs destruction_failed_after_writing snapshots_after_the_recovery={:?}", destroying.snapshots().keys().collect::<Vec<_>>());
}
