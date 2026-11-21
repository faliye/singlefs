//! 两道闸的快照与串、并两种读法、第一闸与第二闸的计算、df 读数。

use super::*;

// ===== 5.2 闸读的量 =====

pub(super) struct GateSnapshot {
    allocatable_free_slots: u64,
    all_free_slots_including_isolated_and_held: u64,
    released: u64,
    live_meta: u64,
    pub(super) checkpoint_reserve: u64,
    push_reserve: u64,
    #[allow(dead_code, reason = "登记「五、5.2」把 Res 列成闸读的量之一；这两个字段只在 gate_snapshot 内部参与 defined_free_space_formula_slots 的计算，ROW 级逐行输出没有实现（见文件头「输出体量说明」），所以字段构造完之后没有第二个读者——留着是为了这份 struct 仍然完整对应 5.2 那张表，不是遗留的死代码")]
    residue: u64,
    #[allow(dead_code, reason = "同上一个字段的理由：lag 对应登记里的滞后量，只在这里参与 defined_free_space_formula_slots 的运算")]
    lag: u64,
    in_flight: u64,
    abandoned_root_exclusive_slots: u64,
    pub(super) occupied_pub: u64,
    switch_reserve: u64,
    held: u64,
    defined_free_space_formula_slots: u64,
}

pub(super) fn gate_snapshot(pool: &Pool, in_flight: u64) -> GateSnapshot {
    let fixpoint_cost = pool.fixpoint_cost;
    let (push_reserve, residue, checkpoint_reserve, switch_reserve) = if pool.damage.zero_reserves {
        (0, 0, 0, 0)
    } else {
        (10 + 7 * fixpoint_cost, 5 + 6 * fixpoint_cost, fixpoint_cost, pool.switch_reserve_slots())
    };
    let allocatable_free_slots = pool.free_count_allocatable();
    let all_free_slots_including_isolated_and_held = pool.free_count_all();
    let released = pool.released_slot_count();
    let live_meta = fixpoint_cost;
    let lag = pool.lag_count();
    let abandoned_root_exclusive_slots = pool.isolated_count();
    let occupied_pub = pool.latest_occupied_pub();
    let held = pool.held_slot_count();
    let defined_free_space_formula_slots = if pool.damage.raw_df {
        all_free_slots_including_isolated_and_held + released
    } else {
        (all_free_slots_including_isolated_and_held + released + live_meta)
            .saturating_sub(push_reserve)
            .saturating_sub(residue)
            .saturating_sub(lag)
    };
    GateSnapshot {
        allocatable_free_slots,
        all_free_slots_including_isolated_and_held,
        released,
        live_meta,
        checkpoint_reserve,
        push_reserve,
        residue,
        lag,
        in_flight,
        abandoned_root_exclusive_slots,
        occupied_pub,
        switch_reserve,
        held,
        defined_free_space_formula_slots,
    }
}

/// 可用_串（D28（挂载期承诺量） 已定项 1，「已分配」读成占着、删掉 defer 待释放）。
fn gate_serial(snapshot: &GateSnapshot, pool: &Pool, arm: Arm) -> u64 {
    let held_penalty = if arm.reclaim.first_gate_excludes_held() {
        snapshot.held
    } else {
        0
    };
    pool.capacity_slots
        .saturating_sub(snapshot.occupied_pub)
        .saturating_sub(snapshot.switch_reserve)
        .saturating_sub(snapshot.abandoned_root_exclusive_slots)
        .saturating_sub(snapshot.checkpoint_reserve)
        .saturating_sub(snapshot.in_flight)
        .saturating_sub(held_penalty)
}

/// 可用_合 = 可用_串 + 已释放且 g ≤ 抬 F 上限的槽数（登记「五、5.2」）。
/// `serial` 已经按 F-双扣 扣过一次扣住槽数（`gate_serial` 对 `arm.reclaim` 独立判断，
/// 与 `arm.gate` 是哪一种读法无关），这里不重复扣。
fn gate_union(pool: &Pool, serial: u64) -> u64 {
    let ceiling = pool.rollback_floor_ceiling().unwrap_or(0);
    let creditable = pool
        .slots
        .iter()
        .filter(|state| {
            matches!(state, SlotState::Released { released_at_txg, .. } if *released_at_txg <= ceiling)
        })
        .count() as u64;
    serial + creditable
}

/// 第一道闸（gate1）：按闸读法分派。
pub(super) fn compute_gate1(pool: &Pool, arm: Arm, in_flight: u64) -> (u64, GateSnapshot) {
    let snapshot = gate_snapshot(pool, in_flight);
    let serial = gate_serial(&snapshot, pool, arm);
    let value = match arm.gate {
        GateReading::Serial | GateReading::SerialRejudge => serial,
        GateReading::Union | GateReading::UnionFullDeduction => gate_union(pool, serial),
    };
    (value, snapshot)
}

/// 第二道闸：可分配_D16 = min(U − InF + L_meta − P_push, df_D16 − InF)，四条闸读法共用。
pub(super) fn compute_gate2(snapshot: &GateSnapshot) -> u64 {
    let left = snapshot
        .allocatable_free_slots
        .saturating_sub(snapshot.in_flight)
        .saturating_add(snapshot.live_meta)
        .saturating_sub(snapshot.push_reserve);
    let right = snapshot.defined_free_space_formula_slots.saturating_sub(snapshot.in_flight);
    left.min(right)
}

/// `df`（字节，按臂）：串族取可用_串，合族取 D16（发布语义） 那条 `df` 式子算出的槽数；
/// 合-全扣 在这个基础上再减一次切换预留、被抛弃根独占量与在飞已批准（见「五、5.3」闸的读法表）。
/// PC2（阳性对照）：改坏时一律报 U_all + Rel，不管闸读法是哪一种。
pub(super) fn df_bytes(pool: &Pool, arm: Arm, snapshot: &GateSnapshot, serial: u64) -> u64 {
    if pool.damage.raw_df {
        return (snapshot.all_free_slots_including_isolated_and_held + snapshot.released) * 16384;
    }
    let slots = match arm.gate {
        GateReading::Serial | GateReading::SerialRejudge => serial,
        GateReading::Union => snapshot.defined_free_space_formula_slots,
        GateReading::UnionFullDeduction => snapshot
            .defined_free_space_formula_slots
            .saturating_sub(snapshot.switch_reserve)
            .saturating_sub(snapshot.abandoned_root_exclusive_slots)
            .saturating_sub(snapshot.in_flight),
    };
    slots * 16384
}

#[cfg(test)]
#[path = "gates_and_reserves_tests.rs"]
mod tests;
