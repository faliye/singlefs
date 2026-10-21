//! checker 档模块：crash_point_replay_progress、crash_given_states、crash_amplification
//! 整理搬过单元之后的管理员回退（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F1；
//! `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「映射是唯一权威」；D23（journal 的角色与格式） 已定项 14）：
//! 整理把单元从源落点搬到目的落点、映射条目换到目的地；回退到搬之前的一版时，复活的是源落点、释放的是目的落点，回退那一版的中央映射
//! 就得指回源落点（`mount::roll_back_by_a_forward_publish` 交给发布路径的那几把位置不同的 key，装着它们的叶连同到根的路重写）。
//! 映射照抄现行那一版时，回退那一版的映射指着回退自己释放掉的目的落点：池级 checker 在回退那一步红 I-3.9（释放了而候选根还引用）与
//! I-3.15（记录罩住的单元在映射里没有 key 指着这一槽），之后的覆盖写经映射释放被拒（`ReleaseTargetAlreadyReleased`，模型答该成）。
//! 这里照攻方腿的两组复现（`research/prompts/m4-segment-code-r1-opus-model/src/attack_rollback_after_compaction.rs` 的
//! `compaction_then_remount_then_rollback_sweep` 与 `seed_126_prefix_then_suffixes`）各写成用例：执行器按模型逐步比、历史要走完，
//! 每一步的镜像池级 checker 一条都不红；再加整理夹着回退那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`）的一批种子。
//!
//! 这几条原先在 harness 档（`crates/singlefs-harness/tests/rollback_after_compaction_takes_the_mapping_positions_of_the_target.rs`），每一步的镜像
//! 就地交池级 checker 判；用户 2026-10-18 定「挪进 checker 档走两段」：历史照旧在 harness 的执行器上跑（理想模型逐步比、结局照旧核），
//! 交给观察者的每一张镜像（与原先就地判的那几张逐张相同：起点之后一张、每一步之后各一张，整理夹着回退那一组只取回退那几步之后的）
//! 各作一张给定状态表（基线就是那张盘、写表空、一个空持久集合），交崩溃点重放的给定状态引擎：录入段写进判定库、判段从库里读回判
//! （配了 GPU 且编进 `gpu` 特性只由 GPU 判器判，否则由 CPU 判器判），判过的复用。只录那一趟（配置开了 GPU 而这份二进制没编进 `gpu` 特性）、
//! 有表归别的机器判的那一趟打 `NOT_RUN_NO_GPU_ADAPTER` 那一行报本次未跑。
#![cfg(feature = "verdict-store")]

use singlefs_checker_tier::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_harness::gpu_memory_ledger::gpu_not_run_line;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, CompactionChoice,
    CompactionRunEnding, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
    GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation, HistoryRun, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice, StepOutcome, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

/// 攻方腿那段前缀里整理强制选段用的选择子（执行时对候选段个数取余）。
const REGION_SELECTOR_OF_THE_ATTACK: u64 = 16_033_315_046_779_662_336;
/// 回退目标在根环里从最新往旧数的下标，扫这么多条（攻方腿扫的 0 到 7）。
const RING_ROOTS_SWEPT_BY_THE_ROLLBACK: u64 = 8;
/// 整理夹着回退那一组比重的种子区间：从种子基往后的这一段（整理那一组在偏移 50000 起，这一段在它之后）。
const SEED_OFFSET_OF_THE_COMPACTION_AND_ROLLBACK_SAMPLING: u64 = 60_000;
/// 那一组比重跑的几段、每段几步（与 harness 档原先那一条逐项相同）。
const COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS: u64 = 4;
const COMPACTION_AND_ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;

/// 一段历史在 harness 的执行器上跑完：执行器交回的这一趟，与交给观察者、要过池级 checker 的那几张镜像（表名、镜像）。
struct HistoryWithItsStepImages {
    run: HistoryRun,
    step_images: Vec<(String, MemoryPool)>,
}

