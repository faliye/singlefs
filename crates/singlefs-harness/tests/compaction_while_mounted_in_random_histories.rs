//! 随机历史加挂着时整理（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7「对拍：`HistoryOperation` 加 `CompactWhileMounted`
//! （含签发、批、批间崩溃），只比成功操作的内容，另判「GC 不把成功变成 ENOSPC，除非 df 也跟着少报」」与 G8 甲）：会话的分配器装上只供测试的
//! 整理开关（强制批大小与选段），经会话的显式入口推；整理的每一批交理想模型比（内容、快照、deadlist、每个角色的分配代都与搬之前相同），
//! 批间崩溃之后可写挂载一次。这里钉四样：一段写死的历史（挂载 → 建快照 → 整理一批就崩、再挂 → 整理到停 → 快照下覆盖写 → 读快照 →
//! 删快照 → 冷启动）每一步模型都对得上、真的崩在意图半路；整理那一组比重在一批种子下跑完、打到批间崩溃；同一段历史去掉整理的那几步，
//! 每一步之后读出的文件内容逐字相同，有一步只在整理的那一份里因空间不够被拒时，那一刻整理那一份报的空闲（`df`）比另一份少；
//! RefFS 翻译整理那一步不进操作序列，之后崩溃读回覆盖写的内容判绿。

use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, CompactionChoice,
    CompactionRunEnding, ContentChoice, ContentLength, GeneratedHistory, GenerationWeights,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
    HistorySeed, HistoryStartingPoint, PerStepChecker, SnapshotChoice, StepOutcome, StepPosition,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::model::{ObservedReadBack, ObservedSnapshotReadBack};
use singlefs_harness::model_comparison::model_root_key;
use singlefs_harness::reference_filesystem_translation::{
    ReferenceFileSystemCrashMoment, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
};
use singlefs_harness::scenario::first_file_content;
use singlefs_harness::SharedStream;

/// 整理那一组比重的种子区间：从种子基往后的这一段（「要多测别的历史，只往后追加新的种子区间」，`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`
/// 的文档注释），快照那一组在偏移 40000 起的 16 段，这一段在它之后。
const SEED_OFFSET_OF_THE_COMPACTION_SAMPLING: u64 = 50_000;
/// 整理那一组比重随时跑的那一批几段、每段几步。
const COMPACTION_SAMPLING_SEEDS: u64 = 4;
const COMPACTION_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;
/// 慢档那一批：同一个区间从偏移 0 起多跑几段。
const COMPACTION_SAMPLING_SEEDS_OF_THE_SLOW_TIER: u64 = 16;

/// 写死历史里强制选的那一段在候选段（0 号盘上装着未释放记录、不是挂载开着的聚簇段，按段起点升序）里的位置：起点（mkfs 同一个进程里
/// 取号、暖机、第一个文件）之后可写挂载一次，候选是第一个文件的数据单元那一段（50176）与第一个文件那次发布的提交内生块那一段（50240，
/// 挂载开的是 50304），取后一段——装着 extent 根、inode 叶与根这几个读文件要走的节点。
const REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 1;

fn compaction(
    batch_units: u64,
    region_selector: u64,
    ending: CompactionRunEnding,
) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending,
    })
}

fn small_overwrite() -> ContentChoice {
    ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 300 },
        fill_seed: 11,
    }
}

fn snapshot_at(index_among_the_snapshots: u64) -> SnapshotChoice {
    SnapshotChoice {
        index_among_the_snapshots,
    }
}

/// 写死的那一段历史（起点是第一个文件，一个数据单元）：0 可写挂载；1 建快照 S；2 整理第一个文件那次发布的那一段、一批一个、推一批就崩
/// （那一段装着几个节点，意图推到半路）、再可写挂载；3 整理同一段、一批两个、推到停；4 快照下覆盖写；5 读 S（读出第一个文件）；6 删 S；
/// 7 冷启动（读回覆盖写的内容）。
fn history_compacting_the_segment_of_the_first_publish() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CreateSnapshot,
            compaction(
                1,
                REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH,
                CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            ),
            compaction(
                2,
                REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH,
                CompactionRunEnding::RunsUntilItStops,
            ),
            HistoryOperation::PublishOverwrite(small_overwrite()),
            HistoryOperation::ReadSnapshot(snapshot_at(0)),
            HistoryOperation::DestroySnapshot(snapshot_at(0)),
            HistoryOperation::ColdStartRecover,
        ],
    }
}

