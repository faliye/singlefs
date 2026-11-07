//! n_live 与 n_记录（记账标签 6 的两项，D3（空间分配） 已定项 2）随一版带着、每次发布在提交时增量维护（`singlefs_core::admission::InodeRecordCounts`），
//! 准入读它、不遍历 inode 树（`.claude/rules/fs-design.md`「空间统计与配额落在第一格：必须在事务提交时增量维护出来」）。
//! 对照的那一份在这里自己数（[`counted_here`]，逐片逐条数 nlink ≥ 1 的与全部的），不调实现里冷恢复数起始值的那个函数：
//! 运行时与对照不用同一段代码（同一份规则「运行时与 checker 必须用不同的算法」）。
//! 两条写死的：意图那次之后（删除中的那条算进 n_记录、不算进 n_live），维护的与从盘上重建的都对；删到底之后那条记录两边都少了。
//! 随机历史里逐版对照：每一步之后会话现行那一版记着的两个数与它的 inode 树上现数的相等。每组比重一条用例：偏向删除那一组与 broad 那一组
//! （建文件、覆盖写、建 inode、unlink（意图与走空在同一步里做完）、回退、冷启动读回与可写挂载），截断、建删快照、挂着时整理、按偏移写各一组，
//! 再加按 inode 号写与别的 inode 改 size 两组（一步至多建 160 个 inode，段长放到 inode 树长出两片以上叶容器，之后改写最右那片之外的记录：
//! 发布按叶序折半找上一版那条记录要跨片找）。维护那一边的每一条来路（写进、删掉、回退照抄 R_old 那一版的、冷恢复数起始值）都走到。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::collections::BTreeMap;

use common_deletion_in_progress::{pool_with_new_inodes, publish_the_intent_of, unlink_to_the_end};
use singlefs_core::admission::InodeRecordCounts;
use singlefs_core::records::{InodeRecord, NLINK_OF_AN_INODE_BEING_DELETED};
use singlefs_core::recovery::{rebuild_version, RebuiltVersion};
use singlefs_core::transaction::TransactionOutput;
use singlefs_verification::heavy_threads::{
    own_variable_or_default_heavy_threads, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryEnding,
    HistoryExecution, HistoryOperationKind, HistorySeed, StepOutcome,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_verification::SharedStream;

/// 每组比重跑的种子数，写死。
const SEEDS_PER_WEIGHTS: u64 = 48;

/// 偏向删除那一组与 broad 那一组每段的步数，写死。
const OPERATIONS_PER_HISTORY_DELETING_FILES_OR_BROAD: usize = 30;

/// 截断、建删快照、挂着时整理、按偏移写那四组每段的步数：与它们各自对模型的那份用例同长（每段 40 步）。
const OPERATIONS_PER_HISTORY_WITH_TRUNCATIONS_SNAPSHOTS_COMPACTION_OR_PARTIAL_WRITES: usize = 40;

/// 按 inode 号写与别的 inode 改 size 那两组每段的步数：建 inode 那一步一半至多建 160 个、压到最右那片装得下的条数，一片 233 条，
/// 段长放到这么长，inode 树多半长出两片以上叶容器、之后还剩几步改写最右那片之外的记录（用例里断言确实长出了、确实改写了）。
const OPERATIONS_PER_HISTORY_GROWING_SEVERAL_INODE_LEAF_CONTAINERS: usize = 60;

/// 改写已有 inode 记录的几类操作（覆盖写第一个文件、按 inode 号写、按偏移写、截断）：数「改写了最右那片之外的记录」只认它们，
/// 回退与挂载换一版不经发布按叶序找上一版的记录。
const KINDS_REWRITING_INODE_RECORDS: [HistoryOperationKind; 4] = [
    HistoryOperationKind::PublishOverwrite,
    HistoryOperationKind::WriteOfAnInode,
    HistoryOperationKind::WriteAtOffset,
    HistoryOperationKind::Truncate,
];

/// 一处自己的线程变量从进程环境取：设了照它，没设取 `SINGLEFS_THREAD_CAP`，再没设取登记的每台默认重活线程数
/// （判法在 `singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads`）。变量名写在调用的那一条语句里：
/// 门禁 code-tooling 的 heavy-thread-defaults 格按读线程变量的那一条语句认它有没有回退到登记值。
///
/// # Panics
/// 自己的变量设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn own_variable_or_default_heavy_threads_from_the_process_environment(
    own_variable: &str,
) -> std::num::NonZeroUsize {
    own_variable_or_default_heavy_threads(
        own_variable,
        std::env::var(own_variable).ok().as_deref(),
        std::env::var(THREAD_CAP_ENVIRONMENT_VARIABLE)
            .ok()
            .as_deref(),
    )
}

