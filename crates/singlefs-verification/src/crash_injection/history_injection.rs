//! 在一段历史上注入崩溃：开着内容保留的录制流跑历史、按屏障与 FUA 切段、按开关摆崩溃状态逐个判，收成这一段历史的发现。

use super::*;

/// 一段历史注入崩溃之后交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryCrashInjection {
    pub seed: HistorySeed,
    /// 摆出来的崩溃状态，按段号从小到大。
    pub crash_points: Vec<CrashPoint>,
    pub tally: CrashInjectionTally,
    pub known_red_hits: Vec<KnownRedAtACrashPoint>,
    /// 按段号从小到大排；同一段历史里同一个签名只留第一个。
    pub new_findings: Vec<CrashPointFinding>,
}

/// 每次掉电之后第一个写在写表里的下标，按次序：录制流下标（[`singlefs_verification::history::HistoryRun::earliest_stream_index_after_each_power_loss`]）
/// 换成写表下标，取录制流下标不小于它的第一个写（写表不算屏障，`stream_indexes` 与写表逐条对应）；掉电之后一个写都没录下的取写表长度。
fn first_write_after_each_power_loss_in_the_write_table(
    earliest_stream_index_after_each_power_loss: &[usize],
    stream_indexes: &[usize],
) -> Vec<usize> {
    earliest_stream_index_after_each_power_loss
        .iter()
        .map(|earliest_stream_index_after_the_power_loss| {
            stream_indexes
                .iter()
                .position(|stream_index| {
                    *stream_index >= *earliest_stream_index_after_the_power_loss
                })
                .unwrap_or(stream_indexes.len())
        })
        .collect()
}

/// 写表下标上的几道掉电接缝（[`RecordStreamSeam::AfterAPowerLoss`]），次序照给的。
pub(super) fn power_loss_seams(
    first_write_after_each_power_loss: &[usize],
) -> Vec<RecordStreamSeam> {
    first_write_after_each_power_loss
        .iter()
        .map(
            |first_write_after_the_power_loss| RecordStreamSeam::AfterAPowerLoss {
                first_write_after_the_power_loss: *first_write_after_the_power_loss,
            },
        )
        .collect()
}

/// 崩溃窗口末尾那个写在录制流里的下标：窗口从写表第 `first_write_of_the_segment` 个写起数 `writes_from_the_segment_on` 个
/// （靶向用例的窗口可以越过段尾），末尾是写表第 `first_write_of_the_segment + writes_from_the_segment_on - 1` 个写。
/// 读故障的认定（[`StreamMarks::read_faults_in_a_crash_state`]）按录制流下标与每一步跑完时流里的步数比；写表下标不算屏障，
/// 流里每过一道屏障两者就差一，拿写表下标去比会把窗口末尾认早（攻方 K2-c，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节）。
///
/// # Panics
/// `writes_from_the_segment_on` 是 0（崩溃窗口里至少有一个写：段内真子集的长度是段的写数，靶向用例的写数由 [`draw_crash_points`] 拒 0），
/// 或窗口末尾越过写表（`stream_indexes` 与写表逐条对应）。
fn stream_index_of_the_last_write_of_the_crash_window(
    stream_indexes: &[usize],
    first_write_of_the_segment: usize,
    writes_from_the_segment_on: usize,
) -> usize {
    let last_write_within_the_crash_window = writes_from_the_segment_on
        .checked_sub(1)
        .expect("崩溃窗口里至少有一个写：段内真子集的长度是段的写数，靶向用例的写数在摆的时候拒 0");
    stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]
}

/// 这一类历史步在注入的读故障下读盘（`singlefs_verification::history` 的 `DeviceReadingZerosOverHiddenRanges`）：崩溃恢复抛弃根（那次可写挂载期间
/// 最新那条根的根槽与它点名的单元读回全 0），最新那一槽系统配置读回全 0 的两种可写挂载——只藏第一块盘的那一种（C696（崩溃注入的读故障一支
/// 在随机历史里从没触发）：随机历史里真的在读故障下往盘上发写的一种步）与每块盘都藏的那一种（乙入判不出拒可写，一个写都不发）。
/// 别的步读的就是盘上的字节。
pub(super) fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
    match kind {
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => true,
        HistoryOperationKind::PublishFirstFile
        | HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::RollBackWhileMounted
        | HistoryOperationKind::RaiseRollbackFloor
        | HistoryOperationKind::ColdStartRecover
        | HistoryOperationKind::Unlink
        | HistoryOperationKind::CreateInode
        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
        // 建快照、删快照经会话发、读的是盘上的字节；读快照只读、不经注入的读故障。
        | HistoryOperationKind::CreateSnapshot
        | HistoryOperationKind::ReadSnapshot
        | HistoryOperationKind::DestroySnapshot
        // 挂着时整理经会话发、读的是盘上的字节（区域扫描读源落点）；批间崩溃之后的可写挂载不藏任何一处。
        | HistoryOperationKind::CompactWhileMounted
        // 截断经会话发，读的是盘上的字节（重写尾单元读它的旧净荷）。
        | HistoryOperationKind::Truncate
        // 建 inode、按 inode 号写经会话发，读的是盘上的字节。
        | HistoryOperationKind::CreateInodes
        | HistoryOperationKind::WriteOfAnInode
        // 按偏移写经会话发，读的是盘上的字节（读—改—写读回旧单元、读新槽残留）。
        | HistoryOperationKind::WriteAtOffset => false,
    }
}

