//! 整理的一批：取这一批的输入，在 CPU 或 GPU 上扫区、数活槽、提出这一批，回锁时判算好的那一批并发布，
//! 发选中的区与活意图那一批，按扫描结果计划的那一步，停下时丢掉活意图。

use super::*;

/// 取一批的输入交回的（[`take_the_input_of_one_compaction_batch`]）。封闭集合。
pub(super) enum CompactionBatchTaken {
    /// 这一批在会话锁里就推完了：整理停着、这一批开头占用不超过补跑阈值，这一批不开（有活意图就删）。装箱：一批推完的结局带着发布被拒的原因与这一批的后台计算记账，比另一个成员大得多。
    AdvancedInTheLock(Box<CompactionPublishAdvanced>),
    /// 交到算的那一段去（装箱：输入带着整份 R 的单元字节与几张集合）。
    Input(Box<CompactionBatchInput>),
}

/// 取一批的输入（会话锁里）：开这一批（读设置、定走法），取走只供测试注入的错，把发布路径隔离过的槽并进常驻隔离集，拷现行那一版的指针与算要的几张集合；
/// 有活意图而 R 里没有隔离槽时把 R 里的单元读进来（读盘留在锁里）。整理停着而这一批不补跑时不开这一批，有活意图就单发一次只删意图的发布。
/// 会话记着一条回锁时删意图被回退门压住的活意图、它仍是现行那一版里的活意图时（[`stop_of_the_abandoned_batch_whose_intent_is_still_to_drop`]），
/// 不开这一批，先单发一次只删意图的发布（这一趟随后停在放弃那一批时停的那一条）：回退门与破例在这之前那一判里照停着时那一次只删意图的发布判过
/// （[`judge_before_the_next_compaction_batch`]：门关就停在门上，走不到这里），不在这条意图下接着搬。现行那一版里已不是它，就清掉那一条照常取。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、这一趟的账、这次挂载的后台计算状态、一批读完设置之后做什么"
)]
pub(super) fn take_the_input_of_one_compaction_batch<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    run: &CompactionRunUnderWay,
    background_compute: &mut BackgroundComputeOfThisMount,
    in_the_middle_of_each_batch: &mut InTheMiddleOfEachBackgroundBatch<'_>,
) -> CompactionBatchTaken {
    if let Some(stop_of_the_abandoned_batch) =
        stop_of_the_abandoned_batch_whose_intent_is_still_to_drop(compaction, current)
    {
        return CompactionBatchTaken::AdvancedInTheLock(Box::new(CompactionPublishAdvanced {
            step: drop_the_live_intent_on_a_stop(
                parameters,
                devices,
                allocator,
                compaction,
                current,
                instance,
                stop_of_the_abandoned_batch,
            ),
            background_compute_of_the_batch: None,
            txg_the_step_was_judged_on: current.root().checkpoint_txg,
        }));
    }
    compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();
    let Ok(batch_under_way) = background_compute.begin_a_batch(
        BackgroundTask::CompactionRelocation,
        AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(allocator),
    ) else {
        // 停着而这一批开头占用不超过补跑阈值（上一批补跑腾出了空间、或一趟开头就停着而还有活意图）：这一批不开，有活意图就删。
        return CompactionBatchTaken::AdvancedInTheLock(Box::new(CompactionPublishAdvanced {
            step: drop_the_live_intent_on_a_stop(
                parameters,
                devices,
                allocator,
                compaction,
                current,
                instance,
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError,
            ),
            background_compute_of_the_batch: None,
            txg_the_step_was_judged_on: current.root().checkpoint_txg,
        }));
    };
    in_the_middle_of_each_batch.in_the_middle_of_the_batch(background_compute);
    let compute_error_of_the_batch =
        background_compute.take_the_compute_error_of_the_batch_under_way();
    if compute_error_of_the_batch.is_none() {
        compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
    }
    let version = Arc::clone(
        current
            .shared_version_of_trees()
            .expect("整理只在有树的一版上推：入口判过，整理的发布与删除续做都交回有树的一版"),
    );
    let live_on_disk = live_compaction_intent_of_the_entries(&version.logged_operations);
    assert_eq!(
        allocator
            .compaction()
            .live_intent()
            .map(|live| live.intent),
        live_on_disk.map(|(_, intent)| intent),
        "分配器这一侧的活意图就是现行那一版 logged_ops 里那一条：会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上"
    );
    let unit_area = {
        let device_map = allocator
            .devices
            .first()
            .expect("池里至少一块盘：两盘同槽，选段与判类按第一块盘的记录做（判决 G1 攻方 K-n）");
        UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        }
    };
    let quarantined_slots = compaction.quarantined_slots().clone();
    let live_intent = live_on_disk.map(|(_issued_txg, intent)| intent);
    // 有活意图那一支只扫 R 一段，R 里的单元在取输入时读（读盘留在会话锁里，D26（后台整理与放置回收） 已定项 13 第 5 条）。
    let unit_bytes_of_the_live_region = match (&compute_error_of_the_batch, live_intent) {
        (None, Some(intent))
            if !region_holds_a_quarantined_slot(intent.region, &quarantined_slots) =>
        {
            let devices_to_read: &[(DeviceIdentity, Device)] = &*devices;
            Some(
                unit_reads_of_the_region_scan(
                    intent.region,
                    &version.allocation_records,
                    &pinned_slots_of_the_version(&version),
                    &quarantined_slots,
                )
                .read_with(|device, slot, unit_bytes| {
                    devices_to_read.read(device, slot.to_device_offset(), unit_bytes)
                }),
            )
        }
        (None, Some(_)) | (None, None) | (Some(_), Some(_)) | (Some(_), None) => None,
    };
    let switches = allocator.compaction().switches;
    CompactionBatchTaken::Input(Box::new(CompactionBatchInput {
        batch_under_way,
        compute_error_of_the_batch,
        live_intent,
        quarantined_slots,
        segments_refused_by_the_region_scan: compaction.segments_refused_by_the_region_scan.clone(),
        cluster_segments: allocator.cluster_segments().clone(),
        unit_area,
        forced_region: switches.forced_region,
        batch_units: run.batch_units,
        reach: run.reach,
        last_completed_intent: compaction.last_completed_intent,
        region_check: run.region_check,
        unit_bytes_of_the_live_region,
        version,
    }))
}

/// 一批整理这一次在哪算（取输入时定下，一批之内不变）。封闭集合。
enum CompactionComputeOfTheBatch {
    /// 区域扫描、选段打分都照 CPU 写法（`scan_region_over_the_identity_index`、`choose_region`）。
    OnTheCpu,
    /// 甲、乙两段经 [`scan_region_on_the_gpu`]，丙段经 [`live_slots_per_cluster_segment_computing_on`]，都交这一份实现；
    /// 各段的条目数记进 `gpu_work`。实现那把锁每一段调用借一次、调完就放（[`BackgroundGpuComputeHandle::lock`]）：回会话锁读盘、交回之前都不持着它。
    /// `gpu_work` 装箱：三段各自的条目数与卡上的交法，比在 CPU 上算的那一个成员大得多。
    OnTheGpu {
        gpu_compute: BackgroundGpuComputeHandle,
        gpu_work: Box<GpuWorkOfTheBatch>,
    },
}

