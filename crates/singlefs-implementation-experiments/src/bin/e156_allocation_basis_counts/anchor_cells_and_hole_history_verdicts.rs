//! 判定（登记第六节 6.2、第八节 8.2）：静态锚点、锚点格、洞历史的判定与全零钉的格。

use super::*;

// ============================================================================================
// main
// ============================================================================================

pub(super) fn emit_static_anchors(emitter: &mut Emitter) {
    let default_parameters = parameters();
    // 装置的本地 S 与实现侧回比（入库装置第 ① 条）：mkfs 默认、上下界、区域设备。
    assert_eq!(
        E156_SLOTS_PER_REGION,
        default_parameters
            .geometry
            .root_ring_slots_per_region
            .count(),
        "装置写死的 mkfs 默认每区槽数 S 与实现侧不等"
    );
    assert_eq!(
        (E156_SLOTS_PER_REGION_MINIMUM, E156_SLOTS_PER_REGION_MAXIMUM),
        (
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM
        ),
        "装置写死的 S 上下界与实现侧不等"
    );
    assert_eq!(
        E156_REGION_DEVICE_NUMBERS.map(DeviceIdentity),
        default_parameters.region_devices,
        "装置写死的区域设备与 mkfs 参数不等"
    );
    assert_eq!(
        E156_ROOT_RING_REGIONS, ROOT_RING_REGIONS,
        "装置写死的区域数 R 与实现侧不等"
    );
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        assert!(
            RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region).is_ok(),
            "K8：S = {slots_per_region} 应被接受"
        );
    }
    assert_eq!(
        (
            E156_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
            E156_SYSTEM_CONFIGURATION_SLOT_BYTES
        ),
        (
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
            singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES
        ),
        "装置写死的系统配置槽数与槽宽与实现侧不等"
    );
    // A1：单元区起点与实现侧同名常量回比（入库装置第 ① 条）。
    assert_eq!(
        E156_UNIT_AREA_START_SLOT, UNIT_AREA_START_SLOT,
        "A1 的单元区起点常量与实现侧不等"
    );
    let unit_area_slot_count = IMAGE_BYTES / SLOT_BYTES - E156_UNIT_AREA_START_SLOT;
    emitter.emit(&format!(
        "name=anchor_a1 unit_area_slots={unit_area_slot_count}"
    ));
    assert_eq!(
        unit_area_slot_count, 211968,
        "A1：4 GiB 设备的单元区槽数应为 211968"
    );
    let allocation_record_node_capacity =
        (16384 - E156_ALLOCATION_RECORD_NODE_HEADER_BYTES) / E156_ALLOCATION_RECORD_BYTES;
    emitter.emit(&format!(
        "name=anchor_a7 capacity={allocation_record_node_capacity}"
    ));
    assert_eq!(
        allocation_record_node_capacity, 812,
        "A7：分配记录节点容量应为 812"
    );
    let leaf_slots_match_format_crate = E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
        == singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS;
    let internal_fanout_match_format_crate = E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
        == singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
    let my_root_level = e156_root_level_for_symmetric_devices(unit_area_slot_count, 2);
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&default_parameters, &mut devices).expect("A-D8 mkfs");
    let allocator = allocator_after_make_filesystem(&default_parameters, &devices, &genesis);
    let real_root_level =
        singlefs_core::allocation_record_tree::AllocationRecordTreeGeometry::of_allocator(
            &allocator,
        )
        .root_level();
    emitter.emit(&format!(
        "name=anchor_a_d8 local_leaf_slots={E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS} local_internal_fanout={E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} local_leaf_slots_matches_format_crate={leaf_slots_match_format_crate} local_internal_fanout_matches_format_crate={internal_fanout_match_format_crate} my_root_level={my_root_level} real_root_level={real_root_level} root_level_matches={}",
        my_root_level == real_root_level
    ));
    // A2：根环 3S 槽、txg 落的槽与 u mod 3S 一一对应（每个取样点的 S 各核一次，用真实几何函数）。
    for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
        let geometry = RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
            .expect("S 落在区间里");
        let ring_length = E156_ROOT_RING_REGIONS * slots_per_region;
        let mut distinct_slots: BTreeSet<RootRingSlot> = BTreeSet::new();
        let mut mismatches = 0u64;
        for probe_txg in 0..(ring_length * 4) {
            let target = target_for_publish(CheckpointTxg(probe_txg), geometry);
            if probe_txg < ring_length {
                distinct_slots.insert(target);
            }
            let wrapped = target_for_publish(CheckpointTxg(probe_txg % ring_length), geometry);
            if target != wrapped {
                mismatches += 1;
            }
        }
        let holds =
            distinct_slots.len() == usize::try_from(ring_length).expect("环长") && mismatches == 0;
        emitter.emit(&format!(
            "name=anchor_a2 s={slots_per_region} ring_slots={ring_length} distinct_slots_in_one_ring={} wrap_mismatches={mismatches} holds={holds}",
            distinct_slots.len()
        ));
        assert!(
            holds,
            "A2：S = {slots_per_region} 时 txg 的槽与 u mod 3S 应一一对应"
        );
    }
}