/// 写死的另一段（起点同上）：0 可写挂载；1 整理第一个文件那次发布的那一段、一批两个、推到停（搬走 extent 根、inode 叶与根这几个读文件要走的节点，
/// 不崩、不重挂，读的就是整理写出的那几版）；2 覆盖写；3 冷启动。
fn history_compacting_the_segment_of_the_first_publish_without_a_crash() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            compaction(
                2,
                REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH,
                CompactionRunEnding::RunsUntilItStops,
            ),
            HistoryOperation::PublishOverwrite(small_overwrite()),
            HistoryOperation::ColdStartRecover,
        ],
    }
}

/// 一步整理的结局：发了几批、停下时意图活不活着、崩没崩。
fn compacted_of(outcome: &StepOutcome) -> (u64, bool, bool) {
    match outcome {
        StepOutcome::Applied(AppliedEffect::Compacted {
            batches_published,
            is_the_intent_live_when_the_run_stopped,
            crashed_then_mounted,
            ..
        }) => (
            *batches_published,
            *is_the_intent_live_when_the_run_stopped,
            *crashed_then_mounted,
        ),
        other @ (StepOutcome::Applied(_)
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_)) => panic!("要是一次做成的整理：{other:?}"),
    }
}

/// 写死的那一段历史每一步模型都对得上（跑完、没有新发现），而且整理真的推到了：第 2 步签发（一次只写 logged_ops 的发布）、发了一批、
/// 崩在意图半路（意图还活着），之后可写挂载做成、重建出的那一版带着 logged_ops 里那一条；第 3 步接着推盘上那一条、至少再发一批，
/// 最后一批同一次发布删意图；模型比过的整理批与发的批一样多、签发与删意图各至少一次、每一版 logged_ops 里的意图都比过；
/// 搬走了用户可见树的根（那一段装着 extent 根与 inode 根）的那几批算新状态、模型比过；快照读出第一个文件。
/// 判别力：模型把整理的一批当成别的发布（重写用户可见的角色、不写固定点、事务号不加）、签发不写 logged_ops 根、删意图之后还留着它，
/// 或执行器把批间崩溃之后的挂载漏掉、把整理的批当走空的一步问，都在那一步判对不上（`crates/mutations.tsv` 里以「m4 整理对拍」「m4 模型跟上意图」
/// 起头的几行）。
#[test]
fn a_history_compacting_the_segment_of_the_first_publish_and_crashing_between_batches_agrees_with_the_model(
) {
    let history = history_compacting_the_segment_of_the_first_publish();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    let (batches_before_the_crash, is_the_intent_live_at_the_crash, crashed) =
        compacted_of(&run.outcomes[2]);
    assert_eq!(batches_before_the_crash, 1, "第 2 步推够一批就崩");
    assert!(crashed, "第 2 步崩了、再可写挂载做成");
    assert!(
        is_the_intent_live_at_the_crash,
        "那一段的节点不止一个、一批一个：崩在意图半路"
    );
    let (batches_after_the_crash, _, crashed_after_the_run) = compacted_of(&run.outcomes[3]);
    assert!(
        batches_after_the_crash >= 1,
        "崩溃之后再整理同一段，剩下的照样搬得动"
    );
    assert!(!crashed_after_the_run, "第 3 步推到停、不崩");
    assert_eq!(
        run.outcomes[5],
        StepOutcome::Applied(AppliedEffect::SnapshotRead {
            snapshot: match &run.outcomes[1] {
                StepOutcome::Applied(AppliedEffect::SnapshotPublished { snapshot, .. }) =>
                    *snapshot,
                other @ (StepOutcome::Applied(_)
                | StepOutcome::Refused { .. }
                | StepOutcome::NotApplicable(_)) => {
                    panic!("第 1 步要是建快照做成：{other:?}")
                }
            },
            read_back: ObservedSnapshotReadBack::File(first_file_content()),
        }),
        "整理之后快照仍读出第一个文件"
    );
    assert_eq!(run.tally.crashes_between_compaction_batches, 1);
    let counts = &run.tally.model_counts;
    assert_eq!(
        counts.compaction_batches_matched,
        batches_before_the_crash + batches_after_the_crash,
        "每一批都交模型比过"
    );
    assert!(
        counts.compaction_intents_issued_matched >= 1,
        "签发交模型比过：{counts:?}"
    );
    assert!(
        counts.compaction_intents_deleted_with_the_last_batch_matched
            + counts.compaction_intents_deleted_without_a_batch_matched
            >= 1,
        "崩溃之前签的那一条在第 3 步删掉、交模型比过：{counts:?}"
    );
    assert!(
        counts.logged_compaction_intents_compared >= 3,
        "签发那一版、崩溃之前那一批、挂载那几版都带着意图：{counts:?}"
    );
    assert!(
        counts.compaction_publishes_relocating_a_user_visible_root_matched >= 1,
        "那一段装着 extent 根与 inode 根：搬它们的那一批算非空、交模型比过：{counts:?}"
    );
}

