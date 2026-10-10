//! O1（理想模型对拍，D13（验证路线） 已定项 6）罩 inode 树第三片、第四片叶容器：删完第一个文件之后一次建一个 inode，
//! 建到第四片；回退到第三片在之后被重写过的那一版；回退到还没有第四片的那一版。模型逐片记各片叶容器的分配代、
//! 逐次与实现交回的分配记录比，每一步都对得上。
//! 只建到第三片的两条（`model_compares_inode_creations_into_the_third_inode_leaf_container`、
//! `model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers`）分不开「第 k 片」与「第三片及以后都当第三片」，
//! 也分不开「回退取回、丢掉右侧每一片」与「只取回第二片」「只丢第三片」（第 29 批代码轮判决
//! `research/prompts/m3-formal-code-b29-r1-main-verification.md` L1 第三行）。
//! 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这几条判的是 O1 逐片记的账。

use singlefs_core::admission::SpaceAdmission;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistoryOperation, HistoryOperationKind, HistoryRun, HistorySeed,
    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::SharedStream;

/// 建到第四片：建满三片（3 × 233 条），再往第四片里建 4 条——第 700 条分裂出第四片，之后三条只重写第四片。
const INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER: u64 = 3 * INODE_LEAF_RECORDS + 4;

/// 第三片在目标那一版之后被重写过的那一次回退之前建几个 inode：建满两片，再往第三片里建 5 条——
/// 根环里最新那条根的第三片 5 条、第 3 新的那条 3 条。
const INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_REWRITTEN_THIRD_LEAF_CONTAINER: u64 =
    2 * INODE_LEAF_RECORDS + 5;

/// 回退到没有第四片的那一版之前建几个 inode：建满三片，再往第四片里建 2 条——根环里最新那条根的第四片 2 条、
/// 第 2 新的 1 条、第 3 新的那一版三片、第三片满、还没有第四片。
const INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_VERSION_WITHOUT_THE_FOURTH_LEAF_CONTAINER: u64 =
    3 * INODE_LEAF_RECORDS + 2;

/// 两次回退都退到根环里按 (txg, 实例) 从新到旧第几条（从 0 数）：第 3 新那条，也就是最新那一版往前数两次建 inode 的那一版。
const ROLLBACK_TARGET_INDEX_FROM_NEWEST: u64 = 2;

/// 回退之后再建几个 inode：第一个重写目标那一版的最右一片（或在它的末尾分出下一片），后两个接着重写。
const INODE_CREATIONS_AFTER_THE_ROLLBACK: u64 = 3;

fn inode_creations(count: u64) -> impl Iterator<Item = HistoryOperation> {
    std::iter::repeat_n(
        HistoryOperation::CreateInode,
        usize::try_from(count).expect("几百次装得进 usize"),
    )
}

/// 起点 `AfterFirstFile`（mkfs、取号、暖机、第一个文件），每一步不跑池级 checker、4 GiB 设备、空间按公式准入。
fn execute_from_after_the_first_file_without_the_per_step_checker(
    operations: Vec<HistoryOperation>,
) -> (GeneratedHistory, HistoryRun) {
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
    (history, run)
}

/// unlink 第一个文件，一次建一个 inode 建 703 个。模型与实现每一步都对得上：历史跑完、没有新发现，
/// unlink 与每一次建 inode 都做成。
/// 判别力：胶水把第四片及以后都当第三片（实现交回的叶容器号封顶在 2），或模型把落点封顶在第三片，
/// 建第 700 个 inode（分出第四片）那一步判对不上、历史停在那里。
#[test]
fn creating_inodes_one_at_a_time_into_the_fourth_inode_leaf_container_is_judged_by_the_model_container_by_container(
) {
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(inode_creations(
        INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
    ));
    let (history, run) = execute_from_after_the_first_file_without_the_per_step_checker(operations);
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
    let not_published: Vec<(usize, &StepOutcome)> = run
        .outcomes
        .iter()
        .enumerate()
        .filter(|(_, outcome)| {
            !matches!(
                outcome,
                StepOutcome::Applied(AppliedEffect::Published { .. })
            )
        })
        .collect();
    assert!(
        not_published.is_empty(),
        "unlink 与每一次建 inode 都做成：{not_published:?}"
    );
    let created = run
        .tally
        .operations_by_kind
        .get(&HistoryOperationKind::CreateInode)
        .copied()
        .expect("建过 inode：这一类在计数里");
    assert_eq!(
        created.applied, INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
        "建过的 inode 多于三片叶容器装的 3 × {INODE_LEAF_RECORDS} 条：{created:?}"
    );
    let counts = &run.tally.model_counts;
    assert!(
        counts.successes_matched > INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
        "模型判过 unlink 与每一次建 inode：{counts:?}"
    );
    assert!(
        counts.allocation_records_compared > INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
        "每一次建 inode 都逐个角色比过分配记录：{counts:?}"
    );
}

