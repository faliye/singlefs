//! checker 档模块：crash、layer0_progress、crash_amplification
//! 崩溃放量流水线在固定脚本到回退 D 那条流上（取号加暖机 → 新池新建文件 A → 覆盖写 B → 重开可写挂载 → 覆盖写 C → 挂着回退到 A）：
//! 六个入口在调时用 `CrashPointRecorder` 记下（共用的步骤在 `common_crash_points` 里，几条流共一个身份），换成崩溃点时每个屏障段一个点、
//! 名字 `<入口>.<写的种类>.<步>`（`publish_overwrite` 出现两次，同名、树上位置不同）；
//! 展开不超过 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`（默认 6）次写的段，块之间多线程核对；红必须是 0，状态数与闭式逐个相等，
//! 第二趟全部复用。展开到 18 时约 39 万个状态（一个 18 写段、两个 16 写段）——用户 2026-09-28 定先跑二十万级。要 `verdict-store` 特性。
#![cfg(feature = "verdict-store")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::path::PathBuf;
use std::time::Instant;

use common::{build_pool, file_content, geometry};
use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
use singlefs_checker_tier::crash::{layer0_plan_state_count, Layer0SegmentExpansion};
use singlefs_checker_tier::crash_amplification::{
    checking_threads, plan_crash_points, run_from_environment, CrashFlow, CrashPointRecorder,
    JudgingCodes,
};
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::RollbackTarget;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, PublishedVersion};
use singlefs_harness::segments::StepKind;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;
const DEFAULT_EXPAND_UP_TO: usize = 6;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