/// 在一版的 inode 树上逐片逐条数：nlink ≥ 1 的记录数与全部记录数（对照用，与实现的数法分开写）。
fn counted_here(version: &TransactionOutput) -> InodeRecordCounts {
    let mut live_objects: u64 = 0;
    let mut records: u64 = 0;
    for container in &version.inode_leaf_containers {
        for record in &container.contents.records {
            records += 1;
            if record.nlink != NLINK_OF_AN_INODE_BEING_DELETED {
                live_objects += 1;
            }
        }
    }
    InodeRecordCounts {
        live_objects,
        records,
    }
}

/// 一版 inode 树最右那片叶容器之外各片里的全部记录（只有一片叶容器时是空的）：下一步改写了其中一条，发布按叶序找上一版那条记录就要跨片找。
fn records_outside_the_rightmost_inode_leaf_container(
    version: &TransactionOutput,
) -> Vec<InodeRecord> {
    let Some((_rightmost, containers_left_of_the_rightmost)) =
        version.inode_leaf_containers.split_last()
    else {
        return Vec::new();
    };
    containers_left_of_the_rightmost
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect()
}

/// `records_before` 里有没有一条在 `version` 里还在、内容换了（被这一步改写过）。删掉的不算：多片叶容器的树上 unlink 不调。
fn rewrites_a_record_among(records_before: &[InodeRecord], version: &TransactionOutput) -> bool {
    let records_now: BTreeMap<u64, InodeRecord> = version
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| (record.inode, *record))
        .collect();
    records_before.iter().any(|record_before| {
        records_now
            .get(&record_before.inode)
            .is_some_and(|record_now| record_now != record_before)
    })
}

/// 从盘上重建 `version` 那一版（冷恢复读出它的那一条路，n_live 与 n_记录的起始值在这里数一遍）。
fn rebuilt_from_disk(pool: &common::BuiltPool, version: &TransactionOutput) -> TransactionOutput {
    match rebuild_version(
        &pool.memory_pool(),
        &version.root,
        Some(version.record.clone()),
    )
    .expect("从盘上重建")
    {
        RebuiltVersion::WithFile(rebuilt) | RebuiltVersion::WithTreesAndNoFile(rebuilt) => rebuilt,
        RebuiltVersion::WithoutFile => panic!("有树的一版"),
    }
}

/// 第一个文件加两个空 inode（号 2、3，与第一个文件同在最左那片叶里），unlink 号 3（那片叶的最后一条）的意图那次之后：三条记录、两条活的。
/// 发布维护出来的与从盘上重建时数出来的都是这两个数。判别力：意图那次把活的换成删除中的不减 n_live，按叶序找上一版那条记录找不到片里的
/// 最后一条（当成新记录、n_记录多一），或重建时把 nlink = 0 的也算活的，这里就红。
#[test]
fn the_record_being_deleted_counts_among_the_records_but_not_among_the_live_objects() {
    let mut pool = pool_with_new_inodes("inode-record-counts-after-the-intent", 2);
    let intent = publish_the_intent_of(&mut pool, 3);
    let expected = InodeRecordCounts {
        live_objects: 2,
        records: 3,
    };
    assert_eq!(
        counted_here(&intent),
        expected,
        "前提：意图之后三条记录、两条活的"
    );
    assert_eq!(intent.inode_record_counts, expected, "发布维护出来的");
    assert_eq!(
        rebuilt_from_disk(&pool, &intent).inode_record_counts,
        expected,
        "冷恢复从盘上读出这一版时数出来的"
    );
}

/// 同上的池，把号 3 删到底之后：两条记录、两条活的（意图那次 n_live 少一，最后一批 n_记录少一）。
#[test]
fn unlinking_to_the_end_takes_the_removed_record_out_of_the_records() {
    let mut pool = pool_with_new_inodes("inode-record-counts-after-unlinking-to-the-end", 2);
    unlink_to_the_end(&mut pool, 3);
    let emptied = pool.output.clone();
    let expected = InodeRecordCounts {
        live_objects: 2,
        records: 2,
    };
    assert_eq!(
        counted_here(&emptied),
        expected,
        "前提：删到底之后两条记录、都活着"
    );
    assert_eq!(emptied.inode_record_counts, expected, "发布维护出来的");
    assert_eq!(
        rebuilt_from_disk(&pool, &emptied).inode_record_counts,
        expected,
        "冷恢复从盘上读出这一版时数出来的"
    );
}