/// 在两块 4 GiB 的盘上跑 `history`，`image_is_judged` 认下的那几步交给观察者的镜像拷下来（表名以 `table_name_prefix` 起头、接这一步的位置）。
/// 观察者在每一步没判出失败时调（起点之后一次、之后每一步一次），交的就是 harness 档原先就地交池级 checker 的那一张。
fn run_the_history_keeping_the_step_images(
    history: &GeneratedHistory,
    table_name_prefix: &str,
    image_is_judged: &dyn Fn(Option<&HistoryOperation>) -> bool,
) -> HistoryWithItsStepImages {
    let mut step_images: Vec<(String, MemoryPool)> = Vec::new();
    let run = execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            if image_is_judged(observation.operation) {
                step_images.push((
                    format!("{table_name_prefix}/step-image/{:?}", observation.position),
                    observation.image.clone(),
                ));
            }
        },
    );
    HistoryWithItsStepImages { run, step_images }
}

/// `step_images` 每一张作一张给定状态表（基线就是那张盘、写表空、一个空持久集合）交给定状态引擎（项名 `test_name`）：录入段写进判定库、
/// 判段从库里读回判。交回池级 checker 判红的那几条（「<表名> <不变量>：<细节>」，GPU 判的与从库里读回的结论不带细节，细节写那一句占位）；
/// 这一趟有表没判（只录，或归别的机器判）时把每张表都交过一遍（录进库）之后打本次未跑那一行、交回 `None`。
///
/// # Panics
/// 引擎起不了；有一张表判不了（不退回别处判）；一张表交回的结论不是恰好一份；GPU 判不了的盘（照第三轮正文 H6 判红）。
fn pool_checker_violations_judged_through_the_library(
    test_name: &str,
    step_images: &[(String, MemoryPool)],
) -> Option<Vec<String>> {
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!("{test_name}::fixed"),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let mut violations: Vec<String> = Vec::new();
    let mut tables_left_unjudged: Vec<&str> = Vec::new();
    for (table_name, image) in step_images {
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
            violations.push(format!("{table_name} {invariant}：{detail}"));
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

/// 历史要走完、每一步与模型对上；交给观察者的每一张镜像经判定库判，池级 checker 一条都不红。只录或归别的机器判的那一趟报本次未跑、不判镜像。
fn assert_completed_and_every_step_image_green(
    test_name: &str,
    histories: &[(String, HistoryWithItsStepImages)],
) {
    for (scenario, history) in histories {
        assert_eq!(
            history.run.ending,
            HistoryEnding::Completed,
            "{scenario}：历史要走完、每一步与模型对上（回退那一版的映射指着被释放的目的落点时，之后的覆盖写被拒 ReleaseTargetAlreadyReleased）"
        );
    }
    let step_images: Vec<(String, MemoryPool)> = histories
        .iter()
        .flat_map(|(_scenario, history)| history.step_images.iter().cloned())
        .collect();
    let Some(violations) =
        pool_checker_violations_judged_through_the_library(test_name, &step_images)
    else {
        return;
    };
    assert!(
        violations.is_empty(),
        "每一步的镜像池级 checker 一条都不红（映射照抄现行那一版时回退那一步红 I-3.9、I-3.15），实际 {violations:?}"
    );
}

/// 一段写死的历史（起点是第一个文件）在每一步交出镜像。
fn fixed_history_keeping_every_step_image(
    table_name_prefix: &str,
    operations: Vec<HistoryOperation>,
) -> HistoryWithItsStepImages {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    run_the_history_keeping_the_step_images(&history, table_name_prefix, &|_operation| true)
}

/// 顺序写跨 `data_units` 个数据单元（一个的走覆盖写入口）。
fn content_spanning(data_units: u64, fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: if data_units <= 1 {
            ContentLength::InsideOneDataUnit { selector: 777 }
        } else {
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units,
                selector: 5,
            }
        },
        fill_seed,
    }
}

/// 攻方腿种子下标 126 缩出来的前缀：第一个文件 → 顺序写 152 单元 → 顺序写 290 单元 → 整理（一批 3 个、推两批后崩、可写挂载）
/// → 回退到根环里第 4 新的根（整理之前那一版）。
fn prefix_of_the_attack_seed_126() -> Vec<HistoryOperation> {
    vec![
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: 152,
                selector: 9_852_620_462_860_606_767,
            },
            fill_seed: 2_832_512_656_265_544_978,
        }),
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units: 290,
                selector: 15_334_540_613_569_234_429,
            },
            fill_seed: 10_383_977_899_441_184_023,
        }),
        HistoryOperation::CompactWhileMounted(CompactionChoice {
            batch_units: 3,
            region_selector: REGION_SELECTOR_OF_THE_ATTACK,
            ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
        }),
        HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
            index_from_newest: 3,
        }),
    ]
}

