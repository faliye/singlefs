//! checker 档模块：crash_point_replay_progress、crash_given_states、crash_amplification
//! 随机历史加截断那一段写死的历史（`common_truncation::history_through_every_truncation_target`，harness 档与理想模型逐版比的那一条跑的也是它）
//! 每一步写过盘之后的镜像池级 checker 一条都不红：截断的意图、各批、最后一批、变长留洞与把洞截掉的每一版都判过（判决
//! `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 的 T-1 到 T-5 由池级 checker 判，K8「池级 checker 不红」）；截断字段那两步
//! （回退到写字段之前再变长、截到洞里更靠后再变长，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6 必红世界）的每一版同样判过、不红。
//! 攻方 `research/prompts/m4-trunc-r2-opus-output.md`「打中三」那条五步历史同样每一步的镜像判过、不红，理想模型每一版都对得上。
//!
//! 这两条原先在 harness 档（`crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs`），每一步的镜像就地交池级 checker 判；
//! 用户 2026-10-18 定「挪进 checker 档走两段」：历史照旧在 harness 的执行器上跑（理想模型逐版对拍照旧在执行器里做），交给观察者的每一张镜像
//! （起点之后一张、每一步之后各一张，与原先就地判的那几张逐张相同）各作一张给定状态表（基线就是那张盘、写表空、一个空持久集合），
//! 交崩溃点重放的给定状态引擎：录入段写进判定库、判段从库里读回判（配了 GPU 且编进 `gpu` 特性只由 GPU 判器判，否则由 CPU 判器判），
//! 判过的复用。只录那一趟（配置开了 GPU 而这份二进制没编进 `gpu` 特性）、有表归别的机器判的那一趟打 `NOT_RUN_NO_GPU_ADAPTER` 那一行报本次未跑。
#![cfg(feature = "verdict-store")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;

use common_truncation::{
    content_of_data_units_in_a_history, history_through_every_truncation_target, payload_capacity,
    truncate_in_a_history,
};
use singlefs_checker_tier::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_harness::gpu_memory_ledger::gpu_not_run_line;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryEnding,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, StepPosition,
    TruncationTarget,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

/// 写死的那一段历史这条用例的名字（引擎项名与本次未跑那一行用它）。
const TEST_NAME: &str =
    "every_step_image_of_the_fixed_truncation_history_is_green_under_the_pool_checker";

/// 起点之后一张、22 步之后各一张：观察者在每一步没判出失败时调一次，与 harness 档原先就地判的张数相同。
const STEP_IMAGES_OF_THE_FIXED_TRUNCATION_HISTORY: usize = 23;

/// 五步历史这条用例的名字（引擎项名与本次未跑那一行用它）。
const FIVE_STEP_HISTORY_TEST_NAME: &str =
    "the_five_step_history_cutting_to_zero_then_growing_then_cutting_to_one_payload_keeps_unit_zero_and_agrees_with_the_model";

/// 五步历史起点之后一张、5 步之后各一张，与 harness 档原先就地判的张数相同。
const STEP_IMAGES_OF_THE_FIVE_STEP_HISTORY: usize = 6;

/// 池级 checker 判红的一处：哪一步之后的镜像、哪条不变量、判红的说明。
type PoolCheckerViolationAtAStep = (StepPosition, &'static str, String);

/// 一段历史交给观察者的每一张镜像各作一张给定状态表，交给定状态引擎（项名 `item_name`）录进判定库、再从库里读回判，
/// 交回池级 checker 判红的每一处。有表这一趟没判（只录，或归别的机器判）时以 `test_name` 打本次未跑那一行、交回 `None`。
/// 引擎起不了、表判不了、一个状态交回的结论不是一份、引擎收尾报红项，都当场 panic（不退回别处判）。
fn pool_checker_violations_of_the_step_images_read_back_from_the_library(
    test_name: &str,
    item_name: String,
    step_images: &[(StepPosition, MemoryPool)],
) -> Option<Vec<PoolCheckerViolationAtAStep>> {
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem { item_name },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let mut violations: Vec<PoolCheckerViolationAtAStep> = Vec::new();
    let mut tables_left_unjudged: Vec<String> = Vec::new();
    for (position, image) in step_images {
        let table_name = format!("step-image/{position:?}");
        let judged = engine
            .judge_the_given_states(&GivenStatesOfATable {
                table_name: table_name.clone(),
                base: image,
                writes: &[],
                segments: &[],
                seams: &[],
                persisted_sets: vec![Vec::new()],
            })
            .unwrap_or_else(|reason| {
                panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
            });
        match judged.source {
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
            GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
            | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => {
                tables_left_unjudged.push(table_name);
                continue;
            }
        }
        let [conclusion] = judged.states.as_slice() else {
            panic!(
                "{table_name}：一个状态一份结论，交回 {} 份",
                judged.states.len()
            );
        };
        for (invariant, detail) in conclusion.pool_checker_violations() {
            violations.push((*position, invariant, detail));
        }
    }
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的盘照第三轮正文 H6 判红"
    );
    if !tables_left_unjudged.is_empty() {
        println!(
            "{}",
            gpu_not_run_line(
                test_name,
                &format!(
                    "{} 张给定状态表这一趟没判（只录，或归别的机器判）：录进了库，判段那一趟接着判；头一张 {}",
                    tables_left_unjudged.len(),
                    tables_left_unjudged[0]
                )
            )
        );
        return None;
    }
    Some(violations)
}