/// 扫一个 R，照这一批在哪算：CPU 那一条交回 `scan_region_over_the_identity_index` 的结果（没有不确定的记录）；GPU 那一条经 [`scan_region_on_the_gpu`]。
/// 两条读同一版、同一份索引与同两张集合，单元的字节都取 `unit_bytes_read`（取输入或回锁时读好的）。
///
/// # Errors
/// GPU 实现交回出错、或实现那把锁中毒（上一次借出时 panic 过）⇒ [`BackgroundComputeError`]。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：R、这一版的记录、按身份索引、钉住与隔离的槽、读好的单元字节、这一批在哪算"
)]
fn scan_the_region_computing_on(
    region: CompactionRegion,
    allocation_records: &[AllocationRecord],
    mapping_index: &CentralMappingIdentityIndex<'_>,
    pinned_slots: &BTreeSet<SlotNumber>,
    quarantined_slots: &BTreeSet<SlotNumber>,
    unit_bytes_read: &UnitBytesReadBeforeTheRegionScan,
    compute: &mut CompactionComputeOfTheBatch,
) -> Result<RegionScanWithTheRecordsUncertainThisRound, BackgroundComputeError> {
    match compute {
        CompactionComputeOfTheBatch::OnTheCpu => Ok(RegionScanWithTheRecordsUncertainThisRound {
            scan: scan_region_over_the_identity_index(
                region,
                allocation_records,
                mapping_index,
                pinned_slots,
                quarantined_slots,
                |device, slot, unit_bytes| unit_bytes_read.unit_bytes(device, slot, unit_bytes),
            ),
            records_uncertain_this_round: Vec::new(),
        }),
        CompactionComputeOfTheBatch::OnTheGpu {
            gpu_compute,
            gpu_work,
        } => {
            let mut implementation = gpu_compute
                .lock()
                .map_err(BackgroundComputeError::of_the_gpu_implementation_poisoned)?;
            scan_region_on_the_gpu(
                region,
                allocation_records,
                mapping_index,
                pinned_slots,
                quarantined_slots,
                |device, slot, unit_bytes| unit_bytes_read.unit_bytes(device, slot, unit_bytes),
                &mut *implementation,
                gpu_work,
            )
        }
    }
}

/// 丙段（选段打分）在 GPU 上数每个整段的活槽数：CPU 那一条交 `None`（由 `choose_region` 自己数）；GPU 那一条一个整段一个，
/// 这一轮不确定的段记成 [`LiveSlotsOfAClusterSegment::UncertainThisRound`]。实现那把锁只在这一次调用里借着。
///
/// # Errors
/// GPU 实现交回出错 ⇒ [`BackgroundComputeError::GpuSegmentFailed`]；实现那把锁中毒 ⇒ [`BackgroundComputeError::GpuImplementationPanickedWhileLent`]。
fn live_slots_per_cluster_segment_computing_on(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    compute: &mut CompactionComputeOfTheBatch,
) -> Result<Option<Vec<LiveSlotsOfAClusterSegment>>, BackgroundComputeError> {
    match compute {
        CompactionComputeOfTheBatch::OnTheCpu => Ok(None),
        CompactionComputeOfTheBatch::OnTheGpu {
            gpu_compute,
            gpu_work,
        } => {
            let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
            let counted_on_the_gpu = gpu_compute
                .lock()
                .map_err(BackgroundComputeError::of_the_gpu_implementation_poisoned)?
                .count_live_slots_per_cluster_segment(allocation_records, unit_area, &segments);
            let decisions = decisions_of_one_segment(
                GpuComputedSegment::LiveSlotsPerClusterSegment,
                segments.len(),
                counted_on_the_gpu,
                &mut gpu_work.live_slots_per_cluster_segment,
            )?;
            Ok(Some(
                decisions
                    .into_iter()
                    .map(|decision| match decision {
                        GpuEntryDecision::ConfirmedByBothKernels(live_slots)
                        | GpuEntryDecision::JudgedByTheCpuReference(live_slots) => {
                            LiveSlotsOfAClusterSegment::Counted(live_slots)
                        }
                        GpuEntryDecision::UncertainThisRound => {
                            LiveSlotsOfAClusterSegment::UncertainThisRound
                        }
                    })
                    .collect(),
            ))
        }
    }
}

/// 锁外那一段在票里那一版上算出的提议。封闭集合。
pub(super) enum CompactionBatchProposal {
    /// 照判据停（没有活意图可续、上一条无产出、没有合格的段、选中那一段有这一轮不确定的记录、R 里只剩不确定的记录），这一批不发。
    Stopped(CompactionRunStop),
    /// 没有活意图：选中这一段（扫描里至少一个能搬的），回锁里重判资格与签发前那一判再签。
    Issue {
        scan: RegionScan,
        /// 票里那一版码 1 数据单元落着的槽（签发前那一判用；现行那一版没变时照用）。
        slots_holding_user_data: BTreeSet<SlotNumber>,
    },
    /// 有活意图：对 R 扫过（R 里有隔离槽、扫描拒了 R、扫出隔离类时没有扫描），按它计划的头一批（跳过的集合是空的）。
    LiveIntentBatch {
        intent: CompactionIntent,
        scan: Option<(RegionScan, Vec<Placement>)>,
        plan_computed_out_of_the_lock: (
            Vec<PlannedRelocation>,
            CompactionIntentStep,
            CompactionPublishKind,
        ),
    },
}

/// 锁外那一段（D26（后台整理与放置回收） 已定项 13 第 5 条）：只读取输入时拿到的那一份，不碰会话。建这一版的映射条目、钉住集与按身份索引，
/// 选段（扫描、打分）或对 R 扫描，算出提议。要读盘上一段单元时（选段那一圈要扫的那一段）交给 `read_in_the_session_lock`：
/// 调用方回会话锁里读完交回字节再接着算（放锁的入口每一圈回锁里读，三段首尾相接时当场读）。GPU 实现那把锁每一段调用借一次，
/// 读盘之前、交回之前都不持着；三段在卡上是怎么交的在交回之前取完（[`crate::background_compute::BackgroundGpuCompute::take_the_submissions_of_the_batch`]）。
pub(super) fn compute_the_compaction_batch(
    input: CompactionBatchInput,
    read_in_the_session_lock: &mut dyn FnMut(
        UnitReadsOfARegionScan,
    ) -> UnitBytesReadBeforeTheRegionScan,
) -> CompactionBatchComputed {
    let CompactionBatchInput {
        batch_under_way,
        compute_error_of_the_batch,
        version,
        live_intent,
        quarantined_slots,
        mut segments_refused_by_the_region_scan,
        cluster_segments,
        unit_area,
        forced_region,
        batch_units,
        reach,
        last_completed_intent,
        region_check,
        unit_bytes_of_the_live_region,
    } = input;
    let mut compute = match batch_under_way.gpu_compute_of_the_route() {
        None => CompactionComputeOfTheBatch::OnTheCpu,
        Some(gpu_compute) => CompactionComputeOfTheBatch::OnTheGpu {
            gpu_compute: gpu_compute.clone(),
            gpu_work: Box::default(),
        },
    };
    let proposal = match compute_error_of_the_batch {
        Some(error) => Err(error),
        None => {
            let proposed = propose_the_compaction_batch(
                &version,
                live_intent,
                unit_bytes_of_the_live_region,
                &quarantined_slots,
                &mut segments_refused_by_the_region_scan,
                &cluster_segments,
                unit_area,
                forced_region,
                batch_units,
                reach,
                last_completed_intent,
                &mut compute,
                read_in_the_session_lock,
            );
            // 三段在卡上是怎么交的（几片、重交几次、哪几张卡）随这一批的记账报出来，出错放弃的那一批也报。
            if let CompactionComputeOfTheBatch::OnTheGpu {
                gpu_compute,
                gpu_work,
            } = &mut compute
            {
                if let Ok(mut implementation) = gpu_compute.lock() {
                    gpu_work.submissions_on_the_gpus =
                        implementation.take_the_submissions_of_the_batch();
                }
            }
            proposed
        }
    };
    CompactionBatchComputed {
        batch_under_way,
        version,
        region_check,
        batch_units,
        segments_refused_by_the_region_scan,
        gpu_work: match compute {
            CompactionComputeOfTheBatch::OnTheCpu => None,
            CompactionComputeOfTheBatch::OnTheGpu {
                gpu_compute: _,
                gpu_work,
            } => Some(*gpu_work),
        },
        proposal,
    }
}