/// 一段历史里逐步对照下来的：对照了几版；按这一步的操作种类数、只数这一步做成了的（入口返回 Ok）之后对照到的几版；
/// 记着的记录数比上一步少的有几次（最后一批删掉那条记录）；inode 树有两片以上叶容器的几版；改写已有记录的那几类操作里，
/// 上一步那一版有两片以上叶容器、这一步改写了最右那片之外的记录的几次（发布按叶序找上一版那条记录跨片找到的）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ComparedVersions {
    versions: u64,
    after_an_applied_step_of_each_kind: BTreeMap<HistoryOperationKind, u64>,
    records_went_down: u64,
    versions_over_several_inode_leaf_containers: u64,
    rewrites_of_a_record_outside_the_rightmost_inode_leaf_container: u64,
}

impl ComparedVersions {
    fn plus(self, other: Self) -> Self {
        let mut after_an_applied_step_of_each_kind = self.after_an_applied_step_of_each_kind;
        for (kind, versions) in other.after_an_applied_step_of_each_kind {
            *after_an_applied_step_of_each_kind.entry(kind).or_default() += versions;
        }
        Self {
            versions: self.versions + other.versions,
            after_an_applied_step_of_each_kind,
            records_went_down: self.records_went_down + other.records_went_down,
            versions_over_several_inode_leaf_containers: self
                .versions_over_several_inode_leaf_containers
                + other.versions_over_several_inode_leaf_containers,
            rewrites_of_a_record_outside_the_rightmost_inode_leaf_container: self
                .rewrites_of_a_record_outside_the_rightmost_inode_leaf_container
                + other.rewrites_of_a_record_outside_the_rightmost_inode_leaf_container,
        }
    }

    /// 这几类操作做成了之后对照到的版数之和。
    fn after_an_applied_step_of(&self, kinds: &[HistoryOperationKind]) -> u64 {
        kinds
            .iter()
            .map(|kind| {
                self.after_an_applied_step_of_each_kind
                    .get(kind)
                    .copied()
                    .unwrap_or(0)
            })
            .sum()
    }
}

/// 跑一段历史，每一步之后拿会话现行那一版记着的两个数与这里现数的比；对不上就带着种子与步号 panic。
fn compare_the_counts_of_every_version_in_one_history(
    seed: HistorySeed,
    weights: &GenerationWeights,
    operations_per_history: usize,
) -> ComparedVersions {
    let history = generate_history_with_weights(seed, operations_per_history, weights);
    let mut compared = ComparedVersions::default();
    let mut records_at_the_previous_step: Option<u64> = None;
    let mut records_outside_the_rightmost_at_the_previous_step: Vec<InodeRecord> = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            let Some(version) = observation.current_version_of_trees else {
                records_at_the_previous_step = None;
                records_outside_the_rightmost_at_the_previous_step = Vec::new();
                return;
            };
            let counted = counted_here(version);
            assert_eq!(
                version.inode_record_counts, counted,
                "种子 {seed:?}、{:?}（{:?}）之后：记着的 n_live 与 n_记录与现数的不等",
                observation.position, observation.operation
            );
            compared.versions += 1;
            let applied_kind = match observation.outcome {
                Some(StepOutcome::Applied(_)) => {
                    observation.operation.map(|operation| operation.kind())
                }
                Some(StepOutcome::Refused { .. } | StepOutcome::NotApplicable(_)) | None => None,
            };
            if let Some(kind) = applied_kind {
                *compared
                    .after_an_applied_step_of_each_kind
                    .entry(kind)
                    .or_default() += 1;
                if KINDS_REWRITING_INODE_RECORDS.contains(&kind)
                    && rewrites_a_record_among(
                        &records_outside_the_rightmost_at_the_previous_step,
                        version,
                    )
                {
                    compared.rewrites_of_a_record_outside_the_rightmost_inode_leaf_container += 1;
                }
            }
            if version.inode_leaf_containers.len() >= 2 {
                compared.versions_over_several_inode_leaf_containers += 1;
            }
            if records_at_the_previous_step.is_some_and(|before| counted.records < before) {
                compared.records_went_down += 1;
            }
            records_at_the_previous_step = Some(counted.records);
            records_outside_the_rightmost_at_the_previous_step =
                records_outside_the_rightmost_inode_leaf_container(version);
        },
    );
    assert!(
        matches!(run.ending, HistoryEnding::Completed),
        "种子 {seed:?}：历史没跑完（{:?}），这一段对照不全",
        run.ending
    );
    compared
}