/// 判别力：池级 checker 在截断的意图、各批或截断字段那几版上误判（`crates/mutations.tsv` 以「m4 截断截辛」起头、点名这一条的那一行），
/// 那几步之后的镜像判红，这里红。
#[test]
fn every_step_image_of_the_fixed_truncation_history_is_green_under_the_pool_checker() {
    let history = history_through_every_truncation_target();
    let mut step_images: Vec<(StepPosition, MemoryPool)> = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| step_images.push((observation.position, observation.image.clone())),
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert_eq!(
        step_images.len(),
        STEP_IMAGES_OF_THE_FIXED_TRUNCATION_HISTORY,
        "起点与 22 步各判一张"
    );
    let Some(violations) = pool_checker_violations_of_the_step_images_read_back_from_the_library(
        TEST_NAME,
        format!("{TEST_NAME}::fixed"),
        &step_images,
    ) else {
        return;
    };
    assert!(
        violations.is_empty(),
        "池级 checker 在截断历史的镜像上判红 {} 处：{violations:#?}",
        violations.len()
    );
}

/// 攻方 `research/prompts/m4-trunc-r2-opus-output.md`「打中三」那条五步历史（顺序写 4 个净荷 → 覆盖写半个净荷 → 截到 0 → 变长到 1 个净荷 + 1 字节
/// → 截到 1 个净荷）：每一步的镜像经判定库交池级 checker 判，一条都不红（单元 0 不许被判死），模型每一版都对得上（截断字段逐版比过）。
/// 按用户 2026-10-17 定的写法（放了单元才写），这段历史上没有一次截断放单元，字段一直没写过：模型或实现在截断没放单元时写了字段
/// （`crates/mutations.tsv` 以「m4 截断截癸」起头、点名这一条的那两行）这一条判对不上；池级 checker 在 size 为 0 的记录上一个单元都不罩
/// （以「m4 五步截断历史挪档」起头的那一行）时截到 0 之后那几张镜像判死单元 0，这一条红。
#[test]
fn the_five_step_history_cutting_to_zero_then_growing_then_cutting_to_one_payload_keeps_unit_zero_and_agrees_with_the_model(
) {
    let payload = payload_capacity();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(content_of_data_units_in_a_history(4, payload - 1)),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit { selector: 16_316 },
                fill_seed: 5,
            }),
            truncate_in_a_history(TruncationTarget::ToZero, 0),
            truncate_in_a_history(TruncationTarget::GrowingAcrossDataUnits, 3),
            truncate_in_a_history(TruncationTarget::ToADataUnitBoundary, 0),
        ],
    };
    let mut step_images: Vec<(StepPosition, MemoryPool)> = Vec::new();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| step_images.push((observation.position, observation.image.clone())),
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert_eq!(
        run.sizes_requested_by_the_truncations
            .iter()
            .map(|(step, size)| (*step, *size))
            .collect::<Vec<_>>(),
        vec![(2, 0), (3, payload + 1), (4, payload)],
        "每次截断交给会话的目标 size"
    );
    assert_eq!(
        step_images.len(),
        STEP_IMAGES_OF_THE_FIVE_STEP_HISTORY,
        "起点与 5 步各判一张"
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.truncation_intents_matched,
            counts.size_changes_without_shrinking_matched,
            counts.written_truncation_fields_compared,
        ),
        (2, 1, 0),
        "截断意图两次、变长一次，字段一次都没写过"
    );
    assert!(
        counts.truncation_fields_compared > 0,
        "截断字段比过：{counts:?}"
    );
    let Some(violations) = pool_checker_violations_of_the_step_images_read_back_from_the_library(
        FIVE_STEP_HISTORY_TEST_NAME,
        format!("{FIVE_STEP_HISTORY_TEST_NAME}::fixed"),
        &step_images,
    ) else {
        return;
    };
    assert!(
        violations.is_empty(),
        "池级 checker 在这段历史的镜像上判红 {} 处：{violations:#?}",
        violations.len()
    );
}