/// 跑一段历史，按 `draw` 在它的录制流上摆崩溃状态，逐个重建镜像、跑恢复、池级 checker 与记录核对器、问模型；
/// 再在同一份崩溃后镜像上起可写挂载、发一次布、跑 checker，挂载途中摆二次崩溃（[`CrashStateStage`] 的后两截）。
/// `image_directory` 给了就把判红的那几个崩溃状态（哪一截判红就落哪一截的镜像）落成镜像文件放进去（留现场）。
///
/// # Panics
/// 录制流没开内容保留（重建镜像要字节）——这里自己建流，走不到；`draw` 是 [`CrashPointDraw::ExactlyOneCrashState`] 而写死的段号、
/// 写数或扣下的下标在这段历史的录制流上不成立（见那一支）。
#[must_use]
pub fn inject_crashes_into_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_drawing_second_crashes(
        history,
        execution,
        draw,
        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
        image_directory,
        given_states_judge,
    )
}

/// [`inject_crashes_into_history`]，挂载途中的二次崩溃按 `second_crash_draw` 摆（靶向用例经它摆写死的那一个二次崩溃）。
///
/// # Panics
/// 同 [`inject_crashes_into_history`]；`second_crash_draw` 是 [`SecondCrashPointDraw::ExactlyOneSecondCrashState`] 而写死的段号、
/// 写数或扣下的下标在某一次挂载写出的写表上不成立（见那一支）。
#[must_use]
pub fn inject_crashes_into_history_drawing_second_crashes(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    second_crash_draw: SecondCrashPointDraw,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_with_every_switch(
        history,
        execution,
        draw,
        second_crash_draw,
        InstanceTableSweepConfiguration::PRODUCT,
        image_directory,
        given_states_judge,
    )
}

/// [`inject_crashes_into_history`]，第一截与第三截上只读一张镜像的那几样（池级 checker、两遍恢复、记录核对器）交 `given_states_judge` 判
/// （给定状态表；崩溃注入战役经它把判器换成引擎，[`run_crash_injection_campaign_judging_the_given_states_with`]）。
///
/// # Panics
/// 同 [`inject_crashes_into_history`]；`given_states_judge` 判不了某一张表。
#[must_use]
pub fn inject_crashes_into_history_judging_the_given_states_with(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_with_every_switch(
        history,
        execution,
        draw,
        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
        InstanceTableSweepConfiguration::PRODUCT,
        image_directory,
        given_states_judge,
    )
}

/// [`inject_crashes_into_history`]，崩溃后镜像上那次可写挂载与每个二次崩溃镜像上的再挂末尾那一判实例表清扫按 `instance_table_sweep`
/// （[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`] 这一份让短历史摆得到清扫那两段；别的入口取产品路径 `InstanceTableSweepConfiguration::PRODUCT`）。
///
/// # Panics
/// 同 [`inject_crashes_into_history`]。
#[must_use]
pub fn inject_crashes_into_history_with_the_instance_table_sweep_configuration(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    instance_table_sweep: InstanceTableSweepConfiguration,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_crashes_into_history_with_every_switch(
        history,
        execution,
        draw,
        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
        instance_table_sweep,
        image_directory,
        given_states_judge,
    )
}

/// [`history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row`] 里实例 2 覆盖写几次。根环 24 槽
/// （`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 每区 8 槽、三区）里实例 1 的根要全被轮出去：实例 2 的写行、两次暖机与崩溃之前的 24 次
/// 覆盖写一共 27 条根；最后一次是摆崩溃状态的那一步。
const OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW: u64 = 25;

/// 崩溃后那次可写挂载（有一行就扫，[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）删得了行的一段历史：第一个文件之后关掉会话、可写挂载成实例 2
/// （写行把实例 1 那一行写进表），再覆盖写 [`OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW`] 次——根环里实例 1 的根
/// 全被轮出去，根环每一槽都是自证过的根，实例 1 那一行是候选行（D18（块里携带什么信息） 已定项 11「回收」一条）。每次发布都做完了，
/// 盘上没有按实例 1 那一行判未发布的单元：清扫删行、一个头都不抹。
#[must_use]
pub fn history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row(
) -> GeneratedHistory {
    let mut operations = vec![HistoryOperation::CloseAndMountWritable];
    operations.extend(
        (0..OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW).map(|overwrite| {
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: overwrite,
                },
                fill_seed: overwrite,
            })
        }),
    );
    GeneratedHistory {
        seed: HistorySeed(3),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    }
}