/// 整理那一组比重在它自己那一段种子里跑前 `seeds` 段：每一段都跑完，交回打到批间崩溃的种子、发了批的整理趟数与模型比过的批数。
struct CompactionSampling {
    seeds_crashing_between_batches: Vec<u64>,
    runs_with_a_batch: u64,
    batches_matched: u64,
    stops: BTreeMap<&'static str, u64>,
}

fn run_the_compaction_sampling(seeds: u64) -> CompactionSampling {
    let mut sampling = CompactionSampling {
        seeds_crashing_between_batches: Vec::new(),
        runs_with_a_batch: 0,
        batches_matched: 0,
        stops: BTreeMap::new(),
    };
    for offset in 0..seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_COMPACTION_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            COMPACTION_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::COMPACTION_WHILE_MOUNTED,
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
            "种子 {seed}（偏移 {offset}）：{:?}；结局 {:?}",
            history.operations,
            run.outcomes
        );
        if run.tally.crashes_between_compaction_batches > 0 {
            sampling.seeds_crashing_between_batches.push(seed);
        }
        sampling.runs_with_a_batch += run.tally.compaction_runs_with_a_batch;
        sampling.batches_matched += run.tally.model_counts.compaction_batches_matched;
        for (stop, count) in &run.tally.compaction_stops {
            *sampling.stops.entry(stop).or_insert(0) += count;
        }
    }
    eprintln!(
        "整理那一组比重 {seeds} 段：批间崩溃的种子 {:?}；发了批的整理 {} 趟、模型比过 {} 批；停机原因 {:?}",
        sampling.seeds_crashing_between_batches,
        sampling.runs_with_a_batch,
        sampling.batches_matched,
        sampling.stops
    );
    sampling
}

/// 整理那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED`）在它自己那一段种子下：每一段都跑完（`KNOWN_RED_FORMS` 不新增，清单是空的，
/// 撞到什么都算新发现），整理发过批、模型比过，至少一段崩在意图半路。打到批间崩溃的种子打出来。
/// 判别力：比重里整理的份数改成 0、执行器不压预算（批间崩溃那一种推到停才崩）时这一条红。
#[test]
fn the_compaction_weights_publish_batches_crash_between_them_and_every_history_agrees_with_the_model(
) {
    let sampling = run_the_compaction_sampling(COMPACTION_SAMPLING_SEEDS);
    assert!(sampling.runs_with_a_batch >= 1, "整理一批都没发过");
    assert!(sampling.batches_matched >= 1, "模型一批都没比过");
    assert!(
        !sampling.seeds_crashing_between_batches.is_empty(),
        "{COMPACTION_SAMPLING_SEEDS} 段里一次都没崩在意图半路"
    );
}

/// 同一组比重、同一段种子多跑几段（慢档）：每一段都跑完、模型每一步都对得上。
#[test]
#[ignore = "harness 慢档用例：整理那一组比重 16 段、每段 40 步"]
fn the_compaction_weights_over_more_seeds_run_every_history_to_the_end_with_the_model_agreeing() {
    let sampling = run_the_compaction_sampling(COMPACTION_SAMPLING_SEEDS_OF_THE_SLOW_TIER);
    assert!(sampling.batches_matched >= 1, "模型一批都没比过");
}

