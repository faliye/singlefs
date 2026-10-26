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

mod common_compaction_and_rollback_histories;

use common_compaction_and_rollback_histories::{
    compaction_and_rollback_history, is_a_rollback_step, run_the_history_keeping_the_step_images,
    seed_of_the_compaction_and_rollback_sampling, HistoryWithItsStepImages,
};
use singlefs_checker_tier::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_harness::gpu_memory_ledger::gpu_not_run_line;
use singlefs_harness::heavy_threads::default_heavy_threads;
use singlefs_harness::history::{
    AppliedEffect, CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength,
    FloorTargetChoice, GeneratedHistory, HistoryEnding, HistoryOperation, HistoryRun, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice, StepOutcome,
};
use singlefs_harness::memory_pool::MemoryPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// 攻方腿那段前缀里整理强制选段用的选择子（执行时对候选段个数取余）。
const REGION_SELECTOR_OF_THE_ATTACK: u64 = 16_033_315_046_779_662_336;
/// 回退目标在根环里从最新往旧数的下标，扫这么多条（攻方腿扫的 0 到 7）。
const RING_ROOTS_SWEPT_BY_THE_ROLLBACK: u64 = 8;
/// 那一组比重跑的几段、每段几步（与 harness 档原先那一条逐项相同）。
const COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS: u64 = 4;
const COMPACTION_AND_ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;

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
/// 各段种子的历史在执行器上跑那一截按段切片、在每台默认重活线程数条线程上跑（[`compaction_and_rollback_segments_on_worker_threads`]），
/// 按偏移的次序并回来：交判定库的表的次序、张数与串行跑逐字相同（对拍
/// `the_compaction_and_rollback_segments_on_several_threads_come_back_like_the_segments_run_one_after_another`）；判定库那一截照旧在这条线程上逐张交。
#[test]
fn the_compaction_and_rollback_weights_run_every_history_to_the_end_with_the_model_agreeing() {
    let mut rollbacks_after_a_relocation = 0_u64;
    let mut step_images_after_the_rollbacks: Vec<(String, MemoryPool)> = Vec::new();
    for (offset, segment) in
        compaction_and_rollback_segments_on_worker_threads(default_heavy_threads().get())
            .into_iter()
            .enumerate()
    {
        assert_eq!(
            segment.run.ending,
            HistoryEnding::Completed,
            "种子 {}（偏移 {offset}）：{:?}；结局 {:?}",
            segment.seed,
            segment.history.operations,
            segment.run.outcomes
        );
        step_images_after_the_rollbacks.extend(segment.step_images);
        rollbacks_after_a_relocation += segment.rollbacks_after_a_relocation;
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

/// 整理夹着回退那一组比重一段种子的历史在执行器上跑完的样子：种子、生成的历史、执行器交回的这一趟、回退那几步之后交给观察者的镜像、
/// 这一段里整理搬过单元之后做成的回退几次。
#[derive(PartialEq, Eq)]
struct CompactionAndRollbackSegment {
    seed: u64,
    history: GeneratedHistory,
    run: HistoryRun,
    step_images: Vec<(String, MemoryPool)>,
    rollbacks_after_a_relocation: u64,
}

/// 种子区间里偏移 `offset` 那一段：生成历史、在两块 4 GiB 的盘上跑、只拷回退那几步之后的镜像（与 harness 档原先那一条同一个取法），
/// 数这一段里整理搬过单元之后做成的回退。每段自己生成历史、自己建池，不读别的段留下的东西。
fn run_the_compaction_and_rollback_segment(offset: u64) -> CompactionAndRollbackSegment {
    let seed = seed_of_the_compaction_and_rollback_sampling(offset);
    let history = compaction_and_rollback_history(
        seed,
        COMPACTION_AND_ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY,
    );
    let HistoryWithItsStepImages { run, step_images } = run_the_history_keeping_the_step_images(
        &history,
        &format!("seed-{seed}"),
        &is_a_rollback_step,
    );
    let mut rollbacks_after_a_relocation = 0_u64;
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
    CompactionAndRollbackSegment {
        seed,
        history,
        run,
        step_images,
        rollbacks_after_a_relocation,
    }
}

/// 每一段在这条线程上依次跑，按偏移的次序交回：多线程那一份的对照。
fn compaction_and_rollback_segments_one_after_another() -> Vec<CompactionAndRollbackSegment> {
    (0..COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS)
        .map(run_the_compaction_and_rollback_segment)
        .collect()
}

/// 同一批段切成片，在 `worker_threads` 条线程上跑（多于段数的不起）：每条线程按偏移领下一段，跑完的段放进它偏移那一格，
/// 全部跑完按偏移的次序并起来交回（与 [`compaction_and_rollback_segments_one_after_another`] 逐段相同，与线程数、谁先跑完无关）。
///
/// # Panics
/// 有一段跑的时候 panic（`std::thread::scope` 等齐之后照样 panic）；并的时候有一格空着。
fn compaction_and_rollback_segments_on_worker_threads(
    worker_threads: usize,
) -> Vec<CompactionAndRollbackSegment> {
    let segment_count = usize::try_from(COMPACTION_AND_ROLLBACK_SAMPLING_SEEDS)
        .expect("段数是个小常量，装得进 usize");
    let next_segment_index = AtomicUsize::new(0);
    let segment_of_each_offset: Mutex<Vec<Option<CompactionAndRollbackSegment>>> =
        Mutex::new((0..segment_count).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _worker in 0..worker_threads.clamp(1, segment_count) {
            scope.spawn(|| loop {
                let segment_index = next_segment_index.fetch_add(1, Ordering::SeqCst);
                if segment_index >= segment_count {
                    break;
                }
                let offset = u64::try_from(segment_index).expect("段的下标小于段数，装得进 u64");
                let segment = run_the_compaction_and_rollback_segment(offset);
                segment_of_each_offset
                    .lock()
                    .expect("别的跑段线程 panic 时 scope 等齐之后照样 panic，走不到这里")
                    [segment_index] = Some(segment);
            });
        }
    });
    let segments_in_the_order_of_their_offsets = segment_of_each_offset
        .into_inner()
        .expect("跑段的线程都已等齐，没有一条 panic 着放下锁");
    segments_in_the_order_of_their_offsets
        .into_iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            segment.unwrap_or_else(|| panic!("偏移 {segment_index} 那一段没有线程跑"))
        })
        .collect()
}