/// 在 [`history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row`] 最后一步的开头摆一个崩溃状态（更早的写全落、
/// 最后一步一个写都没落），崩溃后那次可写挂载与每个二次崩溃镜像上的再挂有一行就扫（[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）。
/// 最后一步的第一个写落在第几段随历史的写数变：先把这段历史跑一遍、按同一条切段规则取出来，不写死段号。
///
/// # Panics
/// 历史没跑到最后一步；含最后一步第一个写的那一段里，段首到它之前有 discard 以外的写（上一步发布返回之前那道屏障把上一步的写与它隔开，
/// 隔不开的只有上一步发布返回之后发的 discard：它们与最后一步之间没有屏障，切段把它们并进同一段；有别的写说明切段变了，
/// 「最后一步一个写都没落」那个状态摆不准）。
#[must_use]
pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
    execution: HistoryExecution,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
        &history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row(),
        execution,
        given_states_judge,
    )
}

/// 在 [`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`] 最后一步的开头摆一个崩溃状态（摆法同
/// [`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]）：那段历史开头那次覆盖写掉电在
/// journal 记录之前、留下实例 1 的孤儿，之后实例 2 把实例 1 的根全轮出根环；崩溃后那次可写挂载（有一行就扫）末尾的清扫先抹孤儿的头、
/// 再发清扫发布删实例 1 那一行，抹头那一段（D18（块里携带什么信息） 已定项 11「回收」一条）的二次崩溃从它来。
///
/// # Panics
/// 同 [`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]。
#[must_use]
pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
    execution: HistoryExecution,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
        &history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(),
        execution,
        given_states_judge,
    )
}

/// 在 `history` 最后一步的开头摆一个崩溃状态（更早的写全落、最后一步一个写都没落），崩溃后那次可写挂载与每个二次崩溃镜像上的再挂
/// 有一行就扫（[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）。最后一步的第一个写落在第几段随历史的写数变：先把这段历史跑一遍、
/// 按同一条切段规则取出来，不写死段号。
///
/// 上一步发布返回之后核心层发的 discard（整段可再分配的那几段）与最后一步之间没有屏障（核心层 discard 之后不补屏障；补不补连着
/// D26（后台整理与放置回收） 未定项 9，没定，这里照今天的写流摆），切段把它们与最后一步的写并成一段，最后一步的第一个写就不是段首。这里不要求它是段首：找含它的那一段，
/// 段首到它之前那几个 discard 算更早的写、持久，它扣下，它之后的一个都不持久。没有这样的 discard 时它就是段首，摆法不变。
///
/// # Panics
/// 历史没跑到最后一步；含最后一步第一个写的那一段里，段首到它之前有 discard 以外的写。
fn inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    let step_before_the_last = StepPosition::Operation(history.operations.len() - 2);
    let stream = SharedStream::retaining_contents();
    let mut operations_before_the_last_step = None;
    let _run = execute_history_with(history, execution, &stream, &mut |observation| {
        if observation.position == step_before_the_last {
            operations_before_the_last_step = Some(stream.operation_count());
        }
    });
    let operations_before_the_last_step = operations_before_the_last_step.expect(
        "这段历史的每一步都做得成（覆盖写落在 4 GiB 的盘上），倒数第二步之后观察者一定被调过",
    );
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let first_write_of_the_last_step = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= operations_before_the_last_step)
        .expect("最后一步（覆盖写）写过东西");
    let segment_index = segments
        .iter()
        .position(|segment| segment.contains(&first_write_of_the_last_step))
        .expect("切段时录到的每个写都恰好落在一段里");
    let position_of_the_first_write_of_the_last_step = segments[segment_index]
        .iter()
        .position(|write| *write == first_write_of_the_last_step)
        .expect("这一段是按含它挑出来的");
    assert!(
        segments[segment_index][..position_of_the_first_write_of_the_last_step]
            .iter()
            .all(|write| writes[*write].kind == StepKind::Discard),
        "段首到最后一步的第一个写之间只许是上一步发布返回之后发的 discard：上一步发布返回之前那道屏障把上一步别的写与它隔开"
    );
    inject_crashes_into_history_with_the_instance_table_sweep_configuration(
        history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index,
            writes_from_the_segment_on: position_of_the_first_write_of_the_last_step + 1,
            withheld_among_them: vec![position_of_the_first_write_of_the_last_step].leak(),
        },
        INSTANCE_TABLE_SWEPT_ON_ANY_ROW,
        None,
        given_states_judge,
    )
}