/// 一个 S 下锚点模型的全部格：k = 0 与 (k, 位置) ∈ {1, 2, 4} × {前, 后}。
pub(super) struct AnchorCellsOfOneSlotsPerRegion {
    pub(super) without_holes: anchor_model::AnchorCell,
    pub(super) cells: Vec<(u64, HolePosition, anchor_model::AnchorCell)>,
}

pub(super) fn anchor_cells_of(slots_per_region: u64) -> AnchorCellsOfOneSlotsPerRegion {
    let without_holes_end = anchor_model::without_holes_end(slots_per_region);
    let mut cells = Vec::new();
    for position in [HolePosition::Front, HolePosition::Back] {
        for hole_count in [1u64, 2, 4] {
            cells.push((
                hole_count,
                position,
                anchor_model::run_the_anchor_hole_script(
                    slots_per_region,
                    hole_count,
                    position == HolePosition::Front,
                    None,
                ),
            ));
        }
    }
    AnchorCellsOfOneSlotsPerRegion {
        without_holes: anchor_model::run_the_anchor_hole_script(
            slots_per_region,
            0,
            false,
            Some(without_holes_end),
        ),
        cells,
    }
}

pub(super) fn anchor_cell_for(
    anchors: &BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion>,
    slots_per_region: u64,
    hole_count: u64,
    position: Option<HolePosition>,
) -> &anchor_model::AnchorCell {
    let of_slots = &anchors[&slots_per_region];
    match position {
        None => &of_slots.without_holes,
        Some(position) => of_slots
            .cells
            .iter()
            .find(|(count, cell_position, _)| *count == hole_count && *cell_position == position)
            .map(|(_, _, cell)| cell)
            .expect("锚点模型有这一格"),
    }
}

/// 一格的 A10–A16、K8、S1 与 V 系列核对的汇总（第二段的 V / S / F 判定从这里取）。
#[derive(Default, Clone, Copy)]
pub(super) struct HoleHistoryVerdictTotals {
    pub(super) hole_count_anchor_mismatches: u64,
    pub(super) threshold_anchor_mismatches: u64,
    pub(super) every_publish_below_tree_table_bound: u64,
    pub(super) real_below_tree_table_bound: u64,
    pub(super) every_publish_nonzero_without_holes: u64,
    pub(super) real_nonzero_without_holes: u64,
    pub(super) interval_rule_every_publish_nonzero: u64,
    interval_rule_real_nonzero: u64,
    pub(super) reclaim_rule_violation_points: u64,
    pub(super) devices_unequal: u64,
    pub(super) root_ring_missing: u64,
    pub(super) anchor_rows_missing: u64,
    pub(super) real_every_differ: u64,
    pub(super) q1a_plus_positive: u64,
    pub(super) device_and_crates_disagreements: u64,
    pub(super) admission_touches: u64,
    pub(super) abandoned_or_unknown: u64,
    pub(super) slots_per_region_read_back_mismatches: u64,
    pub(super) tree_table_lag_every_publish_mismatch: u64,
    pub(super) tree_table_lag_real_mismatch: u64,
    /// 第 7 次重跑登记 A23、A24：门槛滞后、被钉住的槽数与锚点模型不同的观测点数。
    pub(super) lag_anchor_mismatches: u64,
    pub(super) pinned_anchor_mismatches: u64,
}

