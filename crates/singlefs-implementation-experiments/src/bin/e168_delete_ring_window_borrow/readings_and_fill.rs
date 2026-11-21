//! 读数与压满：每块盘的可用字节、保留量读数、分配记录树位置与从单元区末尾压满。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 读数与压满
// ---------------------------------------------------------------------------------------------------------------------

/// 每块盘的式子可用（字节），账取此刻线程局部里那一对。
fn available_bytes_of_each_device(pool: &MountedPool) -> Vec<i128> {
    admission_reading_before_a_publish(&pool.session.allocator, Some(pool.current_file_version()))
        .available_on_each_device()
        .into_iter()
        .map(|(_, available)| available.0)
        .collect()
}

/// 这一刻的读数：保留量（`crates/` 读数，逐盘槽）与 ckpt_cost。保留量 = 账置 0 时「空闲槽 − 式子可用 ÷ 16384」（登记 1.2），
/// 读完把账换回 `ledger_of_the_arm`；与本地按条款算的 8 + 13c 逐盘比（停机条款 S3），被抛弃根独占量不为 0 也在这里对不上。
pub(super) fn reserve_reading(
    pool: &MountedPool,
    hooks: &DeletePathHooks,
    ledger_of_the_arm: Option<(u64, u64)>,
) -> (Vec<i128>, u64) {
    (hooks.set_ledger)(Some((0, 0)));
    let available = available_bytes_of_each_device(pool);
    (hooks.set_ledger)(ledger_of_the_arm);
    let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on(
        Some(pool.current_file_version()),
        &pool.session.allocator,
    )
    .0;
    let reserve_by_the_clauses = i128::from(reserve_slots_by_the_clauses(checkpoint_cost));
    let slot_bytes = i128::from(SLOT_BYTES);
    let reserve: Vec<i128> = pool
        .free_slots_of_each_device()
        .into_iter()
        .zip(available)
        .map(|(free_slots, available_bytes)| {
            if available_bytes % slot_bytes != 0 {
                stop_with(
                    "reserve_reading_not_whole_slots",
                    &format!(
                        "geometry={} available_bytes={available_bytes}",
                        pool.geometry.label()
                    ),
                );
            }
            i128::from(free_slots) - available_bytes / slot_bytes
        })
        .collect();
    for (device_position, reserve_on_the_device) in reserve.iter().enumerate() {
        if *reserve_on_the_device != reserve_by_the_clauses {
            stop_with(
                "s3_reserve_differs_from_the_clauses",
                &format!(
                    "geometry={} device={device_position} implementation={reserve_on_the_device} clauses={reserve_by_the_clauses} checkpoint_cost={checkpoint_cost}",
                    pool.geometry.label()
                ),
            );
        }
    }
    (reserve, checkpoint_cost)
}

/// 停机条款 S4 与第七节 B4：本地 K1 与 `crates/` ckpt_cost 里分配记录树那一项比，单元区槽数与分配器读数比。交回 (根层级, K1)。
pub(super) fn allocation_record_tree_positions_checked(
    pool: &MountedPool,
    hooks: &DeletePathHooks,
) -> (u32, u64) {
    for map in &pool.session.allocator.devices {
        if map.unit_area_slots() != pool.geometry.unit_area_slots() {
            stop_with(
                "b4_unit_area_slots_differ",
                &format!(
                    "geometry={} allocator={} local={}",
                    pool.geometry.label(),
                    map.unit_area_slots(),
                    pool.geometry.unit_area_slots()
                ),
            );
        }
    }
    let implementation = (hooks.allocation_record_tree_positions)(
        pool.current_file_version(),
        &pool.session.allocator,
    );
    let (root_level, by_the_clause) =
        allocation_record_tree_positions_by_the_clause(&pool.geometry.unit_areas());
    if implementation != by_the_clause {
        stop_with(
            "s4_allocation_record_tree_positions_differ",
            &format!("geometry={} implementation={implementation} local={by_the_clause} root_level={root_level}", pool.geometry.label()),
        );
    }
    (root_level, by_the_clause)
}

/// W_t：这次发布写出的分配记录里落在这块盘、分配代 = 这次 txg、未释放的跨度之和（登记 1.2）。
pub(super) fn new_slots_on_device(output: &TransactionOutput, device: DeviceIdentity) -> u64 {
    output
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == device
                && !record.is_released
                && record.generation == output.root.checkpoint_txg
        })
        .map(|record| u64::from(record.span_slots))
        .sum()
}

/// 一次发布按空间归哪一格拆的槽数（`space_budget_of_role`）与落点那一判的需求（M6 的 W_land）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PublishShape {
    pub(super) ordinary_slots: u64,
    pub(super) fixed_point_slots: u64,
    pub(super) switch_reserve_slots: u64,
    pub(super) landing_demand_slots: u64,
}

