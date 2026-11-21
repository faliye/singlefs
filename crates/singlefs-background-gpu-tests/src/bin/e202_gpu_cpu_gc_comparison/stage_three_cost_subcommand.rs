//! 子命令 stage-three-cost：丙段代价，CPU 臂与 GPU 点名一张卡臂各数每段活槽数、各跑一次 GC 调用量丙段那一截。

use super::*;

// ---- 子命令 stage-three-cost（登记第十二节修订第 9 条：丙段代价另量一遍，只交用户定「强制选段之后对照缺不缺丙段」那一条岔路，不进判据）----

/// 每条臂数几遍、取中位（修订第 9 条「各量 5 次……取中位」）。
pub(super) const STAGE_THREE_COST_REPEATS: usize = 5;

/// 丙段代价量的两条臂（修订第 9 条：CPU 臂与 GPU 点名一张卡臂；挂载参数同交替跑那两条臂，每张卡上限 1 GiB）。
const STAGE_THREE_COST_ARMS: [ComputeOfTheArm; 2] = [
    ComputeOfTheArm::Cpu,
    ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
];

/// CPU 臂一次 GC 调用里丙段那一截的读数：核心层在 `choose_region` 里自己数、不经任何从包外递得进去的实现，外面量不到。
const STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE: &str = "not_reachable_from_outside_the_core";

/// 装置自己从分配记录数 `unit_area` 那块盘上每个整段的活槽数，按段起点从小到大一段一个。不调核心层与产品 GPU 包：
/// 按每条记录的槽区间与段区间求交，口径同核心层的 CPU 写法（只数这块盘上未释放的记录、只数落在整段里的槽、末尾凑不满一段的那一截不算）。
pub(super) fn live_slots_of_each_segment_counted_by_the_apparatus(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<u64> {
    let unit_area_start_slot = unit_area.start.slot().0;
    let full_segments = unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    let full_segments_end_slot = unit_area_start_slot + full_segments * LOCAL_CLUSTER_SEGMENT_SLOTS;
    let mut live_slots =
        vec![0_u64; usize::try_from(full_segments).expect("段数装得进 usize")];
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        let first_slot = record.slot.0.max(unit_area_start_slot);
        let end_slot = (record.slot.0 + u64::from(record.span_slots)).min(full_segments_end_slot);
        // 迭代上界：这条记录跨过的段数（跨度至多 u16 能装的槽数）；每一轮至少前进一槽。
        let mut slot = first_slot;
        while slot < end_slot {
            let segment = (slot - unit_area_start_slot) / LOCAL_CLUSTER_SEGMENT_SLOTS;
            let segment_end_slot =
                unit_area_start_slot + (segment + 1) * LOCAL_CLUSTER_SEGMENT_SLOTS;
            let slots_in_the_segment = segment_end_slot.min(end_slot) - slot;
            live_slots[usize::try_from(segment).expect("段下标")] += slots_in_the_segment;
            slot += slots_in_the_segment;
        }
    }
    live_slots
}

/// 一条臂数一遍每段活槽数用的那一份。封闭集合。
enum StageThreeCounter<Gpu> {
    /// 核心层的 CPU 写法（`live_slots_per_cluster_segment_by_the_records`，`choose_region` 打分前数的就是它）。
    OnTheCpu,
    /// 产品 GPU 包公开的丙段（`BackgroundGpuCompute::count_live_slots_per_cluster_segment`：两份核各数一次、比对，连续不确定的交 CPU 判）。
    OnTheGpu(Gpu),
}