fn expand_up_to() -> usize {
    std::env::var("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(DEFAULT_EXPAND_UP_TO)
}

struct TemporaryStore {
    directory: PathBuf,
}

impl Drop for TemporaryStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn the_fixed_script_through_the_rollback_is_recorded_checked_in_parallel_and_reused() {
    let started = Instant::now();
    let mut pool = build_pool("crash-amplification-fixed-script-rollback");
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let second = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(4),
        content: second_content(),
    });
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let third = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
        content: third_content(),
    });
    pool.output = third;
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(9),
        content: file_content(),
    });

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![
            2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1,
            2, 24, 2, 1, 2, 20, 2, 1, 2
        ],
        "固定脚本到 D 为止的段序列（与 crash_enumeration_fixed_script_stream 登记的相同）"
    );
    assert_eq!(writes.len(), 191, "到 C 166 + D 25");
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    assert_eq!(crash_points.len(), segments.len(), "每个屏障段一个崩溃点");
    // 按入口归拢：名字 `<入口>.<周期>.<步>` 的第一段是入口
    let mut entries: Vec<(&str, usize)> = Vec::new();
    for point in &crash_points {
        let entry = point.name.split('.').next().expect("名字以入口起头");
        let count = point.writes.end - point.writes.start;
        match entries.last_mut() {
            Some((last, total)) if *last == entry => *total += count,
            _ => entries.push((entry, count)),
        }
    }
    assert_eq!(
        entries,
        vec![
            ("acquire_instance_and_warm_up", 12),
            ("publish_first_file", 29),
            ("publish_overwrite", 29),
            ("mount_writable", 67),
            ("publish_overwrite", 29),
            ("roll_back_by_a_forward_publish", 25)
        ],
        "六个入口各管的写数：取号 2 + 暖机 5 × 2；A 29；B 29；重开取号 2 + 写行 23 + 暖机 21 × 2；C 29；D 25"
    );
    let names: Vec<&str> = crash_points
        .iter()
        .map(|point| point.name.as_str())
        .collect();
    eprintln!("CRASH_AMPLIFICATION_NAMES {names:?}");
    assert_eq!(
        names,
        vec![
            "acquire_instance_and_warm_up.system_configuration_slot.1",
            "acquire_instance_and_warm_up.journal_record.1",
            "acquire_instance_and_warm_up.root_record_fua.1",
            "acquire_instance_and_warm_up.system_configuration_slot.2",
            "acquire_instance_and_warm_up.journal_record.2",
            "acquire_instance_and_warm_up.root_record_fua.2",
            "acquire_instance_and_warm_up.system_configuration_slot.3",
            "publish_first_file.data_unit+index_node+packed_records.1",
            "publish_first_file.journal_record.1",
            "publish_first_file.root_record_fua.1",
            "publish_first_file.system_configuration_slot.1",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "mount_writable.system_configuration_slot.1",
            "mount_writable.packed_records+index_node.1",
            "mount_writable.journal_record.1",
            "mount_writable.root_record_fua.1",
            "mount_writable.system_configuration_slot.2",
            "mount_writable.index_node.1",
            "mount_writable.journal_record.2",
            "mount_writable.root_record_fua.2",
            "mount_writable.system_configuration_slot.3",
            "mount_writable.index_node.2",
            "mount_writable.journal_record.3",
            "mount_writable.root_record_fua.3",
            "mount_writable.system_configuration_slot.4",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "roll_back_by_a_forward_publish.index_node.1",
            "roll_back_by_a_forward_publish.journal_record.1",
            "roll_back_by_a_forward_publish.root_record_fua.1",
            "roll_back_by_a_forward_publish.system_configuration_slot.1"
        ],
        "36 个点的名字 <入口>.<周期：写的种类，单元写按类标签分 data_unit / index_node / packed_records>.<这个入口里这一种的第几段>：取号加暖机 7 段、A 4 段、B 4 段、重开 13 段（写行那一段是打包记录加索引节点、暖机两段只有索引节点）、C 4 段、D 4 段（回退只写索引节点）"
    );
    assert!(
        crash_points
            .iter()
            .all(|point| point.code_file_in_repository == common_crash_points::THIS_FILE),
        "六个入口都在公共模块里设点"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    let limit = expand_up_to();
    // 每一段都不超过上限就是全域：每个屏障段的每个真子集都枚举到了
    let exhaustive = segments.iter().all(|segment| segment.len() <= limit);
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index,
        versions: &versions,
        expansion: &expansion,
        crash_points,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("仓根");
    let judging = JudgingCodes::of_the_flow(
        &root,
        &flow,
        &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("判法摘要");
    let plan = plan_crash_points(&flow, &judging);
    let state_count = layer0_plan_state_count(&base, &writes, &segments, &expansion);
    assert_eq!(plan.points.last().expect("有点").ordinals.end, state_count);
    let overwrite_points: Vec<usize> = plan
        .points
        .iter()
        .enumerate()
        .filter(|(_, point)| point.name.starts_with("publish_overwrite."))
        .map(|(index, _)| index)
        .collect();
    let (first_overwrite, second_overwrite) = (
        overwrite_points[0],
        overwrite_points[overwrite_points.len() / 2],
    );
    assert_eq!(
        plan.points[first_overwrite].name, plan.points[second_overwrite].name,
        "两次 publish_overwrite 的第一段同名"
    );
    assert_ne!(
        plan.points[first_overwrite].reuse_key, plan.points[second_overwrite].reuse_key,
        "两次 publish_overwrite 同名，复用键不同"
    );
    assert_ne!(
        plan.points[first_overwrite].path_number, plan.points[second_overwrite].path_number,
        "两次 publish_overwrite 在树上是两个位置（路径深度不同）"
    );
    let library_guard = match std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY") {
        Some(_) => None,
        None => {
            let directory = std::env::temp_dir().join(format!(
                "singlefs-crash-amplification-fixed-script-{}",
                std::process::id()
            ));
            std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", &directory);
            Some(TemporaryStore { directory })
        }
    };
    let checking_started = Instant::now();
    let run = run_from_environment(&flow, &judging, &root).expect("按环境跑得完");
    let checking_seconds = checking_started.elapsed().as_secs_f64();
    assert_eq!(
        run.recording.states_recorded + run.recording.states_already_judged,
        state_count,
        "每个状态都录了"
    );
    assert_eq!(
        run.checking.red_states, 0,
        "健康的固定脚本一个红状态都不该有：{:?}",
        run.checking.red_texts
    );
    assert_eq!(run.checking.gpu_cpu_disagreements, 0);
    let imported = std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_IMPORT").is_some();
    if run.share.count == 1 || imported {
        assert_eq!(
            run.unjudged_blocks, 0,
            "整份自己跑、或导入了别的机器的库之后，对账为空"
        );
        assert!(run.violations.is_empty());
    }
    if run.share.count == 1 {
        assert_eq!(run.checking.states_checked, state_count, "每个状态都核了");
        let second = run_from_environment(&flow, &judging, &root).expect("第二趟跑得完");
        assert_eq!(
            second.recording.blocks_already_judged, run.recording.blocks_recorded,
            "第二趟全部复用"
        );
        assert_eq!(second.checking.blocks_checked, 0, "复用的块不再核");
    }
    // 第 ②③ 段跑了就要：核对红 ⇒ 判器红，一条不一致都不许
    if let Some(comparison) = &run.comparison {
        assert_eq!(
            comparison.disagreements, 0,
            "核对红而判器绿：{:?}",
            comparison.disagreement_samples
        );
        assert_eq!(
            comparison.blocks_without_verifier, 0,
            "这台领的块第 ② 段都核过"
        );
    }
    let verifier_line = match (&run.verification, &run.comparison) {
        (Some(outcome), Some(comparison)) => format!(
            " verifier={} verified_states={} verifier_red={} compared={} both_red={} judge_red_only={} disagreements={}",
            outcome.verifier,
            outcome.tally.states_verified,
            outcome.tally.verifier_red_states,
            comparison.states_compared,
            comparison.both_red,
            comparison.judge_red_only,
            comparison.disagreements
        ),
        _ => " verifier=none".to_string(),
    };
    drop(library_guard);
    for line in &run.coverage_lines {
        println!("CRASH_AMPLIFICATION_POINT {line}");
    }
    println!(
        "CRASH_AMPLIFICATION mode={} share={}/{} cards={} states={state_count} states_checked_here={} blocks_checked_here={} imported_blocks={} unjudged_blocks={} red={} threads={} expand_up_to={limit} checking_seconds={checking_seconds:.1} states_per_second={:.0} total_seconds={:.1} exhaustive={exhaustive}{verifier_line}",
        run.mode.name(),
        run.share.index,
        run.share.count,
        run.cards_used,
        run.checking.states_checked,
        run.checking.blocks_checked,
        run.imported_blocks,
        run.unjudged_blocks,
        run.checking.red_states,
        checking_threads(),
        f64::from(u32::try_from(run.checking.states_checked).expect("装得进 u32")) / checking_seconds.max(0.001),
        started.elapsed().as_secs_f64()
    );
}
