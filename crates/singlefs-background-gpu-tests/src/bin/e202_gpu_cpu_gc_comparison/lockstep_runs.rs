//! 交替跑的一次跑：前台与 GC 两条线程、一次跑的汇总与甲类锚点。

use super::*;

/// 交替跑里前台线程与 GC 线程之间来回交的那一份：会话与两块盘（同一时刻只在一条线程手里）。
pub(super) struct SessionAndDevices {
    pub(super) session: MountedSession,
    pub(super) devices: PoolDevices,
}

/// 前台线程交给 GC 线程的。封闭集合。
enum ToTheGcThread {
    /// 第 `after_the_user_change` 次用户改动发成了，GC 线程一次次调后台入口直到一次整理发布都没发，再交回来。
    RunAfterTheUserChange {
        state: Box<SessionAndDevices>,
        after_the_user_change: u64,
        foreground_thread_task: u64,
    },
    /// 用户改动都做完了。
    Finish,
}

/// 一次交替跑记下的（窗口外读）。
struct LockstepRunRecords {
    records: Vec<GcCallRecord>,
    forced_fallback: ForcedFallbackReading,
    the_opening_thread_was_running_at_the_first_call: bool,
    state: SessionAndDevices,
    write_generation_of_each_unit: Vec<u64>,
}

/// 交替跑里 GC 线程交回的：每次调用的记录、头一次调用时开卡线程还在不在跑、强制选段后备记下的。
type GcThreadRecordsOfALockstepRun = (Vec<GcCallRecord>, bool, ForcedFallbackReading);