impl<Gpu: BackgroundGpuCompute> StageThreeCounter<Gpu> {
    /// 数一遍：交回挂钟与每段的活槽数（GPU 那一份这一轮不确定的段是 `None`）。挂钟只罩数的那一次调用（GPU 那一份交卡、等卡、两份核比对都在里面），
    /// 不罩换成 `Option` 那一步与读这一遍的交卡记录。GPU 那一份出错交回 `Err`。
    fn count_once(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<(Duration, Vec<Option<u64>>), String> {
        match self {
            StageThreeCounter::OnTheCpu => {
                let started = Instant::now();
                let live_slots =
                    live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
                let wall = started.elapsed();
                Ok((wall, live_slots.into_iter().map(Some).collect()))
            }
            StageThreeCounter::OnTheGpu(gpu) => {
                let started = Instant::now();
                let counted = gpu.count_live_slots_per_cluster_segment(
                    allocation_records,
                    unit_area,
                    segments,
                );
                let wall = started.elapsed();
                // 核心层每一批三段算完之后读一次交卡记录（读了才清零）；这里每一遍之后读一次，不让它攒着。
                let _submissions_of_the_pass = gpu.take_the_submissions_of_the_batch();
                let decisions = counted.map_err(|failed| format!("GPU 丙段出错：{failed:?}"))?;
                Ok((
                    wall,
                    decisions
                        .into_iter()
                        .map(|decision| match decision {
                            GpuEntryDecision::ConfirmedByBothKernels(live_slots)
                            | GpuEntryDecision::JudgedByTheCpuReference(live_slots) => {
                                Some(live_slots)
                            }
                            GpuEntryDecision::UncertainThisRound => None,
                        })
                        .collect(),
                ))
            }
        }
    }
}

/// 一条臂数 [`STAGE_THREE_COST_REPEATS`] 遍的读数：每一遍的挂钟与数出的每段活槽数，与头一遍之前挂载开卡另起的那条线程还在不在跑。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StageThreeCountsOfTheArm {
    pub(super) walls: Vec<Duration>,
    live_slots_of_each_pass: Vec<Vec<Option<u64>>>,
    the_opening_thread_was_running_at_the_first_pass: bool,
}

/// 同一份记录、同一块单元区上数 [`STAGE_THREE_COST_REPEATS`] 遍（修订第 9 条「数一遍全部分配记录的每段活槽数」）。
fn stage_three_counts_of_the_counter<Gpu: BackgroundGpuCompute>(
    counter: &mut StageThreeCounter<Gpu>,
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Result<StageThreeCountsOfTheArm, String> {
    let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
    let the_opening_thread_was_running_at_the_first_pass =
        the_gpu_opening_thread_is_still_running().unwrap_or(true);
    let mut walls: Vec<Duration> = Vec::new();
    let mut live_slots_of_each_pass: Vec<Vec<Option<u64>>> = Vec::new();
    for _pass in 0..STAGE_THREE_COST_REPEATS {
        let (wall, live_slots) = counter.count_once(allocation_records, unit_area, &segments)?;
        walls.push(wall);
        live_slots_of_each_pass.push(live_slots);
    }
    Ok(StageThreeCountsOfTheArm {
        walls,
        live_slots_of_each_pass,
        the_opening_thread_was_running_at_the_first_pass,
    })
}

/// 几遍挂钟的中位：奇数遍取正中那一遍，偶数遍取正中两遍的平均；一遍都没有是零。
fn median_wall_of(walls: &[Duration]) -> Duration {
    let mut sorted = walls.to_vec();
    sorted.sort();
    match sorted.len() {
        0 => Duration::ZERO,
        length if length % 2 == 1 => sorted[length / 2],
        length => (sorted[length / 2 - 1] + sorted[length / 2]) / 2,
    }
}

/// 每一遍数出的每段活槽数都与装置自己数的逐段相同（这一轮不确定的段、段数对不上都算不同）。
pub(super) fn every_pass_agrees_with_the_apparatus_count(
    counts: &StageThreeCountsOfTheArm,
    counted_by_the_apparatus: &[u64],
) -> bool {
    counts.live_slots_of_each_pass.iter().all(|live_slots| {
        live_slots.len() == counted_by_the_apparatus.len()
            && live_slots
                .iter()
                .zip(counted_by_the_apparatus)
                .all(|(counted, by_the_apparatus)| *counted == Some(*by_the_apparatus))
    })
}

/// `name=stage_three_cost` 一行（修订第 9 条写的四个键在前、次序照写，其后是装置另报的几样）。
fn stage_three_cost_fields(
    tier: UnitAreaTier,
    arm: ComputeOfTheArm,
    counts: &StageThreeCountsOfTheArm,
    counted_by_the_apparatus: &[u64],
) -> String {
    format!(
        "tier={} arm={} median_nanoseconds={} repeats={} nanoseconds_of_each_repeat={} segments={} live_slots={} \
         every_repeat_agrees_with_the_apparatus_count={} gpu_opening_thread_was_running_at_the_first_repeat={}",
        tier.name(),
        arm.name(),
        median_wall_of(&counts.walls).as_nanos(),
        counts.walls.len(),
        counts
            .walls
            .iter()
            .map(|wall| wall.as_nanos().to_string())
            .collect::<Vec<String>>()
            .join(","),
        counted_by_the_apparatus.len(),
        counted_by_the_apparatus.iter().sum::<u64>(),
        every_pass_agrees_with_the_apparatus_count(counts, counted_by_the_apparatus),
        counts.the_opening_thread_was_running_at_the_first_pass,
    )
}

/// 丙段在一次 GC 调用里被调了几次、合起来用了多少挂钟（GPU 臂外面套的计时记下的）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StageThreeTiming {
    calls: u64,
    wall: Duration,
}