/// 一组比重下的全部种子，每段 `operations_per_history` 步，按种子区间切片、几个线程各跑一片（每段历史各建自己的池），合起来按片的次序相加。
fn compare_the_counts_over_the_seeds(
    weights: &GenerationWeights,
    operations_per_history: usize,
) -> ComparedVersions {
    // 随机历史那几条用例自己的线程数变量
    let threads = own_variable_or_default_heavy_threads_from_the_process_environment(
        "SINGLEFS_INODE_RECORD_COUNTS_HISTORY_THREADS",
    );
    let thread_count = u64::try_from(threads.get()).expect("线程数装得进 u64");
    let seeds_per_thread = SEEDS_PER_WEIGHTS.div_ceil(thread_count);
    let compared_of_each_slice: Vec<ComparedVersions> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..thread_count)
            .map(|slice| {
                let first_seed = (slice * seeds_per_thread).min(SEEDS_PER_WEIGHTS);
                let end_seed = (first_seed + seeds_per_thread).min(SEEDS_PER_WEIGHTS);
                scope.spawn(move || {
                    (first_seed..end_seed).fold(ComparedVersions::default(), |sum, offset| {
                        sum.plus(compare_the_counts_of_every_version_in_one_history(
                            HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + offset),
                            weights,
                            operations_per_history,
                        ))
                    })
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("对照的那一片没有 panic"))
            .collect()
    });
    compared_of_each_slice
        .into_iter()
        .fold(ComparedVersions::default(), ComparedVersions::plus)
}

/// 偏向删除的那组比重：unlink（意图与走空）、删完之后建 inode、回退、可写挂载都抽得到。
/// 判别力：发布路径改 n_live 或 n_记录漏一样（例如建 inode 时 n_记录不加一、最后一批删掉时 n_live 照减），第一个走到那里的版本就对不上。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_histories_deleting_files(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
        OPERATIONS_PER_HISTORY_DELETING_FILES_OR_BROAD,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::Unlink]) > 0,
        "unlink 做成之后的版本对照到了：{compared:?}"
    );
    assert!(
        compared.after_an_applied_step_of(&[
            HistoryOperationKind::CreateInode,
            HistoryOperationKind::CreateInodes
        ]) > 0,
        "建 inode 做成之后的版本对照到了：{compared:?}"
    );
    assert!(
        compared.records_went_down > 0,
        "最后一批删掉记录的那一步对照到了：{compared:?}"
    );
}

/// broad 那组比重（建文件、零单元发布、覆盖写、抬 F、unlink、unlink 之后建 inode、回退、冷启动读回、可写挂载、崩溃恢复抛弃根都抽；
/// 不抽快照与整理，`GenerationWeights::BROAD`）：同上逐版对照。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_broad_histories(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::BROAD,
        OPERATIONS_PER_HISTORY_DELETING_FILES_OR_BROAD,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::CloseAndMountWritable]) > 0,
        "关掉再可写挂载（冷恢复数起始值）之后的版本对照到了：{compared:?}"
    );
}

/// 截断那一组比重（`GenerationWeights::TRUNCATION`：截断的意图与各批、快照下截断、回退到截断半路）：同上逐版对照。
/// 判别力：发布路径把换掉的那一条记录不按上一版的找（每次改写都当成新记录、n_记录多一），截断做成的第一版就对不上。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_histories_with_truncations(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::TRUNCATION,
        OPERATIONS_PER_HISTORY_WITH_TRUNCATIONS_SNAPSHOTS_COMPACTION_OR_PARTIAL_WRITES,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::Truncate]) > 0,
        "截断做成之后的版本对照到了：{compared:?}"
    );
}

/// 建快照、读快照、删快照那一组比重（`GenerationWeights::SNAPSHOT_OPERATIONS`）：同上逐版对照。
/// 判别力：发布路径改写一条活着的记录时 n_live 只减不加（换掉的活的减一、写进的活的不加一），覆盖写做成的第一版就对不上。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_histories_with_snapshots(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::SNAPSHOT_OPERATIONS,
        OPERATIONS_PER_HISTORY_WITH_TRUNCATIONS_SNAPSHOTS_COMPACTION_OR_PARTIAL_WRITES,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::CreateSnapshot]) > 0,
        "建快照做成之后的版本对照到了：{compared:?}"
    );
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::DestroySnapshot]) > 0,
        "删快照做成之后的版本对照到了：{compared:?}"
    );
}

