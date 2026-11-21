//! 各臂（df 字面、可用、载荷、写需求与总是多报的对照）的读数与按臂的判定。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// F2 的臂（登记 5.1）：每条臂报的字节
// ---------------------------------------------------------------------------------------------------------------------

/// 登记 5.1 的候选臂加两条控制（恒多报的阳性对照、恒报 0 的阴性对照）。次序固定，产物行按这个次序打。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Arm {
    Candidate(CandidateArm),
    PositiveControlAlwaysOverReports,
    NegativeControlReportsZero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CandidateArm {
    /// D16 已定项 1 `df` 字面，活元数据取 0。
    DfLiteralWithoutLiveMetadata,
    /// D16 已定项 1 `df` 字面，活元数据取现行账里活着的非数据槽。
    DfLiteralWithAllLiveMetadata,
    /// D28 式子可用、各盘取小，按槽字节报。
    AvailableMinimumOverDevicesInSlotBytes,
    /// 同一臂，按单元换载荷字节。
    AvailableMinimumOverDevicesInPayloadBytes,
    /// D28 可用两盘相加 ÷ 副本数。
    AvailableSummedOverDevicesDividedByReplicas,
    /// W2 再扣写 n 单元那一次的非数据需求上界。
    AvailableMinimumLessTheNonDataDemandOfTheWrite,
}

const ALL_CANDIDATE_ARMS: [CandidateArm; 6] = [
    CandidateArm::DfLiteralWithoutLiveMetadata,
    CandidateArm::DfLiteralWithAllLiveMetadata,
    CandidateArm::AvailableMinimumOverDevicesInSlotBytes,
    CandidateArm::AvailableMinimumOverDevicesInPayloadBytes,
    CandidateArm::AvailableSummedOverDevicesDividedByReplicas,
    CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite,
];

impl CandidateArm {
    /// 产物里的臂名（登记 5.1 的 W1-零、W1-全、W2-字节、W2-载荷、W3、W4）。
    pub(super) fn label(self) -> &'static str {
        match self {
            CandidateArm::DfLiteralWithoutLiveMetadata => "df_literal_without_live_metadata",
            CandidateArm::DfLiteralWithAllLiveMetadata => "df_literal_with_all_live_metadata",
            CandidateArm::AvailableMinimumOverDevicesInSlotBytes => {
                "available_minimum_over_devices_in_slot_bytes"
            }
            CandidateArm::AvailableMinimumOverDevicesInPayloadBytes => {
                "available_minimum_over_devices_in_payload_bytes"
            }
            CandidateArm::AvailableSummedOverDevicesDividedByReplicas => {
                "available_summed_over_devices_divided_by_replicas"
            }
            CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite => {
                "available_minimum_less_the_non_data_demand_of_the_write"
            }
        }
    }
}

impl Arm {
    pub(super) fn label(self) -> &'static str {
        match self {
            Arm::Candidate(candidate) => candidate.label(),
            Arm::PositiveControlAlwaysOverReports => "always_over_reports_control",
            Arm::NegativeControlReportsZero => "zero_control",
        }
    }
}

pub(super) fn all_arms() -> Vec<Arm> {
    let mut arms: Vec<Arm> = ALL_CANDIDATE_ARMS
        .iter()
        .copied()
        .map(Arm::Candidate)
        .collect();
    arms.push(Arm::PositiveControlAlwaysOverReports);
    arms.push(Arm::NegativeControlReportsZero);
    arms
}

fn ceiling_division(numerator: u64, denominator: u64) -> u64 {
    numerator.div_ceil(denominator)
}

