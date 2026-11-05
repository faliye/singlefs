
/// 攻方探针（m4-gpugc-code-r1，T4 第 ② 处）：`mounted_session.rs` 整理一趟里「删除续做在整理批之前」那一支进得去。
/// 历史：另一次挂载顺序写第一个文件 → 重挂选 gpu、在 GPU 上签发意图、注入甲段出错让整理停下（同 `gpu_session_stopped_after_a_gpu_failure_with_the_intent_on_disk`）
/// → 会话里 unlink 第一个文件（一次走完）→ 挂着回退到 unlink 意图那一版（队列带回第一个文件，同 `unlink_mount_continuation_and_the_emptied_version.rs`
/// 那条「回退到意图那一版」的用例的做法）→ 显式入口推一趟整理。阈值 0%（停着且超过阈值，补跑）时这一趟先走删除续做；阈值默认 85%（停着且不超过）时一个写都不发。
fn rolled_back_to_the_unlink_intent_while_stopped(
    tag: &str,
    threshold: &CompensationThresholdAtTheMount,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(tag);
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_with_the_intent_on_disk(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            threshold,
        );
    let unlinked = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("停着时会话照常 unlink：一次走完");
    let intent_root = unlinked.versions_published[0].root;
    let mut current = current_of(&session).clone();
    assert!(current.deleted_inodes.is_empty(), "unlink 一次走完：队列空");
    singlefs_core::mount::roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut current,
        singlefs_core::mount::RollbackTarget {
            instance: intent_root.instance,
            checkpoint_txg: intent_root.checkpoint_txg,
        },
    )
    .expect("挂着时回退到 unlink 意图那一版");
    println!(
        "ATTACK name=rolled_back_queue tag={tag} deleted_inodes_after_the_rollback={:?} stopped={:?}",
        current.deleted_inodes,
        session.background_compute().stopped_after_a_compute_error().len()
    );
    session.current = singlefs_core::transaction::PoolVersion::of_a_version_of_trees(current);
    (session, devices)
}

#[test]
fn attack_probe_a_compensating_run_continues_the_deletion_queue_brought_back_by_a_rollback() {
    let (mut session, mut devices) = rolled_back_to_the_unlink_intent_while_stopped(
        "attack-deletion-continuation-compensating",
        &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
    );
    assert!(!current_of(&session).deleted_inodes.is_empty(), "前提：队列不空");
    let txg_before = current_of(&session).root.checkpoint_txg.0;
    let run = run_one_compaction_publish(&mut session, &mut devices);
    println!(
        "ATTACK name=compensating_run_after_rollback outcome={:?} versions_published={} txg_before={} txgs_published={:?} deleted_inodes_after_the_run={:?} batches_recorded={}",
        run.outcome,
        run.versions_published.len(),
        txg_before,
        run.versions_published
            .iter()
            .map(|version| version.root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        current_of(&session).deleted_inodes,
        run.background_compute_of_each_batch.len()
    );
    assert!(
        !run.versions_published.is_empty() && current_of(&session).deleted_inodes.is_empty(),
        "停着而补跑：这一趟进了删除续做那一支、把队列走空"
    );
}

#[test]
fn attack_probe_a_run_left_stopped_does_not_continue_the_deletion_queue_brought_back_by_a_rollback() {
    let (mut session, mut devices) = rolled_back_to_the_unlink_intent_while_stopped(
        "attack-deletion-continuation-left-stopped",
        &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
    );
    let run = run_one_compaction_publish(&mut session, &mut devices);
    println!(
        "ATTACK name=left_stopped_run_after_rollback outcome={:?} versions_published={} deleted_inodes_after_the_run={:?}",
        run.outcome,
        run.versions_published.len(),
        current_of(&session).deleted_inodes
    );
    assert!(
        run.versions_published.is_empty() && !current_of(&session).deleted_inodes.is_empty(),
        "停着而不补跑：一个写都不发，队列照旧不空"
    );
}