/// 在票里那一版上算这一批的提议（[`compute_the_compaction_batch`] 的本体）。活意图读票里那一版的 logged_ops（盘上的权威）。
/// 1. 没有活意图：可写挂载续做停在「没有活意图可续」（只续做、不签新的）；上一条完成的意图全局差值为假就停（攻方 K-h）；选段（强制选的那一段，
///    或 greedy 先扫后选），选不出就停；选中那一段交回去签（签发前判与签发在回锁里、在现行那一版上做）。
/// 2. 有活意图：R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，都交回没有扫描（回锁里丢意图）；
///    对 R 重扫（每批之前都重扫，丁 1 交判第 3 点），照 [`compaction_publish_planned_with_the_records_uncertain_this_round`] 计划头一批。
///
/// 选段那一圈的上界是单元区的整段数；每要扫一段就经 `read_in_the_session_lock` 读一次。区域扫描与选段打分照 `compute` 在哪算：走 GPU 时三段交实现，
/// 判定都确认（或交 CPU 判）时每一步与 CPU 那一条逐项相同；这一轮不确定的条目这一批不回收、不搬（D26（后台整理与放置回收） 已定项 13 第 8 条）——
/// 不确定的段不进选段候选，不确定的记录不当可搬；选中要签的那一段里有不确定的记录就这一批不签、停在 [`CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound`]；
/// 有活意图时 R 里除了不确定的记录再没有可搬的就这一批不发、停在 [`CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove`]。
///
/// # Errors
/// 实现交回出错、实现那把锁中毒 ⇒ [`BackgroundComputeError`]，在任何一次发布之前。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：票里那一版、活意图、读好的 R 的单元、隔离集、不合格段表、开过的段、单元区、强制选的段、批大小、推到哪为止、上一条完成的意图、这一批在哪算、回锁里读盘"
)]
fn propose_the_compaction_batch(
    version: &TransactionOutput,
    live_intent: Option<CompactionIntent>,
    unit_bytes_of_the_live_region: Option<UnitBytesReadBeforeTheRegionScan>,
    quarantined_slots: &BTreeSet<SlotNumber>,
    segments_refused_by_the_region_scan: &mut SegmentsRefusedByTheRegionScan,
    cluster_segments: &BTreeSet<SlotNumber>,
    unit_area: UnitAreaOfOneDevice,
    forced_region: Option<CompactionRegion>,
    batch_units: CompactionBatchUnits,
    reach: CompactionRunReach,
    last_completed_intent: Option<CompactionIntent>,
    compute: &mut CompactionComputeOfTheBatch,
    read_in_the_session_lock: &mut dyn FnMut(
        UnitReadsOfARegionScan,
    ) -> UnitBytesReadBeforeTheRegionScan,
) -> Result<CompactionBatchProposal, BackgroundComputeError> {
    let read_of_the_version = version_as_the_compaction_reads_it(version);
    // 映射条目的按身份索引每读一版建一次（用户 2026-10-26 定「每读一版建一次」）：这一版里扫几个 R（选段时逐段先扫、续做意图时扫 R）都共用它。
    let mapping_index =
        CentralMappingIdentityIndex::of_the_mapping_entries(&read_of_the_version.mapping_entries);
    let scan_with_the_unit_bytes_read =
        |region: CompactionRegion,
         unit_bytes_read: &UnitBytesReadBeforeTheRegionScan,
         compute_of_the_batch: &mut CompactionComputeOfTheBatch| {
            scan_the_region_computing_on(
                region,
                &version.allocation_records,
                &mapping_index,
                &read_of_the_version.pinned_slots,
                quarantined_slots,
                unit_bytes_read,
                compute_of_the_batch,
            )
        };
    let Some(intent) = live_intent else {
        if reach == CompactionRunReach::ContinueTheLiveIntentAtTheMount {
            return Ok(CompactionBatchProposal::Stopped(
                CompactionRunStop::NoLiveIntentLeftToContinue,
            ));
        }
        if let Some(completed) = last_completed_intent {
            let empty_now =
                empty_cluster_segments_by_the_records(&version.allocation_records, unit_area);
            if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {
                return Ok(CompactionBatchProposal::Stopped(
                    CompactionRunStop::NoYieldFromTheLastIntent {
                        empty_cluster_segments_now: empty_now,
                        empty_cluster_segments_at_issue: completed.empty_cluster_segments_at_issue,
                    },
                ));
            }
        }
        // 钉住的槽与常驻隔离集里的槽都让一段没资格：带隔离槽的段签了也立刻要丢（攻方 K-c）。
        let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = read_of_the_version
            .pinned_slots
            .union(quarantined_slots)
            .copied()
            .collect();
        // 选中的那一段的扫描与它这一轮不确定的记录（CPU 那一条恒空）。
        let chosen_scan: Option<(RegionScan, Vec<Placement>)> = match forced_region {
            Some(forced_region) => {
                let is_disqualified = cluster_segments.contains(&forced_region.start_slot())
                    || slots_that_disqualify_a_segment
                        .range(forced_region.start_slot()..forced_region.end_slot())
                        .next()
                        .is_some()
                    || segments_refused_by_the_region_scan.contains(forced_region.start_slot());
                let unit_bytes_read = read_in_the_session_lock(unit_reads_of_the_region_scan(
                    forced_region,
                    &version.allocation_records,
                    &read_of_the_version.pinned_slots,
                    quarantined_slots,
                ));
                let scanned =
                    scan_with_the_unit_bytes_read(forced_region, &unit_bytes_read, &mut *compute)?;
                match (is_disqualified, scanned.scan) {
                    (false, Ok(scan)) if scan.movable_units().next().is_some() => {
                        Some((scan, scanned.records_uncertain_this_round))
                    }
                    (false, Ok(_)) | (true, _) => None,
                    (false, Err(refusal)) => {
                        segments_refused_by_the_region_scan.note_refusal(forced_region, refusal);
                        None
                    }
                }
            }
            None => {
                let mut region_scans: Vec<RegionScan> =
                    region_scans_of_the_segments_without_an_unreleased_record(
                        &version.allocation_records,
                        unit_area,
                    );
                // 扫过的段这一轮不确定的记录（按段起点；CPU 那一条恒空）。
                let mut records_uncertain_in_each_scanned_segment: BTreeMap<
                    SlotNumber,
                    Vec<Placement>,
                > = BTreeMap::new();
                // 丙段：GPU 那一条这一批数一次每段的活槽数（这一圈里记录不变）；CPU 那一条是 `None`，照 `choose_region` 自己数。
                let live_slots_counted_on_the_gpu = live_slots_per_cluster_segment_computing_on(
                    &version.allocation_records,
                    unit_area,
                    compute,
                )?;
                // 迭代上界：每答一次「先扫」下一次就多一段有结论（扫成了进 region_scans，被拒了进不合格段表），不超过单元区的整段数。
                // 每一圈要扫的那一段回会话锁里读（D26（后台整理与放置回收） 已定项 13 第 5 条）。
                loop {
                    let choice = match &live_slots_counted_on_the_gpu {
                        None => choose_region(
                            &version.allocation_records,
                            unit_area,
                            cluster_segments,
                            None,
                            &slots_that_disqualify_a_segment,
                            &region_scans,
                            segments_refused_by_the_region_scan,
                            RegionChoiceCriterion::Greedy,
                        ),
                        Some(live_slots_per_segment) => choose_region_over_the_live_slots(
                            live_slots_per_segment,
                            unit_area,
                            cluster_segments,
                            None,
                            &slots_that_disqualify_a_segment,
                            &region_scans,
                            segments_refused_by_the_region_scan,
                            RegionChoiceCriterion::Greedy,
                        ),
                    };
                    match choice {
                        RegionChoice::Chosen(region) => {
                            break region_scans
                                .into_iter()
                                .find(|scan| scan.region == region)
                                .map(|scan| {
                                    let records_uncertain =
                                        records_uncertain_in_each_scanned_segment
                                            .remove(&region.start_slot())
                                            .unwrap_or_default();
                                    (scan, records_uncertain)
                                });
                        }
                        RegionChoice::ScanBeforeChoosing(region) => {
                            let unit_bytes_read =
                                read_in_the_session_lock(unit_reads_of_the_region_scan(
                                    region,
                                    &version.allocation_records,
                                    &read_of_the_version.pinned_slots,
                                    quarantined_slots,
                                ));
                            let scanned = scan_with_the_unit_bytes_read(
                                region,
                                &unit_bytes_read,
                                &mut *compute,
                            )?;
                            match scanned.scan {
                                Ok(scan) => {
                                    records_uncertain_in_each_scanned_segment.insert(
                                        region.start_slot(),
                                        scanned.records_uncertain_this_round,
                                    );
                                    region_scans.push(scan);
                                }
                                Err(refusal) => {
                                    segments_refused_by_the_region_scan
                                        .note_refusal(region, refusal);
                                }
                            }
                        }
                        RegionChoice::NoEligibleSegment => break None,
                    }
                }
            }
        };
        let Some((scan, records_uncertain_in_the_chosen_region)) = chosen_scan else {
            return Ok(CompactionBatchProposal::Stopped(
                CompactionRunStop::NoEligibleSegment,
            ));
        };
        // GPU 那一条选中的这一段里有这一轮不确定的记录：R 里要搬几个还数不准，签发前那一判（空洞够不够装下 R 里要搬的）判不了，
        // 这一批不签，下一批重扫照判（第 8 条「这一批不回收、不搬」；CPU 那一条恒空）。
        if !records_uncertain_in_the_chosen_region.is_empty() {
            return Ok(CompactionBatchProposal::Stopped(
                CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound {
                    records_uncertain_this_round: u64::try_from(
                        records_uncertain_in_the_chosen_region.len(),
                    )
                    .expect("一段 64 槽"),
                },
            ));
        }
        return Ok(CompactionBatchProposal::Issue {
            scan,
            slots_holding_user_data: read_of_the_version.slots_holding_user_data.clone(),
        });
    };
    let region = intent.region;
    // 对 R 重扫的结果与这一轮不确定的记录（CPU 那一条恒空）。
    let scan: Option<(RegionScan, Vec<Placement>)> =
        if region_holds_a_quarantined_slot(region, quarantined_slots) {
            None
        } else {
            let unit_bytes_read = unit_bytes_of_the_live_region
                .expect("R 里没有隔离槽时取输入那一刻读了 R：两处判的是同一张隔离集");
            let scanned = scan_with_the_unit_bytes_read(region, &unit_bytes_read, &mut *compute)?;
            match scanned.scan {
                Ok(scan)
                    if !scan
                        .allocations
                        .iter()
                        .any(|allocation| allocation.class == AllocationInRegion::Quarantined) =>
                {
                    Some((scan, scanned.records_uncertain_this_round))
                }
                Ok(_scan_with_a_quarantined_slot) => None,
                Err(refusal) => {
                    segments_refused_by_the_region_scan.note_refusal(region, refusal);
                    None
                }
            }
        };
    let Some(plan_computed_out_of_the_lock) =
        compaction_publish_planned_with_the_records_uncertain_this_round(
            scan.as_ref()
                .map(|(scan, records_uncertain)| (scan, records_uncertain.as_slice())),
            batch_units,
            &BTreeSet::new(),
            &version.allocation_records,
            region,
        )
    else {
        return Ok(CompactionBatchProposal::Stopped(
            CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove {
                records_uncertain_this_round: scan.as_ref().map_or(0, |(_, records_uncertain)| {
                    u64::try_from(records_uncertain.len()).expect("一段 64 槽")
                }),
            },
        ));
    };
    Ok(CompactionBatchProposal::LiveIntentBatch {
        intent,
        scan,
        plan_computed_out_of_the_lock,
    })
}

