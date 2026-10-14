//! 三·2 第 8 条：理想模型加删除（`singlefs_harness::model` 的 `answer_unlink_intent` / `answer_deletion_step`，删后读回「没有文件」），
//! 随机历史的操作加 unlink（`HistoryOperation::Unlink`，经挂着的会话发），模型对拍罩上删除那两棵树（`deleted_inodes` 根、墓碑树的容器与根）。
//! 这里钉两样：一段写死的短历史里 unlink 之后每一步模型都对得上；随机历史按快档那组比重真的生成到了 unlink、跑完对得上。
//! 随机历史加「建 inode」（`HistoryOperation::CreateInode`，用户 2026-10-03 定案）之后再钉三样：删完之后建 inode、挂载、再建一个，
//! 模型每一步对得上；删完之后建 inode、回退到删之前那一版，第一个文件又能覆盖写；快档那组比重真的在 unlink 之后抽到了建 inode。

use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, ColdStartReadBack,
    ContentChoice, ContentLength, GeneratedHistory, GenerationWeights, HistoryEnding,
    HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    MissingPrecondition, RollbackTargetChoice, StepOutcome, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

/// 快档那一批的种子数与每段步数（`tests/random_histories.rs` 的 `FAST_TIER_SEEDS`、`FAST_TIER_OPERATIONS_PER_HISTORY`）：
/// 在这一批里找第一段带 unlink 的历史。
const FAST_TIER_SEEDS: u64 = 96;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 30;

/// 从第一个文件起：unlink 第一个文件（意图那次 + 最后一批，模型逐次比两条根、删除那两棵树的分配代）、冷启动读回「没有文件」、
/// 可写挂载（写行与暖机接在有树而文件集合为空的一版后面，写行与暖机都重写固定点那几个单元：分配记录树要改的那几个节点 + 记账 + 映射 + 树表，写行另加实例表单元）、在它上面发第一个文件被拒（树表不是 0 条）、
/// 再冷启动仍读回「没有文件」。每一步之后跑池级 checker，模型与实现每一步都对得上：历史跑完，没有新发现。
/// 判别力：模型删除那一臂改坏（最后一批不去掉数据单元与 `deleted_inodes` 根的角色、删后读回旧内容）时那一步判对不上、历史停在那里。
#[test]
fn unlinking_the_first_file_then_reading_back_and_mounting_runs_to_the_end_with_the_model_agreeing()
{
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::ColdStartRecover,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 2999 },
                fill_seed: 5,
            }),
            HistoryOperation::ColdStartRecover,
        ],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert!(
        matches!(
            run.outcomes[0],
            StepOutcome::Applied(AppliedEffect::Published { .. })
        ),
        "unlink 做成：{:?}",
        run.outcomes[0]
    );
    for step in [1, 4] {
        assert!(
            matches!(
                &run.outcomes[step],
                StepOutcome::Applied(AppliedEffect::Recovered {
                    read_back: ColdStartReadBack::NoFile,
                    ..
                })
            ),
            "第 {step} 步冷启动读回「没有文件」：{:?}",
            run.outcomes[step]
        );
    }
    assert!(
        matches!(
            run.outcomes[2],
            StepOutcome::Applied(AppliedEffect::Mounted { .. })
        ),
        "删完之后可写挂载做成：{:?}",
        run.outcomes[2]
    );
    assert_eq!(
        run.outcomes[3],
        StepOutcome::Refused {
            member: "PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile".to_string()
        },
        "有树而文件集合为空的一版上发第一个文件被拒"
    );
    let counts = &run.tally.model_counts;
    assert!(
        counts.successes_matched >= 4 && counts.required_refusals_matched >= 1,
        "模型判过 unlink 的两次发布、两次冷启动、挂载与那一次该拒：{counts:?}"
    );
}