/// 多线程与单线程的对拍（`.claude/rules/test-script-discipline.md`「编译与算法都要快」：合并要确定、逐字相同，进快档）：整理夹着回退那一组比重的各段
/// 切片在几条线程上跑、按偏移并回来的，与依次跑的逐段相同——种子、历史、结局与每一步的结局、回退之后的镜像（表名与逐字节的盘面）、
/// 整理搬过单元之后的回退次数都一样，交判定库的表的次序与张数因此不变。不经判定库、不起 GPU。线程数取每台默认重活线程数，至少两条
/// （一条就比不出合并的次序）。放 checker 档：被并的那份函数住在这个测试文件里，harness 档的包不依赖 checker 档。
/// 判别力：合并时不按偏移的次序（`crates/mutations.tsv` 里点名这一条的那一行把并好的次序倒过来）这条红。
#[test]
fn the_compaction_and_rollback_segments_on_several_threads_come_back_like_the_segments_run_one_after_another(
) {
    let worker_threads = default_heavy_threads().get().max(2);
    let on_worker_threads = compaction_and_rollback_segments_on_worker_threads(worker_threads);
    let one_after_another = compaction_and_rollback_segments_one_after_another();
    let seeds_and_table_names = |segments: &[CompactionAndRollbackSegment]| {
        segments
            .iter()
            .map(|segment| {
                (
                    segment.seed,
                    segment
                        .step_images
                        .iter()
                        .map(|(table_name, _image)| table_name.clone())
                        .collect::<Vec<String>>(),
                )
            })
            .collect::<Vec<(u64, Vec<String>)>>()
    };
    assert_eq!(
        seeds_and_table_names(&on_worker_threads),
        seeds_and_table_names(&one_after_another),
        "{worker_threads} 条线程上跑的各段按偏移的次序并回来：种子与交判定库的表名同依次跑的逐个相同"
    );
    assert!(
        on_worker_threads == one_after_another,
        "{worker_threads} 条线程上跑的各段：历史、结局、每一步的结局、回退之后的镜像逐字节、搬过单元之后的回退次数同依次跑的逐段相同"
    );
}
