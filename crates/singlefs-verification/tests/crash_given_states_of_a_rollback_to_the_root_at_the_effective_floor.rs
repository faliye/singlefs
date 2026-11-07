//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_amplification
//! 回退到 txg = F_生效 的根（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）那一段历史上，每一步之后的镜像池级 checker 一条都不红。
//! harness 里同一段历史的那条用例（`crates/singlefs-verification/tests/random_histories.rs` 的
//! `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`）只核实现的结局与理想模型：每一步的池级 checker
//! 归给定状态引擎两段，这一份接那一截。收口表第 26 行 I-7.9 那一格（抬 F 的根只认同一实例里 txg 比它小的有效根，
//! 回退那条根不被当成抬 F）由这一份抓。
//!
//! 每一步写过盘之后的镜像（起点之后一张、每一步写过盘之后各一张，harness 的每步镜像那个钩子交的）各作一张给定状态表（基线就是那张盘、
//! 写表空、一个空持久集合），交崩溃点重放的给定状态引擎：录入段写进给定状态表那一类的判定库、判段从库里读回判（配了 GPU 且编进 `gpu`
//! 特性只由 GPU 判器判，否则由 CPU 判器判），判过的复用。只录那一趟（配置开了 GPU 而这份二进制没编进 `gpu` 特性）、表归别的机器判的那一趟
//! 打 `NOT_RUN_NO_GPU_ADAPTER` 那一行报本次未跑。

use singlefs_verification::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_verification::crash_given_states::{
    judge_the_given_states_on_the_cpu, GivenStatesConclusionsSource, GivenStatesJudge,
    GivenStatesOfATable,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::fault_injection::SharedFaultPlan;
use singlefs_verification::gpu_memory_ledger::gpu_not_run_line;
use singlefs_verification::history::{
    execute_history_handing_out_every_step_image, ContentChoice, ContentLength, FloorTargetChoice,
    GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice, StepPosition,
};
use singlefs_verification::memory_pool::MemoryPool;
use singlefs_verification::SharedStream;

/// 这条用例的名字（引擎项名与本次未跑那一行用它）。
const TEST_NAME: &str =
    "rolling_back_to_the_root_at_the_effective_floor_leaves_the_pool_checker_green_after_every_step";

/// 那一段历史，与同名的 harness 用例逐步相同：第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、
/// 抬 F（选择子 6 ⇒ F = 6，推 txg 10、11 两次）、挂着的时候回退到候选集的下沿 (6, 2)（一次向前发布 txg 12）、冷启动读回。
fn history_rolling_back_to_the_root_at_the_effective_floor() -> GeneratedHistory {
    let overwrite = |fill_seed: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed,
        })
    };
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            overwrite(6),
            overwrite(7),
            overwrite(8),
            overwrite(9),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 6,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRootAtTheNewestFloor),
            HistoryOperation::ColdStartRecover,
        ],
    }
}

/// 跑那一段历史，交回它是怎么结束的与每一步写过盘之后交出来的镜像（起点之后一张、每一步写过盘之后各一张）。
fn step_images_of_the_history_rolling_back_to_the_root_at_the_effective_floor(
) -> (HistoryEnding, Vec<(StepPosition, MemoryPool)>) {
    let history = history_rolling_back_to_the_root_at_the_effective_floor();
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let mut step_images: Vec<(StepPosition, MemoryPool)> = Vec::new();
    let run = execute_history_handing_out_every_step_image(
        &history,
        execution,
        &SharedStream::new(),
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        &mut |_| {},
        &mut |step_image| step_images.push((step_image.position, step_image.image.clone())),
    );
    (run.ending, step_images)
}

/// 每一张镜像在哪一步之后交出来的（核「回退那一步写过盘、交出了它之后的镜像」那条前提时打出来看）。
fn positions_of_the_step_images(step_images: &[(StepPosition, MemoryPool)]) -> Vec<StepPosition> {
    step_images.iter().map(|(position, _)| *position).collect()
}