/// 一步之后从盘上冷读出的文件（`recover`，看 journal）：读出的内容、没有文件、读失败。
#[derive(Clone, Debug, PartialEq, Eq)]
enum FileReadFromTheImage {
    File(Vec<u8>),
    NoFile,
    Failed(String),
}

/// 一段历史跑下来，起点之后与每一步之后的样子（观察者只在没判出失败的步上调）：位置、这一步的结局分成哪一类
/// （[`outcome_class`]）、冷读出的文件、会话现行那一版报的每块盘空闲（`df`）。
struct StepSeenInTheTwin {
    position: StepPosition,
    outcome_class: Option<String>,
    file_read: FileReadFromTheImage,
    free_bytes: Option<Vec<(DeviceIdentity, u64)>>,
}

/// 结局按「哪一种做成 / 被哪个成员拒 / 哪个前提不满足」分类，不看带的数（两份历史的 txg 与实例代号不同）。
fn outcome_class(outcome: &StepOutcome) -> String {
    match outcome {
        StepOutcome::Applied(effect) => {
            let effect_name = match effect {
                AppliedEffect::Published { .. } => "Published",
                AppliedEffect::Mounted { .. } => "Mounted",
                AppliedEffect::RaisedFloor { .. } => "RaisedFloor",
                AppliedEffect::RolledBack { .. } => "RolledBack",
                AppliedEffect::Recovered { .. } => "Recovered",
                AppliedEffect::SnapshotPublished { .. } => "SnapshotPublished",
                AppliedEffect::SnapshotRead { .. } => "SnapshotRead",
                AppliedEffect::Compacted { .. } => "Compacted",
            };
            format!("Applied({effect_name})")
        }
        StepOutcome::Refused { member } => format!("Refused({member})"),
        StepOutcome::NotApplicable(precondition) => {
            format!("NotApplicable({})", precondition.name())
        }
    }
}

/// 拒绝的成员是不是空间不够（会话推满抬 F 仍不够、空间准入拒、取不到落点）。
fn is_a_refusal_for_space(outcome_class: &str) -> bool {
    outcome_class.starts_with("Refused(")
        && (outcome_class.contains("NoSpaceAfterRaisingTheFloor")
            || outcome_class.contains("SpaceAdmissionRefused")
            || outcome_class.contains("PlacementRefused"))
}

fn run_noting_every_step(
    history: &GeneratedHistory,
    execution: HistoryExecution,
) -> (HistoryEnding, Vec<StepSeenInTheTwin>) {
    let mut seen = Vec::new();
    let run = execute_history_with(
        history,
        execution,
        &SharedStream::new(),
        &mut |observation| {
            let file_read = match recover(observation.image, JournalPolicy::Consult).outcome {
                RecoveryOutcome::FileRead { content, .. } => FileReadFromTheImage::File(content),
                RecoveryOutcome::NoFile { .. } => FileReadFromTheImage::NoFile,
                RecoveryOutcome::Failed { failure, .. } => {
                    FileReadFromTheImage::Failed(format!("{failure:?}"))
                }
            };
            seen.push(StepSeenInTheTwin {
                position: observation.position,
                outcome_class: observation.outcome.map(outcome_class),
                file_read,
                free_bytes: observation
                    .free_bytes_reported_by_the_current_version
                    .clone(),
            });
        },
    );
    (run.ending, seen)
}

/// 一段历史与去掉整理那几步的同一段比完交回的计数。
struct TwinComparison {
    steps_after_a_compaction_compared: u64,
    refusals_for_space_only_with_the_compaction: u64,
    /// 只在去掉整理那一份因空间不够被拒、整理那一份做成的次数（整理搬空了段，空间够了：G7 那一判只管反方向，这一种照记）。
    successes_only_with_the_compaction: u64,
}

