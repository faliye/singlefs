//! 随机历史加截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8、第六节 S1）：执行器经挂着的会话发 `UserChange::Truncate`，
//! 会话交回的每一版（先走空的各步、截断意图、各批、最后一批、变长与 N = S 那一次、其间推的抬 F）逐版交理想模型比，数据单元的分配代按单元号逐个比；
//! RefFS 翻译把一次截断翻成一个 `Truncate(n)` 连同它那次发布。这里钉三样：一段写死的历史走遍截断目标的每一类（缩到单元边界、跨单元变长、
//! 把洞整个截掉（S1）、留洞的变长、截到单元中间、留下 144 个单元（叶边界）、回退到截断半路再截到 0、N = S、删完之后点名别的 inode 被拒），
//! 每一版模型都对得上、计数钉绝对值；截断那一组比重在它自己那一段种子下跑完、每一类目标都进过入口、模型每一步都对得上、RefFS 不判红；
//! 慢档多跑几段。第一个文件那条记录的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2、R5，第六节用户定案）
//! 逐版交模型比；写死的历史另补「回退到写字段之前再变长」「截到洞里更靠后再变长」两步。
//! 写死的那一段历史住在 `tests/common_truncation/mod.rs`（`history_through_every_truncation_target`）：它每一步的镜像过池级 checker 那一条
//! 挪进了 checker 档走两段（用户 2026-10-18 定，`crates/singlefs-checker-tier/tests/crash_given_states_of_every_step_of_the_fixed_truncation_history.rs`），
//! 这里只留与理想模型、RefFS 比对那一截。攻方「打中三」那条五步历史每一步的镜像也要交池级 checker 判，整条连同模型对拍挪进了同一份 checker 档文件
//! （`the_five_step_history_cutting_to_zero_then_growing_then_cutting_to_one_payload_keeps_unit_zero_and_agrees_with_the_model`）。

mod common;
mod common_truncation;

use common_truncation::{
    content_of_data_units_in_a_history, history_through_every_truncation_target,
    truncate_in_a_history,
};
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, GeneratedHistory,
    GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
    HistorySeed, HistoryStartingPoint, RollbackTargetChoice, StepOutcome, TruncationTarget,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::model::ObservedReadBack;
use singlefs_harness::model_comparison::observed_read_back;
use singlefs_harness::reference_filesystem_translation::{
    ReferenceFileSystemCrashMoment, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
};
use singlefs_harness::SharedStream;

/// 截断那一组比重的种子区间：从种子基往后的这一段（「要多测别的历史，只往后追加新的种子区间」，`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`
/// 的文档注释；快照那一组在 40 000 起、整理那两组在 50 000、60 000 起）。
const SEED_OFFSET_OF_THE_TRUNCATION_SAMPLING: u64 = 70_000;
/// 截断那一组比重随时跑的那一批几段、每段几步（快档规模，同快照那一组）。
const TRUNCATION_SAMPLING_SEEDS: u64 = 4;
const TRUNCATION_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;
/// 慢档那一批：同一个区间从偏移 0 起多跑几段。
const TRUNCATION_SAMPLING_SEEDS_OF_THE_SLOW_TIER: u64 = 16;