/// 判别力：池级 checker 的 I-7.9 把回退那条根当成抬 F、按它自己的实例表认 txg 比它小的有效根（`crates/mutations.tsv` 收口表第 26 行那一条），
/// 回退之后那几张镜像判红 I-7.9，这里红。配了 GPU 只由 GPU 判器判，CPU 判器上的那一条是
/// [`rolling_back_to_the_root_at_the_effective_floor_leaves_the_pool_checker_green_after_every_step_on_the_cpu_judge`]。
#[test]
fn rolling_back_to_the_root_at_the_effective_floor_leaves_the_pool_checker_green_after_every_step()
{
    let (ending, step_images) =
        step_images_of_the_history_rolling_back_to_the_root_at_the_effective_floor();
    assert_eq!(
        ending,
        HistoryEnding::Completed,
        "这段历史跑完（理想模型与执行器那一截由同名的 harness 用例核）"
    );
    assert!(
        step_images
            .iter()
            .any(|(position, _)| *position == StepPosition::Operation(6)),
        "前提：回退那一步写过盘、交出了它之后的镜像：{:?}",
        positions_of_the_step_images(&step_images)
    );
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!("{}::fixed", TEST_NAME),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let mut red_step_images: Vec<(StepPosition, Vec<&'static str>)> = Vec::new();
    for (position, image) in &step_images {
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
                println!(
                    "{}",
                    gpu_not_run_line(
                        TEST_NAME,
                        &format!(
                            "给定状态表 {table_name} 这一趟没判（只录，或归别的机器判）：判过的在库里，判段那一趟接着判"
                        )
                    )
                );
                return;
            }
        }
        let [conclusion] = judged.states.as_slice() else {
            panic!(
                "{table_name}：一个状态一份结论，交回 {} 份",
                judged.states.len()
            );
        };
        let violated: Vec<&'static str> = conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(invariant, _)| invariant)
            .collect();
        if !violated.is_empty() {
            red_step_images.push((*position, violated));
        }
    }
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的盘照第三轮正文 H6 判红"
    );
    assert_eq!(
        red_step_images,
        Vec::new(),
        "回退到 F_生效 那条根的合法历史上，每一步之后的镜像池级 checker 一条都不红"
    );
}

/// 同一段历史每一步写过盘之后的镜像各作一张给定状态表（基线就是那张盘、写表空、一个空持久集合），不经给定状态引擎与判定库、不看本机配不配 GPU，
/// 直接交 CPU 判器（[`judge_the_given_states_on_the_cpu`]：两遍恢复、池级 checker、记录核对器）判：每一张池级 checker 一条都不红。
/// 引擎那一条（[`rolling_back_to_the_root_at_the_effective_floor_leaves_the_pool_checker_green_after_every_step`]）配了 GPU 只由 GPU 判器判，
/// 池级 checker 的 CPU 代码它跑不到，变异表里改坏那一处的行点名这一条。
/// 判别力（`crates/mutations.tsv` 收口表第 26 行「抬 F 的根不认同一实例的前一条，按它自己的实例表认 txg 比它小的有效根」那一行）：
/// 回退之后那几张镜像判红 I-7.9，这里红。
// gpu-exempt: cpu-judge-mutation-rows
#[test]
fn rolling_back_to_the_root_at_the_effective_floor_leaves_the_pool_checker_green_after_every_step_on_the_cpu_judge(
) {
    let (ending, step_images) =
        step_images_of_the_history_rolling_back_to_the_root_at_the_effective_floor();
    assert_eq!(ending, HistoryEnding::Completed, "这段历史跑完");
    assert!(
        step_images
            .iter()
            .any(|(position, _)| *position == StepPosition::Operation(6)),
        "前提：回退那一步写过盘、交出了它之后的镜像：{:?}",
        positions_of_the_step_images(&step_images)
    );
    let mut red_step_images: Vec<(StepPosition, Vec<&'static str>)> = Vec::new();
    for (position, image) in &step_images {
        let table = GivenStatesOfATable {
            table_name: format!("step-image/{position:?}"),
            base: image,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        };
        let judged = judge_the_given_states_on_the_cpu(&table);
        let [conclusion] = judged.states.as_slice() else {
            panic!(
                "{}：一个状态一份结论，交回 {} 份",
                table.table_name,
                judged.states.len()
            );
        };
        let violated: Vec<&'static str> = conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(invariant, _)| invariant)
            .collect();
        if !violated.is_empty() {
            red_step_images.push((*position, violated));
        }
    }
    assert_eq!(
        red_step_images,
        Vec::new(),
        "回退到 F_生效 那条根的合法历史上，每一步之后的镜像在 CPU 判器上池级 checker 一条都不红"
    );
}