impl HoleHistoryVerdictTotals {
    pub(super) fn add(&mut self, other: HoleHistoryVerdictTotals) {
        self.hole_count_anchor_mismatches += other.hole_count_anchor_mismatches;
        self.threshold_anchor_mismatches += other.threshold_anchor_mismatches;
        self.every_publish_below_tree_table_bound += other.every_publish_below_tree_table_bound;
        self.real_below_tree_table_bound += other.real_below_tree_table_bound;
        self.every_publish_nonzero_without_holes += other.every_publish_nonzero_without_holes;
        self.real_nonzero_without_holes += other.real_nonzero_without_holes;
        self.interval_rule_every_publish_nonzero += other.interval_rule_every_publish_nonzero;
        self.interval_rule_real_nonzero += other.interval_rule_real_nonzero;
        self.reclaim_rule_violation_points += other.reclaim_rule_violation_points;
        self.devices_unequal += other.devices_unequal;
        self.root_ring_missing += other.root_ring_missing;
        self.anchor_rows_missing += other.anchor_rows_missing;
        self.real_every_differ += other.real_every_differ;
        self.q1a_plus_positive += other.q1a_plus_positive;
        self.device_and_crates_disagreements += other.device_and_crates_disagreements;
        self.admission_touches += other.admission_touches;
        self.abandoned_or_unknown += other.abandoned_or_unknown;
        self.slots_per_region_read_back_mismatches += other.slots_per_region_read_back_mismatches;
        self.tree_table_lag_every_publish_mismatch += other.tree_table_lag_every_publish_mismatch;
        self.tree_table_lag_real_mismatch += other.tree_table_lag_real_mismatch;
        self.lag_anchor_mismatches += other.lag_anchor_mismatches;
        self.pinned_anchor_mismatches += other.pinned_anchor_mismatches;
    }
}

fn count_where(observations: &[HhObservation], predicate: impl Fn(&HhObservation) -> bool) -> u64 {
    u64::try_from(
        observations
            .iter()
            .filter(|observation| predicate(observation))
            .count(),
    )
    .expect("落在 u64 内")
}

pub(super) fn judge_hole_history_outcome(
    outcome: &HhRunOutcome,
    expected_tree_table_lags: Option<&BTreeSet<u64>>,
) -> HoleHistoryVerdictTotals {
    let observations = &outcome.observations;
    let has_no_holes = outcome.shape.hole_count == 0;
    let floor_rule =
        |observation: &HhObservation, timing| observation.over_withheld(Arm::FloorRule, timing);
    let interval_rule = |observation: &HhObservation, timing| {
        observation.over_withheld(Arm::LifetimeIntervalRule, timing)
    };
    HoleHistoryVerdictTotals {
        hole_count_anchor_mismatches: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                row.missing_txgs != observation.reading.missing_txgs
                    || row.live_holes != observation.reading.live_holes
            })
        }),
        threshold_anchor_mismatches: count_where(observations, |observation| {
            observation
                .anchor
                .is_some_and(|row| row.floor != observation.reading.ring_threshold)
        }),
        every_publish_below_tree_table_bound: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                floor_rule(observation, ReclaimTiming::EveryPublish) < row.tree_table_lower_bound
            })
        }),
        real_below_tree_table_bound: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                floor_rule(observation, ReclaimTiming::Real) < row.tree_table_lower_bound
            })
        }),
        every_publish_nonzero_without_holes: if has_no_holes {
            count_where(observations, |observation| {
                floor_rule(observation, ReclaimTiming::EveryPublish) != 0
                    || interval_rule(observation, ReclaimTiming::EveryPublish) != 0
            })
        } else {
            0
        },
        real_nonzero_without_holes: if has_no_holes {
            count_where(observations, |observation| {
                floor_rule(observation, ReclaimTiming::Real) != 0
                    || interval_rule(observation, ReclaimTiming::Real) != 0
            })
        } else {
            0
        },
        interval_rule_every_publish_nonzero: count_where(observations, |observation| {
            interval_rule(observation, ReclaimTiming::EveryPublish) != 0
        }),
        interval_rule_real_nonzero: count_where(observations, |observation| {
            interval_rule(observation, ReclaimTiming::Real) != 0
        }),
        reclaim_rule_violation_points: count_where(observations, |observation| {
            observation.reading.device_zero.reclaim_rule_violations > 0
        }),
        devices_unequal: count_where(observations, |observation| {
            !observation.reading.devices_equal
        }),
        root_ring_missing: count_where(observations, |observation| {
            !observation.reading.root_ring_installed
        }),
        anchor_rows_missing: count_where(observations, |observation| observation.anchor.is_none()),
        real_every_differ: count_where(observations, |observation| {
            floor_rule(observation, ReclaimTiming::Real)
                != floor_rule(observation, ReclaimTiming::EveryPublish)
                || interval_rule(observation, ReclaimTiming::Real)
                    != interval_rule(observation, ReclaimTiming::EveryPublish)
        }),
        q1a_plus_positive: count_where(observations, |observation| {
            let device_zero = &observation.reading.device_zero;
            device_zero.floor_rule_real_referenced_but_free > 0
                || device_zero.floor_rule_every_publish_referenced_but_free > 0
                || device_zero.interval_rule_referenced_but_free > 0
        }),
        device_and_crates_disagreements: u64::from(!outcome.genesis_root_in_ring)
            + u64::from(!outcome.new_pool_file_creation_matches_the_registered_anchor)
            + outcome.ring_wrap_overwrite_mismatches
            + outcome.mount_schedule_mismatches
            + outcome.disk_cross_check_mismatches
            + outcome.tree_table_release_violations
            + u64::try_from(
                outcome
                    .recoveries
                    .iter()
                    .filter(|recovery| {
                        outcome.cut_point == CrashCutPoint::BeforeTheRootSlotWrite
                            && (!recovery.cut_matches_the_restored_image
                                || recovery.root_slot_writes_in_the_crashed_publish != 1)
                    })
                    .count(),
            )
            .expect("落在 u64 内"),
        admission_touches: outcome.admission_refusals
            + u64::try_from(
                outcome
                    .recoveries
                    .iter()
                    .filter(|recovery| {
                        recovery.mount_admission != "admitted_before_acquisition"
                            || recovery.mount_floor_raise_sequences > 0
                    })
                    .count(),
            )
            .expect("落在 u64 内")
            + count_where(observations, |observation| {
                observation.reading.effective_floor > 0
            }),
        abandoned_or_unknown: count_where(observations, |observation| {
            observation.reading.abandoned_roots > 0
                || observation.reading.roots_without_version_facts > 0
        }),
        slots_per_region_read_back_mismatches: u64::from(
            outcome.slots_per_region_read_back != outcome.shape.slots_per_region,
        ),
        tree_table_lag_every_publish_mismatch: u64::from(
            expected_tree_table_lags
                .is_some_and(|lags| *lags != outcome.tree_table_lags_every_publish),
        ),
        tree_table_lag_real_mismatch: u64::from(
            expected_tree_table_lags.is_some_and(|lags| *lags != outcome.tree_table_lags_real),
        ),
        lag_anchor_mismatches: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                row.lag != observation.reading.newest_root_txg - observation.reading.ring_threshold
            })
        }),
        pinned_anchor_mismatches: count_where(observations, |observation| {
            observation.anchor.is_some_and(|row| {
                row.pinned_slots != observation.reading.all_zero_slots_still_pinned
            })
        }),
    }
}