/// 回锁里头几判交回的（[`judge_the_computed_compaction_batch_at_the_relock`]）。封闭集合。
pub(super) enum CompactionBatchAtTheRelock {
    /// 回锁里就判完了：出错那一支、提议本来就是停、冻结着丢掉这一批。装箱：一批推完的结局带着发布被拒的原因与这一批的后台计算记账，比另一个成员大得多。
    AdvancedInTheLock(Box<CompactionPublishAdvanced>),
    /// 要在现行那一版上发：放锁的入口先排空待发，再交 [`publish_the_computed_compaction_batch_on_the_current_version`]。
    ToPublishOnTheCurrentVersion(Box<CompactionBatchToPublish>),
}

/// 要发的那两种提议（[`CompactionBatchProposal`] 去掉停的那一种）。封闭集合。
pub(super) enum CompactionBatchProposalToPublish {
    Issue {
        scan: RegionScan,
        slots_holding_user_data: BTreeSet<SlotNumber>,
    },
    LiveIntentBatch {
        intent: CompactionIntent,
        scan: Option<(RegionScan, Vec<Placement>)>,
        plan_computed_out_of_the_lock: (
            Vec<PlannedRelocation>,
            CompactionIntentStep,
            CompactionPublishKind,
        ),
    },
}

/// 回锁里头几判（D26（后台整理与放置回收） 已定项 13 第 5 条）：先把锁外那一段记下被扫描拒的段并进会话；出错先于一切——带着计算出错回来的，
/// 照锁里出错那一支停下整理、报出这个错，不看记录比对与冻结；它单发的那一次只删意图的发布同样过回退门
/// （[`is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate`]：门关就不删，意图与 R 排他留到门开；门关着、或删意图那一次没发成，会话都记下这条意图，
/// 之后不论整理停没停、走哪条路，任一入口取下一批的输入之前先删它，[`take_the_input_of_one_compaction_batch`]）；
/// 提议本来就是停的照停（那一判读的是票里那一版）；
/// 回锁时冻结着一次发布的（取输入之前就冻着的、锁外那一段期间冻结的都算），这一批丢掉、停在
/// [`CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock`]，不记成整理失败。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、锁外算出的一批、这次挂载的后台计算状态、回锁判不判回退门"
)]
pub(super) fn judge_the_computed_compaction_batch_at_the_relock<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    computed: CompactionBatchComputed,
    background_compute: &mut BackgroundComputeOfThisMount,
    rollback_gate_at_the_relock: RollbackGateAtTheRelock,
) -> CompactionBatchAtTheRelock {
    let CompactionBatchComputed {
        batch_under_way,
        version,
        region_check,
        batch_units,
        segments_refused_by_the_region_scan,
        gpu_work,
        proposal,
    } = computed;
    compaction
        .segments_refused_by_the_region_scan
        .note_the_refusals_of(&segments_refused_by_the_region_scan);
    let proposal = match proposal {
        Ok(proposal) => proposal,
        Err(error) => {
            let task_after_the_error = background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                error.clone(),
            );
            let stop = CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                task_after_the_error,
            };
            let step = if is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate(
                parameters,
                &*devices,
                allocator,
                current,
                &version,
                rollback_gate_at_the_relock,
            ) {
                // 门关着这一次不删：会话记下这条活意图，之后任一入口取下一批的输入之前先删它（[`take_the_input_of_one_compaction_batch`]）。
                note_the_live_intent_of_the_abandoned_batch_still_to_drop(
                    compaction, current, stop,
                );
                Ok(CompactionStep::Stopped(stop))
            } else {
                drop_the_live_intent_on_a_stop(
                    parameters, devices, allocator, compaction, current, instance, stop,
                )
            };
            // 删意图这一次没发成（冻结着被发布路径第一道拒、R 里外都取不到落点、写盘报错冻结）：照门压住那一支记下这条活意图，
            // 之后任一入口取下一批的输入之前同样先删它，不在这条放弃的意图下接着搬（D26（后台整理与放置回收） 已定项 13 第 4 条
            // 「正在算的那一批放弃，意图也删掉」）。写盘报错冻结的那一次重发成了，意图随它删掉，取输入那一刻现行那一版里已不是它，清掉。
            // 门压住那一支交回的是停，不进这一判。
            if step.is_err() {
                note_the_live_intent_of_the_abandoned_batch_still_to_drop(
                    compaction, current, stop,
                );
            }
            return CompactionBatchAtTheRelock::AdvancedInTheLock(Box::new(
                CompactionPublishAdvanced {
                    step,
                    background_compute_of_the_batch: Some(background_compute.end_the_batch(
                        batch_under_way,
                        gpu_work,
                        Some(error),
                    )),
                    txg_the_step_was_judged_on: current.root().checkpoint_txg,
                },
            ));
        }
    };
    let proposal_to_publish = match proposal {
        CompactionBatchProposal::Stopped(stop) => {
            return CompactionBatchAtTheRelock::AdvancedInTheLock(Box::new(
                CompactionPublishAdvanced {
                    step: Ok(CompactionStep::Stopped(stop)),
                    background_compute_of_the_batch: Some(background_compute.end_the_batch(
                        batch_under_way,
                        gpu_work,
                        None,
                    )),
                    txg_the_step_was_judged_on: version.root.checkpoint_txg,
                },
            ));
        }
        CompactionBatchProposal::Issue {
            scan,
            slots_holding_user_data,
        } => CompactionBatchProposalToPublish::Issue {
            scan,
            slots_holding_user_data,
        },
        CompactionBatchProposal::LiveIntentBatch {
            intent,
            scan,
            plan_computed_out_of_the_lock,
        } => CompactionBatchProposalToPublish::LiveIntentBatch {
            intent,
            scan,
            plan_computed_out_of_the_lock,
        },
    };
    if allocator.frozen_publish().is_some() {
        compaction
            .observations
            .batches_dropped_while_a_publish_is_frozen_at_the_relock += 1;
        return CompactionBatchAtTheRelock::AdvancedInTheLock(Box::new(
            CompactionPublishAdvanced {
                step: Ok(CompactionStep::Stopped(
                    CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock,
                )),
                background_compute_of_the_batch: Some(background_compute.end_the_batch(
                    batch_under_way,
                    gpu_work,
                    None,
                )),
                txg_the_step_was_judged_on: current.root().checkpoint_txg,
            },
        ));
    }
    CompactionBatchAtTheRelock::ToPublishOnTheCurrentVersion(Box::new(CompactionBatchToPublish {
        batch_under_way,
        version,
        region_check,
        batch_units,
        gpu_work,
        proposal: proposal_to_publish,
    }))
}

