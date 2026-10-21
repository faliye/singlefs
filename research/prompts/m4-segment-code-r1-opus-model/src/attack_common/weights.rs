const ATTACK_SEED_OFFSET: u64 = 7_000_000;

const ATTACK_MIX: GenerationWeights = GenerationWeights {
    name: "攻方混合：快照 + 整理 + 回退 + 抬 F + 崩溃恢复",
    named: NamedGenerationWeights::CompactionWhileMounted,
    starting_points: &[
        (HistoryStartingPoint::AfterMakeFilesystem, 1),
        (HistoryStartingPoint::AfterFirstFile, 9),
    ],
    with_session_closed: &[(K::CloseAndMountWritable, 95), (K::ColdStartRecover, 5)],
    with_session_open_without_file: &[
        (K::PublishFirstFile, 80),
        (K::CloseAndMountWritable, 8),
        (K::ColdStartRecover, 4),
        (K::CompactWhileMounted, 8),
    ],
    with_session_open_with_file: &[
        (K::PublishOverwrite, 25),
        (K::CompactWhileMounted, 20),
        (K::CreateSnapshot, 12),
        (K::ReadSnapshot, 6),
        (K::DestroySnapshot, 10),
        (K::Unlink, 5),
        (K::RollBackWhileMounted, 6),
        (K::RaiseRollbackFloor, 6),
        (K::CloseAndMountWritable, 6),
        (K::ColdStartRecover, 2),
        (K::CrashRecoveryAbandoningTheNewestRoot, 2),
    ],
    with_session_open_after_the_file_was_deleted: Some(&[
        (K::CreateInode, 20),
        (K::CompactWhileMounted, 20),
        (K::CreateSnapshot, 12),
        (K::DestroySnapshot, 12),
        (K::ReadSnapshot, 8),
        (K::RollBackWhileMounted, 8),
        (K::CloseAndMountWritable, 8),
        (K::RaiseRollbackFloor, 6),
        (K::PublishOverwrite, 4),
        (K::ColdStartRecover, 2),
    ]),
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
};


/// 同 [`ATTACK_MIX`]，去掉回退、抬 F、崩溃恢复抛弃根（看不靠回退的打中）。
const ATTACK_MIX_WITHOUT_ROLLBACK: GenerationWeights = GenerationWeights {
    name: "攻方混合，去掉回退与抬 F",
    named: NamedGenerationWeights::CompactionWhileMounted,
    starting_points: &[
        (HistoryStartingPoint::AfterMakeFilesystem, 1),
        (HistoryStartingPoint::AfterFirstFile, 9),
    ],
    with_session_closed: &[(K::CloseAndMountWritable, 95), (K::ColdStartRecover, 5)],
    with_session_open_without_file: &[
        (K::PublishFirstFile, 80),
        (K::CloseAndMountWritable, 8),
        (K::ColdStartRecover, 4),
        (K::CompactWhileMounted, 8),
    ],
    with_session_open_with_file: &[
        (K::PublishOverwrite, 30),
        (K::CompactWhileMounted, 24),
        (K::CreateSnapshot, 14),
        (K::ReadSnapshot, 6),
        (K::DestroySnapshot, 12),
        (K::Unlink, 5),
        (K::CloseAndMountWritable, 7),
        (K::ColdStartRecover, 2),
    ],
    with_session_open_after_the_file_was_deleted: Some(&[
        (K::CreateInode, 24),
        (K::CompactWhileMounted, 24),
        (K::CreateSnapshot, 14),
        (K::DestroySnapshot, 14),
        (K::ReadSnapshot, 8),
        (K::CloseAndMountWritable, 10),
        (K::PublishOverwrite, 4),
        (K::ColdStartRecover, 2),
    ]),
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
};

/// 环境变量 ATTACK_WEIGHTS=without-rollback 时取 [`ATTACK_MIX_WITHOUT_ROLLBACK`]，别的取 [`ATTACK_MIX`]。
fn attack_weights() -> &'static GenerationWeights {
    match std::env::var("ATTACK_WEIGHTS").as_deref() {
        Ok("without-rollback") => &ATTACK_MIX_WITHOUT_ROLLBACK,
        Ok("snapshots") => &GenerationWeights::SNAPSHOT_OPERATIONS,
        Ok("compaction") => &GenerationWeights::COMPACTION_WHILE_MOUNTED,
        _ => &ATTACK_MIX,
    }
}
