//! 崩溃之后的可写挂载与挂载之后的池：GPU 记录核对器在崩溃状态镜像与恢复出的镜像上与 CPU 一致；挂载之后的池恢复落在写过的最新根之下时只在自称的那一版下判红，GPU 判自称的那一版与 CPU 一致。

use super::*;

/// 第二截那张表的一种摆法（[`the_gpu_record_checker_after_the_writable_mount_agrees_with_the_cpu_on_the_crash_state_image_and_the_recovered_image`]）：
/// 第一次崩溃状态（整条历史写表上的持久集合）、挂载那一段的写与切段、之后那一次发布的写与切段，与它在表名里的名字。
#[cfg(feature = "gpu")]
struct FirstCrashAndTheWritableMountAfterIt {
    name: &'static str,
    persisted_over_the_whole_history: Vec<bool>,
    mount_writes: Vec<RetainedWrite>,
    mount_segments: Vec<Vec<usize>>,
    publish_writes: Vec<RetainedWrite>,
    publish_segments: Vec<Vec<usize>>,
}

/// 手摆的那一种：最后一次带 journal 记录的那次发布，根槽写落了、记录写一个都没落；挂载那一段只有一个写，把那次发布的根槽盖成同一块盘上
/// 更早写的一条根（历史那次发布的根槽被挂载的写覆盖），之后没有发布。两份镜像判第一条判据红、只读挂载之后的池时放过
/// （`tests/crash_injection_writable_mount_after_the_crash.rs` 里同一种摆法）。
#[cfg(feature = "gpu")]
fn the_root_of_a_history_publish_overwritten_by_the_mount(
    writes: &[RetainedWrite],
) -> FirstCrashAndTheWritableMountAfterIt {
    let every_write = vec![true; writes.len()];
    let last_publish_with_records = singlefs_verification::memory_pool::publishes_in(
        writes,
        &every_write,
        RecordStreamContinuity::OneRecording,
    )
    .into_iter()
    .rev()
    .find(|publish| !publish.records.is_empty())
    .expect("前提：覆盖写那几次发布写了 journal 记录");
    let overwritten_root = &writes[last_publish_with_records.root];
    let earlier_root_on_the_same_device = writes[..last_publish_with_records.root]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::RootRecordFua
                && write.device == overwritten_root.device
                && write.length_in_bytes() == overwritten_root.length_in_bytes()
        })
        .expect("前提：那次发布之前同一块盘上写过别的根");
    let mut persisted_over_the_whole_history = every_write;
    for record in &last_publish_with_records.records {
        persisted_over_the_whole_history[*record] = false;
    }
    FirstCrashAndTheWritableMountAfterIt {
        name: "root-of-the-last-publish-with-records-overwritten-by-the-mount",
        persisted_over_the_whole_history,
        mount_writes: vec![RetainedWrite {
            device: overwritten_root.device,
            kind: StepKind::RootRecordFua,
            is_force_unit_access: true,
            offset: overwritten_root.offset,
            contents: earlier_root_on_the_same_device.contents.clone(),
        }],
        mount_segments: vec![vec![0]],
        publish_writes: Vec::new(),
        publish_segments: Vec::new(),
    }
}