/// 随机历史真的生成到了 unlink：按快档那组比重（`GenerationWeights::BROAD`）、快档的种子与步数，找第一段带 unlink 的历史，
/// 照两块 4 GiB 的盘、每一步之后跑池级 checker 跑完：没有新发现，unlink 那一步至少做成一次。
/// 判别力：比重里 unlink 那一格改成 0 时快档那一批里一段带 unlink 的历史都没有，这一条在找的那一步就红。
#[test]
fn the_fast_tier_weights_draw_an_unlink_and_the_first_history_with_one_runs_to_the_end_with_the_model_agreeing(
) {
    let Some(history) = (0..FAST_TIER_SEEDS)
        .map(|offset| {
            generate_history_with_weights(
                HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(offset)),
                FAST_TIER_OPERATIONS_PER_HISTORY,
                &GenerationWeights::BROAD,
            )
        })
        .find(|history| {
            history
                .operations
                .iter()
                .any(|operation| operation.kind() == HistoryOperationKind::Unlink)
        })
    else {
        panic!("快档那一批 {FAST_TIER_SEEDS} 段历史里一段带 unlink 的都没有：比重里 unlink 那一格是 0？");
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert!(
        !matches!(run.ending, HistoryEnding::NewFinding { .. }),
        "种子 {:?}：{:?}",
        history.seed,
        run.ending
    );
    let unlink = run
        .tally
        .operations_by_kind
        .get(&HistoryOperationKind::Unlink)
        .copied()
        .unwrap_or_default();
    assert!(
        unlink.applied >= 1,
        "种子 {:?} 的 unlink 一次都没做成：{unlink:?}；{:?}",
        history.seed,
        run.outcomes
    );
}

/// 一个数据单元装得下的内容（覆盖写、第一个文件用）。
const CONTENT_INSIDE_ONE_DATA_UNIT: ContentChoice = ContentChoice {
    length: ContentLength::InsideOneDataUnit { selector: 2999 },
    fill_seed: 5,
};

/// 从第一个文件起删掉它，在删完的一版上一路往下走：建一个 inode（实现归带文件的一版、没有第一个文件那条记录）、冷启动读回「没有文件」、
/// 可写挂载（写行与暖机接在只有 size 0 的 inode 的那一版后面）、再建一个；覆盖写与 unlink 的前提不成立（第一个文件已删完、执行器不调），
/// 第一个文件被拒（树表不是 0 条）。每一步之后跑池级 checker，模型与实现每一步都对得上：历史跑完，没有新发现。
/// 判别力：模型建 inode 那一臂改坏（不重写 inode 叶、或建完之后不再算删完第一个文件的一版）时那一步判对不上、历史停在那里。
#[test]
fn creating_inodes_after_the_first_file_was_deleted_runs_through_a_mount_with_the_model_agreeing() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateInode,
            HistoryOperation::ColdStartRecover,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CreateInode,
            HistoryOperation::PublishOverwrite(CONTENT_INSIDE_ONE_DATA_UNIT),
            HistoryOperation::Unlink,
            HistoryOperation::PublishFirstFile(CONTENT_INSIDE_ONE_DATA_UNIT),
        ],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for step in [0, 1, 4] {
        assert!(
            matches!(
                run.outcomes[step],
                StepOutcome::Applied(AppliedEffect::Published { .. })
            ),
            "第 {step} 步（unlink、两次建 inode）做成：{:?}",
            run.outcomes[step]
        );
    }
    assert!(
        matches!(
            &run.outcomes[2],
            StepOutcome::Applied(AppliedEffect::Recovered {
                read_back: ColdStartReadBack::NoFile,
                ..
            })
        ),
        "建了 inode 之后冷启动读回「没有文件」：{:?}",
        run.outcomes[2]
    );
    assert!(
        matches!(
            run.outcomes[3],
            StepOutcome::Applied(AppliedEffect::Mounted { .. })
        ),
        "建了 inode 之后可写挂载做成：{:?}",
        run.outcomes[3]
    );
    assert_eq!(
        (&run.outcomes[5], &run.outcomes[6]),
        (
            &StepOutcome::NotApplicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile),
            &StepOutcome::NotApplicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile),
        ),
        "第一个文件已删完：覆盖写与 unlink 执行器不调"
    );
    assert_eq!(
        run.outcomes[7],
        StepOutcome::Refused {
            member: "PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile".to_string()
        },
        "删完之后建过 inode 的一版上发第一个文件被拒"
    );
    let counts = &run.tally.model_counts;
    assert!(
        counts.successes_matched >= 5 && counts.required_refusals_matched >= 1,
        "模型判过 unlink、两次建 inode、冷启动、挂载与那一次该拒：{counts:?}"
    );
}

