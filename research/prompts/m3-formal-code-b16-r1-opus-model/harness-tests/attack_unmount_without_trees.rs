/// 攻方 K1：把模型「树表 0 条不答」那一格放开（`crates/mutations.tsv` 第 2217 行的改法施加在副本上）之后，模型照 D16 已定项 1「正常卸载」字面答什么、
/// 实现做了什么、对拍报哪一格。只在攻方副本上跑。
#[test]
fn attack_unmount_on_a_version_without_trees_when_the_model_answers_by_the_clause() {
    let mut pool = first_mount_on_a_fresh_pool();
    let answer = pool.model.answer_unmount().expect("副本上放开了那一格");
    let floors: Vec<(u64, u64)> = answer
        .expected_roots
        .iter()
        .map(|root| (root.key.checkpoint_txg.0, root.rollback_floor.0))
        .collect();
    let mut current = pool.mounted.current.clone();
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.mounted.allocator,
        &mut current,
        singlefs_core::mount::ShadowLedger::On,
    )
    .expect("卸载");
    let implementation = match &unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => format!("推了 {} 次", raised.publishes.len()),
        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => "一个字节都不写".to_string(),
    };
    let verdict = pool
        .model
        .judge_and_advance(&answer, &ObservedOutcome::Succeeded(observed_unmount(&unmounted)))
        .map(|_| ())
        .map_err(|disagreement| disagreement.aspect);
    println!("ATTACK-K1 model_roots(txg,F)={floors:?} implementation={implementation} verdict={verdict:?}");
}