/// 真跑的那一种：第一次崩溃摆在最后一条根槽写所在的段上、只扣下那一条（更早的段整段落、更晚的段一个都没落），在那份崩溃后镜像上
/// 真起一次可写挂载、再发一次布（崩溃注入第二截同一个入口）。
#[cfg(feature = "gpu")]
fn the_last_root_withheld_then_the_writable_mount(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> FirstCrashAndTheWritableMountAfterIt {
    use singlefs_verification::crash_injection::{
        materialized_crash_image, mount_and_publish_once_on_the_crash_image, WritableMountOutcome,
    };
    let last_root = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("历史写过根槽");
    let segment_index = segments
        .iter()
        .position(|segment| segment.contains(&last_root))
        .expect("每个写都在某一段里");
    let segment = &segments[segment_index];
    let first_write_of_the_segment = segment[0];
    let persisted_within_the_segment: Vec<bool> =
        segment.iter().map(|write| *write != last_root).collect();
    let mut persisted_over_the_whole_history = vec![false; writes.len()];
    persisted_over_the_whole_history[..first_write_of_the_segment].fill(true);
    persisted_over_the_whole_history
        [first_write_of_the_segment..first_write_of_the_segment + segment.len()]
        .copy_from_slice(&persisted_within_the_segment);
    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    base.apply_writes(&writes[..first_write_of_the_segment]);
    let crash_point = CrashPoint {
        segment_index,
        persisted_within_the_segment: persisted_within_the_segment.clone(),
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &materialized_crash_image(&CrashImage {
            base: &base,
            writes: &writes[first_write_of_the_segment..first_write_of_the_segment + segment.len()],
            persisted: persisted_within_the_segment,
        }),
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &crash_point,
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：崩溃之后的可写挂载与之后那一次发布都做成：{:?}",
        mount_run.outcome
    );
    let geometry = execution.device_width.fixed_geometry();
    let (mount_writes, mount_segments) =
        writes_and_segments(&mount_run.mount_operations, &geometry);
    let (publish_writes, publish_segments) =
        writes_and_segments(&mount_run.publish_operations, &geometry);
    FirstCrashAndTheWritableMountAfterIt {
        name: "last-root-withheld-then-the-writable-mount",
        persisted_over_the_whole_history,
        mount_writes,
        mount_segments,
        publish_writes,
        publish_segments,
    }
}

/// GPU 对拍的 CPU 参照：第二截那张表经对拍那一类的库读回（`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`），
/// 第一次崩溃那份镜像（崩溃态镜像）与挂载之后的池（实现恢复后的镜像）各从读回的那一份叠出来，记录核对器两份镜像一次判
/// （`crash::check_records_against`：第一条判据读崩溃态镜像、第二条读恢复后的镜像）。恢复自称的那一版照 D13（验证路线） 已定项 7 的字面从读回的
/// 那一份现取：崩溃那道接缝之后（挂载与那次发布）写出的最新那条根，一条都没有时退回接缝上记的第一次崩溃落到的那一版
/// （`crash_injection::newest_root_written_after_the_crash`，判器那一侧照表里那一栏取的同一个）；它与恢复后的镜像上看 journal 那一遍落到的根
/// 不同另记一位。交回（记录核对器两条判据，那一位）；只录那一趟交 None。
// cpu-reference-for-gpu-comparison
#[cfg(feature = "gpu")]
fn cpu_record_check_on_the_two_images_read_back_from_the_pairing_library(
    table: &singlefs_verification::crash_injection::GivenStatesTableAfterTheFirstCrash<'_>,
) -> Option<(RecordCheck, bool)> {
    use singlefs_verification::crash_amplification::RecordedGivenStatesTable;
    use singlefs_verification::crash_injection::{
        materialized_crash_image, newest_root_written_after_the_crash,
        StateOfTheTableAfterTheWritableMount,
    };
    use singlefs_verification::gpu_unit_checks::gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library;
    use singlefs_verification::memory_pool::RecordStreamSeam;
    let read_back = cpu_reference_table_read_back_from_the_pairing_library(
        &RecordedGivenStatesTable::of(&table.given_states_table()),
    )?;
    Some(read_back.with_the_table(|read_back_table| {
        let image_of = |state: StateOfTheTableAfterTheWritableMount| {
            materialized_crash_image(&CrashImage {
                base: read_back_table.base,
                writes: read_back_table.writes,
                persisted: read_back_table.persisted_sets[state.position()].clone(),
            })
        };
        let crash_state_image = image_of(StateOfTheTableAfterTheWritableMount::FirstCrashImage);
        let recovered_image = image_of(StateOfTheTableAfterTheWritableMount::PoolAfterTheMount);
        let (first_write_after_the_crash, version_landed_on_after_the_crash) = read_back_table
            .seams
            .iter()
            .find_map(|seam| match seam {
                RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash,
                    version_landed_on_after_the_crash,
                } => Some((
                    *first_write_after_the_crash,
                    *version_landed_on_after_the_crash,
                )),
                RecordStreamSeam::AfterAPowerLoss { .. } => None,
            })
            .expect("第二截那张表在历史与挂载之间有一道崩溃接缝");
        let claimed_version = newest_root_written_after_the_crash(
            &read_back_table.writes[first_write_after_the_crash..],
            version_landed_on_after_the_crash,
        );
        let landed_on = recover(&recovered_image, JournalPolicy::Consult).effective_root;
        (
            check_records_against(
                &crash_state_image,
                &recovered_image,
                read_back_table.writes,
                &read_back_table.persisted_sets
                    [StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position()],
                read_back_table.continuity(),
                claimed_version,
            ),
            claimed_version != landed_on,
        )
    }))
}