/// unlink 第一个文件，建 471 个 inode（第三片 5 条），回退到根环里第 3 新那条根（第三片 3 条），再建 3 个。
/// 回退那次发布的新根取目标那一版的叶容器（D23（journal 的角色与格式） 已定项 14）：第三片两版都有、现行那一版之后重写过两次，
/// 模型要把第三片的分配代整组取回目标那一版的。模型与实现每一步都对得上：历史跑完、没有新发现，回退做成，
/// 回退前后每一次建 inode 都做成。
/// 判别力：模型回退时只取回目标那一版的第二片（`role_written_at.extend` 那一句的过滤只留第二片），
/// 第三片的分配代留在现行那一版的，回退那一步判对不上、历史停在那里。
#[test]
fn a_rollback_to_a_version_whose_third_inode_leaf_container_was_rewritten_since_then_creating_inodes_is_judged_by_the_model_container_by_container(
) {
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(inode_creations(
        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_REWRITTEN_THIRD_LEAF_CONTAINER,
    ));
    let rollback_step = operations.len();
    operations.push(HistoryOperation::RollBackWhileMounted(
        RollbackTargetChoice::RingRoot {
            index_from_newest: ROLLBACK_TARGET_INDEX_FROM_NEWEST,
        },
    ));
    operations.extend(inode_creations(INODE_CREATIONS_AFTER_THE_ROLLBACK));
    let (history, run) = execute_from_after_the_first_file_without_the_per_step_checker(operations);
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
        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_REWRITTEN_THIRD_LEAF_CONTAINER
            + INODE_CREATIONS_AFTER_THE_ROLLBACK,
        "回退前后的建 inode 都做成：{created:?}"
    );
}

/// unlink 第一个文件，建 701 个 inode（第四片 2 条），回退到根环里第 3 新那条根（三片、第三片满、没有第四片），再建 3 个
/// （第一个在目标那一版的末尾重新分出第四片）。模型要把现行那一版有、目标那一版没有的第四片丢掉。
/// 模型与实现每一步都对得上：历史跑完、没有新发现，回退做成，回退前后每一次建 inode 都做成。
/// 判别力：模型回退时只丢第三片、更右的留着（`role_written_at.retain` 那一句的过滤只丢第三片），
/// 现行那一版的第四片的分配代留下，回退那一步判对不上、历史停在那里；
/// 胶水把第四片及以后都当第三片、模型把落点封顶在第三片、回退只取回第二片，在这条上同样判对不上。
#[test]
fn a_rollback_to_a_version_without_the_fourth_inode_leaf_container_then_creating_inodes_is_judged_by_the_model_container_by_container(
) {
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(inode_creations(
        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_VERSION_WITHOUT_THE_FOURTH_LEAF_CONTAINER,
    ));
    let rollback_step = operations.len();
    operations.push(HistoryOperation::RollBackWhileMounted(
        RollbackTargetChoice::RingRoot {
            index_from_newest: ROLLBACK_TARGET_INDEX_FROM_NEWEST,
        },
    ));
    operations.extend(inode_creations(INODE_CREATIONS_AFTER_THE_ROLLBACK));
    let (history, run) = execute_from_after_the_first_file_without_the_per_step_checker(operations);
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
        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_VERSION_WITHOUT_THE_FOURTH_LEAF_CONTAINER
            + INODE_CREATIONS_AFTER_THE_ROLLBACK,
        "回退前后的建 inode 都做成：{created:?}"
    );
}