/// 写 n 个数据单元那一次的非数据需求上界，单位槽（登记 5.1 W4）：下段叶 ⌈n ÷ 144⌉ 逐层 ⌈÷ 147⌉ 到 1 的节点数之和（n ≤ 1 取 0）
/// + 1（上段叶）+ 2（inode 叶容器）+ inode 树这一版的内部层数 + c_max。
fn non_data_demand_upper_bound_slots(
    units: u64,
    inode_tree_internal_levels: u64,
    checkpoint_cost_blocks: u64,
) -> u64 {
    let lower_tree_nodes = if units <= 1 {
        0
    } else {
        let mut level_nodes = ceiling_division(units, EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
        let mut total = level_nodes;
        while level_nodes > 1 {
            level_nodes = ceiling_division(level_nodes, EXTENT_TREE_INTERNAL_FANOUT);
            total += level_nodes;
        }
        total
    };
    lower_tree_nodes + 1 + 2 + inode_tree_internal_levels + checkpoint_cost_blocks
}

/// W4：满足 2n + U(n) ≤ 各盘可用槽数的最小值 的最大 n（单调，二分）。
fn largest_units_satisfying_the_write_demand(
    available_slots: u64,
    inode_tree_internal_levels: u64,
    checkpoint_cost_blocks: u64,
) -> u64 {
    let fits = |units: u64| {
        2 * units
            + non_data_demand_upper_bound_slots(
                units,
                inode_tree_internal_levels,
                checkpoint_cost_blocks,
            )
            <= available_slots
    };
    if !fits(0) {
        return 0;
    }
    let mut known_to_fit = 0u64;
    let mut known_not_to_fit = available_slots / 2 + 1;
    while known_not_to_fit - known_to_fit > 1 {
        let middle = known_to_fit + (known_not_to_fit - known_to_fit) / 2;
        if fits(middle) {
            known_to_fit = middle;
        } else {
            known_not_to_fit = middle;
        }
    }
    known_to_fit
}

impl StateReading {
    pub(super) fn minimum_available_slots(&self) -> u64 {
        let minimum = self
            .devices
            .iter()
            .map(DeviceReading::available_slots_rounded_down)
            .min()
            .expect("至少一块盘");
        u64::try_from(minimum.max(0)).expect("可用槽数")
    }

    fn summed_available_slots(&self) -> u64 {
        let total: i128 = self
            .devices
            .iter()
            .map(|device| device.available_slots_rounded_down().max(0))
            .sum();
        u64::try_from(total).expect("可用槽数之和")
    }

    fn pool_free_slots_counting_released(&self) -> i128 {
        self.devices
            .iter()
            .map(|device| i128::from(device.capacity_slots) - i128::from(device.live_slots))
            .sum()
    }

    fn live_non_data_slots_summed(&self) -> i128 {
        self.devices
            .iter()
            .map(|device| i128::from(device.live_slots) - i128::from(2 * self.data_units))
            .sum()
    }

    /// 这条臂此刻报的字节（可为负；报给用户时按 0 封底，见 [`reported_bytes`]）。
    pub(super) fn raw_bytes_of(&self, arm: Arm) -> i128 {
        let slot_bytes = i128::from(SLOT_BYTES);
        let payload_bytes = i128::from(DATA_UNIT_PAYLOAD_BYTES);
        let cost = i128::from(self.checkpoint_cost_blocks);
        let reserve_slots = 10 + 7 * cost;
        let residue_slots = 5 + 6 * cost;
        let lag = i128::from(self.lag_slots);
        match arm {
            Arm::Candidate(CandidateArm::DfLiteralWithoutLiveMetadata) => {
                slot_bytes
                    * (floor_half(self.pool_free_slots_counting_released())
                        - reserve_slots
                        - residue_slots
                        - lag)
            }
            Arm::Candidate(CandidateArm::DfLiteralWithAllLiveMetadata) => {
                slot_bytes
                    * (floor_half(self.pool_free_slots_counting_released())
                        + floor_half(self.live_non_data_slots_summed())
                        - reserve_slots
                        - residue_slots
                        - lag)
            }
            Arm::Candidate(CandidateArm::AvailableMinimumOverDevicesInSlotBytes) => {
                slot_bytes * i128::from(self.minimum_available_slots())
            }
            Arm::Candidate(CandidateArm::AvailableMinimumOverDevicesInPayloadBytes) => {
                payload_bytes * i128::from(self.minimum_available_slots() / 2)
            }
            Arm::Candidate(CandidateArm::AvailableSummedOverDevicesDividedByReplicas) => {
                slot_bytes * i128::from(self.summed_available_slots() / DEVICE_COUNT)
            }
            Arm::Candidate(CandidateArm::AvailableMinimumLessTheNonDataDemandOfTheWrite) => {
                payload_bytes
                    * i128::from(largest_units_satisfying_the_write_demand(
                        self.minimum_available_slots(),
                        self.inode_tree_internal_levels,
                        self.checkpoint_cost_blocks,
                    ))
            }
            Arm::PositiveControlAlwaysOverReports => {
                slot_bytes * self.pool_free_slots_counting_released()
            }
            Arm::NegativeControlReportsZero => 0,
        }
    }
}

/// 报给用户的字节：负数按 0 报（`df` 不报负）。
pub(super) fn reported_bytes(raw: i128) -> u64 {
    u64::try_from(raw.max(0)).expect("报的字节装得进 u64")
}

// ---------------------------------------------------------------------------------------------------------------------
// 判定：多报、少报、截尾（登记第一节「多报」「少报」，第六节 Q1–Q4）
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Judgement {
    /// 照这个字节数写会被拒：多报 `over_bytes` = X − T。
    OverReports { over_bytes: u64 },
    /// 不多报；`under_bytes` = T − X。
    DoesNotOverReport { under_bytes: u64 },
    /// T 只知道下界，X 不超过下界所以一定不多报。
    CertainlyDoesNotOverReportBelowTheLowerBound,
    /// T 只知道下界或取不出，判不了。
    Undetermined,
}

impl Judgement {
    pub(super) fn label(&self) -> &'static str {
        match self {
            Judgement::OverReports { .. } => "over",
            Judgement::DoesNotOverReport { .. } => "not_over",
            Judgement::CertainlyDoesNotOverReportBelowTheLowerBound => "not_over_below_lower_bound",
            Judgement::Undetermined => "undetermined",
        }
    }
}