/// 同一段历史跑两份：一份照原样，一份去掉整理的那几步（判决 G7「只比成功操作的内容」「GC 不把成功变成 ENOSPC，除非 df 也跟着少报」）。
/// 原样那一份每一步之后冷读出的文件，要与去掉整理那一份在对应那一步（这一步之前最后一个不是整理的步，没有就是起点）之后读出的逐字相同；
/// 不是整理的每一步两份的结局同一类。有一步只在整理那一份里因空间不够被拒、另一份做成时，那一步之前整理那一份的会话报的空闲（`df`）
/// 要在某块盘上比另一份少（整理把块搬走、源落点进了 defer、R 排他，`df` 跟着少报才不算假性的空间不够）；反过来只在去掉整理那一份因空间不够被拒、
/// 整理那一份做成的照记（整理搬空了段）。分叉之后两份不再逐步比，两份照样都要跑完（模型与实现每一步都对得上）。
fn compare_with_the_same_history_without_the_compaction(
    history: &GeneratedHistory,
    execution: HistoryExecution,
) -> TwinComparison {
    let is_a_compaction = |operation: &HistoryOperation| {
        operation.kind() == HistoryOperationKind::CompactWhileMounted
    };
    let without_the_compaction = GeneratedHistory {
        seed: history.seed,
        starting_point: history.starting_point,
        operations: history
            .operations
            .iter()
            .filter(|operation| !is_a_compaction(operation))
            .copied()
            .collect(),
    };
    let (ending_with, seen_with) = run_noting_every_step(history, execution);
    let (ending_without, seen_without) = run_noting_every_step(&without_the_compaction, execution);
    let seen_without_by_position: BTreeMap<StepPosition, &StepSeenInTheTwin> = seen_without
        .iter()
        .map(|step| (step.position, step))
        .collect();
    let mut comparison = TwinComparison {
        steps_after_a_compaction_compared: 0,
        refusals_for_space_only_with_the_compaction: 0,
        successes_only_with_the_compaction: 0,
    };
    let seed = history.seed.0;
    // 原样那一份的第几步对应去掉整理那一份的哪一步之后：这一步之前（含）最后一个不是整理的步在去掉之后的位置。
    let mut position_without_the_compaction = StepPosition::StartingPoint;
    let mut operations_without_the_compaction_so_far = 0_usize;
    let mut a_compaction_ran = false;
    let mut free_bytes_before_the_step_with: Option<Vec<(DeviceIdentity, u64)>> = None;
    for step_with in &seen_with {
        let free_bytes_before_the_step_without = seen_without_by_position
            .get(&position_without_the_compaction)
            .and_then(|step| step.free_bytes.clone());
        let operation = match step_with.position {
            StepPosition::StartingPoint => None,
            StepPosition::Operation(step_index) => Some(&history.operations[step_index]),
        };
        let is_this_step_a_compaction = operation.is_some_and(is_a_compaction);
        if let (Some(_), false) = (operation, is_this_step_a_compaction) {
            position_without_the_compaction =
                StepPosition::Operation(operations_without_the_compaction_so_far);
            operations_without_the_compaction_so_far += 1;
        }
        a_compaction_ran |= is_this_step_a_compaction;
        let Some(step_without) = seen_without_by_position.get(&position_without_the_compaction)
        else {
            panic!(
                "种子 {seed}：去掉整理那一份没跑到 {position_without_the_compaction:?}（原样那一份的 {:?}）：{ending_without:?}",
                step_with.position
            );
        };
        if !is_this_step_a_compaction && step_with.outcome_class != step_without.outcome_class {
            let with = step_with.outcome_class.as_deref().unwrap_or("");
            let without = step_without.outcome_class.as_deref().unwrap_or("");
            if with.starts_with("Applied(") && is_a_refusal_for_space(without) {
                comparison.successes_only_with_the_compaction += 1;
                break;
            }
            assert!(
                is_a_refusal_for_space(with) && without.starts_with("Applied("),
                "种子 {seed} {:?}：整理那一份 {with}，去掉整理那一份 {without}——两份结局不同只许是一份因空间不够被拒、另一份做成",
                step_with.position
            );
            let reported_less = match (
                &free_bytes_before_the_step_with,
                &free_bytes_before_the_step_without,
            ) {
                (Some(with_compaction), Some(without_compaction)) => {
                    with_compaction.iter().any(|(device, free)| {
                        without_compaction.iter().any(|(other_device, other_free)| {
                            device == other_device && free < other_free
                        })
                    })
                }
                (None, _) | (_, None) => false,
            };
            assert!(
                reported_less,
                "种子 {seed} {:?}：整理把做得成的改动变成了空间不够，而 df 没跟着少报（整理那一份 {:?}，去掉整理那一份 {:?}）",
                step_with.position,
                free_bytes_before_the_step_with,
                free_bytes_before_the_step_without
            );
            comparison.refusals_for_space_only_with_the_compaction += 1;
            break;
        }
        assert_eq!(
            step_with.file_read, step_without.file_read,
            "种子 {seed}：{:?} 之后冷读出的文件与去掉整理那一份的 {position_without_the_compaction:?} 之后不同",
            step_with.position
        );
        if a_compaction_ran {
            comparison.steps_after_a_compaction_compared += 1;
        }
        free_bytes_before_the_step_with = step_with.free_bytes.clone();
    }
    assert_eq!(
        ending_with,
        HistoryEnding::Completed,
        "种子 {seed}：整理那一份没跑完"
    );
    assert_eq!(
        ending_without,
        HistoryEnding::Completed,
        "种子 {seed}：去掉整理那一份没跑完"
    );
    comparison
}