/// 写死的那一段历史每一版模型都对得上（跑完、没有新发现），而且模型真的比到了每一种截断发布：意图 8 次（第 1、3、5、7、9、12、15、17 步）、
/// 不是最后一批的批 2 次（第 1 步）、最后一批 9 次（第 1、3、5、7、12、15、17 步各一次，第 9 步先走完回退带回来的那次截断、再走自己的）、
/// 不缩短的 6 次（第 2、4、10、14、16、18 步），删完之后点名别的 inode 被拒是模型要求的拒；交给会话的目标 size 逐步钉绝对值；
/// 第一个文件那条记录的截断字段逐版比过（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6「理想模型记字段并在对拍里比」），
/// 比过的版数与其中字段写过的版数钉绝对值。
/// 判别力：模型或实现在任一种截断发布上改错一样（重写的角色、标签时点、分配代、截断条目、截断字段），这一段在那一步判对不上
/// （`crates/mutations.tsv` 里以「m4 截断截丁」「m4 截断截癸」起头的几行）；执行器把截断的各批当成这次改动本身比时第 1 步判对不上。
#[test]
fn a_fixed_history_through_every_truncation_target_agrees_with_the_model_at_every_version() {
    let history = history_through_every_truncation_target();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for (step, outcome) in run.outcomes.iter().enumerate().take(21) {
        assert!(
            matches!(
                outcome,
                StepOutcome::Applied(
                    AppliedEffect::Published { .. } | AppliedEffect::RolledBack { .. }
                )
            ),
            "第 {step} 步做成：{outcome:?}"
        );
    }
    assert!(
        matches!(&run.outcomes[21], StepOutcome::Refused { member } if member.contains("SizeChangeOfAnInodeOtherThanTheFirstFile")),
        "删完之后点名别的 inode 改 size：第一版拒：{:?}",
        run.outcomes[21]
    );
    let payload = 32_634_u64;
    assert_eq!(
        run.sizes_requested_by_the_truncations
            .iter()
            .map(|(step, size)| (*step, *size))
            .collect::<Vec<_>>(),
        vec![
            (1, payload),
            (2, 3 * payload + 1),
            (3, payload),
            (4, payload + 6),
            (5, 1),
            (7, 143 * payload + 21),
            (9, 0),
            (10, 0),
            (12, payload),
            (14, 5 * payload + 1),
            (15, payload),
            (16, 4 * payload),
            (17, 3 * payload),
            (18, 5 * payload),
            (21, 6),
        ],
        "每次截断交给会话的目标 size"
    );
    let counts = &run.tally.model_counts;
    println!("{counts:?}");
    assert_eq!(
        (
            counts.truncation_intents_matched,
            counts.truncation_batches_before_the_final_batch_matched,
            counts.final_truncation_batches_matched,
            counts.size_changes_without_shrinking_matched,
            counts.required_refusals_matched,
        ),
        (8, 2, 9, 6, 1),
        "意图、不是最后一批的批、最后一批、不缩短的、模型要求的拒"
    );
    assert_eq!(
        (
            counts.truncation_fields_compared,
            counts.written_truncation_fields_compared,
        ),
        (32, 26),
        "比过截断字段的版数（34 条根里除去删完与建 inode 那两版）、其中字段写过的版数"
    );
}

/// 顺序写 150 个单元、截到单元 143 中间（意图与最后一批两条根）、再回退到根环里次新的那条（意图那一版，截断半路），之后冷启动读回截后的内容：
/// RefFS 翻译把截断那一步从意图那条根起写的根都映到截后那一版（判决第三节 K2「先落目标」：意图那一版读出的就是截后的），回退到意图那条根
/// 就是回退到截后那一版，读回截后的内容判绿。判别力：截断那一步只有最后一条根映到新版本（意图那条映到截前那一版）时 RefFS 要的是截前的
/// 150 个单元，判红（`crates/mutations.tsv` 一行）；执行器数带着改动的根数数错时同样。
#[test]
fn a_rollback_to_the_intent_root_of_a_truncation_reads_back_the_truncated_content_for_the_reference_filesystem(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(content_of_data_units_in_a_history(150, 9)),
            truncate_in_a_history(TruncationTarget::IntoTheMiddleOfADataUnit, 143),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 1,
            }),
        ],
    };
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let mut translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let mut read_back_after_the_last_step: Option<ObservedReadBack> = None;
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        translation.note_an_observed_step(observation, stream.operation_count());
        read_back_after_the_last_step = Some(observed_read_back(
            &recover(observation.image, JournalPolicy::Consult).outcome,
        ));
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert!(
        matches!(
            run.outcomes[2],
            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
        ),
        "回退做成：{:?}",
        run.outcomes[2]
    );
    let read_back = read_back_after_the_last_step.expect("起点那一步也交给观察者");
    let ObservedReadBack::FileRead { content, .. } = &read_back else {
        panic!("冷启动读回文件：{read_back:?}");
    };
    assert_eq!(
        u64::try_from(content.len()).expect("长度"),
        143 * 32_634 + 21,
        "回退到意图那一版，读回截后的长度"
    );
    let translated = translation.translate(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    assert_eq!(
        translated
            .judge_a_crash(
                ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
                &read_back
            )
            .vote,
        ReferenceFileSystemVote::Green
    );
}

/// 截断那一组比重在它自己那一段种子里跑前 `seeds` 段：每一段都跑完、模型每一步都对得上；每段停下时冷启动读回的内容交 RefFS 判
/// （「序列里的操作都返回过之后崩溃」那一刻），不许判红（翻译停在 unlink 那一步的段弃权）。交回每类目标进过入口的次数、RefFS 判绿的段数与
/// 模型比过的各种截断发布。
struct TruncationSampling {
    targets_attempted: std::collections::BTreeMap<&'static str, u64>,
    truncations_applied: u64,
    reference_filesystem_greens: u64,
    model_counts: singlefs_harness::model::ModelJudgementCounts,
}

