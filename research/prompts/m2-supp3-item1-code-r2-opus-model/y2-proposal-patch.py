#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r2 · Y2 的改法（只在我的副本上量过、被攻过零轮）：挂载（可写挂载与回退）写出的写行与暖机那几次发布，
逐次拿「上一版」的分配记录比：第一次的上一版是挂载之前的镜像上生效根那棵分配记录树（`recovery::allocation_records_under_root`），之后是前一次发布的输出；
改写或新增的记录分配代要等于那一次的 txg。用法：python3 y2-proposal-patch.py SLOT"""
import sys, os
slot = sys.argv[1]
def once(path, old, new):
    full = os.path.join(slot, path); text = open(full, encoding="utf-8").read()
    assert text.count(old) == 1, (path, old[:60], text.count(old))
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
H = "crates/singlefs-harness/src/history.rs"
once(H, """fn settle_mount(
    pool: &mut HistoryPool,
    mounted: Result<singlefs_core::mount::Mounted, MountError>,
) -> StepOutcome {
    match mounted {
        Ok(mounted) => {""", """/// 攻方改法：挂载写出的写行与暖机那几次发布，逐次比分配记录。
fn opus_mount_publish_judgement(
    image_before: &MemoryPool,
    mounted: &singlefs_core::mount::Mounted,
) -> Option<HarnessJudgement> {
    let mut previous = singlefs_core::recovery::allocation_records_under_root(
        image_before,
        &mounted.output.effective_root,
    )
    .ok()?;
    let mut judgement = None;
    for version in std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter()) {
        if let PoolVersion::WithFile(output) = version {
            let txg = output.root.checkpoint_txg;
            judgement = judgement.or_else(|| {
                allocation_generation_judgement(&previous, &output.allocation_records, txg, txg)
            });
            previous = output.allocation_records.clone();
        }
    }
    judgement
}

fn settle_mount(
    pool: &mut HistoryPool,
    mounted: Result<singlefs_core::mount::Mounted, MountError>,
    image_before: &MemoryPool,
) -> AppliedStep {
    let judgement = mounted
        .as_ref()
        .ok()
        .and_then(|mounted| opus_mount_publish_judgement(image_before, mounted));
    let outcome = settle_mount_outcome(pool, mounted);
    AppliedStep { outcome, harness_judgement: judgement }
}

fn settle_mount_outcome(
    pool: &mut HistoryPool,
    mounted: Result<singlefs_core::mount::Mounted, MountError>,
) -> StepOutcome {
    match mounted {
        Ok(mounted) => {""")
once(H, """fn apply_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> StepOutcome {
    pool.session = None;
    pool.mount_attempts += 1;
    let mounted = mount_writable(parameters, &mut pool.devices);
    settle_mount(pool, mounted)
}""", """fn apply_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    pool.session = None;
    pool.mount_attempts += 1;
    let image_before = pool.image();
    let mounted = mount_writable(parameters, &mut pool.devices);
    settle_mount(pool, mounted, &image_before)
}""")
once(H, """    choice: RollbackTargetChoice,
) -> StepOutcome {
    pool.session = None;""", """    choice: RollbackTargetChoice,
) -> AppliedStep {
    pool.session = None;""")
once(H, """        return StepOutcome::NotApplicable(MissingPrecondition::NoReadableRootInRing);
    };
    let target = match choice {""", """        return AppliedStep::judged_by_outcome_only(StepOutcome::NotApplicable(MissingPrecondition::NoReadableRootInRing));
    };
    let target = match choice {""")
once(H, """    let mounted = mount_rollback(parameters, &mut pool.devices, target, ShadowLedger::On);
    settle_mount(pool, mounted)""", """    let image_before = pool.image();
    let mounted = mount_rollback(parameters, &mut pool.devices, target, ShadowLedger::On);
    settle_mount(pool, mounted, &image_before)""")
once(H, """        HistoryOperation::CloseAndMountWritable => {
            AppliedStep::judged_by_outcome_only(apply_mount_writable(pool, &parameters))
        }
        HistoryOperation::CloseAndMountRollback(choice) => {
            AppliedStep::judged_by_outcome_only(apply_mount_rollback(pool, &parameters, *choice))
        }""", """        HistoryOperation::CloseAndMountWritable => apply_mount_writable(pool, &parameters),
        HistoryOperation::CloseAndMountRollback(choice) => {
            apply_mount_rollback(pool, &parameters, *choice)
        }""")
print("proposal patched", slot)