pub(super) fn emit_cell_checks(
    emitter: &mut Emitter,
    outcome: &HhRunOutcome,
    totals: &HoleHistoryVerdictTotals,
) {
    emitter.emit(&format!(
        "name=q1_cell_checks family={} {} a11_mismatches={} a14_mismatches={} a15_every_below={} a15_real_below={} a12_every_nonzero={} a12_real_nonzero={} a13_every_nonzero={} a13_real_nonzero={} k10_violation_points={} devices_unequal_points={} root_ring_missing_points={} anchor_rows_missing={} real_every_differ_points={} referenced_but_free_points={} s1_failures={} g_adm_nonzero={} abandoned_or_unknown_root_points={} k8_mismatches={} a10_every_mismatch={} a10_real_mismatch={} anchor_lag_mismatches={} anchor_pinned_slot_mismatches={}",
        outcome.family,
        outcome.shape.fields(),
        totals.hole_count_anchor_mismatches,
        totals.threshold_anchor_mismatches,
        totals.every_publish_below_tree_table_bound,
        totals.real_below_tree_table_bound,
        totals.every_publish_nonzero_without_holes,
        totals.real_nonzero_without_holes,
        totals.interval_rule_every_publish_nonzero,
        totals.interval_rule_real_nonzero,
        totals.reclaim_rule_violation_points,
        totals.devices_unequal,
        totals.root_ring_missing,
        totals.anchor_rows_missing,
        totals.real_every_differ,
        totals.q1a_plus_positive,
        totals.device_and_crates_disagreements,
        totals.admission_touches,
        totals.abandoned_or_unknown,
        totals.slots_per_region_read_back_mismatches,
        totals.tree_table_lag_every_publish_mismatch,
        totals.tree_table_lag_real_mismatch,
        totals.lag_anchor_mismatches,
        totals.pinned_anchor_mismatches,
    ));
}

/// 一格的定乙钉住簿记（第 6 次重跑登记第五节 E10、第六节 R1d）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AllZeroPinCellTotals {
    mounts: u64,
    pub(super) pins_installed: u64,
    pinned_points: u64,
    pinned_slots_peak: u64,
    pinned_slots_end: u64,
    pub(super) threshold_lowered_points: u64,
    pub(super) released_by_write: u64,
    pub(super) released_by_reread_only: u64,
    pub(super) bad_slots_at_mount: u64,
    pub(super) bad_slots_still_pinned: u64,
}