fn compaction_running_until_it_stops(region_selector: u64) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units: 3,
        region_selector,
        ending: CompactionRunEnding::RunsUntilItStops,
    })
}

/// 攻方腿那段前缀接上 `suffix` 跑完，判每一步的镜像（项名 `test_name`，场景 `scenario`）。
fn assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
    test_name: &str,
    scenario: &str,
    suffix: Vec<HistoryOperation>,
) {
    let mut operations = prefix_of_the_attack_seed_126();
    operations.extend(suffix);
    let history =
        fixed_history_keeping_every_step_image("prefix-of-the-attack-seed-126", operations);
    assert_completed_and_every_step_image_green(test_name, &[(scenario.to_string(), history)]);
}

/// 同一段前缀，整理那一步换成推到停、不崩，之后（`remount` 时先可写挂载一次）回退到根环里第 `index_from_newest` 新的根，再覆盖写一个单元。
fn compaction_to_its_stop_then_rollback_then_overwrite(
    remount: bool,
    index_from_newest: u64,
) -> Vec<HistoryOperation> {
    let mut operations = prefix_of_the_attack_seed_126();
    operations.truncate(2);
    operations.push(compaction_running_until_it_stops(
        REGION_SELECTOR_OF_THE_ATTACK,
    ));
    if remount {
        operations.push(HistoryOperation::CloseAndMountWritable);
    }
    operations.push(HistoryOperation::RollBackWhileMounted(
        RollbackTargetChoice::RingRoot { index_from_newest },
    ));
    operations.push(HistoryOperation::PublishOverwrite(content_spanning(1, 9)));
    operations
}

/// 「整理推到停 →（`remount` 时可写挂载）→ 回退 → 覆盖写」扫回退目标在根环里的每一个位置：每一段各跑完、每一段的每一步镜像都经判定库判。
fn assert_every_ring_root_swept_after_a_compaction_ran_to_its_stop(test_name: &str, remount: bool) {
    let histories: Vec<(String, HistoryWithItsStepImages)> = (0..RING_ROOTS_SWEPT_BY_THE_ROLLBACK)
        .map(|index_from_newest| {
            let scenario = if remount {
                format!("可写挂载之后、回退到根环里第 {index_from_newest} 新的根")
            } else {
                format!("不重挂、回退到根环里第 {index_from_newest} 新的根")
            };
            let history = fixed_history_keeping_every_step_image(
                &format!("rollback-to-ring-root-{index_from_newest}-from-the-newest"),
                compaction_to_its_stop_then_rollback_then_overwrite(remount, index_from_newest),
            );
            (scenario, history)
        })
        .collect();
    assert_completed_and_every_step_image_green(test_name, &histories);
}

/// 不重挂的 8 段：整理半路、整理之前、整理之后的根都退一遍。判别力同 harness 档原先那一条：回退那一版的映射照抄现行那一版时
/// 回退那一步的镜像红 I-3.9、I-3.15（`crates/mutations.tsv`「m4 修复批甲 F1」那一行）。
#[test]
fn rolling_back_after_a_compaction_ran_to_its_stop_to_any_ring_root_keeps_the_checker_green_and_the_next_overwrite_lands(
) {
    assert_every_ring_root_swept_after_a_compaction_ran_to_its_stop(
        "rolling_back_after_a_compaction_ran_to_its_stop_to_any_ring_root_keeps_the_checker_green_and_the_next_overwrite_lands",
        false,
    );
}

/// 整理推到停之后先可写挂载、再回退的 8 段。
#[test]
fn rolling_back_after_a_compaction_and_a_writable_mount_to_any_ring_root_keeps_the_checker_green_and_the_next_overwrite_lands(
) {
    assert_every_ring_root_swept_after_a_compaction_ran_to_its_stop(
        "rolling_back_after_a_compaction_and_a_writable_mount_to_any_ring_root_keeps_the_checker_green_and_the_next_overwrite_lands",
        true,
    );
}