/// 崩溃注入的全部开关一处收：第一次崩溃怎么摆、二次崩溃怎么摆、挂载末尾的实例表清扫用哪一份配置、给定状态表交谁判。
fn inject_crashes_into_history_with_every_switch(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    draw: CrashPointDraw,
    second_crash_draw: SecondCrashPointDraw,
    instance_table_sweep: InstanceTableSweepConfiguration,
    image_directory: Option<&Path>,
    given_states_judge: &dyn GivenStatesJudge,
) -> HistoryCrashInjection {
    let stream = SharedStream::retaining_contents();
    let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let mut marks = StreamMarks {
        after_make_filesystem: 0,
        after_the_last_finished_step: 0,
        after_each_step: Vec::new(),
    };
    // RefFS 的输入（第一截与二次崩溃那一截的判定 = RefFS ∧ O1 判官全四条）：跑历史时逐步攒。
    let mut reference_filesystem_translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed_versions.insert(key, file);
        }
        let operations_so_far = stream.operation_count();
        marks.after_the_last_finished_step = operations_so_far;
        marks
            .after_each_step
            .push((observation.position, operations_so_far));
        reference_filesystem_translation.note_an_observed_step(observation, operations_so_far);
    });
    marks.after_make_filesystem = run.operations_written_by_make_filesystem;

    let mut tally = CrashInjectionTally {
        histories: 1,
        most_committed_versions_in_one_history: committed_versions.len(),
        ..CrashInjectionTally::default()
    };
    match &run.ending {
        HistoryEnding::Completed => tally.histories_completed = 1,
        HistoryEnding::KnownRed { form, .. } => {
            tally.histories_ended_known_red.insert(*form, 1);
        }
        HistoryEnding::NewFinding { .. } => tally.histories_ended_new_finding = 1,
    }

    let operations = stream.retained_operations();
    let parameters = execution.device_width.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &geometry);
    let first_write_after_each_power_loss = first_write_after_each_power_loss_in_the_write_table(
        &run.earliest_stream_index_after_each_power_loss,
        &stream_indexes,
    );
    // 崩溃状态摆在最后一个跑完的操作之前，停下那一步不进 RefFS（与 O1 那一路同一个范围）。
    let reference_filesystem_history =
        reference_filesystem_translation.translate(&operations, &geometry);
    assert_eq!(
        reference_filesystem_history.segment_count(),
        segments.len(),
        "RefFS 的崩溃时刻按同一条录制流、同一条切段规则切"
    );
    let step_kinds: Vec<HistoryOperationKind> = history
        .operations
        .iter()
        .map(HistoryOperation::kind)
        .collect();
    let crash_points = draw_crash_points(
        history.seed,
        &segments,
        &stream_indexes,
        &marks,
        &step_kinds,
        draw,
    );
    if crash_points.is_empty() {
        tally.histories_without_any_crash_point = 1;
    }
    let last_committed_version = committed_versions.keys().next_back().copied();
    // 第一截只读一张镜像的那几样交给定状态表（基线是 mkfs 之后的池）：一段历史上的崩溃状态先一起判完，下面逐个取
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        marks.after_make_filesystem,
        execution,
    );
    let first_crash_conclusions = judge_the_first_crashes_on_given_states_tables(
        given_states_judge,
        history.seed,
        &history_after_the_make_filesystem,
        &crash_points,
        &segments,
        writes.len(),
        &first_write_after_each_power_loss,
        &mut tally,
    );

    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    let mut writes_applied_to_the_base = 0usize;
    let mut findings = FindingsOfOneHistory {
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
    };
    for (crash_point, first_crash_conclusion) in crash_points.iter().zip(&first_crash_conclusions) {
        let first_write_of_the_segment = segments[crash_point.segment_index][0];
        let writes_in_the_segment = crash_point.persisted_within_the_segment.len();
        // 段号从小到大，基线只往前叠：更早的段整段持久，按写表整段施加，不为每个崩溃状态从头重建。
        base.apply_writes(&writes[writes_applied_to_the_base..first_write_of_the_segment]);
        writes_applied_to_the_base = first_write_of_the_segment;
        let persisted = persisted_over_the_whole_write_table(
            writes.len(),
            first_write_of_the_segment,
            &crash_point.persisted_within_the_segment,
        );
        let image = CrashImage {
            base: &base,
            writes: &writes
                [first_write_of_the_segment..first_write_of_the_segment + writes_in_the_segment],
            persisted: crash_point.persisted_within_the_segment.clone(),
        };

        tally.crash_points += 1;
        for kind in withheld_write_kinds(&writes, first_write_of_the_segment, crash_point) {
            *tally
                .crash_points_withholding_write_kind
                .entry(kind.name())
                .or_insert(0) += 1;
        }
        match crash_point.step {
            StepPosition::StartingPoint => tally.crash_points_inside_the_starting_point += 1,
            StepPosition::Operation(_) => {}
        }
        if let Some(kind) = crash_point.operation_kind {
            *tally
                .crash_points_by_operation_kind
                .entry(kind)
                .or_insert(0) += 1;
        }
        if crash_point.withholds_a_write_before_a_persisted_one() {
            tally.crash_points_withholding_a_write_before_a_persisted_one += 1;
        }
        let newest_persisted = newest_persisted_root(&writes, &persisted)
            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
        if newest_persisted < last_committed_version {
            tally.crash_points_that_land_before_the_last_committed_version += 1;
        }
        if some_publish_persisted_without_its_root(&writes, &persisted) {
            tally.crash_points_inside_an_unfinished_publish += 1;
        }

        // 只读恢复连同挂载透明要比的字段一起读（R1）：恢复报告就是这一次的。
        let fields_read_after_the_crash =
            read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
        let report = &fields_read_after_the_crash.report;
        match &report.outcome {
            RecoveryOutcome::FileRead { .. } => tally.recoveries_reading_a_file += 1,
            RecoveryOutcome::NoFile { .. } => tally.recoveries_without_a_file += 1,
            RecoveryOutcome::Failed { .. } => tally.recoveries_failed += 1,
        }
        if report.journal.prefix_applied > 0 {
            tally.recoveries_that_applied_journal_records += 1;
        }
        let read_back = observed_read_back_after_a_crash(report);
        tally.model_judgements += 1;
        let disagreement =
            disagreement_with_the_ideal_model(&committed_versions, newest_persisted, &read_back);
        // RefFS 那一票：崩溃时刻按切开当前段的那道屏障由哪一步发出归属（H1-收严，只给 RefFS 用；
        // O1 判官与 `crash_point.step` 照旧按段里第一个被扣下的写）。当前段取持久窗口最后一个写所在那一段：段内的崩溃状态就是
        // `segment_index` 那一段；靶向用例越过段尾的那一个，崩在窗口末尾那一段里（窗口里更晚的写都可能已落）。
        let segment_the_crash_closes = segment_containing_the_write(
            &segments,
            first_write_of_the_segment + writes_in_the_segment - 1,
        );
        let reference_filesystem_judgement = reference_filesystem_judgement_of_a_crash_image(
            &reference_filesystem_history,
            segment_the_crash_closes,
            &read_back,
        );
        tally.note_the_reference_filesystem_vote(
            StageTheReferenceFileSystemJudges::CrashImage,
            &reference_filesystem_judgement.vote,
            disagreement.as_ref(),
        );
        if disagreement.is_none() {
            match &read_back {
                ObservedReadBack::FileRead { .. } => tally.model_contents_compared += 1,
                ObservedReadBack::NoFile { .. } => {
                    tally.model_versions_without_a_file_matched += 1;
                }
                ObservedReadBack::Failed { .. } => {}
            }
        }
        // 记录核对器与池级 checker 在上面那张给定状态表上判过（judge_the_first_crashes_on_given_states_tables）：核的是 mkfs 之后这段历史的
        // 整条记录流——更早的段里发出的单元写与 journal 记录写要算进一次发布里；恢复落到由 journal 记录重建的一版时，那一版的根槽写在更晚的段里，
        // 只给到这一段为止的前缀就找不到它、读不出它的实例表，被它抛弃的发布的豁免整条落空（D23（journal 的角色与格式） 已定项 15）。
        // 这个崩溃状态之前掉过电的，在每道掉电接缝处断开（[`RecordStreamSeam::AfterAPowerLoss`]）；第二、三截照这几道接缝接上挂载。
        let first_write_after_each_power_loss_before_the_crash: Vec<usize> =
            first_write_after_each_power_loss
                .iter()
                .copied()
                .filter(|first_write_after_the_power_loss| {
                    *first_write_after_the_power_loss <= first_write_of_the_segment
                })
                .collect();
        let record_check = first_crash_conclusion.record_check();
        tally.record_checks += 1;
        if record_check.root_without_record {
            tally.record_root_without_record += 1;
        }
        if record_check.claimed_state_missing_unit {
            tally.record_claimed_state_missing_unit += 1;
        }
        tally.checker_runs += 1;
        let mut violations = first_crash_conclusion.pool_checker_violations();
        violations.extend(reference_filesystem_judgement.vote.violation());
        if !(violations.is_empty()
            && disagreement.is_none()
            && record_check == RecordCheck::default())
        {
            let observation = crash_state_observation(
                &image,
                crash_point,
                violations,
                disagreement.clone(),
                record_check,
            );
            findings.settle(
                &mut tally,
                FailureAtAStage {
                    seed: history.seed,
                    crash_point,
                    stage: CrashStateStage::CrashImage,
                    observation,
                    read_back: format!("{read_back:?}"),
                    image: &image,
                },
                image_directory,
            );
        }

        // 第二截与第三截（代码审阅第 2 条）：只读恢复只读盘，取号、写行、暖机在崩溃状态上从没跑过；在同一份崩溃后镜像上
        // 真的起一次可写挂载、再发一次布、跑 checker 与记录核对器，挂载途中再崩一次。
        let crash_image_materialized = materialized_crash_image(&image);
        let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(
            &step_kinds,
            stream_index_of_the_last_write_of_the_crash_window(
                &stream_indexes,
                first_write_of_the_segment,
                writes_in_the_segment,
            ),
        );
        match read_faults_in_the_crash_state {
            ReadFaultsInTheCrashState::NoneInjected => {}
            ReadFaultsInTheCrashState::Injected => {
                tally.crash_points_after_a_step_reading_under_injected_faults += 1;
            }
        }
        let content_read_back_after_the_crash = match &read_back {
            ObservedReadBack::FileRead { content, .. } => Some(Some(Rc::from(content.as_slice()))),
            ObservedReadBack::NoFile { .. } => Some(None),
            ObservedReadBack::Failed { .. } => None,
        };
        let mount_run =
            mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
                &crash_image_materialized,
                &parameters,
                execution,
                history.seed,
                crash_point,
                instance_table_sweep,
            );
        if let Some(rows_deleted) = &mount_run.rows_deleted_by_the_instance_table_sweep {
            tally.writable_mounts_after_the_crash_deleting_instance_table_rows += 1;
            tally.heads_erased_by_the_instance_table_sweep_after_the_crash +=
                u64::try_from(rows_deleted.heads_erased).expect("抹的头数装得进 u64");
        }
        let (mount_writes, mount_segments, _mount_stream_indexes) =
            writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &geometry);
        let (
            writes_of_the_publish_after_the_mount,
            segments_of_the_publish_after_the_mount,
            _publish_stream_indexes,
        ) = writes_and_segments_with_stream_indexes(&mount_run.publish_operations, &geometry);
        let records_across_the_crash = HistoryThenWritableMountRecords::new(
            &writes,
            &persisted,
            &first_write_after_each_power_loss_before_the_crash,
            report.effective_root,
            &mount_writes,
            &writes_of_the_publish_after_the_mount,
        );
        let given_states_of_the_mount = GivenStatesAfterTheFirstCrash::new(
            given_states_judge,
            &history_after_the_make_filesystem,
            &persisted,
            &first_write_after_each_power_loss_before_the_crash,
            report.effective_root,
            &writes_of_the_publish_after_the_mount,
            &segments_of_the_publish_after_the_mount,
        );
        // 第二截经库（「能走两段的都走两段」）：挂载与之后那一次发布之后的盘与第一次崩溃那份镜像当一张表的两个给定状态
        // 写进库、从库里读回，池级 checker 与记录核对器交判器判（配了 GPU 走 GPU）；记录核对器两份镜像各判一条（D13（验证路线） 已定项 7，
        // 「过渡后补回两份镜像」）。可写挂载与那次发布本身真跑实现、只能在 CPU 上（上面那一次）
        let conclusions_after_the_mount = given_states_of_the_mount
            .judge_the_pool_after_the_writable_mount(
                &mount_writes,
                &mount_segments,
                format!(
                    "seed={}/after-the-mount/segment={}/first-crash-fingerprint={:016x}",
                    history.seed.0,
                    crash_point.segment_index,
                    fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment)
                ),
            );
        count_a_table_left_to_another_host(&mut tally, conclusions_after_the_mount.source);
        tally.record_checks_after_the_writable_mount += 1;
        let record_check_after_the_mount = conclusions_after_the_mount.record_check();
        judge_the_writable_mount_on_the_crash_image(
            &mut tally,
            &mut findings,
            &mount_run,
            record_check_after_the_mount,
            conclusions_after_the_mount.violations_on_the_pool_after_the_mount(),
            RefusalOnTheCrashImage {
                crash_image_materialized: &crash_image_materialized,
                version_landed_on_after_the_crash: report.effective_root,
                filesystem_identifier_of_the_pool: &parameters.filesystem_identifier,
                read_only_read_back: if disagreement.is_none() {
                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
                } else {
                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
                },
                read_faults: read_faults_in_the_crash_state,
            },
            history.seed,
            crash_point,
            image_directory,
        );
        // 没崩过的那一次挂载之后的池（挂载那一段的写全落在第一次崩溃之后的镜像上，不接之后那一次发布）：v2a。挂载被拒时没有。
        let fields_after_the_uninterrupted_mount = pool_after_the_uninterrupted_mount(
            &crash_image_materialized,
            &mount_run,
            records_across_the_crash.mount_writes(),
        )
        .map(|pool| {
            let fields = read_recovery_fields(&pool, ComparedVersion::TheRecoveredVersion);
            (pool, fields)
        });
        let content_after_the_uninterrupted_mount = match &fields_after_the_uninterrupted_mount {
            None => Err(RemountIdempotenceNotJudged::TheUninterruptedMountWasRefused),
            Some((_pool, fields)) => Ok(content_read_by_the_recovery_report(&fields.report)),
        };
        // 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：注入过读故障的崩溃态不判，按理由记一次没判。
        match read_faults_in_the_crash_state {
            ReadFaultsInTheCrashState::NoneInjected => judge_mount_transparency_on_the_crash_image(
                &mut tally,
                &mut findings,
                &fields_read_after_the_crash,
                &mount_run,
                fields_after_the_uninterrupted_mount
                    .as_ref()
                    .map(|(pool, fields)| (pool, fields)),
                history.seed,
                crash_point,
                image_directory,
            ),
            ReadFaultsInTheCrashState::Injected => {
                *tally
                    .mount_transparency_not_judged
                    .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                    .or_insert(0) += 1;
            }
        }
        judge_second_crashes_inside_the_writable_mount(
            &mut tally,
            &mut findings,
            SecondCrashInputs {
                crash_image_materialized: &crash_image_materialized,
                records_across_the_crash: &records_across_the_crash,
                mount_segments: &mount_segments,
                committed_versions: &committed_versions,
                content_read_back_after_the_crash,
                newest_persisted_before_the_mount: newest_persisted,
                content_after_the_uninterrupted_mount,
                fields_read_after_the_first_crash: &fields_read_after_the_crash,
                floor_raises_in_the_uninterrupted_mount_writes: mount_run
                    .floor_raises_in_the_mount_writes,
                continuation_of_the_uninterrupted_mount: mount_run
                    .user_visible_tree_root_pointers_of_the_continuation
                    .as_ref(),
                publishes_before_the_deletion_queue_continuation: mount_run
                    .publishes_before_the_deletion_queue_continuation,
                publishes_before_the_instance_table_sweep: mount_run
                    .publishes_before_the_instance_table_sweep,
                parameters: &parameters,
                space_admission: execution.space_admission,
                instance_table_sweep,
                rows_deleted_by_the_sweep_of_the_uninterrupted_mount: mount_run
                    .rows_deleted_by_the_instance_table_sweep
                    .as_ref(),
                reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
                read_faults_in_the_first_crash_state: read_faults_in_the_crash_state,
                second_crash_draw,
                given_states: given_states_of_the_mount,
            },
            history.seed,
            crash_point,
            image_directory,
        );
    }
    HistoryCrashInjection {
        seed: history.seed,
        crash_points,
        tally,
        known_red_hits: findings.known_red_hits,
        new_findings: findings.new_findings,
    }
}

