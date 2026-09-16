//! C355（checkpoint 保留池池级扣而固定点每块要落两列）第一轮攻方腿：问一的计数模型。只用 std。
//! 两块盘（D2（RAID 条带策略） 已定项 9）各自的空闲槽 free_d（已扣第八项以外的全部项）、在飞已批 p 个单元、
//! 一次请求 u 个单元；一个单元 = 一个 16 KiB 槽、两盘各一份（first-txn-layout.md 零节）；固定点每块盘要 c 个槽，
//! c = 准入那一刻现算的 ckpt_cost（问一里假设它等于固定点实际要的，问二另判）。
//! 真值：放行之后固定点每块盘都分得到 ⇔ ∀d: free_d ≥ p + u + c。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    JiaPoolOnce,
    YiPoolTimesReplicas,
    BingInsideSum,
    BingPerDeviceConjunction,
}
const ARMS: [Arm; 4] = [
    Arm::JiaPoolOnce,
    Arm::YiPoolTimesReplicas,
    Arm::BingInsideSum,
    Arm::BingPerDeviceConjunction,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Demand {
    LogicalBytes,
    DiskBytes,
}
const DEMANDS: [Demand; 2] = [Demand::LogicalBytes, Demand::DiskBytes];

/// D3（空间分配） 已定项 12 的第二道闸（D16（发布语义） 已定项 1 的准入行）按什么单位读。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeriesGate {
    Absent,
    D16PerCopy,
    D16Summed,
}
const GATES: [SeriesGate; 3] = [SeriesGate::Absent, SeriesGate::D16PerCopy, SeriesGate::D16Summed];

#[derive(Clone, Copy, Debug)]
struct PoolState {
    free_slots: [i64; 2],
    in_flight_units: i64,
    request_units: i64,
    checkpoint_cost_slots: i64,
}

fn arm_name(arm: Arm) -> &'static str {
    match arm {
        Arm::JiaPoolOnce => "jia_pool_once",
        Arm::YiPoolTimesReplicas => "yi_pool_times_2",
        Arm::BingInsideSum => "bing_inside_sum",
        Arm::BingPerDeviceConjunction => "bing_per_device_conjunction",
    }
}
fn demand_name(demand: Demand) -> &'static str {
    match demand {
        Demand::LogicalBytes => "logical",
        Demand::DiskBytes => "disk",
    }
}
fn gate_name(gate: SeriesGate) -> &'static str {
    match gate {
        SeriesGate::Absent => "none",
        SeriesGate::D16PerCopy => "d16_per_copy",
        SeriesGate::D16Summed => "d16_summed",
    }
}

fn demand_slots(demand: Demand, units: i64) -> i64 {
    match demand {
        Demand::LogicalBytes => units,
        Demand::DiskBytes => 2 * units,
    }
}

fn fixpoint_fits_on_every_device(state: &PoolState) -> bool {
    state
        .free_slots
        .iter()
        .all(|free| *free >= state.in_flight_units + state.request_units + state.checkpoint_cost_slots)
}

/// D28（挂载期承诺量） 已定项 1 的「可用 ≥ 需求」，第八项按臂扣。
fn d28_admits(arm: Arm, demand: Demand, state: &PoolState) -> bool {
    let requested = demand_slots(demand, state.in_flight_units + state.request_units);
    let summed_free = state.free_slots[0] + state.free_slots[1];
    let cost = state.checkpoint_cost_slots;
    match arm {
        Arm::JiaPoolOnce => summed_free - cost >= requested,
        Arm::YiPoolTimesReplicas => summed_free - 2 * cost >= requested,
        Arm::BingInsideSum => (state.free_slots[0] - cost) + (state.free_slots[1] - cost) >= requested,
        // 按设备逐块判：每块盘要的是它自己那一份，「需求」两种读法在这里不分。
        Arm::BingPerDeviceConjunction => state
            .free_slots
            .iter()
            .all(|free| free - cost >= state.in_flight_units + state.request_units),
    }
}

