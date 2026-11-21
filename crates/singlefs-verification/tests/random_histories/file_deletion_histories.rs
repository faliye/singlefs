//! 删文件的历史：删掉占几个下层叶子的文件，每一批与变更之前的走查都由模型比对。

use super::*;

/// 删除多于一片下段叶的文件那一格（代码轮 U13）的取样点：种子区间与每段步数，写死。
const DELETION_SAMPLING_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;

const DELETION_SAMPLING_SEEDS: u64 = 48;

const DELETION_SAMPLING: HistorySampling = HistorySampling {
    operations_per_history: 30,
    weights: GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
    execution: HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
};

/// 删除多于一片下段叶的文件那一格（U13：模型罩多个数据单元的文件、随机历史里出现 ≥ 2 片下段叶的删除批；
/// N3「走空多步」）：比重取 `GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`，种子与步数见常量。
/// 覆盖写的内容另抽跨几个单元的、经会话的顺序写发（多半两片下段叶），unlink 之后建 inode、多回退到第四新的根（意图那一版或第一个删除批那一版）、
/// 再建 inode、覆盖写或 unlink：会话先把队列走空。先判没有新发现，再核这一格真的跑到了：两片下段叶的文件发成过、模型比过不是最后一批的删除批的段数
/// 与会话改动之前先走空走了至少两步的段数都不为 0（「走空被空间拒」在 4 GiB 的盘上走不到，只打数不判）。
#[test]
fn random_histories_deleting_files_of_several_lower_leaves_compare_every_batch_and_the_walks_before_a_change(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        DELETION_SAMPLING_FIRST_SEED,
        DELETION_SAMPLING_SEEDS,
        DELETION_SAMPLING.operations_per_history,
        &DELETION_SAMPLING.weights,
        DELETION_SAMPLING.execution,
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：偏向删除多于一片下段叶的文件的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    let two_lower_leaves = ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
        data_units: 147,
        selector: 0,
    };
    assert!(
        report
            .tally
            .content_lengths_published
            .get(two_lower_leaves.name())
            .copied()
            .unwrap_or(0)
            >= 1,
        "两片下段叶的文件一次都没经顺序写发成：\n{rendered}"
    );
    assert!(
        report
            .tally
            .histories_with_a_deletion_batch_keeping_data_units
            >= 1,
        "没有一段历史里模型比过不是最后一批的删除批（≥ 2 片下段叶）：\n{rendered}"
    );
    assert!(
        report
            .tally
            .histories_with_a_walk_of_several_steps_before_a_user_change
            >= 1,
        "没有一段历史里会话改动之前先走空走了至少两步：\n{rendered}"
    );
}

/// 两片下段叶的文件（150 个数据单元，`UserChange::SequentialWrite` 写的）经会话 unlink：意图那次、一个不是最后一批的删除批
/// （释放尾部 144 个，留 6 个）、最后一批，三版逐版交模型比（代码轮 U13）。判别力：模型把文件当一个数据单元（意图之后的下一步就是最后一批）时，
/// 第一个删除批交回的这一版还带着数据单元、inode 叶与 `deleted_inodes` 根，模型那一版已经没有这几个角色，判「单元的分配代」对不上。
#[test]
fn deleting_a_file_of_two_lower_leaves_publishes_a_batch_keeping_data_units_that_the_model_compares(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 150,
                    selector: 7,
                },
                fill_seed: 11,
            }),
            HistoryOperation::Unlink,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes
            .iter()
            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
        "顺序写与 unlink 都做成：{:?}",
        run.outcomes
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.deletion_batches_keeping_data_units_matched,
            run.tally.histories_with_a_deletion_batch_keeping_data_units,
            counts.file_contents_compared,
        ),
        (1, 1, 2),
        "一个不是最后一批的删除批比过；带文件的两版（起点的第一个文件、150 个单元的那一版）各比过一次整份内容"
    );
}

/// 两片下段叶的文件 unlink 之后建一个 inode（删完的那一版没有文件、回退不接，建过 inode 的才接），再回退到 unlink 意图那一版
/// （第四新的根：建 inode、最后一批、删除批、意图），号回到队列；随后再建一个 inode，会话先走空——一个删除批、最后一批两步——再建 inode，
/// 三版逐版交模型比（N3「走空多步」）。判别力：模型把删除中的那一版的下一步一律当最后一批时，走空的第一步交回的这一版
/// 还带着数据单元，判「单元的分配代」对不上。
#[test]
fn deleting_a_file_of_two_lower_leaves_then_rolling_back_to_its_intent_walks_two_steps_before_the_next_change(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 150,
                    selector: 7,
                },
                fill_seed: 11,
            }),
            HistoryOperation::Unlink,
            HistoryOperation::CreateInode,
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 3,
            }),
            HistoryOperation::CreateInode,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        matches!(
            run.outcomes[3],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退到意图那一版做成：{:?}",
        run.outcomes[3]
    );
    assert!(
        matches!(run.outcomes[4], StepOutcome::Applied(_)),
        "走空之后建 inode 做成：{:?}",
        run.outcomes[4]
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.walks_of_several_steps_before_a_user_change_matched,
            run.tally
                .histories_with_a_walk_of_several_steps_before_a_user_change,
            counts.deletion_batches_keeping_data_units_matched,
        ),
        (1, 1, 2),
        "建 inode 之前走空两步比过一次；不是最后一批的删除批 unlink 里一个、走空里一个"
    );
}

/// 跨三片下段叶的文件（292 个单元，`UserChange::SequentialWrite` 写的）：映射条目多于一片叶装的 294 条，实现的中央映射树从中间切开、
/// 长出根之下的节点（D8（核心索引结构） 已定项 11）；再经会话 unlink：意图那次、两个不是最后一批的删除批（292 → 148 → 4）、最后一批，
/// 逐版交模型比（U13 那几段取样在种子 …115 等六段上撞到的那一形）。判别力：胶水把映射树根之下的节点当成模型罩不到的池而 panic 时，
/// 顺序写那一步就以 panic 收尾。
#[test]
fn a_file_of_three_lower_leaves_grows_the_central_mapping_tree_below_its_root_and_the_model_compares_every_version(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 292,
                    selector: 7,
                },
                fill_seed: 11,
            }),
            HistoryOperation::Unlink,
        ],
    };
    let run = execute_history_without_the_pool_checker(&history);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes
            .iter()
            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
        "顺序写与 unlink 都做成：{:?}",
        run.outcomes
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.deletion_batches_keeping_data_units_matched,
            counts.file_contents_compared,
        ),
        (2, 2),
        "两个不是最后一批的删除批比过；带文件的两版（起点的第一个文件、292 个单元的那一版）各比过一次整份内容"
    );
}