type SharedStageThreeTiming = Arc<Mutex<StageThreeTiming>>;

/// 产品 GPU 包那一份外面套的计时：每次丙段（`count_live_slots_per_cluster_segment`）的挂钟与次数记进共享的那一份，别的方法原样转。
struct GpuComputeTimingTheStageThree<Inner> {
    inner: Inner,
    timing: SharedStageThreeTiming,
}

impl<Inner: GpuInventory> GpuInventory for GpuComputeTimingTheStageThree<Inner> {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.inner.devices_with_total_memory()
    }
}

impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeTimingTheStageThree<Inner> {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        self.inner.verify_unit_checksums(units)
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        self.inner
            .classify_region_records(region, records, mapping_entries)
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        let started = Instant::now();
        let counted = self.inner.count_live_slots_per_cluster_segment(
            allocation_records,
            unit_area,
            segments,
        );
        let wall = started.elapsed();
        let mut timing = self
            .timing
            .lock()
            .expect("丙段计时只在调后台入口的那条线程上写，没在别处 panic 过");
        timing.calls += 1;
        timing.wall += wall;
        counted
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.inner.use_the_settings_from_the_next_batch(settings);
    }

    fn use_no_gpu_from_the_next_batch(&mut self) {
        self.inner.use_no_gpu_from_the_next_batch();
    }

    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
        &mut self,
    ) -> LatestGpuSettingsHandedIn {
        self.inner
            .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
    }

    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        self.inner.failure_under_the_settings_handed_in()
    }

    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        self.inner.take_the_submissions_of_the_batch()
    }
}

/// 同一镜像上不强制选段、照产品 greedy 跑的那一次 GC 调用：挂钟、丙段那一截（CPU 臂量不到，`None`）、这次调用搬了几批，
/// 与调用之前挂载开卡另起的那条线程还在不在跑。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StageThreeShareOfOneGreedyCall {
    gc_call_wall: Duration,
    stage_three: Option<StageThreeTiming>,
    batches: u64,
    the_opening_thread_was_running_at_the_call: bool,
}

/// `name=stage_three_share` 一行（修订第 9 条写的四个键在前、次序照写；CPU 臂丙段那一截写 [`STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE`]）。
fn stage_three_share_fields(
    tier: UnitAreaTier,
    arm: ComputeOfTheArm,
    share: &StageThreeShareOfOneGreedyCall,
) -> String {
    let (stage_three_nanoseconds, stage_three_calls) = share.stage_three.map_or_else(
        || {
            (
                STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE.to_string(),
                STAGE_THREE_OF_THE_CPU_ARM_NOT_REACHABLE.to_string(),
            )
        },
        |timing| (timing.wall.as_nanos().to_string(), timing.calls.to_string()),
    );
    format!(
        "tier={} arm={} gc_call_nanoseconds={} stage_three_nanoseconds={stage_three_nanoseconds} \
         stage_three_calls={stage_three_calls} batches={} gpu_opening_thread_was_running_at_the_call={}",
        tier.name(),
        arm.name(),
        share.gc_call_wall.as_nanos(),
        share.batches,
        share.the_opening_thread_was_running_at_the_call,
    )
}