/// 判器交的是占位结论就记一张：两台分片、这张表归别的机器判（[`CrashInjectionTally::given_states_tables_left_to_other_hosts`]），
/// 或这一趟只跑录入段（[`CrashInjectionTally::given_states_tables_recorded_only`]）。
pub(super) fn count_a_table_left_to_another_host(
    tally: &mut CrashInjectionTally,
    source: GivenStatesConclusionsSource,
) {
    match source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost => {
            tally.given_states_tables_left_to_other_hosts += 1;
        }
        GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => {
            tally.given_states_tables_recorded_only += 1;
        }
    }
}

/// 一张表的结论并进崩溃注入的计数：逐条不变量判绿（判到减违例）、没判到（不适用或部分没判）各几次。
pub(super) fn absorb_the_pool_checker_counts_of_the_table(
    tally: &mut CrashInjectionTally,
    count_vector: &BlockCountVector,
) {
    for (invariant, counts) in &count_vector.per_invariant {
        let invariant = singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .find(|candidate| **candidate == invariant.as_str())
            .expect(
                "计数向量按 IMPLEMENTED_INVARIANTS 逐条一行（StateCountsOfABlock::into_count_vector）；判定库读回的在同一把版本下存，清单相同",
            );
        *tally.invariant_holds.entry(invariant).or_insert(0) += counts.judged - counts.violated;
        *tally.invariant_not_judged.entry(invariant).or_insert(0) += counts.not_applicable;
    }
}