/// 攻方腿那段前缀（整理推两批后崩、可写挂载、回退到整理之前）到此为止：回退那一版的映射指回源落点，checker 不红。
/// 判别力：回退那一版的映射照抄现行那一版（`crates/mutations.tsv`「m4 修复批甲 F1」那一行）时回退那一步的镜像红 I-3.9、I-3.15。
#[test]
fn rolling_back_to_before_a_crashed_compaction_keeps_the_checker_green() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "rolling_back_to_before_a_crashed_compaction_keeps_the_checker_green",
        "前缀到回退为止",
        Vec::new(),
    );
}

/// 回退之后覆盖写一个单元：经映射释放的是回退那一版引用的源落点，做得成。
#[test]
fn an_overwrite_of_one_unit_after_rolling_back_a_compaction_lands() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "an_overwrite_of_one_unit_after_rolling_back_a_compaction_lands",
        "回退之后覆盖写一个单元",
        vec![HistoryOperation::PublishOverwrite(content_spanning(1, 9))],
    );
}

/// 回退之后顺序写 290 个单元：换下回退那一版引用的每一个数据单元，做得成。
#[test]
fn a_sequential_write_of_every_unit_after_rolling_back_a_compaction_lands() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "a_sequential_write_of_every_unit_after_rolling_back_a_compaction_lands",
        "回退之后顺序写 290 个单元",
        vec![HistoryOperation::PublishOverwrite(content_spanning(290, 9))],
    );
}

/// 回退之后可写挂载、再覆盖写：重建按映射覆盖提示时读到的是源落点，分配代与模型对得上。
#[test]
fn a_writable_mount_then_an_overwrite_after_rolling_back_a_compaction_read_the_source_placements() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "a_writable_mount_then_an_overwrite_after_rolling_back_a_compaction_read_the_source_placements",
        "回退之后可写挂载、再覆盖写",
        vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(content_spanning(1, 9)),
        ],
    );
}

/// 回退之后 unlink 第一个文件：删除批经映射释放回退那一版引用的单元。
#[test]
fn an_unlink_after_rolling_back_a_compaction_releases_through_the_mapping_of_the_target() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "an_unlink_after_rolling_back_a_compaction_releases_through_the_mapping_of_the_target",
        "回退之后 unlink",
        vec![HistoryOperation::Unlink],
    );
}

/// 回退之后抬 F 再回退一次：第二次回退的目标是第一次回退那一版，映射同样对得上（攻方腿在这一格多红 I-3.11）。
#[test]
fn raising_the_floor_then_rolling_back_again_after_rolling_back_a_compaction_keeps_the_checker_green(
) {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "raising_the_floor_then_rolling_back_again_after_rolling_back_a_compaction_keeps_the_checker_green",
        "回退之后抬 F、再回退",
        vec![
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 4_491_436_106_160_728_736,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 1,
            }),
        ],
    );
}

/// 回退之后再整理一趟：盘上那一条意图随现行那一版跨回退留着（D23 已定项 14「logged_ops 随现行那一版」），会话核分配器这一侧的活意图
/// 与盘上那一条相同、接着推，复活回 R 里的单元照搬。
#[test]
fn compacting_again_after_rolling_back_a_compaction_keeps_the_checker_green() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "compacting_again_after_rolling_back_a_compaction_keeps_the_checker_green",
        "回退之后再整理",
        vec![compaction_running_until_it_stops(
            REGION_SELECTOR_OF_THE_ATTACK,
        )],
    );
}

/// 回退之后冷启动：从盘上重建回退那一版，映射指着源落点。
#[test]
fn a_cold_start_after_rolling_back_a_compaction_keeps_the_checker_green() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "a_cold_start_after_rolling_back_a_compaction_keeps_the_checker_green",
        "回退之后冷启动",
        vec![HistoryOperation::ColdStartRecover],
    );
}