pub(super) fn shape_of_the_publish(
    output: &TransactionOutput,
    checkpoint_cost_before: u64,
) -> PublishShape {
    let slots_of = |budget: SpaceBudgetOfARole| -> u64 {
        output
            .rewritten
            .iter()
            .filter(|role| space_budget_of_role(**role) == budget)
            .map(|role| role.span_slots())
            .sum()
    };
    let ordinary_roles: Vec<TransactionUnit> = output
        .rewritten
        .iter()
        .copied()
        .filter(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation)
        .collect();
    let landing = units_of_a_publish_to_land(
        &ordinary_roles,
        singlefs_core::admission::MetadataBlocks(checkpoint_cost_before),
    );
    PublishShape {
        ordinary_slots: slots_of(SpaceBudgetOfARole::OrdinaryAllocation),
        fixed_point_slots: slots_of(SpaceBudgetOfARole::CheckpointReservePool),
        switch_reserve_slots: slots_of(SpaceBudgetOfARole::InstanceSwitchReserve),
        landing_demand_slots: landing.commit_generated_slots
            + landing.data_units * (DATA_UNIT_BYTES / SLOT_BYTES),
    }
}

/// 5.3 压满的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FillOutcome {
    /// 每块盘的式子可用都落在 (−16384, 0]；`free_slots` 是压满之后每块盘的空闲槽，`filled_slots_of_each_device` 是每块盘标掉的槽号
    /// （停机条款 S7 判 checker 之前要撤掉它们，见 [`check_the_finished_delete`]）。
    WithinOneSlotAtOrBelowZero {
        free_slots: Vec<u64>,
        filled_slots_of_each_device: Vec<Vec<u64>>,
    },
    /// 压满之前式子可用就已经 ≤ −16384（这个几何上写进文件、重挂载之后保留量已经超出空闲），造不出登记的「满池」。
    AlreadyBelowTheIntervalBeforeFilling { available_bytes: Vec<i128> },
}

/// 5.3 F-end：从单元区末尾往前把没挡的空闲槽标成已分配（只动分配器，不写盘），直到式子可用（按这条臂此刻的账）落进 (−16384, 0]。
/// 标掉的槽不改式子里别的项，一轮就该落进区间；不落就停（压满的算术与式子对不上）。
pub(super) fn fill_from_the_unit_area_end(pool: &mut MountedPool) -> FillOutcome {
    let available_before = available_bytes_of_each_device(pool);
    if available_before
        .iter()
        .any(|available| *available <= -i128::from(SLOT_BYTES))
    {
        return FillOutcome::AlreadyBelowTheIntervalBeforeFilling {
            available_bytes: available_before,
        };
    }
    let filled_slots_of_each_device: Vec<Vec<u64>> = pool
        .session
        .allocator
        .devices
        .iter_mut()
        .zip(available_before)
        .map(|(map, available)| {
            mark_free_slots_from_the_unit_area_end(
                map,
                slots_to_mark_so_available_is_within_one_slot_at_or_below_zero(available),
            )
        })
        .collect();
    let available_after = available_bytes_of_each_device(pool);
    if !available_after
        .iter()
        .all(|available| available_is_within_one_slot_at_or_below_zero(*available))
    {
        stop_with(
            "fill_did_not_land_in_the_interval",
            &format!(
                "geometry={} available_after={available_after:?}",
                pool.geometry.label()
            ),
        );
    }
    FillOutcome::WithinOneSlotAtOrBelowZero {
        free_slots: pool.free_slots_of_each_device(),
        filled_slots_of_each_device,
    }
}

/// 从单元区末尾往前把至多 `slots_to_mark` 个空闲槽一槽一槽标成已分配，交回标掉的槽号（从高到低）。
fn mark_free_slots_from_the_unit_area_end(map: &mut DeviceFreeMap, slots_to_mark: u64) -> Vec<u64> {
    let mut filled_slots = Vec::new();
    let start_slot = map.unit_area_start().slot().0;
    let mut slot = start_slot + map.unit_area_slots();
    while u64::try_from(filled_slots.len()).expect("标掉的槽数") < slots_to_mark
        && slot > start_slot
    {
        slot -= 1;
        if map.is_free(SlotNumber(slot)) {
            map.mark_allocated(SlotNumber(slot), 1);
            filled_slots.push(slot);
        }
    }
    filled_slots
}

/// 撤掉压满：把 [`mark_free_slots_from_the_unit_area_end`] 标掉的槽逐槽放回空闲。分配器没有「已分配直接回空闲」的入口，
/// 照它自己的两步走——`mark_released` 挪进待释放、`mark_reclaimed` 从待释放回空闲——待释放一加一减，已分配、空闲、空闲段数、
/// 每段占用与没挡的槽数都回到压满之前。压满的槽从没进过分配记录，也就不在 `PoolAllocator` 的记录与 defer 队列里，只动空闲图。
pub(super) fn unmark_the_filled_slots(
    device_maps: &mut [DeviceFreeMap],
    filled_slots_of_each_device: &[Vec<u64>],
) {
    assert_eq!(
        device_maps.len(),
        filled_slots_of_each_device.len(),
        "压满逐盘标过，撤也逐盘撤"
    );
    for (map, filled_slots) in device_maps.iter_mut().zip(filled_slots_of_each_device) {
        for slot in filled_slots {
            map.mark_released(SlotNumber(*slot), 1);
            map.mark_reclaimed(SlotNumber(*slot), 1);
        }
    }
}

#[cfg(test)]
mod tests;