/// 报 `reported` 字节的臂要的单元数 ⌈X ÷ 32634⌉（登记第一节）：判定与按需收紧共用这一处。
fn units_asked_for_by(reported: u64) -> u64 {
    reported.div_ceil(DATA_UNIT_PAYLOAD_BYTES)
}

impl StateReading {
    /// 这个状态上每条臂（[`all_arms`] 的次序）要的单元数：按需收紧看它们落没落在（当前下界, 记录上限] 里。
    pub(super) fn units_asked_for_by_every_arm(&self) -> Vec<u64> {
        all_arms()
            .into_iter()
            .map(|arm| units_asked_for_by(reported_bytes(self.raw_bytes_of(arm))))
            .collect()
    }
}

/// 候选在 `reported` 字节上对着 T 的判定：⌈X ÷ 32634⌉ > n 即多报（登记第一节）。
pub(super) fn judge(reported: u64, writable: &WritableUnits) -> Judgement {
    let units_asked_for = units_asked_for_by(reported);
    match writable {
        WritableUnits::Exact { units, .. } => {
            let writable_bytes = units * DATA_UNIT_PAYLOAD_BYTES;
            if units_asked_for > *units {
                Judgement::OverReports {
                    over_bytes: reported - writable_bytes,
                }
            } else {
                Judgement::DoesNotOverReport {
                    under_bytes: writable_bytes - reported.min(writable_bytes),
                }
            }
        }
        WritableUnits::AtLeastBecauseOfTheJournalCap {
            lower_bound_units, ..
        } => {
            if units_asked_for <= *lower_bound_units {
                Judgement::CertainlyDoesNotOverReportBelowTheLowerBound
            } else {
                Judgement::Undetermined
            }
        }
        WritableUnits::Unreadable(_) => Judgement::Undetermined,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod arm_and_residual_tests;
