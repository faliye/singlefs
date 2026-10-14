//! checker 档模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification
//! 里程碑「覆盖写、释放、回退与复用」步 4（管理员回退改成挂着时的向前发布）：回退那次发布 D 的每个崩溃点各恢复一次。
//! 每个前缀那一张镜像上的池级 checker 走两段（用户 2026-10-13 定「能走两段的都走两段」）：D 的每个前缀作一个给定状态（基线是 D 之前的盘、
//! 写表是 D 那一段录制流里的写），整张表交崩溃点重放的给定状态引擎（`crash_amplification::GivenStatesEngine`）：录入段写进给定状态表那一类的
//! 判定库，判段从库里读回判（配了 GPU 走 GPU 判器，没配由 CPU 判器判），判过的复用。
//! 它逐个前缀造崩溃状态，归 checker 档（`.claude/rules/verification.md`「定义与名字」）；
//! 同一条脚本的日常用例在 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`，
//! 这里用到的几个搭建函数从那里抄了一份（两档宁可各留一份，不让 harness 依赖 checker 档）。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker_tier::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_library::repository_root_of_this_crate;
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool};
use singlefs_harness::RetainedOperation;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C 的池（D 之前的样子）。
struct ThroughTheThirdPublish {
    pool: BuiltPool,
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> ThroughTheThirdPublish {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    ThroughTheThirdPublish { pool }
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 挂着的时候回退：交回回退的结局，做成时 `pool.output` 已换成回退那次发布之后的一版。
fn roll_back(pool: &mut BuiltPool, target: RollbackTarget) -> Result<RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        target,
    )
}

/// 这份文件的仓内路径：给定状态表那一路的设点文件（CPU 结论的版本按它出发的闭包取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_points_of_the_rollback_publish.rs";

/// D 的每个前缀那一张镜像上的池级 checker 违例（不变量、细节），与 `prefixes` 同序：D 的每个前缀作一个给定状态
/// （基线是 D 之前的盘、写表是 D 那一段录制流里的写，前缀 p 的持久集合是录制流下标小于 p 的写），整张表交崩溃点重放的给定状态引擎两段判
/// （录入段写进库、判段从库里读回判，配了 GPU 走 GPU 判器）。只录那一趟、表归别的机器判的那一趟交 `None`（用例当场返回）。
///
/// # Panics
/// 引擎起不了、判不了这张表（不退回别处判）；GPU 判不了的状态（第三轮正文 H6 判红）。
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
                item_name: "crash_points_of_the_rollback_publish".to_string(),
                setting_file: THIS_FILE.to_string(),
                seeds_and_scale: format!("prefixes={}", prefixes.len()),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
            &repository_root_of_this_crate(),
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
