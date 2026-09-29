//! 崩溃放量共用的流程步骤。几个测试文件调这里的同一个函数，就得到同一个身份、同一串路径段：前缀节点在库里只核一次，
//! 别的流直接复用（用户 2026-09-28 定：前缀共享是剪枝的基础，没有它每次放量都从头核）。
//! 崩溃点的文件是这一份（`file!()`），不是调它的测试文件：改调用方的测试文件不动这里的节点。判法摘要与设点的文件无关，
//! 只随两个判器的闭包变（`crash_identity::JUDGE_ENTRY_FILES`）。
#![allow(
    dead_code,
    reason = "共用的步骤模块：每个测试文件只调其中几步，没调的在那个测试二进制里就是没用到"
)]

use super::common::{parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
use singlefs_core::address::CheckpointTxg;
use singlefs_core::address::InstanceGeneration;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, Mounted,
    RaisedFloor, RollbackTarget, ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::RecordedPublishEntry;

/// 这一份的仓内路径，与 `file!()` 归一之后相同；测试拿它核身份里的文件。
pub const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/common_crash_points/mod.rs";

/// `build_pool` 已经发出去的两段：取号加暖机、新池新建文件。
pub fn record_pool_build(recorder: &mut CrashPointRecorder, pool: &BuiltPool) {
    recorder.record_span(
        file!(),
        "acquire_instance_and_warm_up",
        pool.mkfs_operation_count..pool.warm_up_operation_count,
    );
    recorder.record_span(
        file!(),
        "publish_first_file",
        pool.warm_up_operation_count..pool.stream.operation_count(),
    );
}

/// 覆盖写：把现行那一版的内容换成 `content`。
pub fn overwrite_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    recorder.step(file!(), "publish_overwrite", || {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("覆盖写")
    })
}

/// 关掉镜像再可写挂载（重开：取号、写行、暖机）。
pub fn mount_writable_step(recorder: &mut CrashPointRecorder, pool: &mut BuiltPool) -> Mounted {
    recorder.step(file!(), "mount_writable", || {
        let mut devices = pool.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        pool.devices = Some(devices);
        mounted
    })
}

/// 挂着的时候用一次向前发布回退到 `target`。
pub fn roll_back_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    target: RollbackTarget,
) {
    recorder.step(file!(), "roll_back_by_a_forward_publish", || {
        let rollback_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        roll_back_by_a_forward_publish(
            &rollback_parameters,
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            target,
        )
        .expect("挂着的时候回退");
    });
}

/// 抬回退下界 F 到 `new_floor`：先把新 F 写进每块盘的系统配置，再推带新 F 的空发布直到每块盘上都有一条（D16 已定项 1「抬 F 那一串」）。
pub fn raise_rollback_floor_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    new_floor: CheckpointTxg,
) -> RaisedFloor {
    recorder.step(file!(), "raise_rollback_floor", || {
        let mut current = pool.output.clone();
        let raise_devices = pool.devices.as_mut().expect("镜像还开着");
        let raised = raise_rollback_floor(
            &parameters(),
            raise_devices,
            &mut pool.allocator,
            &mut current,
            new_floor,
            ShadowLedger::On,
        )
        .expect("抬 F");
        pool.output = current;
        raised
    })
}

/// 正常卸载（影子账开着），录制流上把这一段记成卸载入口发的（C557：带卸载记号的根槽写只许落在卸载入口里）：
/// 带文件的一版上卸载必抬 F，交回那一串；现行版本换成那一串最后落盘的那一次。
pub fn unmount_step(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
) -> UnmountRaisedTheFloor {
    recorder.step(file!(), "unmount", || {
        let mut current = PoolVersion::WithFile(pool.output.clone());
        let stream = pool.stream.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let allocator = &mut pool.allocator;
        let unmounted = stream
            .record_entry(RecordedPublishEntry::Unmount, || {
                unmount(
                    &parameters(),
                    devices,
                    allocator,
                    &mut current,
                    ShadowLedger::On,
                )
            })
            .expect("正常卸载");
        pool.output = current
            .into_file_version()
            .expect("带文件的一版卸载之后仍带文件");
        match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
            Unmounted::NothingWrittenOnAVersionWithoutFile {
                current: reported_version,
            } => {
                panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
            }
        }
    })
}