fn run_the_truncation_sampling(seeds: u64) -> TruncationSampling {
    let mut sampling = TruncationSampling {
        targets_attempted: std::collections::BTreeMap::new(),
        truncations_applied: 0,
        reference_filesystem_greens: 0,
        model_counts: singlefs_harness::model::ModelJudgementCounts::default(),
    };
    for offset in 0..seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_TRUNCATION_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            TRUNCATION_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::TRUNCATION,
        );
        let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
        let stream = SharedStream::retaining_contents();
        let mut translation =
            ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
        let mut read_back_after_the_last_step: Option<ObservedReadBack> = None;
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            translation.note_an_observed_step(observation, stream.operation_count());
            read_back_after_the_last_step = Some(observed_read_back(
                &recover(observation.image, JournalPolicy::Consult).outcome,
            ));
        });
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "种子 {seed}（偏移 {offset}）：{:?}",
            history.operations
        );
        let translated = translation.translate(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let read_back = read_back_after_the_last_step.expect("起点那一步也交给观察者");
        let judgement = translated.judge_a_crash(
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
            &read_back,
        );
        match judgement.vote {
            ReferenceFileSystemVote::Green => sampling.reference_filesystem_greens += 1,
            ReferenceFileSystemVote::Abstained(_) => {}
            ReferenceFileSystemVote::Red { reason } => {
                panic!("种子 {seed}（偏移 {offset}）：RefFS 判红：{reason}")
            }
        }
        for (target, count) in &run.tally.truncation_targets_attempted {
            *sampling.targets_attempted.entry(target).or_insert(0) += count;
        }
        sampling.truncations_applied += run
            .tally
            .operations_by_kind
            .get(&HistoryOperationKind::Truncate)
            .map_or(0, |tally| tally.applied);
        sampling.model_counts.add(&run.tally.model_counts);
    }
    eprintln!(
        "截断那一组比重 {seeds} 段：各类目标进过入口 {:?}；做成 {} 次；RefFS 判绿 {} 段；模型比过意图 {}、不是最后一批的批 {}、最后一批 {}、不缩短的 {}",
        sampling.targets_attempted,
        sampling.truncations_applied,
        sampling.reference_filesystem_greens,
        sampling.model_counts.truncation_intents_matched,
        sampling
            .model_counts
            .truncation_batches_before_the_final_batch_matched,
        sampling.model_counts.final_truncation_batches_matched,
        sampling.model_counts.size_changes_without_shrinking_matched
    );
    sampling
}

/// 截断那一组比重（`GenerationWeights::TRUNCATION`）在它自己那一段种子下（快档规模 4 段、每段 40 步）：每一段都跑完、模型每一版都对得上，
/// 截断的七类目标都进过入口，意图、最后一批与不缩短的那一次模型都比过，RefFS 判过绿、一次都没判红。
/// 判别力：比重里截断的份数改成 0（`crates/mutations.tsv` 一行）时七类目标一次都没进过入口，这一条红；模型或实现在某种截断发布上改错一样时
/// 那一段判对不上、这一条红。
#[test]
fn the_truncation_weights_draw_every_truncation_target_and_every_history_agrees_with_the_model() {
    let sampling = run_the_truncation_sampling(TRUNCATION_SAMPLING_SEEDS);
    for target in [
        TruncationTarget::ToZero,
        TruncationTarget::ToADataUnitBoundary,
        TruncationTarget::IntoTheMiddleOfADataUnit,
        TruncationTarget::GrowingWithinTheLastDataUnit,
        TruncationTarget::GrowingAcrossDataUnits,
        TruncationTarget::BackIntoTheHole,
        TruncationTarget::ToTheCurrentSize,
    ] {
        assert!(
            sampling
                .targets_attempted
                .get(target.name())
                .is_some_and(|count| *count > 0),
            "截断目标「{}」在 {TRUNCATION_SAMPLING_SEEDS} 段里一次都没进过入口：{:?}",
            target.name(),
            sampling.targets_attempted
        );
    }
    let counts = &sampling.model_counts;
    assert!(
        counts.truncation_intents_matched > 0
            && counts.final_truncation_batches_matched > 0
            && counts.size_changes_without_shrinking_matched > 0,
        "意图、最后一批与不缩短的那一次模型都比过：{counts:?}"
    );
    assert!(
        sampling.reference_filesystem_greens > 0,
        "RefFS 一段都没判过绿"
    );
}

/// 同一组比重、同一段种子多跑几段（慢档）：每一段都跑完、模型每一步都对得上、RefFS 不判红，不是最后一批的截断批也比过。
#[test]
#[ignore = "harness 慢档用例：截断那一组比重 16 段、每段 40 步，每段几百个单元的文件逐版对拍"]
fn the_truncation_weights_over_more_seeds_run_every_history_to_the_end_with_the_model_agreeing() {
    let sampling = run_the_truncation_sampling(TRUNCATION_SAMPLING_SEEDS_OF_THE_SLOW_TIER);
    assert!(
        sampling
            .model_counts
            .truncation_batches_before_the_final_batch_matched
            > 0,
        "不是最后一批的截断批一次都没比过：{:?}",
        sampling.model_counts
    );
}
