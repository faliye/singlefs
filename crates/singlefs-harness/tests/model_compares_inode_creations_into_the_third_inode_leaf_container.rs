//! O1（理想模型对拍，D13（验证路线） 已定项 6）罩 inode 树第二片及以后的叶容器（C673 ⑦「历史加跨一片叶」）：
//! 删完第一个文件之后一次建一个 inode，建满 233 条之后下一条在末尾分裂出第二片、建满 2 × 233 条之后下一条分裂出第三片
//! （D8（核心索引结构） 已定项 6：左半不动、只重写新记录落的那一片与根），模型逐片记各片叶容器的分配代、逐次与实现交回的分配记录比，
//! 每一步都对得上。

use singlefs_core::admission::SpaceAdmission;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
    HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepOutcome,
};
use singlefs_harness::SharedStream;

/// 删完第一个文件之后建几个 inode：建满第一片、第二片各 233 条，再往第三片里建 4 条——第 234 条分裂出第二片、
/// 第 467 条分裂出第三片，之后三条只重写第三片；第一片的分配代停在第 233 条那次、第二片的停在第 466 条那次。
/// 建到第三片才分得开「第 k 片」与「第二片及以后都当第二片」（第 25 批代码轮正推 KX1、KX2，
/// 判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md` K1 第三行）。
const INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER: u64 = 2 * INODE_LEAF_RECORDS + 4;

/// mkfs、取号、暖机、第一个文件（起点 `AfterFirstFile`），unlink 第一个文件（意图那次 + 最后一批，队列走空），
/// 之后一次建一个 inode，建到 inode 树有三片叶容器再多建三个。模型与实现每一步都对得上：历史跑完、没有新发现，
/// unlink 与每一次建 inode 都做成。
/// 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这条用例判的是 O1 逐片记的账，跑 checker 时 debug 下要六分多钟。
/// 判别力：模型把第二片的分配代记成第一片的（建第二片里的 inode 时重写的叶容器记成最左那片），或胶水把第二片当成最左那片，
/// 建第 234 个 inode 那一步判对不上、历史停在那里；模型把落点封顶在第二片，或胶水把第三片及以后都当第二片，
/// 建第 467 个 inode 那一步判对不上。
#[test]
fn creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container(
) {
    let creations =
        usize::try_from(INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER).expect("几百次装得进 usize");
    let mut operations = vec![HistoryOperation::Unlink];
    operations.extend(std::iter::repeat_n(
        HistoryOperation::CreateInode,
        creations,
    ));
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
        created.applied, INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER,
        "建过的 inode 多于两片叶容器装的 2 × {INODE_LEAF_RECORDS} 条：{created:?}"
    );
    let counts = &run.tally.model_counts;
    assert!(
        counts.successes_matched > INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER,
        "模型判过 unlink 与每一次建 inode：{counts:?}"
    );
    assert!(
        counts.allocation_records_compared > INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER,
        "每一次建 inode 都逐个角色比过分配记录：{counts:?}"
    );
}