/// mkfs 写了写表的前几个写：录制流下标在 `after_make_filesystem` 之前的那几个（写表与 `stream_indexes` 逐条对应、按录制流次序）。
pub(super) fn writes_of_the_make_filesystem(
    stream_indexes: &[usize],
    after_make_filesystem: usize,
) -> usize {
    stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= after_make_filesystem)
        .unwrap_or(stream_indexes.len())
}

/// 写表从第 `first_write` 个起的那一截的切段（下标减去 `first_write`）：整段在它之前的不要，跨过它的只留它之后那一部分
/// （跨过它的段不是候选段，上面不摆状态；留下的那一部分只为让每个写都在一段里）。
pub(super) fn segments_from_the_write(
    segments: &[Vec<usize>],
    first_write: usize,
) -> Vec<Vec<usize>> {
    segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .filter(|write| **write >= first_write)
                .map(|write| write - first_write)
                .collect::<Vec<usize>>()
        })
        .filter(|segment| !segment.is_empty())
        .collect()
}

/// 整条写表上一个崩溃状态的持久集合：段首之前全持久，从段首起照 `persisted_within_the_segment`（靶向用例可以越过段尾），之后都没持久。
pub(super) fn persisted_over_the_whole_write_table(
    write_count: usize,
    first_write_of_the_segment: usize,
    persisted_within_the_segment: &[bool],
) -> Vec<bool> {
    let mut persisted = vec![false; write_count];
    persisted[..first_write_of_the_segment].fill(true);
    persisted[first_write_of_the_segment
        ..first_write_of_the_segment + persisted_within_the_segment.len()]
        .copy_from_slice(persisted_within_the_segment);
    persisted
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 带读故障的步与别的步交错时登记的那几段流：起点发到第 10 步，关掉重挂发到 20，崩溃恢复抛弃根（带读故障）发到 25，零单元发布发到 40。
    fn marks_with_an_abandoning_recovery_sending_stream_steps_15_through_24() -> StreamMarks {
        StreamMarks {
            after_make_filesystem: 4,
            after_the_last_finished_step: 40,
            after_each_step: vec![
                (StepPosition::StartingPoint, 10),
                (StepPosition::Operation(0), 20),
                (StepPosition::Operation(1), 25),
                (StepPosition::Operation(2), 40),
            ],
        }
    }

    /// C670：崩溃窗口末尾那个写之前（含）历史里有一步「崩溃恢复抛弃根」往录制流里发过东西，这个崩溃态才算注入过读故障；
    /// 那一步落在窗口末尾之后的不算。
    #[test]
    fn crash_state_counts_as_read_faulted_from_the_earliest_stream_step_of_an_abandoning_recovery_on(
    ) {
        let step_kinds = [
            HistoryOperationKind::CloseAndMountWritable,
            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperationKind::PublishWithoutUnits,
        ];
        let marks = marks_with_an_abandoning_recovery_sending_stream_steps_15_through_24();
        assert_eq!(
            marks.read_faults_in_a_crash_state(&step_kinds, 19),
            ReadFaultsInTheCrashState::NoneInjected,
            "窗口末尾落在带读故障那一步发的第一步之前：没注入"
        );
        assert_eq!(
            marks.read_faults_in_a_crash_state(&step_kinds, 20),
            ReadFaultsInTheCrashState::Injected,
            "窗口末尾就是带读故障那一步发的第一步：注入过"
        );
        assert_eq!(
            marks.read_faults_in_a_crash_state(&step_kinds, 35),
            ReadFaultsInTheCrashState::Injected,
            "窗口末尾落在带读故障那一步之后的零单元发布里：注入过"
        );
    }

    /// 读故障认定拿的崩溃窗口末尾（攻方改法 F5，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：
    /// 写表 6 个写落在录制流第 0、1、3、4、6、9 步（第 2、5、7、8 步是屏障）。从写表第 2 个写起数 3 个，窗口末尾是写表第 4 个写、
    /// 录制流第 6 步——交出去的是 6，不是写表下标 4；窗口越到写表末尾、只有一个写时同样按录制流下标交。
    #[test]
    fn the_end_of_a_crash_window_is_named_by_its_stream_index_and_not_by_its_write_index() {
        let stream_indexes = [0, 1, 3, 4, 6, 9];
        assert_eq!(
            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 2, 3),
            6,
            "写表第 2、3、4 个写，末尾那个在录制流第 6 步"
        );
        assert_eq!(
            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 4, 2),
            9,
            "窗口越到写表末尾：写表第 5 个写在录制流第 9 步"
        );
        assert_eq!(
            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 3, 1),
            4,
            "窗口只有写表第 3 个写：录制流第 4 步"
        );
    }

    /// C670：「崩溃恢复抛弃根」那一步一个都没往录制流里发（取号之前就拒了），盘面与没走这一步的历史一样：之后的崩溃态都不算注入过读故障。
    #[test]
    fn an_abandoning_recovery_that_sent_nothing_leaves_every_crash_state_without_read_faults() {
        let step_kinds = [
            HistoryOperationKind::CloseAndMountWritable,
            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperationKind::PublishWithoutUnits,
        ];
        let marks = StreamMarks {
            after_make_filesystem: 4,
            after_the_last_finished_step: 40,
            after_each_step: vec![
                (StepPosition::StartingPoint, 10),
                (StepPosition::Operation(0), 20),
                (StepPosition::Operation(1), 20),
                (StepPosition::Operation(2), 40),
            ],
        };
        for last_stream_index_of_the_crash_window in [12, 19, 20, 35] {
            assert_eq!(
                marks.read_faults_in_a_crash_state(&step_kinds, last_stream_index_of_the_crash_window),
                ReadFaultsInTheCrashState::NoneInjected,
                "带读故障那一步一个都没发，窗口末尾在第 {last_stream_index_of_the_crash_window} 步：没注入"
            );
        }
    }
}