/// 修订第 9 条后一半：从起步镜像量挂载（照这条臂的设置，GPU 臂外面套丙段计时；不装强制选段，核心层照产品 greedy 选段），
/// 前台先发一次用户改动（同交替跑头一次调用之前那一次），再调一次后台入口、量这一次的挂钟。开卡或挂载不成交回 `Err`。
fn stage_three_share_of_one_greedy_call(
    image: &StartingImage,
    arm: ComputeOfTheArm,
    discrete_gpus: &[DiscreteGpuSeenByThisProcess],
    memory_leaked: &GpuMemoryLeakedInThisProcess,
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<StageThreeShareOfOneGreedyCall, String> {
    let settings = background_compute_settings_of_the_arm(arm, discrete_gpus);
    let (gpu_compute, timing, gpus_leaked_when_dropped) = if arm.uses_the_gpu() {
        let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
            &BackgroundGpuSelection::of_the_settings(&settings),
            memory_leaked,
            openings_not_returned_in_this_process(),
            vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
            |_gpus_to_open| {},
        )
        .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
        let gpus_leaked_when_the_gpu_is_dropped = background_gpu.gpus_leaked_when_dropped();
        let shared_timing: SharedStageThreeTiming =
            Arc::new(Mutex::new(StageThreeTiming::default()));
        let timed = GpuComputeTimingTheStageThree {
            inner: BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(
                background_gpu,
            ),
            timing: Arc::clone(&shared_timing),
        };
        (
            Some(BackgroundGpuComputeHandle::of_an_implementation(timed)),
            Some(shared_timing),
            Some(gpus_leaked_when_the_gpu_is_dropped),
        )
    } else {
        (None, None, None)
    };
    let (mut session, mut devices, _recording) = measuring_mount(
        image,
        settings,
        gpu_compute,
        BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
    )?;
    let mut write_generation_of_each_unit = image.write_generation_of_each_unit.clone();
    publish_the_user_change(
        &mut session,
        &mut devices,
        image.lowest_live_data_inode,
        0,
        &mut write_generation_of_each_unit,
        image.next_write_time_seconds,
    );
    let the_opening_thread_was_running_at_the_call =
        the_gpu_opening_thread_is_still_running().unwrap_or(true);
    let timing_before_the_call = timing.as_ref().map(|shared_timing| {
        *shared_timing
            .lock()
            .expect("丙段计时只在这条线程上写，没在别处 panic 过")
    });
    let caller = CallerOfTheLockstepRun::starting_at_zero();
    caller.advance_before_the_next_call();
    let started = Instant::now();
    let call = session
        .run_compaction_in_the_background(&mut devices, pacing_of_the_measuring_mount(), &caller)
        .expect("盘表是挂载核过的那一份、现行那一版有树：后台入口不拒");
    let gc_call_wall = started.elapsed();
    let stage_three =
        timing
            .as_ref()
            .zip(timing_before_the_call)
            .map(|(shared_timing, before_the_call)| {
                let after_the_call = *shared_timing
                    .lock()
                    .expect("丙段计时只在这条线程上写，没在别处 panic 过");
                StageThreeTiming {
                    calls: after_the_call.calls - before_the_call.calls,
                    wall: after_the_call.wall - before_the_call.wall,
                }
            });
    let batches =
        record_of_the_call(&call, 0, None, Duration::ZERO, RejudgeTally::default()).batches;
    drop(session);
    if let Some(leaked) = gpus_leaked_when_dropped
        .as_ref()
        .and_then(GpusLeakedWhenTheBackgroundGpuIsDropped::gpus_leaked)
    {
        memory_leaked.note_the_gpus_leaked_when_dropped(&leaked);
    }
    Ok(StageThreeShareOfOneGreedyCall {
        gc_call_wall,
        stage_three,
        batches,
        the_opening_thread_was_running_at_the_call,
    })
}