/// 挂着时整理那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED`）：同上逐版对照。
/// 判别力：发布路径写进一条新的活记录时 n_live 不加一，第一个文件那一版就对不上。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_histories_compacting_while_mounted(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::COMPACTION_WHILE_MOUNTED,
        OPERATIONS_PER_HISTORY_WITH_TRUNCATIONS_SNAPSHOTS_COMPACTION_OR_PARTIAL_WRITES,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::CompactWhileMounted]) > 0,
        "挂着时整理做成之后的版本对照到了：{compared:?}"
    );
}

/// 按偏移写那一组比重（`GenerationWeights::PARTIAL_WRITES`）：同上逐版对照。
/// 判别力：n_live 判活不按 nlink = 0 判（nlink 1 的也不算活），第一个文件那一版就对不上。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_histories_with_partial_writes(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::PARTIAL_WRITES,
        OPERATIONS_PER_HISTORY_WITH_TRUNCATIONS_SNAPSHOTS_COMPACTION_OR_PARTIAL_WRITES,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::WriteAtOffset]) > 0,
        "按偏移写做成之后的版本对照到了：{compared:?}"
    );
}

/// 按 inode 号写那一组比重（`GenerationWeights::WRITES_BY_INODE_NUMBER`，一步至多建 160 个 inode），每段放长到 inode 树长出两片以上叶容器：
/// 同上逐版对照，并断言确实对照到了两片以上叶容器的版本、确实有一步改写了最右那片之外的记录（发布按叶序找上一版那条记录跨片找）。
/// 判别力：按叶序找上一版那条记录恒取最右那片（`TransactionOutput::inode_record_found_by_its_leaf`），最右那片之外的记录找不到、当成新记录，
/// 改写它的那一版 n_记录多一、就对不上。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_histories_writing_by_inode_number_over_several_inode_leaf_containers(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::WRITES_BY_INODE_NUMBER,
        OPERATIONS_PER_HISTORY_GROWING_SEVERAL_INODE_LEAF_CONTAINERS,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::WriteOfAnInode]) > 0,
        "按 inode 号写做成之后的版本对照到了：{compared:?}"
    );
    assert!(
        compared.versions_over_several_inode_leaf_containers > 0,
        "inode 树长出了两片以上叶容器、那几版对照到了：{compared:?}"
    );
    assert!(
        compared.rewrites_of_a_record_outside_the_rightmost_inode_leaf_container > 0,
        "两片以上叶容器的树上改写最右那片之外的记录（跨片找上一版那条记录）对照到了：{compared:?}"
    );
}

/// 别的 inode 改 size 那一组比重（`GenerationWeights::SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST`：截断点名别的 inode、按 inode 号写、
/// 一步至多建 160 个 inode），每段放长到 inode 树长出两片以上叶容器：同上逐版对照，并断言确实对照到了两片以上叶容器的版本、
/// 确实有一步改写了最右那片之外的记录。
/// 判别力：按叶序找上一版那条记录时差一片、取它左邻那片，最左那片之外的记录找不到、当成新记录，改写它的那一版 n_记录多一、就对不上。
#[test]
fn inode_record_counts_kept_by_each_publish_equal_the_counts_over_the_inode_tree_in_histories_changing_sizes_of_other_inodes_over_several_inode_leaf_containers(
) {
    let compared = compare_the_counts_over_the_seeds(
        &GenerationWeights::SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST,
        OPERATIONS_PER_HISTORY_GROWING_SEVERAL_INODE_LEAF_CONTAINERS,
    );
    assert!(compared.versions > 0, "对照到了版本：{compared:?}");
    assert!(
        compared.after_an_applied_step_of(&[HistoryOperationKind::Truncate]) > 0,
        "截断做成之后的版本对照到了：{compared:?}"
    );
    assert!(
        compared.versions_over_several_inode_leaf_containers > 0,
        "inode 树长出了两片以上叶容器、那几版对照到了：{compared:?}"
    );
    assert!(
        compared.rewrites_of_a_record_outside_the_rightmost_inode_leaf_container > 0,
        "两片以上叶容器的树上改写最右那片之外的记录（跨片找上一版那条记录）对照到了：{compared:?}"
    );
}
