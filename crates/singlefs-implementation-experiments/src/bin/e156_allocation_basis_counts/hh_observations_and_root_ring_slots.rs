//! 岔路 1 的族 Hh：观测点、臂与回收时机、一趟的请求与结局、树表的谱系与根环槽的内容、挂载时读回全零的槽。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ObservationPoint {
    NewPoolFileCreation,
    Workload(WorkloadPublishKind),
    MountReturn,
}

impl ObservationPoint {
    pub(super) fn label(self) -> &'static str {
        match self {
            ObservationPoint::NewPoolFileCreation => "new_pool_file_creation",
            ObservationPoint::Workload(kind) => kind.label(),
            ObservationPoint::MountReturn => "mount_return",
        }
    }
}

/// 逐观测点一行（产物 `name=q1_observation`）。
#[derive(Clone, Debug)]
pub(super) struct HhObservation {
    pub(super) txg: u64,
    pub(super) point: ObservationPoint,
    pub(super) workload_publishes: u64,
    pub(super) holes_injected: u64,
    pub(super) reading: HhStateReading,
    pub(super) self_reclaimed_real: Option<u64>,
    pub(super) anchor: Option<anchor_model::AnchorRow>,
}

impl HhObservation {
    pub(super) fn over_withheld(&self, arm: Arm, timing: ReclaimTiming) -> u64 {
        let device_zero = &self.reading.device_zero;
        match (arm, timing) {
            (Arm::FloorRule, ReclaimTiming::Real) => device_zero.floor_rule_real,
            (Arm::FloorRule, ReclaimTiming::EveryPublish) => device_zero.floor_rule_every_publish,
            (Arm::LifetimeIntervalRule, ReclaimTiming::Real) => device_zero.interval_rule_real,
            (Arm::LifetimeIntervalRule, ReclaimTiming::EveryPublish) => {
                device_zero.interval_rule_every_publish
            }
        }
    }
    pub(super) fn delta(&self, timing: ReclaimTiming) -> i64 {
        i64::try_from(self.over_withheld(Arm::FloorRule, timing)).expect("落在 i64 内")
            - i64::try_from(self.over_withheld(Arm::LifetimeIntervalRule, timing))
                .expect("落在 i64 内")
    }
}

/// 两条臂：甲-T1（回收按门槛 max(F_生效, 环里最旧有效根)，今天的实现）与 G12（按寿命区间）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Arm {
    FloorRule,
    LifetimeIntervalRule,
}

/// 两个回收时点（W1、W2）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub(super) enum ReclaimTiming {
    Real,
    EveryPublish,
}

impl ReclaimTiming {
    pub(super) fn label(self) -> &'static str {
        match self {
            ReclaimTiming::Real => "real",
            ReclaimTiming::EveryPublish => "every_publish",
        }
    }
}

/// 一次 c2 之后那一次可写挂载的报告（PQ1 的 ①–④、K14 / S1(k)、G-adm）。
#[derive(Clone, Debug)]
pub(super) struct HoleRecovery {
    pub(super) hole_txg: u64,
    pub(super) crashed_publish_kind: WorkloadPublishKind,
    pub(super) prefix_applied: usize,
    pub(super) first_txg_of_the_new_instance: u64,
    pub(super) last_txg_of_the_mount: u64,
    pub(super) publishes_in_the_mount: u64,
    pub(super) ring_has_a_root_at_the_hole_txg: bool,
    pub(super) hole_slot_content: String,
    pub(super) cut_matches_the_restored_image: bool,
    pub(super) sectors_differing_from_the_root_slot_only_image: Vec<(u32, u64)>,
    pub(super) steps_after_the_root_slot_write: Vec<&'static str>,
    pub(super) root_slot_writes_in_the_crashed_publish: usize,
    pub(super) mount_admission: &'static str,
    pub(super) mount_floor_raise_sequences: u64,
}

