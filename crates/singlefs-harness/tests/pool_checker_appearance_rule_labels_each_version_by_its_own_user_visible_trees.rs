//! 池级 checker I-3.5（引用区间的精确性）出现规则 (b) 款认「b 落在 R 树表里某棵用户可见树的子树里」，按 R 这一版自己走读到的落点与标签认，
//! 不取跨全部版本、按落点查的一张表（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9，攻方 T4）。
//!
//! 合法历史从写路径来（随机历史执行器，每一步模型对得上）：无快照时回退把第一个文件的数据单元复活进回退那一版的活头 extent 树，
//! 再建快照、在快照下覆盖写，那个单元只剩快照那两棵树走得到。最新那一版先走、经快照那两棵树走到它（快照树不是用户可见树），
//! 回退那一版共享这片子树、不重走：跨版本的表里它没有标签，回退那一版的 (b) 款落空、I-3.5 误红（攻方种子下标 2022 缩出的 5 步，
//! `research/prompts/m4-segment-code-r1-opus-model/out/i35-seed2022.txt`）。
use singlefs_checker::image::{InvariantVerdict, REFERENCE_INTERVALS_ARE_EXACT};
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, ContentChoice, ContentLength, GeneratedHistory,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    RollbackTargetChoice, SnapshotChange, StepOutcome, StepPosition,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

/// 攻方种子区间的起点（`research/prompts/m4-segment-code-r1-opus-model/src/attack_common/weights.rs` 的 `ATTACK_SEED_OFFSET`）
/// 加种子下标 2022：只当这段历史的名字，执行不读它。
const SEED_OF_THE_ATTACK_HISTORY: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7_000_000 + 2022;

/// 攻方种子下标 2022 缩出的 5 步（快照那一组比重生成，留第 5、21、22、26、27 步）：挂载、一个数据单元以内的覆盖写、
/// 回退到根环里第三新的那条根（txg 4，覆盖写之前那一版）、建快照、跨 291 个数据单元的顺序写。
fn rollback_without_snapshots_then_a_snapshot_then_an_overwrite_under_it() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_OF_THE_ATTACK_HISTORY),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: 7_879_751_949_923_199_194,
                },
                fill_seed: 17_840_163_520_990_143_491,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 2,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 291,
                    selector: 7_664_500_909_322_504_686,
                },
                fill_seed: 4_676_315_349_287_084_229,
            }),
        ],
    }
}

/// 判别力：I-3.5 的标签改回跨全部版本那张表（`crates/mutations.tsv` 里「m4 修复批丙 F9」那一行），最后一步的镜像上
/// 回退那一版 (txg 7) 对它前一版 (txg 6) 判红，这条红。
#[test]
fn a_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it_keeps_the_reference_intervals_exact_at_every_step(
) {
    let history = rollback_without_snapshots_then_a_snapshot_then_an_overwrite_under_it();
    let mut reference_interval_violations: Vec<(StepPosition, String)> = Vec::new();
    let mut reference_interval_verdict_of_the_last_step: Option<InvariantVerdict> = None;
    let last_position = StepPosition::Operation(history.operations.len() - 1);
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            for (name, verdict) in check_pool_image(observation.image) {
                if name != REFERENCE_INTERVALS_ARE_EXACT {
                    continue;
                }
                if let InvariantVerdict::Violated(why) = &verdict {
                    reference_interval_violations.push((observation.position, why.clone()));
                }
                if observation.position == last_position {
                    reference_interval_verdict_of_the_last_step = Some(verdict);
                }
            }
        },
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "这段历史每一步都做成、模型对得上：{:?}",
        run.outcomes
    );
    assert!(
        matches!(
            run.outcomes[2],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "无快照时的回退做成：{:?}",
        run.outcomes[2]
    );
    assert!(
        matches!(
            run.outcomes[3],
            StepOutcome::Applied(AppliedEffect::SnapshotPublished {
                change: SnapshotChange::Created,
                ..
            })
        ),
        "回退之后建快照做成：{:?}",
        run.outcomes[3]
    );
    assert!(
        matches!(
            run.outcomes[4],
            StepOutcome::Applied(AppliedEffect::Published { .. })
        ),
        "快照下的覆盖写做成：{:?}",
        run.outcomes[4]
    );
    assert_eq!(
        reference_interval_violations,
        Vec::<(StepPosition, String)>::new(),
        "合法历史的每一步镜像上 I-3.5 不红：回退那一版复活的单元在它自己的活头 extent 树里，(b) 款按这一版的标签罩住它"
    );
    assert_eq!(
        reference_interval_verdict_of_the_last_step,
        Some(InvariantVerdict::Holds),
        "最后一步的镜像上 I-3.5 判到了单元、成立（不是不适用）"
    );
}