/// 回锁里出错那一支这一次没删掉活意图（被回退门压住，或删意图那一次没发成）：现行那一版里有活意图就记在会话上，连同放弃那一批时这一趟停在的那一条，
/// 之后任一入口取下一批的输入之前先删它（[`take_the_input_of_one_compaction_batch`]）。现行那一版里没有活意图就什么都不记。
fn note_the_live_intent_of_the_abandoned_batch_still_to_drop(
    compaction: &mut CompactionOfThisMount,
    current: &PoolVersion,
    stop: CompactionRunStop,
) {
    if let Some((_issued_txg, intent_still_to_drop)) =
        current.version_of_trees().and_then(|version_now| {
            live_compaction_intent_of_the_entries(&version_now.logged_operations)
        })
    {
        compaction
            .hold_back_the_drop_of_the_intent_of_an_abandoned_batch(intent_still_to_drop, stop);
    }
}

/// 回锁里出错那一支单发删意图之前那一判（D26（后台整理与放置回收） 已定项 13 第 5 条：停着时那一次只删意图的发布同样过回退门，门关就不发，
/// 意图与 R 排他留到门开；破例的条件成立时这一次也照发）：回锁这一刻在现行那一版上判破例（全空聚簇段数低于底额、前台已写不进，
/// [`foreground_cannot_write_under_the_switches`]），成立就交回假、照发，与取输入时定的 `rollback_gate_at_the_relock` 无关；
/// 不成立而这一批判回退门、现行那一版又与票里那一版不同时，在现行那一版上照取输入之前那一判再判一次，门关（或根环读不通）交回真。
/// 现行那一版就是票里那一版时取输入那一刻刚在同一份根环上判过门开，交回假。只读，分配器与盘一样都不动。
fn is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    allocator: &PoolAllocator,
    current: &PoolVersion,
    version_of_the_ticket: &TransactionOutput,
    rollback_gate_at_the_relock: RollbackGateAtTheRelock,
) -> bool {
    if foreground_cannot_write_under_the_switches(allocator) {
        return false;
    }
    match rollback_gate_at_the_relock {
        RollbackGateAtTheRelock::JudgedWhenTheVersionMoved => {
            current.root().checkpoint_txg != version_of_the_ticket.root.checkpoint_txg
                && stop_of_the_rollback_gate(parameters, devices, allocator, current).is_some()
        }
        RollbackGateAtTheRelock::NotJudged => false,
    }
}

/// 回锁里在现行那一版上发这一批（D26（后台整理与放置回收） 已定项 13 第 5 条；放锁的入口在这之前排空了待发）：现行那一版与票里那一版不同、这一批判回退门时
/// 先在现行那一版上再判一次回退门，门关就不发；把锁外那一段期间发布路径隔离的槽并进常驻隔离集；签发那一支在现行那一版上重判资格与签发前那一判、
/// 意图的量取现行那一版的（[`issue_the_chosen_region_at_the_relock`]），有活意图那一支比 R 里的记录、剔掉变了的单元、在现行那一版上发
/// （[`publish_the_live_intent_batch_at_the_relock`]）。三段首尾相接时现行那一版就是票里那一版：不再判门，比出来什么都没变，与锁里一口气算完逐项相同。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、要发的那一批、这次挂载的后台计算状态、回锁判不判回退门"
)]
pub(super) fn publish_the_computed_compaction_batch_on_the_current_version<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    to_publish: CompactionBatchToPublish,
    background_compute: &BackgroundComputeOfThisMount,
    rollback_gate_at_the_relock: RollbackGateAtTheRelock,
) -> CompactionPublishAdvanced {
    let CompactionBatchToPublish {
        batch_under_way,
        version: version_of_the_ticket,
        region_check,
        batch_units,
        gpu_work,
        proposal,
    } = to_publish;
    let current_version =
        Arc::clone(current.shared_version_of_trees().expect(
            "前台的改动、删除续做与整理的发布都交回有树的一版：取输入那一刻现行那一版有树",
        ));
    let has_the_version_moved =
        current_version.root.checkpoint_txg != version_of_the_ticket.root.checkpoint_txg;
    match rollback_gate_at_the_relock {
        RollbackGateAtTheRelock::JudgedWhenTheVersionMoved if has_the_version_moved => {
            if let Some(stop) = stop_of_the_rollback_gate(parameters, &*devices, allocator, current)
            {
                return CompactionPublishAdvanced {
                    step: Ok(CompactionStep::Stopped(stop)),
                    background_compute_of_the_batch: Some(background_compute.end_the_batch(
                        batch_under_way,
                        gpu_work,
                        None,
                    )),
                    txg_the_step_was_judged_on: current_version.root.checkpoint_txg,
                };
            }
        }
        RollbackGateAtTheRelock::JudgedWhenTheVersionMoved | RollbackGateAtTheRelock::NotJudged => {
        }
    }
    compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
    let step = match proposal {
        CompactionBatchProposalToPublish::Issue {
            scan,
            slots_holding_user_data,
        } => issue_the_chosen_region_at_the_relock(
            parameters,
            devices,
            allocator,
            compaction,
            current,
            instance,
            &version_of_the_ticket,
            &current_version,
            scan,
            slots_holding_user_data,
        ),
        CompactionBatchProposalToPublish::LiveIntentBatch {
            intent,
            scan,
            plan_computed_out_of_the_lock,
        } => publish_the_live_intent_batch_at_the_relock(
            parameters,
            devices,
            allocator,
            compaction,
            current,
            instance,
            &version_of_the_ticket,
            &current_version,
            intent,
            scan,
            plan_computed_out_of_the_lock,
            batch_units,
            region_check,
        ),
    };
    CompactionPublishAdvanced {
        step,
        background_compute_of_the_batch: Some(background_compute.end_the_batch(
            batch_under_way,
            gpu_work,
            None,
        )),
        txg_the_step_was_judged_on: current.root().checkpoint_txg,
    }
}

/// 一版里 R 那块盘上起点在 R 里、未释放的分配记录（整条，含隔离记号），按起点槽：回锁里比票里那一版与现行那一版的这一份，比的是集合与整条记录。
fn unreleased_records_starting_in_the_region(
    allocation_records: &[AllocationRecord],
    region: CompactionRegion,
) -> BTreeMap<SlotNumber, AllocationRecord> {
    allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .map(|record| (record.slot, *record))
        .collect()
}