/// 第二截记录核对器两份镜像的输入上 GPU 与 CPU 对拍（欠账 C832，D13（验证路线） 已定项 7「崩溃后镜像是两份」）：第一个文件之后覆盖写两次的历史上
/// 摆两种第一次崩溃状态（手摆的「历史那次发布的根槽被挂载的写覆盖」、真跑的「扣下最后一条根槽写再可写挂载」），各建第二截那张两个状态的表，
/// 交配了 GPU 的引擎判（只由 GPU 判器判），两个状态的结论合成第二截记录核对器的两条判据；CPU 一侧从对拍那一类的库读回同一张表，
/// 叠出两份镜像、记录核对器两份镜像一次判。两侧逐条相同；手摆那一态两份镜像判第一条判据红（对拍的输入里真有一态分得开两种读法）；
/// GPU 判器两态都判得了。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
// crash-case-check:not-a-crash-case 只判写死的两个第一次崩溃状态（手摆一个、扣下最后一条根槽写一个），不枚举崩溃状态
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_record_checker_after_the_writable_mount_agrees_with_the_cpu_on_the_crash_state_image_and_the_recovered_image(
) {
    use singlefs_verification::crash_amplification::{CrashAmplificationEnvironment, JudgedBy};
    use singlefs_verification::crash_injection::GivenStatesAfterTheFirstCrash;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_gpu_record_checker_after_the_writable_mount_agrees_with_the_cpu_on_the_crash_state_image_and_the_recovered_image";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let history = history_of_a_first_file_and_two_overwrites();
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let first_crashes = [
        the_root_of_a_history_publish_overwritten_by_the_mount(&writes),
        the_last_root_withheld_then_the_writable_mount(&history, execution, &writes, &segments),
    ];
    // 库是这条用例自己在进程里起的临时 prune-server 里的一段一次性名字空间（设成这个进程开的那一份，CPU 参照经对拍那一类读回的也是它）：
    // 不连真服务
    let directory = disposable_library_of_the_campaign("record-checker-after-the-mount-on-the-gpu");
    let configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&configuration);
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(
        &settings_of_an_engine_on_the_gpu(&configuration),
    );
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!("{}::history_of_a_first_file_and_two_overwrites record-checker-after-the-writable-mount", CRASH_POINT_REPLAY_ITEM),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("配了 GPU 的引擎起得来（卡用不上在这里红，不退回 CPU）");
    let mut compared = Vec::new();
    for first_crash in &first_crashes {
        let version_landed_on_after_the_first_crash = recover(
            &CrashImage {
                base: history_after_the_make_filesystem.base_pool(),
                writes: history_after_the_make_filesystem.writes(),
                persisted: history_after_the_make_filesystem
                    .persisted_set_of_the_table(&first_crash.persisted_over_the_whole_history),
            },
            JournalPolicy::Consult,
        )
        .effective_root;
        let given = GivenStatesAfterTheFirstCrash::new(
            &engine,
            &history_after_the_make_filesystem,
            &first_crash.persisted_over_the_whole_history,
            &[],
            version_landed_on_after_the_first_crash,
            &first_crash.publish_writes,
            &first_crash.publish_segments,
        );
        let table_name = format!("first-file-and-two-overwrites/{}", first_crash.name);
        let conclusions_on_the_gpu = given.judge_the_pool_after_the_writable_mount(
            &first_crash.mount_writes,
            &first_crash.mount_segments,
            table_name.clone(),
        );
        let gpu = (
            conclusions_on_the_gpu.record_check(),
            conclusions_on_the_gpu
                .on_the_pool_after_the_mount
                .claimed_version_is_not_landed_on(),
        );
        let Some(cpu) = cpu_record_check_on_the_two_images_read_back_from_the_pairing_library(
            &given.table_after_the_writable_mount(
                &first_crash.mount_writes,
                &first_crash.mount_segments,
                table_name,
            ),
        ) else {
            return;
        };
        assert_eq!(
            gpu, cpu,
            "{}：GPU 判器两个状态合成的第二截记录核对器（连同恢复落到的根是不是挂载与那次发布写出的最新那条根）与 CPU 两份镜像一次判的逐条相同",
            first_crash.name
        );
        compared.push((first_crash.name, cpu.0));
    }
    let engine_run = engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    drop(the_library_of_this_process);
    drop(directory);
    assert_eq!(
        engine_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU：GPU 判器判"
    );
    assert_eq!(
        engine_run.red_items(),
        Vec::<String>::new(),
        "两张表的四个状态 GPU 都判得了"
    );
    assert!(
        compared
            .iter()
            .any(|(name, record_check)| *name == first_crashes[0].name
                && record_check.root_without_record),
        "手摆那一态两份镜像判第一条判据红：{compared:?}"
    );
}

