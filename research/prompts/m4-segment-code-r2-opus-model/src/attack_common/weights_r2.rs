// 攻方腿（m4-segment-code-r2）的比重：与第一轮攻方的两组不同，换攻击面（正文第三节 R8「不重复第一轮的取样办法与种子段」）。
// 种子段从 SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 8_000_000 起（第一轮攻方用的是 7_000_000 起）。
const ATTACK_SEED_OFFSET: u64 = 8_000_000;

/// 整理、回退、抬 F、删文件之后的快照、崩溃恢复抛弃根混在一起；带文件时 unlink 份数比第一轮高，删完之后建快照、回退、整理各占一大份。
const R2_MIX: GenerationWeights = GenerationWeights {
    name: "第二轮攻方：整理 + 回退 + 删完之后的快照",
    named: NamedGenerationWeights::CompactionWhileMountedWithRollback,
    starting_points: &[
        (HistoryStartingPoint::AfterMakeFilesystem, 1),
        (HistoryStartingPoint::AfterFirstFile, 9),
    ],
    with_session_closed: &[(K::CloseAndMountWritable, 90), (K::ColdStartRecover, 10)],
    with_session_open_without_file: &[
        (K::PublishFirstFile, 80),
        (K::CloseAndMountWritable, 8),
        (K::ColdStartRecover, 4),
        (K::CompactWhileMounted, 8),
    ],
    with_session_open_with_file: &[
        (K::PublishOverwrite, 22),
        (K::CompactWhileMounted, 22),
        (K::RollBackWhileMounted, 14),
        (K::RaiseRollbackFloor, 5),
        (K::CreateSnapshot, 6),
        (K::ReadSnapshot, 4),
        (K::DestroySnapshot, 6),
        (K::Unlink, 6),
        (K::CloseAndMountWritable, 8),
        (K::ColdStartRecover, 4),
        (K::CrashRecoveryAbandoningTheNewestRoot, 3),
    ],
    with_session_open_after_the_file_was_deleted: Some(&[
        (K::CreateInode, 15),
        (K::CompactWhileMounted, 20),
        (K::RollBackWhileMounted, 20),
        (K::CreateSnapshot, 15),
        (K::DestroySnapshot, 10),
        (K::ReadSnapshot, 8),
        (K::CloseAndMountWritable, 7),
        (K::RaiseRollbackFloor, 5),
    ]),
    rollback_targets: RollbackTargetDraw::FourthNewestRootHalfTheTime,
    overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
};

/// 文件集合为空时的快照（F5）：带文件时多抽 unlink，删完之后建快照、删快照、读快照、回退、整理、可写挂载。
const R2_EMPTY_FILE_SET: GenerationWeights = GenerationWeights {
    name: "第二轮攻方：删完之后的快照",
    named: NamedGenerationWeights::SnapshotOperations,
    starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
    with_session_closed: &[(K::CloseAndMountWritable, 90), (K::ColdStartRecover, 10)],
    with_session_open_without_file: &[
        (K::PublishFirstFile, 80),
        (K::CloseAndMountWritable, 10),
        (K::ColdStartRecover, 10),
    ],
    with_session_open_with_file: &[
        (K::Unlink, 35),
        (K::PublishOverwrite, 25),
        (K::CreateSnapshot, 15),
        (K::CompactWhileMounted, 10),
        (K::CloseAndMountWritable, 10),
        (K::ColdStartRecover, 5),
    ],
    with_session_open_after_the_file_was_deleted: Some(&[
        (K::CreateSnapshot, 30),
        (K::DestroySnapshot, 15),
        (K::ReadSnapshot, 10),
        (K::RollBackWhileMounted, 15),
        (K::CreateInode, 10),
        (K::CompactWhileMounted, 10),
        (K::CloseAndMountWritable, 5),
        (K::ColdStartRecover, 5),
    ]),
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
};

/// 环境变量 ATTACK_WEIGHTS：with-rollback 取修复批新加的 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`（原样），empty-file-set 取
/// [`R2_EMPTY_FILE_SET`]，别的取 [`R2_MIX`]。
fn attack_weights() -> &'static GenerationWeights {
    match std::env::var("ATTACK_WEIGHTS").as_deref() {
        Ok("with-rollback") => &GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
        Ok("empty-file-set") => &R2_EMPTY_FILE_SET,
        _ => &R2_MIX,
    }
}