/// 交替跑一次（登记第五节「两种跑法」）：主线程起前台线程与 GC 线程，两条线程经通道交替；GC 调用期间前台线程阻塞在通道上。
fn lockstep_threads(
    session_and_devices_at_the_start: SessionAndDevices,
    lowest_live_data_inode: u64,
    write_generation_of_each_unit_at_the_start: Vec<u64>,
    first_write_time_seconds: u64,
    user_changes: u64,
    arm: LockstepArm,
    forced_fallback_of_the_run: ForcedFallbackOfTheRun,
) -> Result<LockstepRunRecords, ClockReadFailed> {
    let main_thread_task = task_identifier_of_this_thread()?;
    let unit_areas = unit_areas_of_every_device(&session_and_devices_at_the_start.session);
    let (to_the_gc_thread, gc_thread_inbox) = mpsc::channel::<ToTheGcThread>();
    let (to_the_foreground_thread, foreground_thread_inbox) = mpsc::channel::<SessionAndDevices>();
    std::thread::scope(|scope| {
        let foreground = scope.spawn(
            move || -> Result<(SessionAndDevices, Vec<u64>), ClockReadFailed> {
                let foreground_thread_task = task_identifier_of_this_thread()?;
                let mut state = session_and_devices_at_the_start;
                let mut write_generation_of_each_unit = write_generation_of_each_unit_at_the_start;
                for change_number in 0..user_changes {
                    publish_the_user_change(
                        &mut state.session,
                        &mut state.devices,
                        lowest_live_data_inode,
                        change_number,
                        &mut write_generation_of_each_unit,
                        first_write_time_seconds + change_number,
                    );
                    to_the_gc_thread
                        .send(ToTheGcThread::RunAfterTheUserChange {
                            state: Box::new(state),
                            after_the_user_change: change_number,
                            foreground_thread_task,
                        })
                        .expect("GC 线程在等");
                    state = foreground_thread_inbox.recv().expect("GC 线程交回会话");
                }
                to_the_gc_thread
                    .send(ToTheGcThread::Finish)
                    .expect("GC 线程在等");
                Ok((state, write_generation_of_each_unit))
            },
        );
        let gc = scope.spawn(move || -> GcThreadRecordsOfALockstepRun {
            let mut forced_fallback = forced_fallback_of_the_run;
            let started = StartOnTheMonotonicRawClock::now();
            let gc_thread_task = task_identifier_of_this_thread().ok();
            let caller = CallerOfTheLockstepRun::starting_at_zero();
            let pacing = pacing_of_the_measuring_mount();
            let mut records: Vec<GcCallRecord> = Vec::new();
            let mut the_opening_thread_was_running_at_the_first_call = false;
            while let Ok(ToTheGcThread::RunAfterTheUserChange {
                mut state,
                after_the_user_change,
                foreground_thread_task,
            }) = gc_thread_inbox.recv()
            {
                let clocks = ClocksOfTheGcThread {
                    started,
                    gc_thread_task,
                    foreground_thread_task,
                    main_thread_task,
                };
                loop {
                    if records.is_empty() {
                        the_opening_thread_was_running_at_the_first_call =
                            the_gpu_opening_thread_is_still_running().unwrap_or(true);
                    }
                    caller.advance_before_the_next_call();
                    let empty_segments_before_the_call =
                        empty_segments_of_every_device(&state.session);
                    let intents_issued_before_the_call =
                        forced_fallback.before_the_call(&mut state.session, &state.devices);
                    let measured = measure_one_gc_call(
                        &clocks,
                        &mut state,
                        |state| {
                            state
                                .session
                                .run_compaction_in_the_background(
                                    &mut state.devices,
                                    pacing,
                                    &caller,
                                )
                                .expect("盘表是挂载核过的那一份、现行那一版有树：后台入口不拒")
                        },
                        |state, call| match arm.rejudging {
                            RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu
                                if !arm.compute.uses_the_gpu() =>
                            {
                                rejudge_the_batches_of_the_call_on_the_cpu(
                                    call,
                                    &state.devices,
                                    &unit_areas,
                                )
                            }
                            RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu
                            | RejudgingOfTheArm::NotRejudged => {
                                (Some(Duration::ZERO), RejudgeTally::default())
                            }
                        },
                    );
                    forced_fallback.after_the_call(
                        &state.session,
                        intents_issued_before_the_call,
                        &measured.outcome,
                    );
                    let mut record = record_of_the_call(
                        &measured.outcome,
                        after_the_user_change,
                        measured.window,
                        measured.inner_rejudge_cpu,
                        measured.rejudge_tally,
                    );
                    record.empty_segments_before_the_call = empty_segments_before_the_call;
                    record.empty_segments_after_the_call =
                        empty_segments_of_every_device(&state.session);
                    let is_done = the_call_published_nothing(&record);
                    records.push(record);
                    if is_done {
                        break;
                    }
                }
                to_the_foreground_thread.send(*state).expect("前台线程在等");
            }
            (
                records,
                the_opening_thread_was_running_at_the_first_call,
                forced_fallback.reading,
            )
        });
        let (records, the_opening_thread_was_running_at_the_first_call, forced_fallback) =
            gc.join().expect("GC 线程没有 panic");
        let (state, write_generation_of_each_unit) =
            foreground.join().expect("前台线程没有 panic")?;
        Ok(LockstepRunRecords {
            records,
            forced_fallback,
            the_opening_thread_was_running_at_the_first_call,
            state,
            write_generation_of_each_unit,
        })
    })
}

/// 一次交替跑的结果：`name=run` 一行的数、锚点与作废原因。
pub(super) struct LockstepRunSummary {
    pub(super) arm: LockstepArm,
    pub(super) repeat: u32,
    calls: u64,
    calls_started: u64,
    pub(super) batches: u64,
    units_relocated: u64,
    pub(super) gc_wall: Duration,
    pub(super) process_cpu: Duration,
    gc_thread_cpu: Duration,
    foreground_thread_cpu: Duration,
    main_thread_cpu: Duration,
    pub(super) inner_rejudge_cpu: Duration,
    pub(super) rejudge: RejudgeTally,
    pub(super) recording_digest: String,
    recording_operations: u64,
    recording_overhead: Duration,
    pub(super) content_digest: String,
    pub(super) content_matches_the_expectation: bool,
    pub(super) host_other_cores: f64,
    path_anchor: PathAnchor,
    pub(super) gpu_work_anchors: GpuWorkAnchors,
    batches_not_adding_up_to_the_entries_handed_in: u64,
    live_slot_calls_not_handing_in_every_full_segment: u64,
    batches_with_live_slot_entries_not_whole_device_passes: u64,
    most_compaction_publishes_between_two_user_changes: u64,
    calls_failing_the_inclusion_checks: u64,
    /// 判不过包含自检的那几次调用的窗口（不进产物行；交替跑每次尝试之后打到标准错误，小池用例判红时也打出来，看是哪一条不等式、差多少）。
    pub(super) windows_failing_the_inclusion_checks: Vec<CallWindow>,
    cpu_arm_thread_cpu_covers_the_wall: Option<bool>,
    gpus_leaked: u64,
    calls_with_the_alarm: u64,
    budget_left_trajectory: TrajectoryOfTheReadings,
    uncertain_entries_trajectory: TrajectoryOfTheReadings,
    empty_segments_trajectory: Vec<EmptySegmentsTrajectoryOfADevice>,
    pub(super) forced_fallback: ForcedFallbackReading,
    stops: BTreeMap<String, u64>,
    pub(super) void_reason: Option<String>,
}