// ─── 第二截恢复自称的那一版照 D13（验证路线） 已定项 7 的字面取（照字面改：表里记下根的来路） ───

/// 只建表、不判的判器：[`second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written`] 拿 `GivenStatesAfterTheFirstCrash`
/// 建第二截那张表，表不交它判。
struct GivenStatesTableBuiltOnly;

impl GivenStatesJudge for GivenStatesTableBuiltOnly {
    fn stop_is_requested(&self) -> bool {
        false
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        Err(format!(
            "这个判器只用来建第二截那张表，表 {} 不该交它判",
            table.table_name
        ))
    }
}

/// 手摆一张第二截的表，挂载之后的池那个状态上恢复落到挂载与那次发布写出的最新那条根之下：第一个文件之后覆盖写两次的历史整条落了、
/// 不崩，在那份镜像上真起一次可写挂载、再发一次布（崩溃注入第二截同一个入口），建第二截那张表（挂载之后的池那个状态那一栏写那次发布的根）；
/// 再把挂载之后的池那个状态里之后那次发布的 journal 记录、它的根槽写与之后的写、它的一个单元（那个偏移上的每一份副本）都改成没落
/// （判段之前把那条根与它的记录从镜像上拿掉：恢复落到挂载最后写的那条根）。交回读回用的那一份（表连同两栏）与那一栏写的那条根。
fn second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written() -> (
    singlefs_verification::crash_amplification::RecordedGivenStatesTableWithItsColumns,
    (InstanceGeneration, CheckpointTxg),
) {
    use singlefs_checker::walk::ContainerCapacityKnobs;
    use singlefs_verification::crash_amplification::{
        RecordedGivenStatesTable, RecordedGivenStatesTableWithItsColumns,
    };
    use singlefs_verification::crash_injection::{
        materialized_crash_image, mount_and_publish_once_on_the_crash_image,
        ClaimedVersionOfAGivenState, GivenStatesAfterTheFirstCrash,
        StateOfTheTableAfterTheWritableMount, WritableMountOutcome,
    };
    let history = history_of_a_first_file_and_two_overwrites();
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let every_history_write = vec![true; writes.len()];
    let version_landed_on_after_the_first_crash = recover(
        &CrashImage {
            base: history_after_the_make_filesystem.base_pool(),
            writes: history_after_the_make_filesystem.writes(),
            persisted: history_after_the_make_filesystem
                .persisted_set_of_the_table(&every_history_write),
        },
        JournalPolicy::Consult,
    )
    .effective_root;
    let last_segment_index = segments.len() - 1;
    let last_segment = &segments[last_segment_index];
    let first_write_of_the_last_segment = last_segment[0];
    let mut base_before_the_last_segment = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    base_before_the_last_segment.apply_writes(&writes[..first_write_of_the_last_segment]);
    let crash_point = CrashPoint {
        segment_index: last_segment_index,
        persisted_within_the_segment: vec![true; last_segment.len()],
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &materialized_crash_image(&CrashImage {
            base: &base_before_the_last_segment,
            writes: &writes[first_write_of_the_last_segment
                ..first_write_of_the_last_segment + last_segment.len()],
            persisted: vec![true; last_segment.len()],
        }),
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &crash_point,
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：可写挂载与之后那一次发布都做成：{:?}",
        mount_run.outcome
    );
    let geometry = execution.device_width.fixed_geometry();
    let (mount_writes, mount_segments) =
        writes_and_segments(&mount_run.mount_operations, &geometry);
    let (publish_writes, publish_segments) =
        writes_and_segments(&mount_run.publish_operations, &geometry);
    let given = GivenStatesAfterTheFirstCrash::new(
        &GivenStatesTableBuiltOnly,
        &history_after_the_make_filesystem,
        &every_history_write,
        &[],
        version_landed_on_after_the_first_crash,
        &publish_writes,
        &publish_segments,
    );
    let mut table = given.table_after_the_writable_mount(
        &mount_writes,
        &mount_segments,
        "first-file-and-two-overwrites/whole-history-landed/pool-after-the-mount-lands-below-the-newest-root-written"
            .to_string(),
    );
    let pool_after_the_mount = StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position();
    let ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some(claimed_root)) =
        table.claimed_versions[pool_after_the_mount]
    else {
        panic!(
            "挂载与那次发布写了根：挂载之后的池那个状态那一栏写那条根：{:?}",
            table.claimed_versions
        );
    };
    let publish_after_the_mount = singlefs_verification::memory_pool::publishes_in(
        &table.writes,
        &table.persisted_sets[pool_after_the_mount],
        RecordStreamContinuity::BrokenAtSeams(&table.seams),
    )
    .pop()
    .expect("写表里有发布");
    assert_eq!(
        (
            InstanceGeneration(publish_after_the_mount.instance),
            CheckpointTxg(publish_after_the_mount.checkpoint_txg)
        ),
        claimed_root,
        "前提：写表里最后那次发布就是挂载之后那一次，它的根就是那一栏写的那一条"
    );
    let first_unit_offset = publish_after_the_mount
        .units
        .iter()
        .map(|unit| table.writes[*unit].offset.0)
        .min()
        .expect("前提：挂载之后那一次发布写了单元");
    let persisted = &mut table.persisted_sets[pool_after_the_mount];
    for record in &publish_after_the_mount.records {
        persisted[*record] = false;
    }
    persisted[publish_after_the_mount.root..].fill(false);
    for unit in &publish_after_the_mount.units {
        if table.writes[*unit].offset.0 == first_unit_offset {
            persisted[*unit] = false;
        }
    }
    let recorded = RecordedGivenStatesTableWithItsColumns {
        table: RecordedGivenStatesTable::of(&table.given_states_table()),
        container_capacity_knobs: ContainerCapacityKnobs::PRODUCT,
        claimed_versions: table.claimed_versions.clone(),
    };
    (recorded, claimed_root)
}

