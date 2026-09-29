//! checker 档模块：crash、layer0_progress、crash_amplification
//! 崩溃放量流水线在里程碑「覆盖写、释放、回退与复用」的整条固定脚本上（步 0 到步 5：取号加暖机 → 新池新建文件 A → 覆盖写 B →
//! 重开可写挂载（取号、写行、暖机两次）→ 覆盖写 C → 挂着回退到 A（D）→ 五次覆盖写（txg 10–14）→ 抬 F 到 11（两次空发布 txg 15、16）→
//! 复用 E（txg 17，数据单元落回 50176）→ 正常卸载（先写 F、两次带卸载记号的空发布 txg 18、19））：与层 0 全量跑的那条流同一条
//! （`crash_enumeration_fixed_script_stream.rs` 的 `ReuseAfterRaisingFloorThenNormalUnmount`，78 段、477 次写、19 条根槽写）。
//! 入口都由 `common_crash_points` 设点，前 36 个点与到 D 那条测试共享。展开上限 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`（默认 6），
//! 28 及以上就是全域（`exhaustive=true`）。要 `verdict-store` 特性。
#![cfg(feature = "verdict-store")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::path::PathBuf;
use std::time::Instant;

use common::{build_pool, file_content, geometry};
use common_crash_points::{
    mount_writable_step, overwrite_step, raise_rollback_floor_step, record_pool_build,
    roll_back_step, unmount_step,
};
use singlefs_checker_tier::crash::{layer0_plan_state_count, Layer0SegmentExpansion};
use singlefs_checker_tier::crash_amplification::{
    checking_threads, plan_crash_points, run_from_environment, CrashFlow, CrashPointRecorder,
    PipelineJudgingCode,
};
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::RollbackTarget;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes_and_entries, PublishedVersion,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RecordedEntrySpan;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;
const DEFAULT_EXPAND_UP_TO: usize = 6;
/// E 的数据单元落回最低的可再分配偶数槽对：mkfs 实例表那两槽（释放代 5，抬 F 到 11 回收了）。
const REUSED_DATA_UNIT_SLOT: u64 = 50176;

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

fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
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

fn version(instance: u32, txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
        content,
    }
}