impl LockstepRunSummary {
    /// 第七节甲类、乙类锚点都成立（停机条款第 4 条看甲类；乙类不符那一档作废）。
    pub(super) fn class_one_anchors_hold(&self) -> bool {
        class_one_anchors_of_a_lockstep_run_hold(
            self.path_anchor,
            self.arm.compute,
            self.gpu_work_anchors,
            self.batches_not_adding_up_to_the_entries_handed_in,
            self.most_compaction_publishes_between_two_user_changes,
        )
    }

    pub(super) fn class_two_anchors_hold(&self) -> bool {
        self.live_slot_calls_not_handing_in_every_full_segment == 0
            && self.batches_with_live_slot_entries_not_whole_device_passes == 0
    }

    pub(super) fn fields(
        &self,
        tier: UnitAreaTier,
        session_number: u64,
        gpu_models: &str,
    ) -> String {
        let stops: Vec<String> = self
            .stops
            .iter()
            .map(|(stop, count)| format!("{stop}:{count}"))
            .collect();
        format!(
            "tier={} arm={} repeat={} session={session_number} calls={} calls_started={} batches={} units_relocated={} \
             gc_wall_nanoseconds={} gc_wall_nanoseconds_per_batch={} process_cpu_nanoseconds={} gc_thread_cpu_nanoseconds={} \
             foreground_thread_cpu_nanoseconds={} main_thread_cpu_nanoseconds={} inner_rejudge_cpu_nanoseconds={} \
             rejudged_entries={} rejudge_mismatches={} recording_digest={} recording_operations={} recording_overhead_nanoseconds={} \
             content_digest={} content_matches_the_expectation={} host_other_cores={:.2} gpu_models={} \
             compute_records={} compute_records_on_the_path_of_the_settings={} path_anchor_holds={} \
             gpu_records={} segments_given_to_a_card_other_than_the_lowest={} segments_without_a_submission={} \
             recuts_after_running_out_of_memory={} segments_with_pieces_other_than_submissions={} uncertain_entries={} \
             entries_judged_by_the_cpu_reference={} batches_abandoned_after_a_compute_error={} \
             batches_not_adding_up_to_the_entries_handed_in={} live_slot_calls_not_handing_in_every_full_segment={} \
             batches_with_live_slot_entries_not_whole_device_passes={} most_compaction_publishes_between_two_user_changes={} \
             calls_failing_the_inclusion_checks={} cpu_arm_thread_cpu_covers_the_wall={} class_one_anchors_hold={} \
             class_two_anchors_hold={} gpus_leaked={} calls_with_the_alarm={} {} {} {} {} stops={} void={}",
            tier.name(),
            self.arm.name(),
            self.repeat,
            self.calls,
            self.calls_started,
            self.batches,
            self.units_relocated,
            self.gc_wall.as_nanos(),
            self.gc_wall.as_nanos() / u128::from(self.batches.max(1)),
            self.process_cpu.as_nanos(),
            self.gc_thread_cpu.as_nanos(),
            self.foreground_thread_cpu.as_nanos(),
            self.main_thread_cpu.as_nanos(),
            self.inner_rejudge_cpu.as_nanos(),
            self.rejudge.entries_rejudged,
            self.rejudge.mismatches,
            self.recording_digest,
            self.recording_operations,
            self.recording_overhead.as_nanos(),
            self.content_digest,
            self.content_matches_the_expectation,
            self.host_other_cores,
            gpu_models,
            self.path_anchor.compute_records,
            self.path_anchor.compute_records_on_the_path_of_the_settings,
            self.path_anchor.holds(),
            self.gpu_work_anchors.gpu_records,
            self.gpu_work_anchors.segments_given_to_a_card_other_than_the_lowest,
            self.gpu_work_anchors.segments_without_a_submission,
            self.gpu_work_anchors.recuts_after_running_out_of_memory,
            self.gpu_work_anchors.segments_with_pieces_other_than_submissions,
            self.gpu_work_anchors.uncertain_entries,
            self.gpu_work_anchors.entries_judged_by_the_cpu_reference,
            self.gpu_work_anchors.batches_abandoned_after_a_compute_error,
            self.batches_not_adding_up_to_the_entries_handed_in,
            self.live_slot_calls_not_handing_in_every_full_segment,
            self.batches_with_live_slot_entries_not_whole_device_passes,
            self.most_compaction_publishes_between_two_user_changes,
            self.calls_failing_the_inclusion_checks,
            self.cpu_arm_thread_cpu_covers_the_wall
                .map_or_else(|| "not_a_cpu_arm".to_string(), |covers| covers.to_string()),
            self.class_one_anchors_hold(),
            self.class_two_anchors_hold(),
            self.gpus_leaked,
            self.calls_with_the_alarm,
            trajectory_fields("budget_left", self.budget_left_trajectory),
            trajectory_fields("uncertain_entries", self.uncertain_entries_trajectory),
            empty_segments_fields(&self.empty_segments_trajectory),
            self.forced_fallback.fields(),
            field_value(&stops.join(",")),
            field_value(self.void_reason.as_deref().unwrap_or("no")),
        )
    }
}

