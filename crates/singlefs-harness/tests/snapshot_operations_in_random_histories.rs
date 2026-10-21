//! 随机历史加建快照、读快照、删快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7、S8 与第七节岔路 1 至 4）：
//! 理想模型把建快照、删快照的意图与每一批当发布推进根环，每一版逐项比活头前驱、快照列表、deadlist 条目数与被钉映射条目数；
//! 读快照读出的内容与模型记的建时内容比；有快照时回退，模型与执行器同样拒。这里钉三样：一段写死的历史（建快照 → 顺序写 → 再建 →
//! 读 → 回退被拒 → 删文件 → 删完文件之后删掉先建的快照 → 建 inode → 读 → 挂载 → 删掉另一个（先一批再最后一批）→ 回退到带快照的那一版被拒）
//! 每一步模型都对得上；
//! 快照那一组比重在一批种子下跑完、三种新操作都做成过；RefFS 翻译这三种操作，读快照的内容与 RefFS 记的相同。

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, ContentChoice,
    ContentLength, GeneratedHistory, GenerationWeights, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    RollbackTargetChoice, SnapshotChange, SnapshotChoice, StepOutcome,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::model::{ObservedReadBack, ObservedSnapshotReadBack};
use singlefs_harness::model_comparison::model_root_key;
use singlefs_harness::reference_filesystem_translation::{
    ReferenceFileSystemCrashMoment, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
};
use singlefs_harness::scenario::first_file_content;
use singlefs_harness::SharedStream;

/// 快照那一组比重的种子区间：从种子基往后的这一段（「要多测别的历史，只往后追加新的种子区间」，`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`
/// 的文档注释），已有各段的偏移都在它之下。
const SEED_OFFSET_OF_THE_SNAPSHOT_SAMPLING: u64 = 40_000;
/// 快照那一组比重随时跑的那一批几段、每段几步：这四段里建快照、读快照、删快照都做成过（种子偏移 0 到 3，跑出来的种子见
/// [`the_snapshot_weights_draw_every_snapshot_operation_and_every_history_agrees_with_the_model`] 的输出），debug 下三十秒上下。
const SNAPSHOT_SAMPLING_SEEDS: u64 = 4;
const SNAPSHOT_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;
/// 慢档那一批：同一个区间从偏移 0 起多跑几段（debug 下两分钟上下）。
const SNAPSHOT_SAMPLING_SEEDS_OF_THE_SLOW_TIER: u64 = 16;

/// 写死历史里那次顺序写：150 个数据单元（两片下段叶加一个下段根，D8（核心索引结构） 已定项 14），删的时候先一个删除批放 144 个、
/// 再最后一批放剩下的 6 个。
fn content_of_two_lower_leaves() -> ContentChoice {
    ContentChoice {
        length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units: 150,
            selector: 77,
        },
        fill_seed: 3,
    }
}

fn snapshot_at(index_among_the_snapshots: u64) -> SnapshotChoice {
    SnapshotChoice {
        index_among_the_snapshots,
    }
}

/// 写死的那一段历史（第一个文件的内容是 `scenario::first_file_content`，一个数据单元）：
/// 0 建快照 S1；1 顺序写 150 个单元（换下第一个文件的四个单元，生在 S1 之前，钉进 deadlist）；2 建快照 S2；3、4 读 S1、S2；
/// 5 回退到根环里第三新的那条（现行那一版带快照：拒）；6 unlink（意图那次、一个删除批、最后一批：换下的生在 S2 之前的全钉住）；
/// 7 删 S1（删完之后有树而文件集合为空：删快照不依赖活头里有没有文件，照样删掉——丙 2 交判第 5 条主 agent 定是缺陷、会话放行；
/// 它那一段是覆盖写钉住的第一个文件那四个单元，放掉，前驱 0 传给 S2）；8 建一个 inode；9 读 S2（文件删了它仍读出那 150 个单元）；
/// 10 可写挂载（写行与暖机带着 S2 与 deadlist）；11 删 S2（它那一段的条目多于 144 条：先一批放按 key 排在最前的 144 条——都是数据单元——
/// 再最后一批放剩下的，模型逐条比）；12 回退到根环里次新的那条（删 S2 不是最后一批的那一版带着销毁中的 S2：拒）。
fn history_with_snapshots_across_an_unlink() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CreateSnapshot,
            HistoryOperation::PublishOverwrite(content_of_two_lower_leaves()),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::ReadSnapshot(snapshot_at(1)),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 2,
            }),
            HistoryOperation::Unlink,
            HistoryOperation::DestroySnapshot(snapshot_at(0)),
            HistoryOperation::CreateInode,
            HistoryOperation::ReadSnapshot(snapshot_at(1)),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::DestroySnapshot(snapshot_at(0)),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 1,
            }),
        ],
    }
}

