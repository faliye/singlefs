use super::*;

// ───────────────────────────── 每个取样点的池与几何（登记第五节「挂好之后、第一次 run_compaction 之前，装置核三样」） ─────────────────────────────

/// 一块内存盘的整份拷贝（`SparseBlockDevice` 不能克隆，拷它的镜像）。
pub(super) fn copy_of_the_memory_devices(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy =
                SparseBlockDevice::new(device.size_in_bytes(), device.probe_physical_block_size());
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

/// 测量会话装的整理开关：`CompactionSwitches::PRODUCT`，只把批大小换成只供测试的 `ForcedForTests(K)`；几何没造成走后备时另点名 R
/// （登记第五节「真实基线」与「负载」那一段的后备）。
pub(super) fn measured_switches(
    batch_units: u64,
    forced_region: Option<CompactionRegion>,
) -> CompactionSwitches {
    let mut switches = CompactionSwitches::PRODUCT;
    switches.batch_size = CompactionBatchSize::ForcedForTests(
        CompactionBatchUnits::new(usize::try_from(batch_units).expect("批大小装得进 usize"))
            .expect("批大小至少 1"),
    );
    switches.forced_region = forced_region;
    switches
}

/// R 怎么选：产品的 greedy，或几何造不成时登记好的后备（只供测试的强制选段，点名一个 f 段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChoiceOfTheRegion {
    Greedy,
    ForcedFallback(CompactionRegion),
}

impl ChoiceOfTheRegion {
    pub(super) const fn label(self) -> &'static str {
        match self {
            ChoiceOfTheRegion::Greedy => "greedy",
            ChoiceOfTheRegion::ForcedFallback(_) => "forced_fallback",
        }
    }

    pub(super) const fn forced_region(self) -> Option<CompactionRegion> {
        match self {
            ChoiceOfTheRegion::Greedy => None,
            ChoiceOfTheRegion::ForcedFallback(region) => Some(region),
        }
    }
}

/// 在一份拷贝上可写挂载一次、照会话的选段那一圈重放，看 greedy 将选哪一段、它的样子，另列出全部 f 段（活槽 = 64f、全是这次写的数据单元）。
struct GreedyPrediction {
    greedy_region: Option<CompactionRegion>,
    greedy_composition: Option<SegmentComposition>,
    geometry_formed_segments: Vec<CompactionRegion>,
    mapped_units: usize,
    /// 单元区里每一段（盘 0 上有未释放记录的段）的成分，按段起点升序（第六节 N11）。
    segment_compositions: Vec<SegmentComposition>,
}

/// 盘 0 单元区里有未释放记录的每一段的成分（N11），取法同几何核的 [`composition_of_the_segment`]。
fn compositions_of_every_segment_holding_an_unreleased_record(
    records: &[AllocationRecord],
    unit_area_start_slot: u64,
    live_data_inode_by_slot: &BTreeMap<u64, u64>,
) -> Vec<SegmentComposition> {
    let segment_starts: BTreeSet<u64> = records
        .iter()
        .filter(|record| {
            record.device == DeviceIdentity(0)
                && !record.is_released
                && record.slot.0 >= unit_area_start_slot
        })
        .map(|record| segment_start_of(record.slot.0, unit_area_start_slot))
        .collect();
    segment_starts
        .into_iter()
        .map(|start| {
            let region =
                CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start))
                    .expect("段起点在聚簇段边界上、装得进 6 字节");
            composition_of_the_segment(records, region, live_data_inode_by_slot)
        })
        .collect()
}

/// N11 的一行：`name=segment_composition`。
fn segment_composition_line(
    point: SamplingPoint,
    overwrite_order: OverwriteOrder,
    workload_changes: usize,
    composition: SegmentComposition,
) -> String {
    format!(
        "name=segment_composition {} overwrite_order={} workload_changes={workload_changes} region_start_slot={} live_slots={} slots_of_this_workload={} other_slots={} formed={}",
        point.label(),
        overwrite_order.label(),
        composition.start_slot,
        composition.live_slots,
        composition.slots_of_this_workload,
        composition.other_slots,
        is_the_geometry_formed(composition, point.fraction),
    )
}