fn all_zero_pin_cell_totals(outcome: &HhRunOutcome) -> AllZeroPinCellTotals {
    let observations = &outcome.observations;
    AllZeroPinCellTotals {
        mounts: u64::try_from(outcome.all_zero_pin_mounts.len()).expect("落在 u64 内"),
        pins_installed: outcome
            .all_zero_pin_mounts
            .iter()
            .map(|mount| u64::try_from(mount.pinned.len()).expect("落在 u64 内"))
            .sum(),
        pinned_points: count_where(observations, |observation| {
            observation.reading.all_zero_slots_still_pinned > 0
        }),
        pinned_slots_peak: observations
            .iter()
            .map(|observation| observation.reading.all_zero_slots_still_pinned)
            .max()
            .unwrap_or(0),
        pinned_slots_end: observations.last().map_or(0, |observation| {
            observation.reading.all_zero_slots_still_pinned
        }),
        threshold_lowered_points: count_where(observations, |observation| {
            observation.reading.threshold_lowered_by_all_zero_pins
        }),
        released_by_write: outcome.all_zero_pins_released_by_write,
        released_by_reread_only: outcome.all_zero_pins_released_by_reread_only,
        bad_slots_at_mount: outcome
            .all_zero_pin_mounts
            .iter()
            .map(|mount| mount.bad_slots_at_mount)
            .sum(),
        bad_slots_still_pinned: observations
            .iter()
            .map(|observation| observation.reading.bad_slots_still_pinned)
            .sum(),
    }
}

pub(super) fn emit_all_zero_pin_cell(
    emitter: &mut Emitter,
    outcome: &HhRunOutcome,
) -> AllZeroPinCellTotals {
    let totals = all_zero_pin_cell_totals(outcome);
    emitter.emit(&format!(
        "name=all_zero_pin_cell family={} {} mounts={} pins_installed={} pinned_points={} pinned_slots_peak={} pinned_slots_end={} threshold_lowered_points={} released_by_write={} released_by_reread_only={} bad_slots_at_mount={} bad_slots_still_pinned={}",
        outcome.family,
        outcome.shape.fields(),
        totals.mounts,
        totals.pins_installed,
        totals.pinned_points,
        totals.pinned_slots_peak,
        totals.pinned_slots_end,
        totals.threshold_lowered_points,
        totals.released_by_write,
        totals.released_by_reread_only,
        totals.bad_slots_at_mount,
        totals.bad_slots_still_pinned,
    ));
    totals
}

/// PQ1 一段历史里「造出了洞」的判定（登记第六节 6.1：② = t + 1 且 ③ 没有）。
pub(super) fn premise_one_reproduced(outcome: &HhRunOutcome) -> bool {
    !outcome.recoveries.is_empty()
        && outcome.recoveries.iter().all(|recovery| {
            recovery.first_txg_of_the_new_instance == recovery.hole_txg + 1
                && !recovery.ring_has_a_root_at_the_hole_txg
        })
}

pub(super) fn hh_request<'anchor>(
    family: &'static str,
    shape: HhCellShape,
    anchor: &'anchor anchor_model::AnchorCell,
) -> HhRunRequest<'anchor> {
    let is_first_geometry = shape.slots_per_region == E156_SLOTS_PER_REGION
        && shape.density == WorkloadDensity::EveryPublishOverwrites;
    // 第 7 次重跑：Hc、Hs 与 PC-同槽 不进第 6 次已有的旁路检查（I-3.1 判洞上状态、不回收对照），这些只对 Hh 一族有定义。
    let is_hole_count_sweep = family == E156_FAMILY_HOLE_COUNT_SWEEP;
    HhRunRequest {
        family,
        shape,
        allocator_source: HhAllocatorSource::ProductPathWithRootRing,
        cut_point: CrashCutPoint::BeforeTheRootSlotWrite,
        anchor,
        check_overwrites_before_the_ring_wraps: is_first_geometry && shape.hole_count == 0,
        judge_checker_on_hole_states: is_first_geometry && is_hole_count_sweep,
        emit_positive_control_no_reclaim: is_hole_count_sweep
            && (shape.hole_count == 0
                || (shape.hole_count == 2 && shape.position == Some(HolePosition::Back))),
        treat_the_hole_ring_slot_as_rewritten: false,
    }
}