/// 没有活意图那一支回锁里签发（D26（后台整理与放置回收） 已定项 13 第 5 条）：现行那一版变了，就对选中的段在现行那一版上重判资格
/// （选段那几样：这次挂载开过的段、钉住与隔离的槽、扫描拒过的段、R 里还有未释放的记录），没资格这一次不签、停在
/// [`CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock`]（下一批重选）；签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条：R 外非全空段的空洞 ≥ R 里要搬的单元数）
/// 在现行那一版与此刻的分配器上判，R 里的记录没变时要搬的取锁外扫出的能搬的单元数，变了取现行那一版 R 里未释放的记录数（上界：只多拒、不少拒）；
/// 意图里的量取现行那一版的：`c0` 是签发那次发布接着的那一版的 txg，`segs0` 是那一版（签发之前那一版）按记录口径的全空段数。
/// 签发：先在分配器上排他 R（签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布；被空间拒就撤排他、停（意图没签成）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、票里那一版、现行那一版的树、选中那一段的扫描、票里那一版的数据单元槽"
)]
fn issue_the_chosen_region_at_the_relock<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    version_of_the_ticket: &TransactionOutput,
    current_version: &TransactionOutput,
    scan: RegionScan,
    slots_holding_user_data_of_the_ticket: BTreeSet<SlotNumber>,
) -> Result<CompactionStep, RelocationBatchRefused> {
    let region = scan.region;
    let records_of_the_region_at_the_input = unreleased_records_starting_in_the_region(
        &version_of_the_ticket.allocation_records,
        region,
    );
    let records_of_the_region_now =
        unreleased_records_starting_in_the_region(&current_version.allocation_records, region);
    let slots_holding_user_data = if current_version.root.checkpoint_txg
        == version_of_the_ticket.root.checkpoint_txg
    {
        slots_holding_user_data_of_the_ticket
    } else {
        let read_of_the_current_version = version_as_the_compaction_reads_it(current_version);
        let slots_that_disqualify_the_region: BTreeSet<SlotNumber> = read_of_the_current_version
            .pinned_slots
            .union(compaction.quarantined_slots())
            .copied()
            .collect();
        let is_the_region_still_eligible =
            !allocator.cluster_segments().contains(&region.start_slot())
                && slots_that_disqualify_the_region
                    .range(region.start_slot()..region.end_slot())
                    .next()
                    .is_none()
                && !compaction
                    .segments_refused_by_the_region_scan
                    .contains(region.start_slot())
                && !records_of_the_region_now.is_empty();
        if !is_the_region_still_eligible {
            compaction
                .observations
                .issues_given_up_for_a_region_no_longer_eligible_at_the_relock += 1;
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock,
            ));
        }
        read_of_the_current_version.slots_holding_user_data
    };
    let units_to_move = if records_of_the_region_at_the_input == records_of_the_region_now {
        u64::try_from(scan.movable_units().count()).expect("一段 64 槽")
    } else {
        u64::try_from(records_of_the_region_now.len()).expect("一段 64 槽")
    };
    let device_map = allocator
        .devices
        .first()
        .expect("池里至少一块盘（取输入时取单元区判过）");
    let unit_area = UnitAreaOfOneDevice {
        device: device_map.device,
        start: device_map.unit_area_start(),
        slots: device_map.unit_area_slots(),
    };
    let holes = holes_outside_the_region_in_segments_holding_user_data(
        &current_version.allocation_records,
        &slots_holding_user_data,
        unit_area,
        scan.region,
        |slot| device_map.is_free(slot),
    );
    if !holes_admit_the_issue(holes, units_to_move) {
        return Ok(CompactionStep::Stopped(
            CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove {
                holes,
                units_to_move,
            },
        ));
    }
    let issued = CompactionIntent {
        region: scan.region,
        allocation_generation_at_issue: current_version.root.checkpoint_txg,
        empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
            &current_version.allocation_records,
            unit_area,
        ),
    };
    // R 从签发那次发布起就排他：那次发布自己的新落点也不进 R。没发成就撤掉（盘上没有这条意图）。
    allocator.compaction_mut().issue(issued);
    match publish_the_compaction_step_once(
        parameters,
        devices,
        allocator,
        current_version,
        &[],
        CompactionIntentStep::Issue(issued),
        instance,
    ) {
        Ok(published) => {
            compaction.observations.intents_issued += 1;
            *current = PoolVersion::of_a_version_of_trees(published.clone());
            Ok(CompactionStep::Published {
                version: Box::new(published),
                kind: CompactionPublishKind::IntentIssued,
            })
        }
        Err(refused) => {
            allocator.compaction_mut().withdraw_the_live_intent();
            if compaction_publish_refused_short_of_space(&refused) {
                Ok(CompactionStep::Stopped(
                    CompactionRunStop::IssueRefusedShortOfSpace,
                ))
            } else {
                Err(refused)
            }
        }
    }
}

/// 有活意图那一支回锁里发这一批（D26（后台整理与放置回收） 已定项 13 第 5 条）：比票里那一版与现行那一版起点在 R 里的未释放记录——整条记录（含隔离记号）
/// 与集合——变了的（释放了、隔离了、多出来的）那几个槽上的单元从计划里剔掉，其余在现行那一版上发；计划里本来有要搬的而剔完一个不剩，这一批不发、停在
/// [`CompactionRunStop::BatchPrunedToNothingAtTheRelock`]。没有扫描的（R 里有隔离槽、扫描拒了 R、扫出隔离类）照计划丢意图。
/// 计划跳过这次发布本来就重写的节点、补上 R 里真要搬的单元：照抄意图的一批在算好的账上核出本来就重写的那几把 key
/// 就并进跳过的集合、照锁外那一份扫描再计划一次、再剔一次。迭代上界是 R 里能搬的单元数加一：被拒时点名的 key 都在这一次的计划里，
/// 计划不取已跳过的，所以每被拒一次跳过的集合至少多一把 key。整理的发布被空间拒就暂停（[`pause_by_dropping_the_live_intent_or_hand_up`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、票里那一版、现行那一版的树、活意图、锁外那一份扫描、头一批的计划、批大小、删意图那一支的核怎么判"
)]
fn publish_the_live_intent_batch_at_the_relock<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    version_of_the_ticket: &TransactionOutput,
    current_version: &TransactionOutput,
    intent: CompactionIntent,
    scan: Option<(RegionScan, Vec<Placement>)>,
    plan_computed_out_of_the_lock: (
        Vec<PlannedRelocation>,
        CompactionIntentStep,
        CompactionPublishKind,
    ),
    batch_units: CompactionBatchUnits,
    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
) -> Result<CompactionStep, RelocationBatchRefused> {
    let region = intent.region;
    let records_of_the_region_at_the_input = unreleased_records_starting_in_the_region(
        &version_of_the_ticket.allocation_records,
        region,
    );
    let records_of_the_region_now =
        unreleased_records_starting_in_the_region(&current_version.allocation_records, region);
    let slots_whose_record_changed: BTreeSet<SlotNumber> = records_of_the_region_at_the_input
        .keys()
        .chain(records_of_the_region_now.keys())
        .filter(|slot| {
            records_of_the_region_at_the_input.get(slot) != records_of_the_region_now.get(slot)
        })
        .copied()
        .collect();
    let mut plan_not_yet_tried = Some(plan_computed_out_of_the_lock);
    let mut units_this_publish_rewrites_anyway: BTreeSet<Vec<u8>> = BTreeSet::new();
    let (published, kind) = loop {
        let (planned, intent_step, kind) = match plan_not_yet_tried.take() {
            Some(plan) => plan,
            None => {
                let Some(plan) = compaction_publish_planned_with_the_records_uncertain_this_round(
                    scan.as_ref()
                        .map(|(scan, records_uncertain)| (scan, records_uncertain.as_slice())),
                    batch_units,
                    &units_this_publish_rewrites_anyway,
                    &version_of_the_ticket.allocation_records,
                    region,
                ) else {
                    return Ok(CompactionStep::Stopped(
                        CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove {
                            records_uncertain_this_round: scan.as_ref().map_or(
                                0,
                                |(_, records_uncertain)| {
                                    u64::try_from(records_uncertain.len()).expect("一段 64 槽")
                                },
                            ),
                        },
                    ));
                };
                plan
            }
        };
        let units_planned_before_pruning = planned.len();
        let planned: Vec<PlannedRelocation> = planned
            .into_iter()
            .filter(|relocation| !slots_whose_record_changed.contains(&relocation.source.slot))
            .collect();
        let units_pruned =
            u64::try_from(units_planned_before_pruning - planned.len()).expect("一批的单元数");
        compaction.observations.units_pruned_at_the_relock += units_pruned;
        if units_planned_before_pruning > 0 && planned.is_empty() {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::BatchPrunedToNothingAtTheRelock { units_pruned },
            ));
        }
        match publish_the_planned_compaction_step_falling_back_on_the_region_check(
            parameters,
            devices,
            allocator,
            current_version,
            &planned,
            intent_step,
            kind,
            region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
            instance,
        ) {
            Ok(published_and_kind) => break published_and_kind,
            Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {
                mapping_keys,
            }) => {
                let skipped_before = units_this_publish_rewrites_anyway.len();
                units_this_publish_rewrites_anyway.extend(mapping_keys);
                assert!(
                    units_this_publish_rewrites_anyway.len() > skipped_before,
                    "被拒时点名的 key 都在这一次的计划里，计划不取已跳过的：跳过的集合每被拒一次至少多一把 key"
                );
            }
            Err(CompactionStepRefused::Refused(refused)) => {
                return pause_by_dropping_the_live_intent_or_hand_up(
                    parameters, devices, allocator, compaction, current, instance, refused,
                )
            }
        }
    };
    match kind {
        CompactionPublishKind::BatchCompletingTheIntent
        | CompactionPublishKind::IntentCompletedWithoutABatch => {
            allocator.compaction_mut().withdraw_the_live_intent();
            compaction.observations.intents_completed += 1;
            compaction.last_completed_intent = Some(intent);
        }
        CompactionPublishKind::IntentDropped
        | CompactionPublishKind::IntentDroppedWithNothingMovable
        | CompactionPublishKind::IntentDroppedOnAStop { .. } => {
            allocator.compaction_mut().withdraw_the_live_intent();
            compaction.observations.intents_dropped += 1;
        }
        CompactionPublishKind::Batch | CompactionPublishKind::IntentIssued => {}
    }
    *current = PoolVersion::of_a_version_of_trees(published.clone());
    Ok(CompactionStep::Published {
        version: Box::new(published),
        kind,
    })
}