/// 一次造池之后的 N11、N12 几行：逐段成分、成分汇总（`formed_segments` 应等于同一次 `name=geometry_attempt` 的
/// `geometry_formed_segments`，第七节「两条路数同一个东西」）、覆盖写副本的落点（没数的写 `status=not_run`）。
/// 几行都带 `workload_changes`，与同一次的 `name=geometry_attempt` 对得上（一个点改过负载时有几次造池）。
fn construction_reading_lines(
    recipe: &PoolRecipe,
    pool: &ConstructedPool,
    prediction: &GreedyPrediction,
    workload_changes: usize,
) -> Vec<String> {
    let point = recipe.point;
    let order_label = recipe.overwrite_order.label();
    let mut lines: Vec<String> = prediction
        .segment_compositions
        .iter()
        .map(|composition| {
            segment_composition_line(
                point,
                recipe.overwrite_order,
                workload_changes,
                *composition,
            )
        })
        .collect();
    let formed_segments = prediction
        .segment_compositions
        .iter()
        .filter(|composition| is_the_geometry_formed(**composition, point.fraction))
        .count();
    lines.push(format!(
        "name=segment_composition_summary {} overwrite_order={order_label} workload_changes={workload_changes} segments={} formed_segments={formed_segments} geometry_formed_segments_of_the_attempt={} counts_agree={}",
        point.label(),
        prediction.segment_compositions.len(),
        prediction.geometry_formed_segments.len(),
        formed_segments == prediction.geometry_formed_segments.len(),
    ));
    lines.push(match pool.overwrite_landing {
        Some(landing) => format!(
            "name=overwrite_landing {} overwrite_order={order_label} workload_changes={workload_changes} status=counted overwritten_units={} copies_landing_in_holes_of_this_construction={} copies_landing_elsewhere={} predicted_copies_landing_in_holes={} root_ring_entries={LOCAL_ROOT_RING_ENTRIES}",
            point.label(),
            pool.overwritten_units,
            landing.copies_landing_in_holes_of_this_construction,
            landing.copies_landing_elsewhere,
            predicted_copies_landing_in_holes(pool.overwritten_units),
        ),
        None => format!(
            "name=overwrite_landing {} overwrite_order={order_label} workload_changes={workload_changes} status=not_run overwritten_units={}",
            point.label(),
            pool.overwritten_units,
        ),
    });
    lines
}

fn predict_the_greedy_choice(
    pool: &ConstructedPool,
    fraction: LiveSlotFraction,
) -> GreedyPrediction {
    let mut devices = copy_of_the_memory_devices(&pool.devices);
    let mounted = mount_writable_with_compaction_switches(
        &experiment_pool::parameters(),
        &mut devices,
        measured_switches(BATCH_SIZES[0], None),
    )
    .expect("几何核那一次可写挂载");
    let (_mount_output, session) = MountedSession::of_the_mount(mounted);
    let version = version_as_the_replay_reads_it(version_of_trees(&session));
    let state = session_state_for_the_choice(&session);
    let replay = replay_the_greedy_choice(&version, &state, |device, slot, length| {
        PoolReader::read(devices.as_slice(), device, slot.to_device_offset(), length)
    });
    let segment_starts: BTreeSet<u64> = pool
        .live_data_inode_by_slot
        .keys()
        .map(|slot| segment_start_of(*slot, pool.unit_area_start_slot))
        .collect();
    let geometry_formed_segments = segment_starts
        .into_iter()
        .map(|start| {
            CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start))
                .expect("段起点在聚簇段边界上、装得进 6 字节")
        })
        .filter(|region| {
            is_the_geometry_formed(
                composition_of_the_segment(
                    &version.allocation_records,
                    *region,
                    &pool.live_data_inode_by_slot,
                ),
                fraction,
            )
        })
        .collect();
    GreedyPrediction {
        greedy_region: replay.chosen,
        greedy_composition: replay.chosen.map(|region| {
            composition_of_the_segment(
                &version.allocation_records,
                region,
                &pool.live_data_inode_by_slot,
            )
        }),
        geometry_formed_segments,
        mapped_units: version_of_trees(&session).mapped_units.len(),
        segment_compositions: compositions_of_every_segment_holding_an_unreleased_record(
            &version.allocation_records,
            pool.unit_area_start_slot,
            &pool.live_data_inode_by_slot,
        ),
    }
}

/// 一个取样点准备好的样子：最后用的配方、R 怎么选、几何造没造成（没造成的点不进判定）、第一次造的池（给最先那一轮用）与它的分块摘要，
/// 以及准备时要打的产物行（准备在别的线程上跑，行先攒着，轮到这个点量的时候按原来的次序打出去，登记第十二节 12.10）。
pub(super) struct PreparedPoint {
    pub(super) recipe: PoolRecipe,
    pub(super) choice: ChoiceOfTheRegion,
    pub(super) is_the_geometry_formed: bool,
    pub(super) pool: ConstructedPool,
    pub(super) reference_digests: Vec<ChunkDigest>,
    pub(super) lines: Vec<String>,
}

/// 造池并核几何：greedy 将选的段是 f 段就用 greedy；不是而那一段全是这次写的数据单元，就把它们并进配方多覆盖写掉重造，至多改三次；
/// 那一段装着别的单元（节点、打包单元）时没有登记好的负载改法，直接走后备（点名槽号最小的 f 段）。一个 f 段都没有，几何没造成。
/// 产物行不直接打，攒进交回的 [`PreparedPoint::lines`]：同一个点的几次重造前后依赖，在一条线程上按次序跑；不同的点互不依赖，由
/// [`PreparationQueue`] 与别的准备单位一起分到几条线程上。
pub(super) fn prepare_the_sampling_point(point: SamplingPoint) -> PreparedPoint {
    prepare_the_sampling_point_with_the_order(point, OverwriteOrder::HighestFullSegmentLast)
}