/// D16（发布语义） 已定项 1：可分配 = min(可再分配 + 活元数据 − 保留池, df)，保留池 10 + 7 c_max；
/// 活元数据取 E139（按盘回退下界的收严形态） 装置的读法（上一次发布的元数据 = c），稳态无滞后、无待释放 ⇒ 这道闸 = df。
fn d16_gate_admits(gate: SeriesGate, demand: Demand, state: &PoolState) -> bool {
    let cost = state.checkpoint_cost_slots;
    let reserve = 10 + 7 * cost;
    let residue = 5 + 6 * cost;
    let wanted_units = state.in_flight_units + state.request_units;
    match gate {
        SeriesGate::Absent => true,
        SeriesGate::D16PerCopy => {
            let smaller = state.free_slots[0].min(state.free_slots[1]);
            smaller + cost - reserve - residue >= wanted_units
        }
        SeriesGate::D16Summed => {
            let summed = state.free_slots[0] + state.free_slots[1];
            summed + 2 * cost - reserve - residue >= demand_slots(demand, wanted_units)
        }
    }
}

/// 例子按 (两盘空闲之差, 两盘空闲之和, 请求, 在飞) 字典序取最小：短盘的例子先看差有多小。
type Example = (i64, i64, i64, i64, i64, i64);

#[derive(Default, Clone, Copy)]
struct Tally {
    states: u64,
    t1_equal_data_fits: u64,
    t1_short_data_fits: u64,
    t1_data_itself_does_not_fit: u64,
    t2_total: u64,
    t2_caused_by_term_eight: u64,
    smallest_t1_equal: Option<Example>,
    smallest_t1_short: Option<Example>,
    smallest_t2_term_eight: Option<Example>,
}

fn keep_smaller(slot: &mut Option<Example>, state: &PoolState) {
    let candidate = (
        (state.free_slots[0] - state.free_slots[1]).abs(),
        state.free_slots[0] + state.free_slots[1],
        state.request_units,
        state.in_flight_units,
        state.free_slots[0],
        state.free_slots[1],
    );
    if slot.map_or(true, |current| candidate < current) {
        *slot = Some(candidate);
    }
}

fn describe(example: Option<Example>) -> String {
    example.map_or("none".to_string(), |(_, _, request, in_flight, free0, free1)| {
        format!("free0={free0}:free1={free1}:in_flight={in_flight}:request={request}")
    })
}