/// 同 [`compaction_publish_planned_for_the_region_scan`]，扫描带着这一轮不确定的记录（GPU 那一条，D26（后台整理与放置回收） 已定项 13 第 8 条）：
/// 没有不确定的记录就是它；有的时候 R 里还留着不确定的未释放记录，这一批只搬判定了可搬的那几个、照抄意图（R 被这一批腾空时照 F2 同一批删意图，
/// 发布路径在算好的账上核），不删意图也不因「一个能搬的都没有」丢意图；判定了可搬的一个都没有时交 `None`：这一批不发。
fn compaction_publish_planned_with_the_records_uncertain_this_round(
    scan: Option<(&RegionScan, &[Placement])>,
    batch_units: CompactionBatchUnits,
    rewritten_anyway: &BTreeSet<Vec<u8>>,
    allocation_records_of_the_version: &[AllocationRecord],
    region: CompactionRegion,
) -> Option<(
    Vec<PlannedRelocation>,
    CompactionIntentStep,
    CompactionPublishKind,
)> {
    match scan {
        Some((scan, records_uncertain)) if !records_uncertain.is_empty() => {
            match plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                scan,
                batch_units,
                rewritten_anyway,
            ) {
                PlannedBatch::Relocate(planned) => {
                    // 照抄意图那一支：R 被这一批腾空时发布路径照 F2 同一批删意图（R 里还留着不确定的记录时腾不空）。
                    let carry_the_intent_unless_the_region_is_emptied =
                        CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion;
                    Some((
                        planned,
                        carry_the_intent_unless_the_region_is_emptied,
                        CompactionPublishKind::Batch,
                    ))
                }
                PlannedBatch::RegionEmptied
                | PlannedBatch::NothingMovableButTheRegionIsNotEmpty => None,
            }
        }
        Some((scan, _no_record_uncertain)) => Some(compaction_publish_planned_for_the_region_scan(
            Some(scan),
            batch_units,
            rewritten_anyway,
            allocation_records_of_the_version,
            region,
        )),
        None => Some(compaction_publish_planned_for_the_region_scan(
            None,
            batch_units,
            rewritten_anyway,
            allocation_records_of_the_version,
            region,
        )),
    }
}

/// 有活意图时这一次整理发布的计划：要搬哪几个、对意图做什么、发成了是哪一种（[`advance_compaction_by_one_publish`]）。`scan` 是对 R 重扫的结果
/// （R 里有隔离槽、区域扫描拒了 R、扫出隔离类时是 `None`），`rewritten_anyway` 是这一次发布上已试算出的、本来就重写而不另搬的映射 key
/// （[`plan_batch_skipping_the_units_this_publish_rewrites_anyway`]）。
fn compaction_publish_planned_for_the_region_scan(
    scan: Option<&RegionScan>,
    batch_units: CompactionBatchUnits,
    rewritten_anyway: &BTreeSet<Vec<u8>>,
    allocation_records_of_the_version: &[AllocationRecord],
    region: CompactionRegion,
) -> (
    Vec<PlannedRelocation>,
    CompactionIntentStep,
    CompactionPublishKind,
) {
    match scan.map(|scan| {
        (
            scan,
            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                scan,
                batch_units,
                rewritten_anyway,
            ),
        )
    }) {
        // R 里有隔离槽、区域扫描拒了 R、扫出隔离类：丢意图（攻方 K-c）。
        None => (
            Vec::new(),
            CompactionIntentStep::Drop,
            CompactionPublishKind::IntentDropped,
        ),
        Some((scan, PlannedBatch::Relocate(planned))) => {
            // 这一批罩住 R 里每一条未释放的记录（搬走，或 R 里只剩这次发布本来就重写的节点）：最后一批，同一次发布删意图
            // （D8（核心索引结构） 已定项 5；判决 m4-segment-code-r1 第三节 F2）。
            if planned.len() == scan.allocations.len() {
                (
                    planned,
                    CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                    CompactionPublishKind::BatchCompletingTheIntent,
                )
            } else {
                // 照抄意图那一支在算好的账上核两样：R 被这一批腾空了（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发
                // （判决 m4-segment-code-r1 第三节 F2）；计划里有本来就重写、不另搬的就跳过它们再计划（判决 m4-segment-code-r2 第三节 H4）。
                (
                    planned,
                    CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
                    CompactionPublishKind::Batch,
                )
            }
        }
        Some((_scan, PlannedBatch::RegionEmptied)) => {
            assert!(
                region_is_empty_by_the_records(allocation_records_of_the_version, region),
                "扫描里一条未释放记录都没有，R 就按记录全空：扫描与停机谓词读的是同一份记录"
            );
            (
                Vec::new(),
                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                CompactionPublishKind::IntentCompletedWithoutABatch,
            )
        }
        Some((_scan, PlannedBatch::NothingMovableButTheRegionIsNotEmpty)) => (
            Vec::new(),
            CompactionIntentStep::Drop,
            CompactionPublishKind::IntentDroppedWithNothingMovable,
        ),
    }
}