/// 写死那一段不崩的历史与去掉整理那一步的同一段：每一步之后冷读出的文件逐字相同（整理搬走了读文件要走的节点，读照样读得出同一份）。
/// 判别力：整理的一批写到目的地的字节不是搬之前那一份（`crates/mutations.tsv` 的「m4 整理对拍：搬到目的地的字节改了」，位置项的校验和照改过的字节算，
/// journal 点名验证照样过）时，整理那一步之后冷读读不出、与去掉整理那一份不同，红在「冷读出的文件」那一句。
#[test]
fn reads_after_compacting_the_segment_of_the_first_publish_equal_the_reads_of_the_same_history_without_it(
) {
    let comparison = compare_with_the_same_history_without_the_compaction(
        &history_compacting_the_segment_of_the_first_publish_without_a_crash(),
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
    );
    assert_eq!(
        comparison.steps_after_a_compaction_compared, 3,
        "整理那一步与之后的两步都比过"
    );
}

/// 整理那一组比重的那几段种子，各与去掉整理的同一段比（[`compare_with_the_same_history_without_the_compaction`]），4 GiB 的盘：
/// 这几段里空间不够走不到，「只在整理那一份因空间不够被拒」那一判没有对象，比的是每一步之后读出的内容。
#[test]
fn reads_of_the_compaction_weights_equal_the_reads_of_the_same_histories_without_the_compaction() {
    let mut steps_compared = 0;
    let mut refusals_for_space_only_with_the_compaction = 0;
    for offset in 0..COMPACTION_SAMPLING_SEEDS {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_COMPACTION_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            COMPACTION_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::COMPACTION_WHILE_MOUNTED,
        );
        let comparison = compare_with_the_same_history_without_the_compaction(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        );
        steps_compared += comparison.steps_after_a_compaction_compared;
        refusals_for_space_only_with_the_compaction +=
            comparison.refusals_for_space_only_with_the_compaction;
    }
    eprintln!(
        "整理那一组比重 {COMPACTION_SAMPLING_SEEDS} 段与去掉整理的同一段：整理之后比过 {steps_compared} 步；只在整理那一份因空间不够被拒、df 跟着少报的 {refusals_for_space_only_with_the_compaction} 次"
    );
    assert!(steps_compared >= 1, "整理之后一步都没比过");
}