fn snapshot_read_of(outcome: &StepOutcome) -> &ObservedSnapshotReadBack {
    match outcome {
        StepOutcome::Applied(AppliedEffect::SnapshotRead { read_back, .. }) => read_back,
        other @ (StepOutcome::Applied(_)
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_)) => panic!("要是一次读快照：{other:?}"),
    }
}

fn snapshot_change_of(outcome: &StepOutcome) -> SnapshotChange {
    match outcome {
        StepOutcome::Applied(AppliedEffect::SnapshotPublished { change, .. }) => *change,
        other @ (StepOutcome::Applied(_)
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_)) => panic!("要是一次做成的快照改动：{other:?}"),
    }
}

/// 写死的那一段历史每一步模型都对得上（跑完、没有新发现），而且模型真的比到了：每一版的快照那几样、带 deadlist 的那几版、
/// 三次读快照、删 S2 时不是最后一批的那一批；回退在现行那一版带快照、目标那一版带快照时两次都拒，删快照在删完文件的一版上做成。
/// 判别力：模型改错任一样快照字段（活头前驱、快照列表、deadlist 条目、被钉映射条目数）这一段在那一步判对不上
/// （`crates/mutations.tsv` 里以「m4 快照模型对拍」起头的几行）；会话或模型把删完文件之后的删快照改回拒、一批放的不是按 key 排在最前的
/// 那几条（模型挑反了、实现从后往前放）都在那一步红（以「m4 模型跟上删快照」起头的几行）。
#[test]
fn a_history_creating_reading_and_destroying_snapshots_across_an_unlink_agrees_with_the_model_at_every_version(
) {
    let history = history_with_snapshots_across_an_unlink();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for step in [0, 2] {
        assert_eq!(
            snapshot_change_of(&run.outcomes[step]),
            SnapshotChange::Created,
            "第 {step} 步建快照"
        );
    }
    assert_eq!(
        snapshot_read_of(&run.outcomes[3]),
        &ObservedSnapshotReadBack::File(first_file_content()),
        "S1 读出第一个文件"
    );
    let content_of_the_sequential_write = content_of_two_lower_leaves().bytes();
    for step in [4, 9] {
        assert_eq!(
            snapshot_read_of(&run.outcomes[step]),
            &ObservedSnapshotReadBack::File(content_of_the_sequential_write.clone()),
            "第 {step} 步 S2 读出那次顺序写的 150 个单元"
        );
    }
    assert_eq!(
        run.outcomes[5],
        StepOutcome::Refused {
            member: "RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses(Current)"
                .to_string()
        },
        "现行那一版带快照：回退拒"
    );
    for step in [7, 11] {
        assert_eq!(
            snapshot_change_of(&run.outcomes[step]),
            SnapshotChange::Destroyed,
            "第 {step} 步删快照（第 7 步在删完文件的一版上）"
        );
    }
    assert_eq!(
        run.outcomes[12],
        StepOutcome::Refused {
            member:
                "RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses(Target)"
                    .to_string()
        },
        "目标那一版带着销毁中的快照：回退拒"
    );
    let counts = &run.tally.model_counts;
    assert_eq!(counts.snapshot_read_backs_compared, 3, "三次读快照都比过");
    assert!(
        counts.snapshot_destruction_batches_keeping_the_snapshot_matched >= 1,
        "删 S2 时它那一段多于一批，不是最后一批的那一批比过：{counts:?}"
    );
    assert!(
        counts.versions_with_deadlist_entries_compared >= 10,
        "建快照之后的每一版都带着 deadlist 比过：{counts:?}"
    );
    assert!(
        counts.snapshots_compared >= counts.versions_with_deadlist_entries_compared,
        "带 deadlist 的每一版都带着快照：{counts:?}"
    );
    assert_eq!(
        counts.required_refusals_matched, 2,
        "两次回退都是模型要求的拒"
    );
}

/// 快照那一组比重在它自己那一段种子里跑前 `seeds` 段（每段 [`SNAPSHOT_SAMPLING_OPERATIONS_PER_HISTORY`] 步）：每一段都跑完、
/// 交回打到每种新操作的种子、带快照被拒的回退次数与带 deadlist 比过的版数。
struct SnapshotSampling {
    seeds_applying: std::collections::BTreeMap<HistoryOperationKind, Vec<u64>>,
    rollbacks_refused_for_the_snapshots: u64,
    versions_with_deadlist_entries_compared: u64,
}