#[derive(Clone, Debug)]
pub(super) struct HhRunOutcome {
    pub(super) family: &'static str,
    pub(super) shape: HhCellShape,
    pub(super) allocator_source: HhAllocatorSource,
    pub(super) cut_point: CrashCutPoint,
    pub(super) registered_end: u64,
    pub(super) actual_end: u64,
    pub(super) truncation: Option<(u64, String)>,
    pub(super) observations: Vec<HhObservation>,
    pub(super) recoveries: Vec<HoleRecovery>,
    pub(super) genesis_root_in_ring: bool,
    pub(super) slots_per_region_read_back: u64,
    pub(super) new_pool_file_creation_matches_the_registered_anchor: bool,
    pub(super) ring_wrap_overwrites_checked: u64,
    pub(super) ring_wrap_overwrite_mismatches: u64,
    pub(super) tree_table_release_checks: u64,
    pub(super) tree_table_release_violations: u64,
    pub(super) mount_schedule_mismatches: u64,
    pub(super) disk_cross_checked_roots: u64,
    pub(super) disk_cross_check_mismatches: u64,
    pub(super) disk_cross_check_failures: Vec<String>,
    pub(super) admission_refusals: u64,
    pub(super) tree_table_lags_real: BTreeSet<u64>,
    pub(super) tree_table_lags_every_publish: BTreeSet<u64>,
    pub(super) checker_red_on_hole_states: u64,
    pub(super) checker_judged_hole_states: u64,
    /// 第 6 次重跑登记 E8：每次挂载那一刻的定乙钉住。
    pub(super) all_zero_pin_mounts: Vec<AllZeroPinsAtMount>,
    /// W18 (a)、(b) 解开的钉住槽数（整格累加，每槽每次挂载最多一次）。
    pub(super) all_zero_pins_released_by_write: u64,
    pub(super) all_zero_pins_released_by_reread_only: u64,
}

/// 一格要怎么跑（第 4 次重跑登记第五节 5.3、5.4）。
pub(super) struct HhRunRequest<'anchor> {
    pub(super) family: &'static str,
    pub(super) shape: HhCellShape,
    pub(super) allocator_source: HhAllocatorSource,
    pub(super) cut_point: CrashCutPoint,
    pub(super) anchor: &'anchor anchor_model::AnchorCell,
    pub(super) check_overwrites_before_the_ring_wraps: bool,
    pub(super) judge_checker_on_hole_states: bool,
    pub(super) emit_positive_control_no_reclaim: bool,
    /// 第 7 次重跑登记 5.3、E25 的 PC-同槽：「每」列的旁路门槛把第一个洞的环槽当作这个进程写过（只改旁路，不碰真实分配器）。
    pub(super) treat_the_hole_ring_slot_as_rewritten: bool,
}

/// 这一格挂在第 4 次重跑登记第十一节停机 S1(k) 的 K13：每一版换下上一版的树表单元，释放代 = 这一版的 txg。
pub(super) struct TreeTableLineage {
    pub(super) previous_tree_table_slot: u64,
    pub(super) pending_release_checks: Vec<(u64, u64)>,
    pub(super) releases: Vec<(u64, u64)>,
}

impl TreeTableLineage {
    pub(super) fn note_version(
        &mut self,
        versions: &mut BTreeMap<u64, PublishedVersionFacts>,
        output: &TransactionOutput,
    ) {
        let facts = facts_of_published_version(output);
        let txg = output.root.checkpoint_txg.0;
        self.pending_release_checks
            .push((self.previous_tree_table_slot, txg));
        self.releases.push((self.previous_tree_table_slot, txg));
        self.previous_tree_table_slot = facts.tree_table_slot;
        versions.insert(txg, facts);
    }
}

pub(super) fn describe_the_root_ring_slot_of_txg(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    parameters: &MakeFilesystemParameters,
    txg: u64,
) -> String {
    let ring_slot = target_for_publish(
        CheckpointTxg(txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    match read_root_ring_slot(
        devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReading::SelfVerified(root) => format!("root_txg_{}", root.checkpoint_txg.0),
        RootRingSlotReading::Bad(_) => {
            let device = parameters.region_devices
                [usize::try_from(ring_slot.region).expect("区域号落在 3 以内")];
            let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
            let root_slot_bytes =
                usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
            match PoolReader::read(devices, device, offset, root_slot_bytes) {
                Some(bytes) if bytes.iter().all(|byte| *byte == 0) => "never_written".to_string(),
                Some(_) => "not_self_verified".to_string(),
                None => "unreadable".to_string(),
            }
        }
    }
}

/// 根环一个槽这一刻读到的样子（第 6 次重跑登记 W17）：读出自证过的根；读不出自证而原字节每个都是 0；
/// 其余（读不出、读得出而不是全零又自证不过）都算读坏，这一族不造，出现了照 S17 停。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RootRingSlotContent {
    SelfVerifiedRoot { checkpoint_txg: u64 },
    AllZero,
    Bad,
}

pub(super) fn read_root_ring_slot_content(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    parameters: &MakeFilesystemParameters,
    ring_slot: RootRingSlot,
) -> RootRingSlotContent {
    match read_root_ring_slot(
        devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReading::SelfVerified(root) => RootRingSlotContent::SelfVerifiedRoot {
            checkpoint_txg: root.checkpoint_txg.0,
        },
        RootRingSlotReading::Bad(_) => {
            let device = parameters.region_devices
                [usize::try_from(ring_slot.region).expect("区域号落在 3 以内")];
            let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
            let root_slot_bytes =
                usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
            match PoolReader::read(devices, device, offset, root_slot_bytes) {
                Some(bytes) if bytes.iter().all(|byte| *byte == 0) => RootRingSlotContent::AllZero,
                Some(_) | None => RootRingSlotContent::Bad,
            }
        }
    }
}

/// 一次挂载那一刻的定乙钉住（第 6 次重跑登记 W16、W17，第五节 E8）：交给 `mount_writable` 之前那份切段镜像里
/// 读回全零的槽，各钉在它可能住过的最旧 txg。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AllZeroPinsAtMount {
    pub(super) hole_txg: u64,
    pub(super) first_txg_of_the_new_instance: u64,
    pub(super) pinned: BTreeMap<RootRingSlot, u64>,
    pub(super) bad_slots_at_mount: u64,
}