/// 交替跑一次的甲类锚点（第七节甲类第 1–4 行）都成立：每一项记账照设置走、条数不少于批数；GPU 臂另要分到活的卡、切片与两份核比对那几样；
/// 多判的 GPU 臂每批三段的判定加起来等于交进去的；相邻两次用户改动之间的整理发布不超过 B_gc。
fn class_one_anchors_of_a_lockstep_run_hold(
    path_anchor: PathAnchor,
    compute: ComputeOfTheArm,
    gpu_work_anchors: GpuWorkAnchors,
    batches_not_adding_up_to_the_entries_handed_in: u64,
    most_compaction_publishes_between_two_user_changes: u64,
) -> bool {
    path_anchor.holds()
        && (!compute.uses_the_gpu() || gpu_work_anchors.hold())
        && batches_not_adding_up_to_the_entries_handed_in == 0
        && most_compaction_publishes_between_two_user_changes
            <= LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE
}

/// 一次交替跑的设定。
pub(super) struct LockstepRunSetting<'run> {
    pub(super) image: &'run StartingImage,
    pub(super) discrete_gpus: &'run [DiscreteGpuSeenByThisProcess],
    /// 全部卡臂开的那几张（`--cards` 挑过的，[`discrete_gpus_named_for_the_every_card_arm`]）。
    pub(super) discrete_gpus_of_the_every_card_arm: &'run [DiscreteGpuSeenByThisProcess],
    pub(super) memory_leaked: &'run GpuMemoryLeakedInThisProcess,
    /// 开卡用的那一份进程级 Vulkan 实例；只跑 CPU 臂的没有。
    pub(super) vulkan_instance: Option<&'run VulkanInstanceOfThisProcess>,
    pub(super) user_changes: u64,
    pub(super) batch_units: usize,
}

impl LockstepRunSetting<'_> {
    /// 这条臂开的卡从哪一份里取（[`discrete_gpus_of_the_arm`]）。
    pub(super) fn discrete_gpus_of_the_arm(
        &self,
        compute: ComputeOfTheArm,
    ) -> &[DiscreteGpuSeenByThisProcess] {
        discrete_gpus_of_the_arm(
            compute,
            self.discrete_gpus,
            self.discrete_gpus_of_the_every_card_arm,
        )
    }
}