/// 修订第 9 条前一半：照这条臂数 [`STAGE_THREE_COST_REPEATS`] 遍。GPU 臂照交替跑那条臂的设置开产品那一份（不挂载：照核心层挂载时的做法
/// 校验设置、交给实现），数完关掉、记下卸载泄漏的卡。开卡、校验不成或这组设置下一张卡都用不上交回 `Err`。
pub(super) fn stage_three_counts_on_the_arm(
    arm: ComputeOfTheArm,
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    discrete_gpus: &[DiscreteGpuSeenByThisProcess],
    memory_leaked: &GpuMemoryLeakedInThisProcess,
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<StageThreeCountsOfTheArm, String> {
    match arm {
        ComputeOfTheArm::Cpu => stage_three_counts_of_the_counter(
            &mut StageThreeCounter::<BackgroundGpuComputeOnTheParticipatingGpus>::OnTheCpu,
            allocation_records,
            unit_area,
        ),
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress | ComputeOfTheArm::GpuOnEveryCard => {
            let settings = background_compute_settings_of_the_arm(arm, discrete_gpus);
            let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                &BackgroundGpuSelection::of_the_settings(&settings),
                memory_leaked,
                openings_not_returned_in_this_process(),
                vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
                |_gpus_to_open| {},
            )
            .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
            let gpus_leaked_when_the_gpu_is_dropped = background_gpu.gpus_leaked_when_dropped();
            let mut gpu =
                BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu);
            let admitted = admit_gpu_settings_against_the_inventory(&settings, &gpu)
                .map_err(|refused| format!("校验 GPU 设置：{refused:?}"))?;
            gpu.use_the_settings_from_the_next_batch(&admitted);
            if let Some(failure) = gpu.failure_under_the_settings_handed_in() {
                return Err(format!("这组设置下一张卡都用不上：{failure:?}"));
            }
            let mut counter = StageThreeCounter::OnTheGpu(gpu);
            let counts =
                stage_three_counts_of_the_counter(&mut counter, allocation_records, unit_area);
            drop(counter);
            if let Some(leaked) = gpus_leaked_when_the_gpu_is_dropped.gpus_leaked() {
                memory_leaked.note_the_gpus_leaked_when_dropped(&leaked);
            }
            counts
        }
    }
}

/// 起步镜像在量挂载那一刻那一版的分配记录与盘 0 的单元区（核心层选段数活槽的就是这一块；挂载照产品设置，不开卡）。挂载不成交回 `Err`。
pub(super) fn allocation_records_and_unit_area_of_the_image(
    image: &StartingImage,
) -> Result<(Vec<AllocationRecord>, UnitAreaOfOneDevice), String> {
    let (session, _devices, _recording) = measuring_mount(
        image,
        BackgroundComputeSettings::PRODUCT,
        None,
        BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
    )?;
    Ok((
        version_of_trees(&session).allocation_records.to_vec(),
        unit_area_of_device_zero(&session),
    ))
}