/// W17：3S 个槽逐个读，交回读回全零的槽与读坏的槽数。槽用 txg 0..3S 经 `target_for_publish` 走一遍（A2：txg 与 u mod 3S 一一对应）。
pub(super) fn root_ring_slots_reading_back_all_zero_at_mount(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    parameters: &MakeFilesystemParameters,
    ring_length: u64,
) -> (BTreeSet<RootRingSlot>, u64) {
    let mut all_zero = BTreeSet::new();
    let mut bad_slots = 0u64;
    for txg in 0..ring_length {
        let ring_slot = target_for_publish(
            CheckpointTxg(txg),
            parameters.geometry.root_ring_slots_per_region,
        );
        match read_root_ring_slot_content(devices, parameters, ring_slot) {
            RootRingSlotContent::SelfVerifiedRoot { .. } => {}
            RootRingSlotContent::AllZero => {
                all_zero.insert(ring_slot);
            }
            RootRingSlotContent::Bad => bad_slots += 1,
        }
    }
    (all_zero, bad_slots)
}

/// W16：落到这一槽、小于这次挂载新实例第一个 txg 的最小 txg，在 [0, min(3S, 第一个 txg)) 里找；一个都没有的槽不钉。
pub(super) fn oldest_txg_an_all_zero_slot_can_have_held(
    ring_slot: RootRingSlot,
    first_txg_of_the_new_instance: u64,
    ring_length: u64,
    parameters: &MakeFilesystemParameters,
) -> Option<u64> {
    (0..ring_length.min(first_txg_of_the_new_instance)).find(|txg| {
        target_for_publish(
            CheckpointTxg(*txg),
            parameters.geometry.root_ring_slots_per_region,
        ) == ring_slot
    })
}

pub(super) fn mount_admission_label(admission: &MountSpaceAdmission) -> &'static str {
    match admission {
        MountSpaceAdmission::AdmittedBeforeAcquisition => "admitted_before_acquisition",
        MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => {
            "admitted_after_the_floor_raises"
        }
        MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => {
            "still_short_after_the_floor_raises"
        }
        MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "not_judged_by_the_test_only_switch",
    }
}

pub(super) fn new_pool_file_creation_matches_the_registered_anchor(
    allocator: &PoolAllocator,
    first: &TransactionOutput,
) -> bool {
    let (allocated, _free, deferred) = accounting_row_slots(allocator, DeviceIdentity(0));
    let mut placements: Vec<(u64, u64)> = first
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.identity.span_slots()))
        .collect();
    placements.sort_unstable();
    first.root.checkpoint_txg.0 == 3
        && allocated == E156_NEW_POOL_FILE_CREATION_ITEM1_SLOTS
        && deferred == E156_NEW_POOL_FILE_CREATION_ITEM5_SLOTS
        && placements == E156_NEW_POOL_FILE_CREATION_PLACEMENTS
        && allocator.records().len() == E156_NEW_POOL_FILE_CREATION_EXPECTED_RECORD_COUNT
}

pub(super) fn released_not_reclaimed_slots_on_device_zero(
    allocator: &PoolAllocator,
) -> BTreeSet<u64> {
    let free_map = allocator
        .devices
        .iter()
        .find(|map| map.device == DeviceIdentity(0))
        .expect("盘 0 在池里");
    expanded_slots(allocator.records().iter().filter(|record| {
        record.device == DeviceIdentity(0) && record.is_released && !free_map.is_free(record.slot)
    }))
}

#[cfg(test)]
mod tests;