/// 发这一次计划好的整理发布，删意图那一支或照抄那一支在算好的账上核不过时换个对意图的做法再算（[`advance_compaction_by_one_publish`]）。
/// 照抄意图那一支经 [`publish_compaction_step_in_the_session`] 发，计划里有这次发布本来就重写的单元就不发、交回
/// [`CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish`]（判决 m4-segment-code-r2 第三节 H4）。发成交回这一版与发成的是哪一种；
/// 别的被拒原样交回。一次调用至多算三次账、发一次。
///
/// 两条「核不过就照抄意图」的退路在产品路径上走得到（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H2）：
/// 计划里的单元在这次发布里都离开 R（搬走，或本来就重写），删意图那一核不过只有一种来历——释放之前读盘核把一份隔离、那条记录留在已分配，
/// 也就是一份副本读与重读都读不出（瞬时读错）；下一遍算账再读时它又读得出，照抄意图那一版里 R 就按记录全空了，下一次发布照「R 已按记录全空」
/// 只删意图。另外两处走得到：只供测试的开关关掉 R 排他、这次发布的新落点落进 R；只供测试的 `region_check` 强制删意图那一核不过。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、计划、对意图做什么、发成了是哪一种、一批腾空了 R 之后删意图那一支的核怎么判、实例代号"
)]
fn publish_the_planned_compaction_step_falling_back_on_the_region_check<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    version: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    kind: CompactionPublishKind,
    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
    instance: InstanceGeneration,
) -> Result<(TransactionOutput, CompactionPublishKind), CompactionStepRefused> {
    let switches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    let first_try = publish_compaction_step_in_the_session(
        &mut writer,
        allocator,
        version,
        planned,
        intent_step,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain,
        instance,
    );
    match first_try {
        Ok(published) => Ok((published, kind)),
        // 最后一批在算好的账上 R 还不按记录全空（一份副本读与重读都读不出而被隔离，那条记录留在已分配；或只供测试的开关关掉排他、
        // 这一批的新落点落进了 R）：这一批不删意图，照搬、照抄意图。
        Err(CompactionStepRefused::Refused(
            RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                ..
            },
        )) if kind == CompactionPublishKind::BatchCompletingTheIntent => {
            match publish_the_compaction_step_once(
                parameters,
                devices,
                allocator,
                version,
                planned,
                CompactionIntentStep::CarryTheIntent,
                instance,
            ) {
                Ok(published) => Ok((published, CompactionPublishKind::Batch)),
                Err(refused) => Err(refused.into()),
            }
        }
        // 这一批在算好的账上腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）：同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
        // 删意图那一支再核 R 按记录全空：两次只差 logged_ops 那一条，取落点不读它，核不过的来历同上一支（瞬时读错、关掉排他），
        // 照这一批不删意图再发（用例经只供测试的 `RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`
        // 或故障注入的瞬时读错强制进入）。
        Err(CompactionStepRefused::Refused(
            RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. },
        )) if kind == CompactionPublishKind::Batch => {
            match publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
                parameters,
                devices,
                allocator,
                version,
                planned,
                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
                instance,
            ) {
                Ok(published) => Ok((published, CompactionPublishKind::BatchCompletingTheIntent)),
                Err(
                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                        ..
                    },
                ) => match publish_the_compaction_step_once(
                    parameters,
                    devices,
                    allocator,
                    version,
                    planned,
                    CompactionIntentStep::CarryTheIntent,
                    instance,
                ) {
                    Ok(published) => Ok((published, CompactionPublishKind::Batch)),
                    Err(refused) => Err(refused.into()),
                },
                Err(refused) => Err(refused.into()),
            }
        }
        Err(refused) => Err(refused),
    }
}

/// 整理的一次发布没发成（活意图在盘上，现行那一版就是没发成的那一次的上一版）：空间不够就暂停——在现行那一版上单发一次丢意图的发布
/// （[`drop_the_live_intent_on_a_stop`]，停在 [`CompactionRunStop::PausedShortOfSpace`]），下一次整理照常从头签发、重扫，不续做这一条。
/// 不推抬 F（判决 G8 g1）。别的拒绝原样交回，什么都不写。
fn pause_by_dropping_the_live_intent_or_hand_up<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    refused: RelocationBatchRefused,
) -> Result<CompactionStep, RelocationBatchRefused> {
    if !compaction_publish_refused_short_of_space(&refused) {
        return Err(refused);
    }
    drop_the_live_intent_on_a_stop(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        CompactionRunStop::PausedShortOfSpace,
    )
}

/// 整理停下时删掉现行那一版里的活意图（被空间拒的暂停、一批出错放弃、停着而不补跑三处共用）：单发一次只删意图的发布
/// （[`CompactionIntentStep::Drop`]，不搬单元），发成之后撤分配器这一侧的活意图连同 R 排他、记一次丢意图，现行那一版换成删意图那一版，
/// 交回那一次发布（种类 [`CompactionPublishKind::IntentDroppedOnAStop`]，这一趟随后停在 `stop`）。分配器这一侧的活意图一直留到删意图那一版落盘，
/// R 排他对别的发布照留；删意图那一次发布自己取落点时放开 R（`transaction::publish_compaction_step_in_the_session`，
/// R 之外没有空槽而 R 里有时照样删得掉）：它落盘的那一版里已没有意图。盘上每一版要么意图活着而 R 里没有分配代 > `c0` 的记录，
/// 要么意图已不在（I-5.6「活意图的 R 内没有分配代 > `c0` 的记录」）。
/// 只删意图、不搬单元的整理发布归删除类：式子那一判不判、落点那一判照判（D28（挂载期承诺量） 已定项 1 的不判名单，
/// `transaction::judge_the_publish_before_any_read_or_write`）。
/// 现行那一版里没有活意图就什么都不写，交回停在 `stop`。删意图那一版发成，会话记着的回锁时被回退门压住的那一条
/// （[`stop_of_the_abandoned_batch_whose_intent_is_still_to_drop`]）一并清掉：现行那一版里至多一条活意图，删的就是它。
///
/// # Errors
/// 删意图那一次没发成（R 里外都取不到落点，或发布路径的错）：原样交回。取落点被拒在任何写之前，盘上逐字节不变，意图留在盘上、R 照旧排他，
/// 下一次整理再走到停下时再删；会话记着的那一条照旧记着（回锁里出错那一支没删成的，调用方把它记在会话上，取下一批的输入之前先删）。
pub(super) fn drop_the_live_intent_on_a_stop<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    stop: CompactionRunStop,
) -> Result<CompactionStep, RelocationBatchRefused> {
    let Some(version_with_the_live_intent) = current
        .version_of_trees()
        .filter(|version| {
            live_compaction_intent_of_the_entries(&version.logged_operations).is_some()
        })
        .cloned()
    else {
        return Ok(CompactionStep::Stopped(stop));
    };
    let published = publish_the_compaction_step_once(
        parameters,
        devices,
        allocator,
        &version_with_the_live_intent,
        &[],
        CompactionIntentStep::Drop,
        instance,
    )?;
    allocator.compaction_mut().withdraw_the_live_intent();
    compaction.observations.intents_dropped += 1;
    compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();
    *current = PoolVersion::of_a_version_of_trees(published.clone());
    Ok(CompactionStep::Published {
        version: Box::new(published),
        kind: CompactionPublishKind::IntentDroppedOnAStop { stop },
    })
}

/// 会话记着的那一条回锁时删意图被回退门压住的活意图（`compaction_plan::CompactionOfThisMount::intent_of_an_abandoned_batch_still_to_drop`）
/// 此刻仍是现行那一版里的活意图：交回放弃那一批时这一趟停在的那一条（删它那一趟照它停）。没记着、或现行那一版里的活意图已不是它
/// （删意图那一次冻结之后重发成了、或现行那一版没有树）交回 `None`。只读。
pub(super) fn stop_of_the_abandoned_batch_whose_intent_is_still_to_drop(
    compaction: &CompactionOfThisMount,
    current: &PoolVersion,
) -> Option<CompactionRunStop> {
    let held_back = compaction.intent_of_an_abandoned_batch_still_to_drop()?;
    let live_intent_now = current.version_of_trees().and_then(|version| {
        live_compaction_intent_of_the_entries(&version.logged_operations)
            .map(|(_issued_txg, intent)| intent)
    });
    (live_intent_now == Some(held_back.intent)).then_some(held_back.stop_of_the_abandoned_batch)
}