/// 丙段代价不拆成先录后判：GPU 臂逐段比的每段活槽数就是计时的那几遍数出来的（[`StageThreeCounter::count_once`] 一次调用同时交回挂钟与每段的数），
/// 比较入库按键复用的话，复用的那一趟就不跑这几遍、量代价那一半跟着没了；另跑几遍不计时的来比，跑在计时之前改了头一遍的冷热与开卡线程那一读数，
/// 跑在之后是多占的卡时、而量代价那一半整趟照样持卡，复用省不下卡。装置自数的参照（[`live_slots_of_each_segment_counted_by_the_apparatus`]）
/// 与造起步镜像都在取卡之前做完。
pub(super) fn stage_three_cost_command(tier: UnitAreaTier, product_path: &Path) -> i32 {
    let mut product = match ProductWriter::appending_to(product_path) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let opening = match opening_of_the_command(
        &mut product,
        product_path,
        &format!(
            "command=stage-three-cost tier={} repeats={STAGE_THREE_COST_REPEATS}",
            tier.name()
        ),
        CardsOfTheCommand::TakenBeforeOpeningTheDevices,
    ) {
        Ok(opening) => opening,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    if !emit_whether_the_make_filesystem_accepts_the_tier(&mut product, tier) {
        product.finish();
        return EXIT_STOPPED;
    }
    // 几何核同一份起步镜像（修订第 9 条）；几何成没成照打、不判：丙段数的是全部记录，不看碎片段。
    let image = match construct_the_starting_image_or_emit_the_stop(&mut product, tier) {
        Ok(image) => image,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    let _is_the_geometry_formed = emit_the_construction(&mut product, &image);
    let (allocation_records, unit_area) =
        match allocation_records_and_unit_area_of_the_image(&image) {
            Ok(records_and_unit_area) => records_and_unit_area,
            Err(reason) => {
                product.emit(
                    "stopped",
                    &format!(
                        "clause=measuring_mount_refused reason={}",
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_RUN_CONDITION_NOT_MET;
            }
        };
    let counted_by_the_apparatus =
        live_slots_of_each_segment_counted_by_the_apparatus(&allocation_records, unit_area);
    let cards = match cards_seen_by_the_command(&mut product) {
        Ok(cards) => cards,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let mut arms_run: Vec<&str> = Vec::new();
    let mut does_every_arm_agree = true;
    for arm in STAGE_THREE_COST_ARMS {
        if let Some(reason) = reason_the_arm_is_not_run_on_the_cards(arm, &cards) {
            product.emit(
                "not_run",
                &format!("tier={} arm={} reason={reason}", tier.name(), arm.name()),
            );
            continue;
        }
        // 点名一张卡臂真跑才记它开的那一张（登记第十二节修订第 16 条：丙段代价只记、不归组）。
        if let (ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress, Some(lowest)) =
            (arm, cards.discrete_gpus.first())
        {
            product.emit(
                "one_card_arm_card",
                &one_card_arm_card_fields(opening.session_number, tier, lowest),
            );
        }
        eprintln!("E202 丙段代价 {} {}", tier.name(), arm.name());
        let measured = stage_three_counts_on_the_arm(
            arm,
            &allocation_records,
            unit_area,
            &cards.discrete_gpus,
            &memory_leaked,
            cards.vulkan_instance.as_ref(),
        )
        .and_then(|counts| {
            stage_three_share_of_one_greedy_call(
                &image,
                arm,
                &cards.discrete_gpus,
                &memory_leaked,
                cards.vulkan_instance.as_ref(),
            )
            .map(|share| (counts, share))
        });
        let (counts, share) = match measured {
            Ok(counts_and_share) => counts_and_share,
            Err(reason) => {
                product.emit(
                    "stopped",
                    &format!(
                        "clause=arm_failed tier={} arm={} reason={}",
                        tier.name(),
                        arm.name(),
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_STOPPED;
            }
        };
        // 这里的比较不拆出去入库复用：逐段比的活槽数就是上面计时那几遍数出来的，拆开就要么少了量代价那一半、要么另跑几遍多占卡时。
        does_every_arm_agree &=
            every_pass_agrees_with_the_apparatus_count(&counts, &counted_by_the_apparatus);
        product.emit(
            "stage_three_cost",
            &stage_three_cost_fields(tier, arm, &counts, &counted_by_the_apparatus),
        );
        product.emit(
            "stage_three_share",
            &stage_three_share_fields(tier, arm, &share),
        );
        arms_run.push(arm.name());
    }
    product.emit(
        "verdict",
        &format!(
            "command=stage-three-cost tier={} arms_run={} every_arm_agrees_with_the_apparatus_count={does_every_arm_agree}",
            tier.name(),
            arms_run.join(",")
        ),
    );
    product.finish();
    if does_every_arm_agree {
        EXIT_SUCCESS
    } else {
        EXIT_STOPPED
    }
}

#[cfg(test)]
mod tests;