/// 回退之后抬 F 到顶、整理到停、冷启动。
#[test]
fn raising_the_floor_compacting_and_a_cold_start_after_rolling_back_a_compaction_keep_the_checker_green(
) {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "raising_the_floor_compacting_and_a_cold_start_after_rolling_back_a_compaction_keep_the_checker_green",
        "回退之后抬 F、整理、冷启动",
        vec![
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: u64::MAX,
            }),
            compaction_running_until_it_stops(7),
            HistoryOperation::ColdStartRecover,
        ],
    );
}

/// 回退之后可写挂载、抬 F 到顶、整理两趟、冷启动。
#[test]
fn a_writable_mount_and_two_compactions_after_rolling_back_a_compaction_keep_the_checker_green() {
    assert_the_prefix_of_the_attack_seed_126_then_completes_and_stays_green(
        "a_writable_mount_and_two_compactions_after_rolling_back_a_compaction_keep_the_checker_green",
        "回退之后可写挂载、抬 F、整理两趟、冷启动",
        vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: u64::MAX,
            }),
            compaction_running_until_it_stops(1),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: u64::MAX,
            }),
            compaction_running_until_it_stops(2),
            HistoryOperation::ColdStartRecover,
        ],
    );
}

/// 整理夹着回退那一组比重在它自己那一段种子下：每一段都跑完、模型逐步对得上、每次回退之后的镜像池级 checker 一条都不红（I-3.5 也判：
/// 它的用户可见树标签按版本记，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9），
/// 至少有一次回退做在同一段历史里整理搬过单元之后（取样点真撞到这一格）。只判回退那几步之后的镜像（与 harness 档原先那一条同一个取法：
/// 回退那一版的映射照抄现行那一版时红在回退那一步，I-3.9、I-3.15）。判别力：整理夹着回退那一组不抽回退
/// （`crates/mutations.tsv`「m4 修复批甲 F1 取样」那一行）时一次都没有在整理搬过单元之后回退，这一条红。
#[test]
fn the_compaction_and_rollback_weights_run_every_history_to_the_end_with_the_model_agreeing() {
    let mut rollbacks_after_a_relocation = 0_u64;
    let mut step_images_after_the_rollbacks: Vec<(String, MemoryPool)> = Vec::new();
    for offset in 0..COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_COMPACTION_AND_ROLLBACK_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            COMPACTION_AND_ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
        );
        let HistoryWithItsStepImages { run, step_images } = run_the_history_keeping_the_step_images(
            &history,
            &format!("seed-{seed}"),
            &|operation| matches!(operation, Some(HistoryOperation::RollBackWhileMounted(_))),
        );
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "种子 {seed}（偏移 {offset}）：{:?}；结局 {:?}",
            history.operations,
            run.outcomes
        );
        step_images_after_the_rollbacks.extend(step_images);
        let mut has_relocated_a_unit = false;
        for outcome in &run.outcomes {
            match outcome {
                StepOutcome::Applied(AppliedEffect::Compacted {
                    units_relocated, ..
                }) if *units_relocated > 0 => has_relocated_a_unit = true,
                StepOutcome::Applied(AppliedEffect::RolledBack { .. }) if has_relocated_a_unit => {
                    rollbacks_after_a_relocation += 1;
                }
                StepOutcome::Applied(_)
                | StepOutcome::Refused { .. }
                | StepOutcome::NotApplicable(_) => {}
            }
        }
    }
    eprintln!(
        "整理夹着回退那一组比重 {COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS} 段：整理搬过单元之后做成的回退 {rollbacks_after_a_relocation} 次；回退之后交判定库的镜像 {} 张",
        step_images_after_the_rollbacks.len()
    );
    assert!(
        rollbacks_after_a_relocation >= 1,
        "{COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS} 段里一次都没有在整理搬过单元之后回退"
    );
    let Some(violations) = pool_checker_violations_judged_through_the_library(
        "the_compaction_and_rollback_weights_run_every_history_to_the_end_with_the_model_agreeing",
        &step_images_after_the_rollbacks,
    ) else {
        return;
    };
    assert!(
        violations.is_empty(),
        "{COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS} 段：每次回退之后的镜像池级 checker 一条都不红，实际 {violations:?}"
    );
}
