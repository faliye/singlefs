//! 随机历史里两格判结局的取样点：留孤儿比重 × 384 槽小盘上覆盖写掉电留在系统配置里的 F（D16（发布语义） 已定项 1「生效」：
//! 实现取系统配置那一半，模型答「只能是原值或这一刻的上限」，`IdealModel::settle_floor_left_in_the_system_configurations_by_a_lost_power`），
//! 与回退到截断半路之后的一次整份重写（会话先把截断走完再发，`queue_of_committed_intents_is_empty`）。
//! 种子偏移写死，取的是这两格走得到掉电留 F 与截断半路重写的那几段；每段一条用例，一条用例只跑一段历史。

use singlefs_core::admission::SpaceAdmission;
use singlefs_verification::history::{
    execute_history_with, generate_history_with_weights, ContentChoice, ContentLength,
    GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryRun, HistorySeed, HistoryStartingPoint, PerStepChecker,
    RollbackTargetChoice, TruncationChoice, TruncationTarget, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_verification::SharedStream;

/// 每段随机历史的步数：这几段种子撞到那两格用的步数。
const OPERATIONS_OF_EACH_RANDOM_HISTORY: usize = 60;

fn run_with_the_space_admission_judged(
    history: &GeneratedHistory,
    device_width: HistoryDeviceWidth,
) -> HistoryRun {
    execute_history_with(
        history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &SharedStream::new(),
        &mut |_observation| {},
    )
}

/// 留孤儿比重 × 384 槽小盘的一段历史：跑到结局，且至少一次掉电把系统配置里的 F 留在模型算的上限上（这一格真的走到了）。
/// 掉电留下的 F 抬过上限、模型不取系统配置那一半，都停在对不上。
fn assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(seed_offset: u64) {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(seed_offset)),
        OPERATIONS_OF_EACH_RANDOM_HISTORY,
        &GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
    );
    let run = run_with_the_space_admission_judged(&history, HistoryDeviceWidth::UnitAreaOf384Slots);
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "种子偏移 {seed_offset}：{:?}",
        history.operations
    );
    assert!(
        run.tally
            .model_counts
            .lost_powers_leaving_the_floor_at_the_ceiling
            >= 1,
        "种子偏移 {seed_offset}：这段历史没有一次掉电把系统配置里的 F 留在上限上，用例没走到要判的那一格"
    );
}

#[test]
fn orphan_history_at_seed_offset_3_completes_with_a_floor_left_at_the_ceiling() {
    assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(3);
}

#[test]
fn orphan_history_at_seed_offset_8_completes_with_a_floor_left_at_the_ceiling() {
    assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(8);
}

#[test]
fn orphan_history_at_seed_offset_10_completes_with_a_floor_left_at_the_ceiling() {
    assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(10);
}

#[test]
fn orphan_history_at_seed_offset_15_completes_with_a_floor_left_at_the_ceiling() {
    assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(15);
}

#[test]
fn orphan_history_at_seed_offset_19_completes_with_a_floor_left_at_the_ceiling() {
    assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(19);
}

#[test]
fn orphan_history_at_seed_offset_20_completes_with_a_floor_left_at_the_ceiling() {
    assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(20);
}

#[test]
fn orphan_history_at_seed_offset_22_completes_with_a_floor_left_at_the_ceiling() {
    assert_orphan_history_completes_with_a_floor_left_at_the_ceiling(22);
}

/// 截断比重 × 384 槽小盘的那一段：一次整份重写接在截断半路的一版上，会话先走完截断，执行器要逐版交模型比。
#[test]
fn truncation_history_at_seed_offset_11_on_small_devices_completes() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(11)),
        OPERATIONS_OF_EACH_RANDOM_HISTORY,
        &GenerationWeights::TRUNCATION,
    );
    let run = run_with_the_space_admission_judged(&history, HistoryDeviceWidth::UnitAreaOf384Slots);
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "{:?}",
        history.operations
    );
}

/// 两块 4 GiB 的盘上写死的最短一形：三个单元的文件截到单元边界（意图一条根、最后一批一条根），回退到意图那条根，再整份重写。
/// 那次重写之前会话先走完截断的最后一批，模型比过的最后一批是两次：截断自己那一次、走空那一次。
#[test]
fn a_whole_file_write_after_a_rollback_to_a_truncation_intent_on_large_devices_walks_the_truncation_first(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 3,
                    selector: 0,
                },
                fill_seed: 5,
            }),
            HistoryOperation::Truncate(TruncationChoice {
                target: TruncationTarget::ToADataUnitBoundary,
                selector: 0,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 1,
            }),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 1 },
                fill_seed: 6,
            }),
        ],
    };
    let run = run_with_the_space_admission_judged(&history, HistoryDeviceWidth::FourGibibytes);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert_eq!(run.tally.model_counts.truncation_intents_matched, 1);
    assert_eq!(run.tally.model_counts.final_truncation_batches_matched, 2);
}