/// 删完第一个文件、建一个 inode 之后回退到删之前那一版（根环里从新到旧第 3 条：建 inode、最后一批、意图之前那一版，第一个文件活着），
/// 第一个文件又能覆盖写，冷启动读回覆盖写的内容。每一步之后跑池级 checker，模型每一步对得上：历史跑完，没有新发现。
/// 判别力：模型的回退在只有 size 0 的 inode 的那一版上不答（当成没有文件）时回退那一步判对不上；回退之后不把第一个文件取回来时覆盖写那一步判对不上。
#[test]
fn after_an_inode_is_created_on_the_emptied_version_a_rollback_to_before_the_unlink_lets_the_first_file_be_overwritten(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateInode,
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::PublishOverwrite(CONTENT_INSIDE_ONE_DATA_UNIT),
            HistoryOperation::ColdStartRecover,
        ],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert!(
        matches!(
            run.outcomes[2],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到删之前那一版做成：{:?}",
        run.outcomes[2]
    );
    assert!(
        matches!(
            run.outcomes[3],
            StepOutcome::Applied(AppliedEffect::Published { .. })
        ),
        "回退之后第一个文件又能覆盖写：{:?}",
        run.outcomes[3]
    );
    assert!(
        matches!(
            &run.outcomes[4],
            StepOutcome::Applied(AppliedEffect::Recovered {
                read_back: ColdStartReadBack::FileRead,
                ..
            })
        ),
        "冷启动读回覆盖写的内容：{:?}",
        run.outcomes[4]
    );
}

/// 随机历史真的在 unlink 之后抽到了建 inode：按快档那组比重（`GenerationWeights::BROAD`，unlink 之后按它那张「删过之后」的表抽）、
/// 快档的种子与步数，找第一段带建 inode 的历史，照两块 4 GiB 的盘、每一步之后跑池级 checker 跑完：没有新发现，建 inode 至少做成一次。
/// 判别力：「删过之后」那张表里建 inode 那一格改成 0 时快档那一批里一段带建 inode 的历史都没有，这一条在找的那一步就红。
#[test]
fn the_fast_tier_weights_draw_an_inode_creation_after_an_unlink_and_the_first_history_with_one_runs_to_the_end(
) {
    let Some(history) = (0..FAST_TIER_SEEDS)
        .map(|offset| {
            generate_history_with_weights(
                HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(offset)),
                FAST_TIER_OPERATIONS_PER_HISTORY,
                &GenerationWeights::BROAD,
            )
        })
        .find(|history| {
            history
                .operations
                .iter()
                .any(|operation| operation.kind() == HistoryOperationKind::CreateInode)
        })
    else {
        panic!(
            "快档那一批 {FAST_TIER_SEEDS} 段历史里一段带建 inode 的都没有：「删过之后」那张表里建 inode 那一格是 0？"
        );
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert!(
        !matches!(run.ending, HistoryEnding::NewFinding { .. }),
        "种子 {:?}：{:?}",
        history.seed,
        run.ending
    );
    let create_inode = run
        .tally
        .operations_by_kind
        .get(&HistoryOperationKind::CreateInode)
        .copied()
        .unwrap_or_default();
    assert!(
        create_inode.applied >= 1,
        "种子 {:?} 的建 inode 一次都没做成：{create_inode:?}；{:?}",
        history.seed,
        run.outcomes
    );
}

