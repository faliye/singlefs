//! 池级 checker 不重走别的版本先走过的单元，按「单元 → 它下面的落点」那张表把落点并进后走的版本（I-5.1 共享子树那一格，
//! `walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。一个单元在先走的那一版里只从快照那两棵树走得到、
//! 它的孩子又是那一版活头已经记过的落点时，孩子照样挂到它下面：后走的那一版若是建快照之后、活头改写那片单元之前的一版，
//! 那个单元在它那里是活头自己的，少挂一个孩子，那一版的引用集合就少一个落点、走读又不记失败，I-3.5（引用区间的精确性）
//! 把那个落点当成下一版新出现的（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9「种子下标 1032 那一形」，攻方 T4）。
//!
//! 合法历史从写路径来（随机历史执行器，每一步模型对得上）：攻方去掉回退那一组比重的种子下标 1032，40 步按块删到再删一步就不红的 16 步。
//! 没有回退；有建快照、unlink、几次可写挂载、批间崩溃再挂载的整理（后一趟搬了一个单元）、删快照。老的候选根按根环槽的次序走，
//! 根环转过一圈之后，建快照之后活头改写那片 extent 叶的那一版先于它前一版被走到，误红才出现（修前红在最后一步：
//! (实例 2、txg 8) → (实例 2、txg 9)，盘 0 槽 50182，标签是活头 extent 树 65536）。
use singlefs_checker::image::{InvariantVerdict, REFERENCE_INTERVALS_ARE_EXACT};
use singlefs_checker::walk::check_pool_image;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, CompactionChoice, CompactionRunEnding, ContentChoice,
    ContentLength, GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, SnapshotChange, SnapshotChoice, StepOutcome, StepPosition,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

/// 攻方种子区间的起点（`research/prompts/m4-segment-code-r1-opus-model/src/attack_common/weights.rs` 的 `ATTACK_SEED_OFFSET`）
/// 加种子下标 1032：只当这段历史的名字，执行不读它。
const SEED_OF_THE_ATTACK_HISTORY: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7_000_000 + 1032;

/// 这段历史里建快照与删快照那几步在操作序列里的下标，连同各自该做成哪一种。
const SNAPSHOT_STEPS: [(usize, SnapshotChange); 7] = [
    (3, SnapshotChange::Created),
    (7, SnapshotChange::Created),
    (9, SnapshotChange::Destroyed),
    (11, SnapshotChange::Destroyed),
    (12, SnapshotChange::Created),
    (13, SnapshotChange::Created),
    (14, SnapshotChange::Destroyed),
];

/// 攻方种子下标 1032 缩出的 16 步（去掉回退那一组比重生成 40 步，留第 0、1、4、5、8、9、24、25、26、27、28、29、35、36、37、38 步），
/// 取值照 `attack_minimize` 打出的那一串原样抄。
fn snapshots_created_and_destroyed_around_writable_mounts_and_crashed_compactions(
) -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_OF_THE_ATTACK_HISTORY),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: 4_908_652_545_014_035_923,
                },
                fill_seed: 1_175_750_450_067_266_269,
            }),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 152,
                    selector: 1_112_582_400_669_913_327,
                },
                fill_seed: 4_144_227_704_568_475_653,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::Unlink,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                region_selector: 9_802_755_752_694_277_730,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 },
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 3,
                region_selector: 16_953_606_325_372_812_583,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
            }),
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 1_939_626_785_022_617_838,
            }),
            HistoryOperation::CreateInode,
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 10_045_164_462_792_776_581,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 849_968_957_743_593_146,
            }),
            HistoryOperation::CreateInode,
        ],
    }
}

/// 判别力：快照那两棵树里这一版已经从活头记过的落点改回不挂父单元（`crates/mutations.tsv` 里「m4 修复批二丙二」那一行），
/// 最后一步的镜像上 (txg 8) → (txg 9) 判红 I-3.5，这条红。
#[test]
fn snapshots_created_and_destroyed_around_writable_mounts_and_crashed_compactions_keep_every_invariant_at_every_step(
) {
    let history = snapshots_created_and_destroyed_around_writable_mounts_and_crashed_compactions();
    let mut violations: Vec<(StepPosition, &'static str, String)> = Vec::new();
    let mut reference_interval_verdict_of_the_last_step: Option<InvariantVerdict> = None;
    let last_position = StepPosition::Operation(history.operations.len() - 1);
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            for (name, verdict) in check_pool_image(observation.image) {
                if let InvariantVerdict::Violated(why) = &verdict {
                    violations.push((observation.position, name, why.clone()));
                }
                if name == REFERENCE_INTERVALS_ARE_EXACT && observation.position == last_position {
                    reference_interval_verdict_of_the_last_step = Some(verdict);
                }
            }
        },
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "这段历史每一步都做成或按前提不适用、模型对得上：{:?}",
        run.outcomes
    );
    for (step, change) in SNAPSHOT_STEPS {
        assert!(
            matches!(
                &run.outcomes[step],
                StepOutcome::Applied(AppliedEffect::SnapshotPublished { change: done, .. }) if *done == change
            ),
            "第 {step} 步的快照操作做成 {change:?}：{:?}",
            run.outcomes[step]
        );
    }
    assert_eq!(
        violations,
        Vec::<(StepPosition, &'static str, String)>::new(),
        "合法历史的每一步镜像上池级 checker 一条都不红：快照独有的父单元下面挂着它与活头共享的孩子，建快照之前那几版并进它时引用集合不少落点（I-3.5 不误红），共享的孩子也不被当成第二次引用（I-5.1 不误红）"
    );
    assert_eq!(
        reference_interval_verdict_of_the_last_step,
        Some(InvariantVerdict::Holds),
        "最后一步的镜像上 I-3.5 判到了单元、成立（不是不适用）"
    );
}