/// 第二截恢复自称的那一版照字面取（D13（验证路线） 已定项 7「恢复自称的那一版取挂载与那次发布写出的最新那条根」）：手摆的那张表
/// （[`second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written`]）上，挂载之后的池那个状态恢复落到那条根之下。
/// 旧取法（第二条判据取恢复落到的根）放过：那次发布比恢复落到的那一版新，它缺的那个单元没人核，那一位也不置；照那一栏取（新取法）判红：
/// 第二条判据核到那次发布缺单元，「恢复落到的根不是挂载与那次发布写出的最新那条根」那一位也红。CPU 判器就地判一次，再经崩溃点重放的引擎
/// （两段：表连同两栏写进判定库、读回再判，照一份单机 CPU 配置）判一次，两次逐字相同。
/// 判别力（`crates/mutations.tsv`）：CPU 判器第二条判据改回取恢复落到的根、那一位恒不置、第二截建表时那一栏不写，各红这一条。
// crash-case-check:not-a-crash-case 一段两次覆盖写的历史上手摆一张两个状态的表，不枚举崩溃状态
#[test]
fn a_pool_after_the_mount_whose_recovery_lands_below_the_newest_root_written_is_red_only_under_the_claimed_version(
) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_verification::crash_injection::{
        judge_the_given_states_on_the_cpu, StateOfTheTableAfterTheWritableMount,
    };
    let (recorded, claimed_root) =
        second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written();
    let pool_after_the_mount = StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position();
    let (under_the_landed_root, under_the_claimed_version) =
        recorded.with_the_table(|table_with_its_columns| {
            (
                judge_the_given_states_on_the_cpu(table_with_its_columns.table),
                judge_the_given_states_on_the_cpu(table_with_its_columns),
            )
        });
    let landed_on = under_the_claimed_version.states[pool_after_the_mount]
        .words
        .consulted_pass_landed_root;
    assert_ne!(
        landed_on.landed_root(),
        Some(claimed_root),
        "前提：挂载之后的池那个状态上看 journal 那一遍恢复落到那一栏写的根 {claimed_root:?} 之下"
    );
    let old_reading = &under_the_landed_root.states[pool_after_the_mount];
    assert!(
        !old_reading.record_check().claimed_state_missing_unit
            && !old_reading.claimed_version_is_not_landed_on(),
        "旧取法（取恢复落到的根）放过：{:?}",
        old_reading.words
    );
    let literal_reading = &under_the_claimed_version.states[pool_after_the_mount];
    assert!(
        literal_reading.record_check().claimed_state_missing_unit,
        "照那一栏取的那一版判第二条判据：挂载之后那一次发布缺了一个单元，判红：{:?}",
        literal_reading.words
    );
    assert!(
        literal_reading.claimed_version_is_not_landed_on(),
        "恢复落到的根不是挂载与那次发布写出的最新那条根，判红：{:?}",
        literal_reading.words
    );
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu("claimed-version");
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine = given_states_engine_under_the_callers_environment_guard(
        "claimed_version_of_the_pool_after_the_mount",
    );
    let through_the_library = recorded
        .with_the_table(|table_with_its_columns| {
            engine.judge_the_given_states_with_its_columns(table_with_its_columns)
        })
        .unwrap_or_else(|reason| panic!("引擎判不了（不退回别处判）：{reason}"));
    let engine_run = engine.finish();
    drop(environment);
    drop(the_library_of_this_process);
    assert_eq!(
        engine_run.tally.tables_judged_here, 1,
        "这一趟从空库起、这张表现判：{:?}",
        engine_run.tally
    );
    assert_eq!(
        through_the_library
            .states
            .iter()
            .map(|state| state.words)
            .collect::<Vec<_>>(),
        under_the_claimed_version
            .states
            .iter()
            .map(|state| state.words)
            .collect::<Vec<_>>(),
        "经判定库两段判的与就地照那一栏判的逐字相同（两栏跟着表录进库、读回照它判）"
    );
}