/// 单元区 256 槽的小盘（每块盘四个聚簇段，第一个文件之后写几次就逼近单元区墙）：整理那一组比重的那几段种子在它上面跑。
const COMPACTION_ON_SMALL_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::UnitAreaOf256Slots,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 整理那一组比重的那几段种子在小盘上（[`COMPACTION_ON_SMALL_DEVICES`]），各与去掉整理的同一段比（[`compare_with_the_same_history_without_the_compaction`]）：
/// 两份都跑完、模型每一步都对得上——整理搬了 inode 树根或 extent 树根的那一批算一个新状态（用户 2026-10-15 定「照字节比，搬根算非空」），
/// 会话为空间推抬 F 时模型数的上限与实现的相同（丙 4 报告停下第 3 条：按「整理不改状态」数，模型在种子偏移 0 第 7 步要求拒「要抬的 F 超过上限」、
/// 实现做成）；有一步只在整理那一份里因空间不够被拒时，那一刻整理那一份报的空闲（`df`）在某块盘上比另一份少（判决 G7 攻方 H-G7-2），
/// 这一判至少真判到一次（2026-10-15 跑出来的是偏移 0、1 两段），判到的种子打出来。
/// 判别力：模型不把搬根的那一批算新状态（`crates/mutations.tsv` 以「m4 搬根算非空」起头的几行）时，偏移 0 那一段在会话为空间推抬 F 那一步
/// 判「模型说该拒、实现做成了」。
#[test]
fn space_refusals_only_with_the_compaction_on_small_devices_come_with_less_free_space_reported() {
    let mut steps_compared = 0;
    let mut seeds_with_a_refusal_for_space_only_with_the_compaction = Vec::new();
    let mut seeds_with_a_success_only_with_the_compaction = Vec::new();
    for offset in 0..COMPACTION_SAMPLING_SEEDS {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_COMPACTION_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            COMPACTION_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::COMPACTION_WHILE_MOUNTED,
        );
        let comparison = compare_with_the_same_history_without_the_compaction(
            &history,
            COMPACTION_ON_SMALL_DEVICES,
        );
        steps_compared += comparison.steps_after_a_compaction_compared;
        if comparison.refusals_for_space_only_with_the_compaction > 0 {
            seeds_with_a_refusal_for_space_only_with_the_compaction.push(seed);
        }
        if comparison.successes_only_with_the_compaction > 0 {
            seeds_with_a_success_only_with_the_compaction.push(seed);
        }
    }
    eprintln!(
        "小盘上整理那一组比重 {COMPACTION_SAMPLING_SEEDS} 段与去掉整理的同一段：整理之后比过 {steps_compared} 步；只在整理那一份因空间不够被拒、df 跟着少报的种子 {seeds_with_a_refusal_for_space_only_with_the_compaction:?}；只在去掉整理那一份因空间不够被拒的种子 {seeds_with_a_success_only_with_the_compaction:?}"
    );
    assert!(steps_compared >= 1, "整理之后一步都没比过");
    assert!(
        !seeds_with_a_refusal_for_space_only_with_the_compaction.is_empty(),
        "「GC 不把成功变成 ENOSPC，除非 df 也跟着少报」那一判一次都没判到"
    );
}

/// RefFS 翻译整理那一步（判决 G7：整理不改用户看得见的东西，不进操作序列）：起点（第一个文件）→ 可写挂载 → 整理到停 → 覆盖写，
/// 翻译一路不停；之后崩溃、读回覆盖写的内容判绿。
/// 判别力：整理那一步翻成别的事件（交一次显式发布、或翻译停下）时这一条的票不是绿。
#[test]
fn the_reference_filesystem_takes_the_compaction_and_a_crash_after_it_reading_the_overwrite_is_green(
) {
    let overwrite = small_overwrite();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            compaction(
                2,
                REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH,
                CompactionRunEnding::RunsUntilItStops,
            ),
            HistoryOperation::PublishOverwrite(overwrite),
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
    assert!(
        compacted_of(&run.outcomes[1]).0 >= 1,
        "整理那一步发了批：{:?}",
        run.outcomes[1]
    );
    let translated = translation.translate(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    // RefFS 只看读出的内容，根取哪一条不进它的判定。
    let judgement = translated.judge_a_crash(
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
        &ObservedReadBack::FileRead {
            root: model_root_key(InstanceGeneration(2), CheckpointTxg(0)),
            content: overwrite.bytes(),
        },
    );
    assert_eq!(
        judgement.vote,
        ReferenceFileSystemVote::Green,
        "整理那一步不进 RefFS 的操作序列，崩溃之后读回覆盖写的内容合法"
    );
}
