//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_amplification
//! 里程碑「覆盖写、释放、回退与复用」步 4（管理员回退改成挂着时的向前发布）：回退那次发布 D 的每个崩溃点各恢复一次。
//! 每个前缀那一张镜像上的池级 checker 走两段：D 的每个前缀作一个给定状态（基线是 D 之前的盘、
//! 写表是 D 那一段录制流里的写），整张表交崩溃点重放的给定状态引擎（`crash_amplification::GivenStatesEngine`）：录入段写进给定状态表那一类的
//! 判定库，判段从库里读回判（配了 GPU 走 GPU 判器，没配由 CPU 判器判），判过的复用。
//! 它逐个前缀造崩溃状态。同一条脚本的日常用例在 `crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs`，
//! 两边的搭建函数都在共用的场景文件 `tests/common_images_for_the_given_states_engine/rollback_by_a_forward_publish.rs` 里，经 `#[path]` 引。

mod common;
mod common_images_for_the_given_states_engine;
mod common_shared_by_harness_scenarios;
#[path = "common_images_for_the_given_states_engine/rollback_by_a_forward_publish.rs"]
mod scenarios_of_rollback_by_a_forward_publish;

use common::{file_content, geometry, IMAGE_BYTES};
use common_shared_by_harness_scenarios::{roll_back, third_content};
use scenarios_of_rollback_by_a_forward_publish::{
    build_through_third_publish, first_root, ThroughTheThirdPublish,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_verification::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_verification::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool};
use singlefs_verification::RetainedOperation;

/// D 的每个前缀那一张镜像上的池级 checker 违例（不变量、细节），与 `prefixes` 同序：D 的每个前缀作一个给定状态
/// （基线是 D 之前的盘、写表是 D 那一段录制流里的写，前缀 p 的持久集合是录制流下标小于 p 的写），整张表交崩溃点重放的给定状态引擎两段判
/// （录入段写进库、判段从库里读回判，配了 GPU 走 GPU 判器）。只录那一趟、表归别的机器判的那一趟交 `None`（用例当场返回）。
///
/// # Panics
/// 引擎起不了、判不了这张表（不退回别处判）；GPU 判不了的状态（H6 判红）。
fn pool_checker_violations_of_the_prefixes(
    operations: &[RetainedOperation],
    operations_before: usize,
    prefixes: &[usize],
) -> Option<Vec<Vec<(&'static str, String)>>> {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    base.apply(&operations[..operations_before]);
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations[operations_before..], &geometry());
    let persisted_sets: Vec<Vec<bool>> = prefixes
        .iter()
        .map(|prefix| {
            stream_indexes
                .iter()
                .map(|stream_index| operations_before + stream_index < *prefix)
                .collect()
        })
        .collect();
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!(
                    "crash_points_of_the_rollback_publish::prefixes={}",
                    prefixes.len()
                ),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let judged = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: "every-prefix-of-the-rollback-publish".to_string(),
            base: &base,
            writes: &writes,
            segments: &segments,
            seams: &[],
            persisted_sets,
        })
        .unwrap_or_else(|reason| panic!("D 的前缀那张给定状态表判不了（不退回别处判）：{reason}"));
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的状态照第三轮正文 H6 判红"
    );
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => return None,
    }
    assert_eq!(
        judged.states.len(),
        prefixes.len(),
        "判器逐状态交回结论，与前缀同序"
    );
    Some(
        judged
            .states
            .iter()
            .map(GivenStateConclusion::pool_checker_violations)
            .collect(),
    )
}

/// 验收「崩在 D 的每个崩溃点」：录制流里 D 那一段的每一个前缀各恢复一次——D 的记录还没持久（它那一段写之前、屏障之前）⇒ 恢复到 C (2, 8)、
/// 读回第三次的内容；D 的记录已持久、根槽还没 ⇒ 恢复由记录重建 D 的根（D23（journal 的角色与格式） 已定项 15）；根槽已持久 ⇒ 择 D。
/// 后两种都读回第一次的内容；前缀越长只会从前一种走到后一种、不回头。每个前缀 checker 全绿。
// harness-test-granularity:one-scenario 循环里每轮建的是同一条回退发布 D 录制流的一个前缀崩溃镜像，不是新场景；「恢复到过 D 之后前缀更长不回到 C」那一句跨轮判，拆开就判不了
// crash-case-check:not-a-crash-case 录制流里回退发布 D 那一段的每个前缀各恢复一次，自己造前缀崩溃镜像，不调枚举一族函数、不经块判定库
#[test]
fn every_crash_point_of_the_rollback_publish_recovers_to_the_third_version_or_to_the_rollback_publish_and_every_image_is_green(
) {
    let ThroughTheThirdPublish { mut pool, .. } =
        build_through_third_publish("step-four-forward-crash-points");
    let operations_before = pool.retained_operations().len();
    roll_back(&mut pool, first_root()).expect("回退到 A");
    let operations = pool.retained_operations();
    assert!(operations.len() > operations_before + 10, "D 写了一段");
    let prefixes: Vec<usize> = (operations_before..=operations.len()).collect();
    let Some(violations_of_each_prefix) =
        pool_checker_violations_of_the_prefixes(&operations, operations_before, &prefixes)
    else {
        return;
    };
    let mut recovered_to_the_rollback_publish_at: Option<usize> = None;
    for (prefix, violations) in prefixes.into_iter().zip(violations_of_each_prefix) {
        let mut image =
            MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        image.apply(&operations[..prefix]);
        let report = recover(&image, JournalPolicy::Consult);
        let (effective_root, content) = match &report.outcome {
            RecoveryOutcome::FileRead { content, .. } => (
                report.effective_root.expect("读回了文件就有实际走的那条根"),
                content.clone(),
            ),
            other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
                panic!("前缀 {prefix}：恢复要读回文件，交回 {other:?}")
            }
        };
        if effective_root == (InstanceGeneration(2), CheckpointTxg(9)) {
            assert_eq!(content, file_content(), "前缀 {prefix}：D 读回第一次的内容");
            recovered_to_the_rollback_publish_at.get_or_insert(prefix);
        } else {
            assert_eq!(
                effective_root,
                (InstanceGeneration(2), CheckpointTxg(8)),
                "前缀 {prefix}：D 没立住就是 C"
            );
            assert_eq!(
                content,
                third_content(),
                "前缀 {prefix}：C 读回第三次的内容"
            );
            assert!(
                recovered_to_the_rollback_publish_at.is_none(),
                "前缀 {prefix}：恢复到过 D 之后前缀更长又回到 C"
            );
        }
        assert!(
            violations.is_empty(),
            "D 的前缀 {prefix}：池级 checker 一条违例都没有：{violations:?}"
        );
    }
    let recovered_to_the_rollback_publish_at =
        recovered_to_the_rollback_publish_at.expect("整条 D 落完之后恢复到 D");
    assert!(
        recovered_to_the_rollback_publish_at < operations.len(),
        "D 的根槽写之前就已经恢复到 D（记录重建那一格）：从前缀 {recovered_to_the_rollback_publish_at} 起"
    );
}
