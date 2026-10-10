//! O1（理想模型对拍，D13（验证路线） 已定项 6）罩管理员回退跨过 inode 树叶容器分裂的那一支：
//! 删完第一个文件之后建到 inode 树有三片叶容器（第三片 2 条），回退到根环里第 3 新那条根（第三片还没分出来的那一版），
//! 再建 3 个 inode（第三片重新分出来）。回退那次发布的新根取目标那一版的叶容器（D23（journal 的角色与格式） 已定项 14），
//! 模型要把现行那一版有、目标那一版没有的那几片丢掉、把目标那一版的那几片的分配代整组取回来，之后逐片记的账才与实现对得上。

use singlefs_core::admission::SpaceAdmission;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    PerStepChecker, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::SharedStream;

/// 回退之前建几个 inode：建满两片（2 × 233 条），再往第三片里建 2 条——根环里最新那条根的第三片 2 条、第 2 新的 1 条、
/// 第 3 新的那一版还没有第三片。
const INODE_CREATIONS_BEFORE_THE_ROLLBACK: u64 = 2 * INODE_LEAF_RECORDS + 2;

/// 回退到根环里按 (txg, 实例) 从新到旧第几条（从 0 数）：第 3 新那条，也就是第三片分出来之前建完第 466 个 inode 的那一版。
const ROLLBACK_TARGET_INDEX_FROM_NEWEST: u64 = 2;

/// 回退之后再建几个 inode：第一个在目标那一版的末尾重新分出第三片，后两个只重写第三片。
const INODE_CREATIONS_AFTER_THE_ROLLBACK: u64 = 3;

/// mkfs、取号、暖机、第一个文件（起点 `AfterFirstFile`），unlink 第一个文件，一次建一个 inode 建 468 个，
/// 回退到根环里第 3 新那条根，再建 3 个。模型与实现每一步都对得上：历史跑完、没有新发现，回退做成，回退之后三次建 inode 都做成。
/// 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这条用例判的是 O1 逐片记的账。
/// 判别力：模型回退时不丢掉现行那一版多出来的第三片（`role_written_at.retain` 那一句没了），或不取回目标那一版的右侧叶容器
/// （`role_written_at.extend` 那一句没了），回退或回退之后的建 inode 那一步判对不上、历史停在那里
/// （第 25 批代码轮攻方 T13，判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md` K1 第二行）。
#[test]
fn a_rollback_past_an_inode_leaf_container_split_then_creating_inodes_is_judged_by_the_model_container_by_container(
) {
    let creations_before =
        usize::try_from(INODE_CREATIONS_BEFORE_THE_ROLLBACK).expect("几百次装得进 usize");
    let creations_after =
        usize::try_from(INODE_CREATIONS_AFTER_THE_ROLLBACK).expect("几次装得进 usize");
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(std::iter::repeat_n(
        HistoryOperation::CreateInode,
        creations_before,
    ));
    operations.push(HistoryOperation::RollBackWhileMounted(
        RollbackTargetChoice::RingRoot {
            index_from_newest: ROLLBACK_TARGET_INDEX_FROM_NEWEST,
        },
    ));
    operations.extend(std::iter::repeat_n(
        HistoryOperation::CreateInode,
        creations_after,
    ));
    let rollback_step = 1 + creations_before;
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
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "跑到第 {} 步：{:?}",
        run.outcomes.len(),
        run.outcomes.last()
    );
    assert_eq!(
        run.outcomes.len(),
        history.operations.len(),
        "每一步操作各有一个结局"
    );
    assert!(
        matches!(
            run.outcomes[rollback_step],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到根环里第 3 新那条根做成：{:?}",
        run.outcomes[rollback_step]
    );
    let not_published: Vec<(usize, &StepOutcome)> = run
        .outcomes
        .iter()
        .enumerate()
        .filter(|(step, _)| *step != rollback_step)
        .filter(|(_, outcome)| {
            !matches!(
                outcome,
                StepOutcome::Applied(AppliedEffect::Published { .. })
            )
        })
        .collect();
    assert!(
        not_published.is_empty(),
        "unlink 与回退前后每一次建 inode 都做成：{not_published:?}"
    );
    let created = run
        .tally
        .operations_by_kind
        .get(&HistoryOperationKind::CreateInode)
        .copied()
        .expect("建过 inode：这一类在计数里");
    assert_eq!(
        created.applied,
        INODE_CREATIONS_BEFORE_THE_ROLLBACK + INODE_CREATIONS_AFTER_THE_ROLLBACK,
        "回退前后的建 inode 都做成：{created:?}"
    );
}