/// 回退到意图那一版（队列带回第一个文件）之后的一步（攻方 `m3-unlink-code-r1` A3 那一形）：unlink 第一个文件（意图 + 最后一批）、
/// 删完之后建一个 inode（实现把这一版归带文件，执行器才调回退）、挂着时回退到从新到旧第 3 条根（意图那一版），再走 `then`。执行器不再跳过删除中的版本（代码轮 U11）：会话先走空、交回走空发成的那一版，
/// 模型照「走空的一步」比，再比这次改动本身。交回整段的结局与第 4 步的结局。
fn history_after_rolling_back_to_the_intent_then(
    then: HistoryOperation,
) -> (
    HistoryEnding,
    StepOutcome,
    singlefs_harness::model::ModelJudgementCounts,
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateInode,
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 2,
            }),
            then,
        ],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert!(
        matches!(
            run.outcomes[2],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到意图那一版做成：{:?}",
        run.outcomes
    );
    (
        run.ending,
        run.outcomes[3].clone(),
        run.tally.model_counts.clone(),
    )
}

/// 回退到意图那一版之后覆盖写：会话先走空（最后一批落盘）、再拒成 `NoFileVersionToChange`，模型比过走空那一版（做成一次）与这次该拒（一次），
/// 每一步之后池级 checker 绿、整段跑完。判别力：执行器在删除中的版本上跳过覆盖写时第 4 步是「前提不满足」，这一项红。
#[test]
fn an_overwrite_after_rolling_back_to_the_intent_is_judged_by_the_model_through_the_walked_step() {
    let (ending, outcome, counts) = history_after_rolling_back_to_the_intent_then(
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed: 5,
        }),
    );
    assert_eq!(ending, HistoryEnding::Completed);
    assert_eq!(
        outcome,
        StepOutcome::Refused {
            member: "UserChangeRefused::NoFileVersionToChange".to_string()
        },
        "走空之后拒成没有可改的文件"
    );
    assert!(
        counts.required_refusals_matched >= 1,
        "模型判过这一次该拒：{counts:?}"
    );
}

/// 回退到意图那一版之后再 unlink：同上，会话先走空、再拒成 `NoFileVersionToChange`，模型逐版比、整段跑完。
/// 判别力：执行器在删除中的版本上跳过 unlink 时第 4 步是「前提不满足」，这一项红。
#[test]
fn an_unlink_after_rolling_back_to_the_intent_is_judged_by_the_model_through_the_walked_step() {
    let (ending, outcome, counts) =
        history_after_rolling_back_to_the_intent_then(HistoryOperation::Unlink);
    assert_eq!(ending, HistoryEnding::Completed);
    assert_eq!(
        outcome,
        StepOutcome::Refused {
            member: "UserChangeRefused::NoFileVersionToChange".to_string()
        },
        "走空之后拒成没有可改的文件"
    );
    assert!(
        counts.required_refusals_matched >= 1,
        "模型判过这一次该拒：{counts:?}"
    );
}

/// 回退到意图那一版之后建 inode：会话先走空、再建，做成；模型比过走空那一版与建的那一版，整段跑完。
/// 判别力：执行器在删除中的版本上跳过建 inode 时第 4 步是「前提不满足」，这一项红。
#[test]
fn creating_an_inode_after_rolling_back_to_the_intent_walks_the_queue_first_and_is_judged_by_the_model(
) {
    let (ending, outcome, _counts) =
        history_after_rolling_back_to_the_intent_then(HistoryOperation::CreateInode);
    assert_eq!(ending, HistoryEnding::Completed);
    assert!(
        matches!(
            outcome,
            StepOutcome::Applied(AppliedEffect::Published { .. })
        ),
        "走空之后建 inode 做成：{outcome:?}"
    );
}