#[test]
fn the_whole_fixed_script_through_the_unmount_is_recorded_checked_in_parallel_and_reused() {
    let started = Instant::now();
    let mut pool = build_pool("crash-amplification-fixed-script-unmount");
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![version(1, 3, file_content())];
    let second = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    versions.push(version(1, 4, second_content()));
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(version(2, txg, second_content()));
    }
    let third = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    versions.push(version(2, 8, third_content()));
    pool.output = third;
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    versions.push(version(2, 9, file_content()));
    // 回退之后再覆盖写五次（txg 10–14）：第一次释放 D 复活的 A 的数据单元、释放代 10
    let mut latest = Vec::new();
    for (txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
        latest = later_content(seed);
        pool.output = overwrite_step(&mut recorder, &mut pool, &latest, InstanceGeneration(2));
        versions.push(version(2, txg, latest.clone()));
    }
    // 抬 F 到 11：先写每块盘的系统配置，再两次空发布（txg 15 落盘 0、txg 16 落盘 1）
    let raised = raise_rollback_floor_step(&mut recorder, &mut pool, CheckpointTxg(11));
    assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
    for txg in 15..=16 {
        versions.push(version(2, txg, latest.clone()));
    }
    // E：数据单元落回最低的可再分配偶数槽对
    let reuse = overwrite_step(
        &mut recorder,
        &mut pool,
        &later_content(31),
        InstanceGeneration(2),
    );
    assert_eq!(
        reuse.data_pointers[0].locations[0].slot.0, REUSED_DATA_UNIT_SLOT,
        "E 的数据单元落回 mkfs 实例表那两槽（释放代 5，抬 F 到 11 回收了）"
    );
    versions.push(version(2, 17, later_content(31)));
    pool.output = reuse;
    // 正常卸载：先把 F = 17 写进每块盘的系统配置、过一道屏障，再推带卸载记号的空发布直到每块盘上都有一条（txg 18、19）
    let unmounted = unmount_step(&mut recorder, &mut pool);
    assert_eq!(
        unmounted
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(18), CheckpointTxg(19)],
        "卸载推两次空发布：txg 18 落盘 0、txg 19 落盘 1"
    );
    for txg in 18..=19 {
        versions.push(version(2, txg, later_content(31)));
    }

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    // 带卸载记号的根槽写只许落在卸载入口发的那一段里（C557）：入口段一起交给切段
    let entry_spans: Vec<RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - pool.mkfs_operation_count
                ..span.operations.end - pool.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations[pool.mkfs_operation_count..],
        &entry_spans,
        &geometry(),
    );
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![
            2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1,
            2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2,
            1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2
        ],
        "固定脚本到 E 再正常卸载的段序列（与 crash_enumeration_fixed_script_stream 登记的相同）"
    );
    assert_eq!(
        writes.len(),
        191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),
        "到 D 191 + 五次覆盖写 165 + 抬 F 44 + E 33 + 卸载 44"
    );
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    assert_eq!(crash_points.len(), segments.len(), "每个屏障段一个崩溃点");
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
            ("roll_back_by_a_forward_publish", 25),
            ("publish_overwrite", 33 * 5),
            ("raise_rollback_floor", 44),
            ("publish_overwrite", 33),
            ("unmount", 44)
        ],
        "十个入口各管的写数（连着的五次覆盖写归拢成一行）"
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
            "roll_back_by_a_forward_publish.system_configuration_slot.1",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "raise_rollback_floor.system_configuration_slot.1",
            "raise_rollback_floor.index_node.1",
            "raise_rollback_floor.journal_record.1",
            "raise_rollback_floor.root_record_fua.1",
            "raise_rollback_floor.system_configuration_slot.2",
            "raise_rollback_floor.index_node.2",
            "raise_rollback_floor.journal_record.2",
            "raise_rollback_floor.root_record_fua.2",
            "raise_rollback_floor.system_configuration_slot.3",
            "publish_overwrite.data_unit+index_node+packed_records.1",
            "publish_overwrite.journal_record.1",
            "publish_overwrite.root_record_fua.1",
            "publish_overwrite.system_configuration_slot.1",
            "unmount.system_configuration_slot.1",
            "unmount.index_node.1",
            "unmount.journal_record.1",
            "unmount.root_record_fua.1",
            "unmount.system_configuration_slot.2",
            "unmount.index_node.2",
            "unmount.journal_record.2",
            "unmount.root_record_fua.2",
            "unmount.system_configuration_slot.3"
        ],
        "78 个点的名字：前 36 个与到 D 那条相同；五次覆盖写各 4 段；抬 F 9 段（先写 F 的系统配置 2、两次空发布各 16+2+1、末尾轮换 2）；E 4 段；卸载 9 段（与抬 F 同形）"
    );
    assert!(
        crash_points
            .iter()
            .all(|point| point.code_file_in_repository == common_crash_points::THIS_FILE),
        "十个入口都在公共模块里设点"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    assert_eq!(
        writes
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .count(),
        19,
        "每次发布一条根槽写"
    );
    let limit = expand_up_to();
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
    let judging = PipelineJudgingCode::of_the_judges(
        &root,
        &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("判法摘要");
    let plan = plan_crash_points(&flow);
    let state_count = layer0_plan_state_count(&base, &writes, &segments, &expansion);
    assert_eq!(plan.points.last().expect("有点").ordinals.end, state_count);
    let library_guard = match std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY") {
        Some(_) => None,
        None => {
            let directory = std::env::temp_dir().join(format!(
                "singlefs-crash-amplification-fixed-script-unmount-{}",
                std::process::id()
            ));
            std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", &directory);
            std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY", "1");
            Some(TemporaryStore { directory })
        }
    };
    let checking_started = Instant::now();
    let run = run_from_environment(&flow, &judging, &root).expect("按环境跑得完");
    let checking_seconds = checking_started.elapsed().as_secs_f64();
    if run.placement_only {
        // 只排派活计划的那一趟一块都没判：不断言、不打汇总行
        drop(library_guard);
        return;
    }
    if run.stopped_early {
        // 被叫停的那一趟：判过的块在库里、第 ②③ 段没做；只报判了多少，不断言对账
        println!(
            "CRASH_AMPLIFICATION mode={} share={}/{} states={state_count} states_checked_here={} blocks_checked_here={} unjudged_blocks={} red={} stopped_early=true",
            run.mode.name(),
            run.share.index,
            run.share.count,
            run.checking.states_checked,
            run.checking.blocks_checked,
            run.unjudged_blocks,
            run.checking.red_states
        );
        assert_eq!(
            run.checking.red_states, 0,
            "健康的固定脚本一个红状态都不该有：{:?}",
            run.checking.red_texts
        );
        drop(library_guard);
        return;
    }
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
        // 库里这一判法版本已经判过的块复用、不重核（续跑）：这一趟核的是它录下来还没判的那些
        assert_eq!(
            run.checking.states_checked, run.recording.states_recorded,
            "这一趟录下来要判的状态都核了"
        );
        let second = run_from_environment(&flow, &judging, &root).expect("第二趟跑得完");
        assert_eq!(
            second.recording.blocks_already_judged,
            run.recording.blocks_recorded + run.recording.blocks_already_judged,
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
        // 两台各跑一份、还没导入对方的库时，本机库里有对方那份的块（本机先判着的那一段判的），它们归对方核，这时不判
        if run.share.count == 1 || imported {
            assert_eq!(
                comparison.blocks_without_verifier, 0,
                "判过的块第 ② 段都核过"
            );
        }
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
    let gpu_judge_line = common_crash_points::judge_fields_of_the_summary_line(&run);
    drop(library_guard);
    for line in &run.coverage_lines {
        println!("CRASH_AMPLIFICATION_POINT {line}");
    }
    println!(
        "CRASH_AMPLIFICATION mode={} share={}/{} cards={} states={state_count} states_checked_here={} blocks_checked_here={} imported_blocks={} unjudged_blocks={} red={} threads={} judge_version={} expand_up_to={limit} checking_seconds={checking_seconds:.1} states_per_second={:.0} total_seconds={:.1} exhaustive={exhaustive}{verifier_line}{gpu_judge_line}",
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
        run.judge_version
            .0
            .iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        if checking_seconds > 0.0 {
            run.checking.states_checked as f64 / checking_seconds
        } else {
            0.0
        },
        started.elapsed().as_secs_f64()
    );
}