fn run_the_snapshot_sampling(seeds: u64) -> SnapshotSampling {
    let mut sampling = SnapshotSampling {
        seeds_applying: std::collections::BTreeMap::new(),
        rollbacks_refused_for_the_snapshots: 0,
        versions_with_deadlist_entries_compared: 0,
    };
    for offset in 0..seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_SNAPSHOT_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            SNAPSHOT_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::SNAPSHOT_OPERATIONS,
        );
        let run = execute_history_with(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |_| {},
        );
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "种子 {seed}（偏移 {offset}）：{:?}",
            history.operations
        );
        for kind in [
            HistoryOperationKind::CreateSnapshot,
            HistoryOperationKind::ReadSnapshot,
            HistoryOperationKind::DestroySnapshot,
        ] {
            if run
                .tally
                .operations_by_kind
                .get(&kind)
                .is_some_and(|tally| tally.applied > 0)
            {
                sampling.seeds_applying.entry(kind).or_default().push(seed);
            }
        }
        sampling.rollbacks_refused_for_the_snapshots += run
            .tally
            .refusals_by_member
            .iter()
            .filter(|(member, _)| {
                member.starts_with(
                    "RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses",
                )
            })
            .map(|(_, count)| *count)
            .sum::<u64>();
        sampling.versions_with_deadlist_entries_compared += run
            .tally
            .model_counts
            .versions_with_deadlist_entries_compared;
    }
    eprintln!(
        "快照那一组比重 {seeds} 段：做成过的种子 {:?}；带快照被拒的回退 {} 次；带 deadlist 比过的版 {} 个",
        sampling.seeds_applying,
        sampling.rollbacks_refused_for_the_snapshots,
        sampling.versions_with_deadlist_entries_compared
    );
    sampling
}

/// 快照那一组比重（`GenerationWeights::SNAPSHOT_OPERATIONS`）在它自己那一段种子下：每一段都跑完（`KNOWN_RED_FORMS` 不新增，清单是空的，
/// 撞到什么都算新发现），建快照、读快照、删快照都做成过，带快照的回退被拒过，模型比过带 deadlist 的版。打到每种新操作的种子逐个打出来。
/// 判别力：比重里三种新操作的份数改成 0、或执行器把删快照的批当成这次改动本身比（`crates/mutations.tsv` 的几行）时这一条红。
#[test]
fn the_snapshot_weights_draw_every_snapshot_operation_and_every_history_agrees_with_the_model() {
    let sampling = run_the_snapshot_sampling(SNAPSHOT_SAMPLING_SEEDS);
    for kind in [
        HistoryOperationKind::CreateSnapshot,
        HistoryOperationKind::ReadSnapshot,
        HistoryOperationKind::DestroySnapshot,
    ] {
        assert!(
            sampling
                .seeds_applying
                .get(&kind)
                .is_some_and(|seeds| !seeds.is_empty()),
            "{kind:?} 在 {SNAPSHOT_SAMPLING_SEEDS} 段里一次都没做成：{:?}",
            sampling.seeds_applying
        );
    }
    assert!(
        sampling.rollbacks_refused_for_the_snapshots >= 1,
        "带快照的回退一次都没被拒过"
    );
    assert!(
        sampling.versions_with_deadlist_entries_compared >= 1,
        "一版带 deadlist 的都没比过"
    );
}

/// 同一组比重、同一段种子多跑几段（慢档）：每一段都跑完、模型每一步都对得上。
#[test]
#[ignore = "harness 慢档用例：快照那一组比重 16 段、每段 40 步，debug 下两分钟上下"]
fn the_snapshot_weights_over_more_seeds_run_every_history_to_the_end_with_the_model_agreeing() {
    let sampling = run_the_snapshot_sampling(SNAPSHOT_SAMPLING_SEEDS_OF_THE_SLOW_TIER);
    assert!(
        sampling.versions_with_deadlist_entries_compared >= 1,
        "一版带 deadlist 的都没比过"
    );
}

/// RefFS 翻译这三种操作（判决 S7「RefFS 加 CreateSnapshot、DestroySnapshot、读快照」）：起点（第一个文件）→ 建快照 → 覆盖写 → 读快照 →
/// 删快照，翻译一路不停、读快照读出的与 RefFS 记的相同；之后崩溃、读回覆盖写的内容判绿。
/// 判别力：读快照那一比反过来、删快照翻成建快照时，翻译在那一步停下，这一条的票不是绿。
#[test]
fn the_reference_filesystem_takes_the_snapshot_operations_and_a_crash_after_them_reading_the_overwrite_is_green(
) {
    let overwrite = ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 200 },
        fill_seed: 7,
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CreateSnapshot,
            HistoryOperation::PublishOverwrite(overwrite),
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::DestroySnapshot(snapshot_at(0)),
        ],
    };
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let mut translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        translation.note_an_observed_step(observation, stream.operation_count());
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    let translated = translation.translate(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    // RefFS 只看读出的内容，根取哪一条不进它的判定。
    let judgement = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        &ObservedReadBack::FileRead {
            root: model_root_key(InstanceGeneration(1), CheckpointTxg(0)),
            content: overwrite.bytes(),
        },
    );
    assert_eq!(
        judgement.vote,
        ReferenceFileSystemVote::Green,
        "建快照、读快照、删快照都翻进了 RefFS，崩溃之后读回覆盖写的内容合法"
    );
}