/// 同 [`prepare_the_sampling_point`]，覆盖写的次序由调用方给（`geometry-check --overwrite-order slot-order` 的对照用 [`OverwriteOrder::SlotOrder`]）。
/// 每次造池（`name=geometry_attempt` 那几次）之后另攒 N11、N12 几行（[`construction_reading_lines`]）；改法甲数副本落点，`slot-order` 不数。
pub(super) fn prepare_the_sampling_point_with_the_order(
    point: SamplingPoint,
    overwrite_order: OverwriteOrder,
) -> PreparedPoint {
    let mut lines = Vec::new();
    let mut recipe = PoolRecipe {
        point,
        overwrite_order,
        extra_overwritten_inodes: BTreeSet::new(),
    };
    let landing_count = match overwrite_order {
        OverwriteOrder::HighestFullSegmentLast => LandingCount::Counted,
        OverwriteOrder::SlotOrder => LandingCount::NotCounted,
    };
    let mut workload_changes = 0_usize;
    loop {
        let pool = construct_the_pool(&recipe, landing_count);
        let prediction = predict_the_greedy_choice(&pool, point.fraction);
        let greedy = prediction.greedy_composition;
        let is_greedy_formed =
            greedy.is_some_and(|composition| is_the_geometry_formed(composition, point.fraction));
        lines.push(format!(
            "name=geometry_attempt {} workload_changes={workload_changes} construction_seconds={:.3} overwritten_units={} filler_units={} floor_raises_pushed_by_the_session={} release_rounds={} every_overwritten_slot_released={} mapped_units={} geometry_formed_segments={} greedy_region_start_slot={} greedy_live_slots={} greedy_slots_of_this_workload={} greedy_other_slots={} greedy_formed={is_greedy_formed}",
            point.label(),
            pool.construction_seconds,
            pool.overwritten_units,
            pool.filler_units,
            pool.floor_raises_pushed_by_the_session,
            pool.release_rounds,
            pool.is_every_overwritten_slot_released,
            prediction.mapped_units,
            prediction.geometry_formed_segments.len(),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.start_slot.to_string()),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.live_slots.to_string()),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.slots_of_this_workload.to_string()),
            greedy.map_or_else(|| "none".to_string(), |composition| composition.other_slots.to_string()),
        ));
        lines.extend(construction_reading_lines(
            &recipe,
            &pool,
            &prediction,
            workload_changes,
        ));
        if is_greedy_formed {
            lines.push(format!(
                "name=geometry {} choice=greedy region_start_slot={} formed=true workload_changes={workload_changes}",
                point.label(),
                prediction
                    .greedy_region
                    .map_or_else(|| "none".to_string(), |region| region.start_slot().0.to_string()),
            ));
            let reference_digests = digests_of_the_pool(&pool);
            return PreparedPoint {
                recipe,
                choice: ChoiceOfTheRegion::Greedy,
                is_the_geometry_formed: true,
                pool,
                reference_digests,
                lines,
            };
        }
        let competitor_inodes: Vec<u64> = match (prediction.greedy_region, greedy) {
            (Some(region), Some(composition)) if composition.other_slots == 0 => pool
                .live_data_inode_by_slot
                .iter()
                .filter(|(slot, _inode)| region.contains_slot(SlotNumber(**slot)))
                .map(|(_slot, inode)| *inode)
                .collect(),
            (Some(_), Some(_)) | (None, _) | (Some(_), None) => Vec::new(),
        };
        if workload_changes < WORKLOAD_CHANGES_AT_MOST && !competitor_inodes.is_empty() {
            workload_changes += 1;
            recipe.extra_overwritten_inodes.extend(competitor_inodes);
            continue;
        }
        let fallback = prediction.geometry_formed_segments.first().copied();
        lines.push(format!(
            "name=geometry {} choice={} fallback_region_start_slot={} formed={} reason={}",
            point.label(),
            if fallback.is_some() {
                "forced_fallback"
            } else {
                "none"
            },
            fallback.map_or_else(
                || "none".to_string(),
                |region| region.start_slot().0.to_string()
            ),
            fallback.is_some(),
            if competitor_inodes.is_empty() {
                "greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change"
            } else {
                "workload_changed_three_times"
            },
        ));
        let reference_digests = digests_of_the_pool(&pool);
        return PreparedPoint {
            recipe,
            choice: fallback.map_or(ChoiceOfTheRegion::Greedy, ChoiceOfTheRegion::ForcedFallback),
            is_the_geometry_formed: fallback.is_some(),
            pool,
            reference_digests,
            lines,
        };
    }
}

#[cfg(test)]
mod tests;