fn main() {
    let costs = [1i64, 2, 3, 4, 5, 9];
    let mut sum_versus_times_two_disagreements = 0u64;
    let mut compared = 0u64;
    for cost in costs {
        for arm in ARMS {
            for demand in DEMANDS {
                for gate in GATES {
                    let mut tally = Tally::default();
                    for free0 in 0..=80i64 {
                        for free1 in 0..=80i64 {
                            for in_flight_units in 0..=3i64 {
                                for request_units in 1..=40i64 {
                                    let state = PoolState {
                                        free_slots: [free0, free1],
                                        in_flight_units,
                                        request_units,
                                        checkpoint_cost_slots: cost,
                                    };
                                    tally.states += 1;
                                    let gate_admits = d16_gate_admits(gate, demand, &state);
                                    let term_eight_admits = d28_admits(arm, demand, &state);
                                    let admitted = term_eight_admits && gate_admits;
                                    let fits = fixpoint_fits_on_every_device(&state);
                                    let data_fits = state.free_slots.iter().all(|free| *free >= in_flight_units + request_units);
                                    if admitted && !fits {
                                        if !data_fits {
                                            tally.t1_data_itself_does_not_fit += 1;
                                        } else if free0 == free1 {
                                            tally.t1_equal_data_fits += 1;
                                            keep_smaller(&mut tally.smallest_t1_equal, &state);
                                        } else {
                                            tally.t1_short_data_fits += 1;
                                            keep_smaller(&mut tally.smallest_t1_short, &state);
                                        }
                                    }
                                    if !admitted && fits {
                                        tally.t2_total += 1;
                                        if gate_admits && !term_eight_admits {
                                            tally.t2_caused_by_term_eight += 1;
                                            keep_smaller(&mut tally.smallest_t2_term_eight, &state);
                                        }
                                    }
                                    if arm == Arm::YiPoolTimesReplicas && gate == SeriesGate::Absent {
                                        compared += 1;
                                        if d28_admits(Arm::YiPoolTimesReplicas, demand, &state) != d28_admits(Arm::BingInsideSum, demand, &state) {
                                            sum_versus_times_two_disagreements += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    println!(
                        "Q1RESULT name=grid c={cost} arm={} demand={} series_gate={} states={} t1_equal_data_fits={} t1_short_data_fits={} t1_data_itself_does_not_fit={} t2_total={} t2_caused_by_term8={} t1_equal_smallest={} t1_short_smallest={} t2_term8_smallest={}",
                        arm_name(arm), demand_name(demand), gate_name(gate), tally.states,
                        tally.t1_equal_data_fits, tally.t1_short_data_fits, tally.t1_data_itself_does_not_fit,
                        tally.t2_total, tally.t2_caused_by_term_eight,
                        describe(tally.smallest_t1_equal), describe(tally.smallest_t1_short), describe(tally.smallest_t2_term_eight)
                    );
                }
            }
        }
    }
    println!("Q1RESULT name=yi_versus_bing_inside_sum states={compared} disagreements={sum_versus_times_two_disagreements}");
    assert_eq!(sum_versus_times_two_disagreements, 0, "两块盘时 Σ(free_d − c) 与 Σfree − 2c 是同一个数");
    df_cells();
}

/// D3（空间分配） 已定项 9 第 1 条：df ≥ s ⇒ 写 s 必须成功。两盘等量 f，挂载期承诺量里的切换预留每块盘
/// S = (N_switch + 1) × (链重写 2 + R × c) = 8 + 12c（D28（挂载期承诺量） 已定项 3，rows0 = 0）。
/// 读法一 df = D16（发布语义） 已定项 1 那一行的式子（不扣切换预留）；读法二 df = D28 的可用（已定项 3「df 报可用」）折成用户槽；
/// 读法三 df = 两者取小。请求按盘字节计。
fn df_cells() {
    for arm in ARMS {
        for reading in ["df_is_d16_row", "df_is_d28_available", "df_is_min_of_both"] {
            let mut first_cost_with_cell: Option<i64> = None;
            let mut cells_at_cost = [0u64; 2];
            let mut smallest_at_nine: Option<(i64, i64)> = None;
            for cost in 1..=30i64 {
                let switch_reserve = 8 + 12 * cost;
                let mut cells = 0u64;
                for free in 0..=600i64 {
                    let d16_df = free + cost - (10 + 7 * cost) - (5 + 6 * cost);
                    let after_mount_commitment = 2 * (free - switch_reserve);
                    let d28_available_slots = match arm {
                        Arm::JiaPoolOnce => after_mount_commitment - cost,
                        Arm::YiPoolTimesReplicas | Arm::BingInsideSum => after_mount_commitment - 2 * cost,
                        Arm::BingPerDeviceConjunction => after_mount_commitment - 2 * cost,
                    };
                    let d28_df = d28_available_slots.div_euclid(2);
                    let reported_df = match reading {
                        "df_is_d16_row" => d16_df,
                        "df_is_d28_available" => d28_df,
                        _ => d16_df.min(d28_df),
                    };
                    for request in 1..=600i64 {
                        let d28_ok = d28_available_slots >= 2 * request;
                        let d16_ok = d16_df >= request;
                        if reported_df >= request && !(d28_ok && d16_ok) {
                            cells += 1;
                            if cost == 9 && smallest_at_nine.map_or(true, |(best_free, _)| free < best_free) {
                                smallest_at_nine = Some((free, request));
                            }
                        }
                    }
                }
                if cells > 0 && first_cost_with_cell.is_none() {
                    first_cost_with_cell = Some(cost);
                }
                if cost == 4 { cells_at_cost[0] = cells }
                if cost == 9 { cells_at_cost[1] = cells }
            }
            println!(
                "Q1RESULT name=df_false_enospc arm={} reading={reading} first_c_with_cell={} cells_c4={} cells_c9={} smallest_c9={}",
                arm_name(arm),
                first_cost_with_cell.map_or("none".to_string(), |cost| cost.to_string()),
                cells_at_cost[0], cells_at_cost[1],
                smallest_at_nine.map_or("none".to_string(), |(free, request)| format!("free_each={free}:request={request}"))
            );
        }
    }
    println!("Q1RESULT name=done");
}