/// 交替跑一次：拷池、开卡、挂载、两条线程交替、读摘要与内容、核锚点、记机器负载。开卡或挂载不成交回 `Err`（交主 agent，不当跑过）。
pub(super) fn run_lockstep_once(
    setting: &LockstepRunSetting<'_>,
    arm: LockstepArm,
    repeat: u32,
) -> Result<LockstepRunSummary, String> {
    let host_at_the_start = host_load_reading()?;
    let settings = background_compute_settings_of_the_arm(
        arm.compute,
        setting.discrete_gpus_of_the_arm(arm.compute),
    );
    let gpu = open_the_gpu_of_the_arm(
        arm,
        &settings,
        setting.memory_leaked,
        setting.vulkan_instance,
    )?;
    let (session, devices, recording) = measuring_mount(
        setting.image,
        settings,
        gpu.gpu_compute,
        setting.batch_units,
    )?;
    let threads = lockstep_threads(
        SessionAndDevices { session, devices },
        setting.image.lowest_live_data_inode,
        setting.image.write_generation_of_each_unit.clone(),
        setting.image.next_write_time_seconds,
        setting.user_changes,
        arm,
        ForcedFallbackOfTheRun::of_the_image(setting.image),
    );
    let LockstepRunRecords {
        records,
        forced_fallback,
        the_opening_thread_was_running_at_the_first_call,
        state,
        write_generation_of_each_unit,
    } = threads.map_err(|failed| format!("拿线程的 CPU 钟：{}", failed.description()))?;
    let (recording_digest, recording_operations, recording_overhead, recording_clock_failures) = {
        let recording = recording.lock().expect("录制摘要没在别处 panic 过");
        (
            recording.digest.clone().finish_as_hex(),
            recording.operations,
            recording.cpu_spent_feeding,
            recording.clock_read_failures,
        )
    };
    let content_digest = read_content_digest(
        &state.devices,
        version_of_trees(&state.session),
        setting.image.lowest_live_data_inode,
        u64::try_from(write_generation_of_each_unit.len()).expect("P"),
    )?;
    let content_matches_the_expectation =
        content_digest == expected_content_digest(&write_generation_of_each_unit);
    let full_segments_of_a_device =
        unit_area_of_device_zero(&state.session).slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    drop(state);
    let gpus_leaked = match gpu
        .gpus_leaked_when_dropped
        .as_ref()
        .and_then(GpusLeakedWhenTheBackgroundGpuIsDropped::gpus_leaked)
    {
        Some(leaked) => {
            setting
                .memory_leaked
                .note_the_gpus_leaked_when_dropped(&leaked);
            u64::try_from(leaked.len()).expect("卡数")
        }
        None => 0,
    };
    let windows: Vec<CallWindow> = records.iter().filter_map(|record| record.window).collect();
    let every_window_read = windows.len() == records.len();
    let (
        handed_in_of_each_batch,
        live_slot_calls_not_handing_in_every_full_segment,
        rejudge_clock_failures,
        wrapper_tally,
        wrapper_inner,
    ) = match &gpu.rejudge_log {
        Some(log) => {
            let log = log.lock().expect("多判记录没在别处 panic 过");
            (
                Some(log.handed_in_of_each_batch.clone()),
                log.live_slot_calls_not_handing_in_every_full_segment,
                log.clock_read_failures,
                log.tally,
                log.inner_rejudge_cpu,
            )
        }
        None => (None, 0, 0, RejudgeTally::default(), Duration::ZERO),
    };
    let mut rejudge = wrapper_tally;
    for record in &records {
        rejudge.add(record.rejudge_tally);
    }
    let gc_wall = gc_wall_clock_of_the_run(&windows, &records);
    let gc_thread_cpu: Duration = windows.iter().map(|window| window.gc_thread_cpu).sum();
    let cpu_arm_thread_cpu_covers_the_wall = (!arm.compute.uses_the_gpu())
        .then(|| cpu_arm_thread_cpu_covers_the_wall(gc_thread_cpu, gc_wall));
    let count_of_calls_failing_the_inclusion_checks = calls_failing_the_inclusion_checks(&windows);
    let windows_failing_the_inclusion_checks: Vec<CallWindow> = windows
        .iter()
        .filter(|window| calls_failing_the_inclusion_checks(std::slice::from_ref(*window)) > 0)
        .copied()
        .collect();
    let lowest_card =
        setting
            .discrete_gpus_of_the_arm(arm.compute)
            .first()
            .map(|lowest_discrete_gpu| {
                gpu_device_identifier_of_the_pci_bus_address(&lowest_discrete_gpu.pci_bus_address)
            });
    let gpu_work_anchors = lowest_card
        .as_ref()
        .map_or_else(GpuWorkAnchors::default, |lowest_card| {
            gpu_work_anchors(&records, lowest_card)
        });
    let void_reason =
        if !every_window_read || recording_clock_failures > 0 || rejudge_clock_failures > 0 {
            Some("a_clock_read_failed_or_went_backwards".to_string())
        } else if count_of_calls_failing_the_inclusion_checks > 0
            || cpu_arm_thread_cpu_covers_the_wall == Some(false)
        {
            Some("an_inclusion_check_failed".to_string())
        } else if the_opening_thread_was_running_at_the_first_call {
            Some("the_gpu_opening_thread_was_running_at_the_first_call".to_string())
        } else {
            None
        };
    let mut stops: BTreeMap<String, u64> = BTreeMap::new();
    for record in &records {
        *stops.entry(record.stop_name.clone()).or_insert(0) += 1;
    }
    let host_at_the_end = host_load_reading()?;
    Ok(LockstepRunSummary {
        arm,
        repeat,
        calls: u64::try_from(records.len()).expect("调用数"),
        calls_started: u64::try_from(records.iter().filter(|record| record.started).count())
            .expect("调用数"),
        batches: records.iter().map(|record| record.batches).sum(),
        units_relocated: records
            .iter()
            .flat_map(|record| record.units_relocated_of_each_batch.iter())
            .sum(),
        gc_wall,
        process_cpu: windows.iter().map(|window| window.process_cpu).sum(),
        gc_thread_cpu,
        foreground_thread_cpu: windows
            .iter()
            .map(|window| window.foreground_thread_cpu)
            .sum(),
        main_thread_cpu: windows.iter().map(|window| window.main_thread_cpu).sum(),
        inner_rejudge_cpu: wrapper_inner
            + records
                .iter()
                .map(|record| record.inner_rejudge_cpu)
                .sum::<Duration>(),
        rejudge,
        recording_digest,
        recording_operations,
        recording_overhead,
        content_digest,
        content_matches_the_expectation,
        host_other_cores: cores_used_outside_this_process(host_at_the_start, host_at_the_end),
        path_anchor: path_anchor(&records, arm.compute),
        gpu_work_anchors,
        batches_not_adding_up_to_the_entries_handed_in: handed_in_of_each_batch.map_or(
            0,
            |handed_in| {
                batches_whose_decisions_do_not_add_up_to_the_entries_handed_in(&records, &handed_in)
            },
        ),
        live_slot_calls_not_handing_in_every_full_segment,
        batches_with_live_slot_entries_not_whole_device_passes:
            batches_whose_live_slot_entries_are_not_whole_device_passes(
                &records,
                full_segments_of_a_device,
            ),
        most_compaction_publishes_between_two_user_changes:
            most_compaction_publishes_between_two_user_changes(&records),
        calls_failing_the_inclusion_checks: count_of_calls_failing_the_inclusion_checks,
        windows_failing_the_inclusion_checks,
        cpu_arm_thread_cpu_covers_the_wall,
        gpus_leaked,
        calls_with_the_alarm: u64::try_from(
            records.iter().filter(|record| record.has_the_alarm).count(),
        )
        .expect("调用数"),
        budget_left_trajectory: trajectory_of_the_readings(
            &budget_left_before_and_after_each_call(&records),
        ),
        uncertain_entries_trajectory: trajectory_of_the_readings(
            &uncertain_entries_of_each_gpu_record(&records),
        ),
        empty_segments_trajectory: empty_segments_trajectory_of_each_device(&records),
        forced_fallback,
        stops,
        void_reason,
    })
}

#[cfg(test)]
mod tests;