/// 一条已经录好的流（起点镜像、写表、段、被判的根槽写、版本表、崩溃点）按环境跑一趟崩溃放量：展开上限从
/// `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 取（没设用 `default_expand_up_to`），库目录从 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 取
/// （没设建在临时目录、跑完删），判法摘要取两个判器的闭包；健康的流一个红状态都不该有。只排派活计划的那一趟（`placement_only`）不判，原样交回。
/// 打一行 `CRASH_AMPLIFICATION flow=<名> …`，交回这一趟的结果。
#[allow(
    clippy::too_many_arguments,
    reason = "一条流的七样各自独立，收成一个结构体只是把七个字段挪个地方"
)]
pub fn amplify_prepared_flow(
    flow_name: &str,
    base: &singlefs_harness::memory_pool::MemoryPool,
    writes: &[singlefs_harness::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[singlefs_harness::memory_pool::PublishedVersion],
    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    default_expand_up_to: usize,
) -> singlefs_checker_tier::crash_amplification::EnvironmentRun {
    use singlefs_checker_tier::crash::Layer0SegmentExpansion;
    use singlefs_checker_tier::crash_amplification::{
        run_from_environment, CrashFlow, PipelineJudgingCode,
    };
    use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
    let limit: usize = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(default_expand_up_to);
    let exhaustive = segments.iter().all(|segment| segment.len() <= limit);
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion: &expansion,
        crash_points,
    };
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("仓根");
    let judging = PipelineJudgingCode::of_the_judges(
        &root,
        &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("判法摘要");
    let temporary_library = match std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY") {
        Some(_) => None,
        None => {
            let directory = std::env::temp_dir().join(format!(
                "singlefs-crash-amplification-{flow_name}-{}",
                std::process::id()
            ));
            std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY", &directory);
            std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY", "1");
            Some(directory)
        }
    };
    let started = std::time::Instant::now();
    let run = run_from_environment(&flow, &judging, &root).expect("按环境跑得完");
    let seconds = started.elapsed().as_secs_f64();
    if let Some(directory) = &temporary_library {
        std::env::remove_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY");
        std::env::remove_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY");
        let _ = std::fs::remove_dir_all(directory);
    }
    if run.placement_only {
        return run;
    }
    if run.stopped_early {
        // 被叫停的那一趟：判过的块在库里、第 ②③ 段没做；只报判了多少，不断言对账
        println!(
            "CRASH_AMPLIFICATION flow={flow_name} mode={} states_checked_here={} blocks_checked_here={} unjudged_blocks={} red={} stopped_early=true",
            run.mode.name(),
            run.checking.states_checked,
            run.checking.blocks_checked,
            run.unjudged_blocks,
            run.checking.red_states
        );
        assert_eq!(
            run.checking.red_states, 0,
            "健康的流 {flow_name} 一个红状态都不该有：{:?}",
            run.checking.red_texts
        );
        return run;
    }
    let judge_line = judge_fields_of_the_summary_line(&run);
    let judge_version: String = run
        .judge_version
        .0
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!(
        "CRASH_AMPLIFICATION flow={flow_name} mode={} states_checked_here={} blocks_checked_here={} unjudged_blocks={} red={} violations_in_the_library={} judge_version={judge_version} expand_up_to={limit} exhaustive={exhaustive} seconds={seconds:.1}{judge_line}",
        run.mode.name(),
        run.checking.states_checked,
        run.checking.blocks_checked,
        run.unjudged_blocks,
        run.checking.red_states,
        run.violations.len()
    );
    assert_eq!(
        run.checking.red_states, 0,
        "健康的流 {flow_name} 一个红状态都不该有：{:?}",
        run.checking.red_texts
    );
    // 这一趟复用的块（库里这一判法版本早先判过的）判红过也要红：只看这一趟判出的红数，红过一次的流再跑一遍就成了绿
    if run.unjudged_blocks == 0 {
        assert!(
            run.violations.is_empty(),
            "健康的流 {flow_name}：库里这条流的块有 {} 个红状态（含早先判过、这一趟复用的），前几个：{:?}",
            run.violations.len(),
            run.violations.iter().take(3).collect::<Vec<_>>()
        );
    }
    if let Some(comparison) = &run.comparison {
        assert_eq!(
            comparison.disagreements, 0,
            "流 {flow_name}：核对红而判器绿：{:?}",
            comparison.disagreement_samples
        );
    }
    run
}

/// 汇总行里说第 ① 段在哪判的那几个字段：GPU 判的带用了几张卡、判了多少、多快、写库花了多久；CPU 判的只有 `judge=cpu`。
#[must_use]
pub fn judge_fields_of_the_summary_line(
    run: &singlefs_checker_tier::crash_amplification::EnvironmentRun,
) -> String {
    match &run.gpu_judge {
        Some(gpu) => format!(
            " judge=gpu gpu_cards={} gpu_states={} gpu_compile_seconds={:.1} gpu_judge_seconds={:.1} gpu_states_per_second={:.0} gpu_seconds_storing={:.1}",
            gpu.cards_used,
            gpu.states_judged,
            gpu.compile_seconds,
            gpu.judge_seconds,
            gpu.states_per_second(),
            gpu.seconds_storing
        ),
        None => " judge=cpu".to_string(),
    }
}

/// 一次录下的整条写表当一个崩溃点（起点镜像不枚举、只录一次发布那几条流）：名字是这条流的名字，设点的文件给 `file!()`。
pub fn one_crash_point_over_every_write(
    code_file: &str,
    name: &str,
    write_count: usize,
) -> Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan> {
    vec![singlefs_checker_tier::crash_amplification::CrashPointSpan {
        name: name.to_string(),
        code_file_in_repository:
            singlefs_checker_tier::crash_amplification::repository_relative_file(code_file),
        writes: 0..write_count,
        ignored: false,
    }]
}