/// 上一条那张表上 GPU 判器与 CPU 判器对拍（两趟独立、各从自己那一份库读回）：一趟配了 GPU（只由 GPU 判器判，恢复内核照判表里那一栏取那一版），
/// 一趟没开 GPU（CPU 判器），两个状态的对拍字经 `cross_check_words_compared_by_the_cross_check` 去掉内核不置位、对拍不比的那几位
/// （`.claude/kb/checks-owed.md` C821（GPU 判器没有扫描重建内核） 的 I-1.13）之后逐字相同，挂载之后的池那个状态两边都判第二条判据红、那一位红。要本机一张 NVIDIA 独显：
/// 一张适配器都没有时报「本次未跑」。判别力（`crates/mutations.tsv`）：恢复内核第二条判据改回取看 journal 那一遍落到的根，GPU 那一趟放过，这条红。
// crash-case-check:not-a-crash-case 一段两次覆盖写的历史上手摆一张两个状态的表，不枚举崩溃状态
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_judges_the_claimed_version_of_the_pool_after_the_mount_like_the_cpu_when_the_recovery_lands_below_it(
) {
    use singlefs_verification::crash_amplification::{CrashAmplificationEnvironment, JudgedBy};
    use singlefs_verification::crash_injection::StateOfTheTableAfterTheWritableMount;
    use singlefs_verification::crash_judge_conclusions::cross_check_words_compared_by_the_cross_check;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_gpu_judges_the_claimed_version_of_the_pool_after_the_mount_like_the_cpu_when_the_recovery_lands_below_it";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let (recorded, _claimed_root) =
        second_stage_table_whose_pool_after_the_mount_lands_below_the_newest_root_written();
    let (directory, cpu_configuration) =
        disposable_library_with_a_configuration_on_the_cpu("claimed-version-paired");
    let gpu_configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&gpu_configuration);
    let judged_in_a_run = |run: &str, gpu_enabled: bool| {
        let library = disposable_library_of_the_campaign(&format!("claimed-version-paired-{run}"));
        let settings: Vec<(&str, &std::ffi::OsStr)> = if gpu_enabled {
            settings_of_an_engine_on_the_gpu(&gpu_configuration).to_vec()
        } else {
            vec![("SINGLEFS_MULTI_HOST_CONFIG", cpu_configuration.as_os_str())]
        };
        let _the_library_of_this_process = library.make_it_the_library_of_this_process();
        let environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
        let cards_of_the_caller =
            gpu_enabled.then(common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run);
        let engine = given_states_engine_under_the_callers_environment_guard(&format!(
            "claimed_version_of_the_pool_after_the_mount_paired {run}"
        ));
        let judged = recorded
            .with_the_table(|table_with_its_columns| {
                engine.judge_the_given_states_with_its_columns(table_with_its_columns)
            })
            .unwrap_or_else(|reason| panic!("{run} 那一趟判不了（不退回别处判）：{reason}"));
        let engine_run = engine.finish();
        drop(cards_of_the_caller);
        drop(environment);
        (judged, engine_run)
    };
    let (on_the_gpu, gpu_run) = judged_in_a_run("on-the-gpu", true);
    let (on_the_cpu, cpu_run) = judged_in_a_run("on-the-cpu", false);
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        gpu_run.red_items(),
        Vec::<String>::new(),
        "GPU 判器判得了这张表的两个状态"
    );
    assert_eq!(
        on_the_gpu
            .states
            .iter()
            .map(|state| cross_check_words_compared_by_the_cross_check(state.words))
            .collect::<Vec<_>>(),
        on_the_cpu
            .states
            .iter()
            .map(|state| cross_check_words_compared_by_the_cross_check(state.words))
            .collect::<Vec<_>>(),
        "两个状态 GPU 与 CPU 的对拍字去掉内核不置位、对拍不比的那几位（C821 的 I-1.13）之后逐字相同"
    );
    let pool_after_the_mount =
        &on_the_gpu.states[StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position()];
    assert!(
        pool_after_the_mount
            .record_check()
            .claimed_state_missing_unit
            && pool_after_the_mount.claimed_version_is_not_landed_on(),
        "GPU 判器照那一栏取那一版：第二条判据红、恢复落到的根不是那一栏写的那一条也红：{:?}",
        pool_after_the_mount.words
    );
}
