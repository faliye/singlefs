//! C355（checkpoint 保留池池级扣而固定点每块要落两列）第二轮攻方腿，判据 V4：「需求」取盘字节时两份 `df` 怎么写，
//! 才不在甲 / 乙 / 丙′ 上出假性 ENOSPC（D3（空间分配） 已定项 9 第 1 条：`df` 报出的空闲 ≥ s，写 s 字节就必须成功）。只用 std，确定性。
//!
//! 单位：16 KiB 槽。两块盘。用户写 u 个数据单元（每份 2 槽，两盘各一份）。每块盘的量：
//! unused_d = 没用过的 + 已可再分配的；pinned = 已释放、推空发布抬 F 之后能回来的；lag = 滞后量（D16（发布语义） 已定项 1 的式子，
//! 含正在攒的窗口放掉的）；current = 正在攒的窗口里的释放（D28（挂载期承诺量） 已定项 1 的「defer 待释放」，current ≤ lag）；
//! live = 活元数据（取 E139（按盘回退下界的收严形态） 装置 `live_metadata_credit` 的读法：最近一次发布的元数据块）；c = ckpt_cost = c_max；
//! 切换预留每块盘 8 + 12c（D28（挂载期承诺量） 已定项 3 第一个事务几何：链重写 8 + (3 + 1) × 3 × c）；
//! k = 池级承诺的墓碑容器个数（每份 2 槽）；prepay = 这次写要预付的「解开自己」每份槽数（D3（空间分配） 已定项 2，量没有条款，取 0 / 2 两档）。
//!
//! 两道闸（D3（空间分配） 已定项 12：先过 D28 的「可用 ≥ 需求」，再过 D16 的抬 F 逻辑）：
//! D28 闸按臂；D16 闸取 E139 的性质——推满发布之后「可分配」就是这条臂自己的 `df`（E139 判决：各格按这条臂自己的 `df` 假性 ENOSPC 为 0），
//! 所以 D16 闸 ⇔ df16 ≥ 需求。df16 = unused + pinned + live − (10 + 7c) − (5 + 6c)（D16（发布语义） 已定项 1 的 `df` 式子，lag 两边相消）。
//! 假设（写明）：推空发布不改变 D28 闸的读数；D28 闸与 D16 闸在准入那一刻各读一次。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm { JiaPoolOnce, YiPoolTimesTwo, BingPrimePerDevice }
const ARMS: [Arm; 3] = [Arm::JiaPoolOnce, Arm::YiPoolTimesTwo, Arm::BingPrimePerDevice];

/// D28 式子里「已分配」的两种读法：只算仍分配的；或按 checker 读法甲（根环里全部有效根引用的并集，`02-second-txn.md` 步 2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocatedReading { LiveOnly, CheckerReadingJia }
const ALLOCATED_READINGS: [AllocatedReading; 2] = [AllocatedReading::LiveOnly, AllocatedReading::CheckerReadingJia];

/// D16 那一道闸（与它的 `df`）在两块盘上怎么读：E139 装置没有设备维，取小的那块盘或两块之和的一半。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum D16DeviceReading { SmallerDevice, HalfOfSum }
const D16_READINGS: [D16DeviceReading; 2] = [D16DeviceReading::SmallerDevice, D16DeviceReading::HalfOfSum];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DfWriting {
    /// `28-挂载期承诺量.md:90`「`df` 报可用」，按这条臂自己的「可用」，盘字节不折。
    D28RawAvailable,
    /// 同上折成一份副本（丙′ 取逐设备最小）。
    D28PerCopyOwnArm,
    /// `16-发布语义.md:378` 的 `df`，按小的那块盘。
    D16SmallerDevice,
    /// 同上按两块之和的一半。
    D16HalfOfSum,
    /// 两者取小：这条臂自己的一份副本形态 ∧ D16 取小盘。
    MinOwnArmAndD16Smaller,
    /// 与臂无关的一种写法：逐设备形态 ∧ D16 取小盘。
    MinPerDeviceFormAndD16Smaller,
    /// 与臂无关的另一种：求和形态（乙）∧ D16 取一半。
    MinSummedFormAndD16Half,
}
const WRITINGS: [DfWriting; 7] = [
    DfWriting::D28RawAvailable,
    DfWriting::D28PerCopyOwnArm,
    DfWriting::D16SmallerDevice,
    DfWriting::D16HalfOfSum,
    DfWriting::MinOwnArmAndD16Smaller,
    DfWriting::MinPerDeviceFormAndD16Smaller,
    DfWriting::MinSummedFormAndD16Half,
];

const DATA_SLOTS_PER_COPY: i64 = 2;
const TOMBSTONE_SLOTS_PER_COPY: i64 = 2;

#[derive(Clone, Copy, Debug)]
struct State {
    unused: [i64; 2],
    pinned: i64,
    lag: i64,
    current: i64,
    live: i64,
    cost: i64,
    containers: i64,
    request_units: i64,
    prepay: i64,
}

fn switch_reserve(cost: i64) -> i64 { 8 + 12 * cost }
fn d16_reserve(cost: i64) -> i64 { 10 + 7 * cost }
fn d16_residue(cost: i64) -> i64 { 5 + 6 * cost }

/// 每块盘括号里六项之后的量。
fn bracket(state: &State, reading: AllocatedReading, device: usize) -> i64 {
    match reading {
        // 容量 − 已分配 = unused + pinned + lag；再减 defer 待释放（current）与切换预留。
        AllocatedReading::LiveOnly => state.unused[device] + state.pinned + state.lag - state.current - switch_reserve(state.cost),
        // 已分配含全部已释放而仍被根引用的（pinned + lag），再减 defer 待释放：current 扣两次。
        AllocatedReading::CheckerReadingJia => state.unused[device] - state.current - switch_reserve(state.cost),
    }
}

fn need_per_copy(state: &State) -> i64 { DATA_SLOTS_PER_COPY * state.request_units + state.prepay }
fn pool_per_copy(state: &State) -> i64 { TOMBSTONE_SLOTS_PER_COPY * state.containers }
fn pool_both_copies(state: &State) -> i64 { 2 * pool_per_copy(state) }

fn d28_available_raw(arm: Arm, state: &State, reading: AllocatedReading) -> i64 {
    let summed = bracket(state, reading, 0) + bracket(state, reading, 1);
    match arm {
        Arm::JiaPoolOnce => summed - pool_both_copies(state) - state.cost,
        Arm::YiPoolTimesTwo => summed - pool_both_copies(state) - 2 * state.cost,
        Arm::BingPrimePerDevice => (0..2).map(|device| bracket(state, reading, device) - pool_per_copy(state) - state.cost).sum(),
    }
}

fn d28_per_copy(arm: Arm, state: &State, reading: AllocatedReading) -> i64 {
    match arm {
        Arm::JiaPoolOnce | Arm::YiPoolTimesTwo => d28_available_raw(arm, state, reading).div_euclid(2),
        Arm::BingPrimePerDevice => (0..2).map(|device| bracket(state, reading, device) - pool_per_copy(state) - state.cost).min().expect("两块盘"),
    }
}

fn d28_gate(arm: Arm, state: &State, reading: AllocatedReading) -> bool {
    match arm {
        Arm::JiaPoolOnce | Arm::YiPoolTimesTwo => d28_available_raw(arm, state, reading) >= 2 * need_per_copy(state),
        Arm::BingPrimePerDevice => d28_per_copy(arm, state, reading) >= need_per_copy(state),
    }
}

fn df16_device(state: &State, device: usize) -> i64 {
    state.unused[device] + state.pinned + state.live - d16_reserve(state.cost) - d16_residue(state.cost)
}

fn df16(state: &State, reading: D16DeviceReading) -> i64 {
    match reading {
        D16DeviceReading::SmallerDevice => df16_device(state, 0).min(df16_device(state, 1)),
        D16DeviceReading::HalfOfSum => (df16_device(state, 0) + df16_device(state, 1)).div_euclid(2),
    }
}

fn admitted(arm: Arm, state: &State, allocated: AllocatedReading, d16: D16DeviceReading) -> bool {
    d28_gate(arm, state, allocated) && df16(state, d16) >= need_per_copy(state)
}

/// 放得下（乐观：不计推空发布自己的开销）：每块盘上 unused + pinned 装得下数据与固定点。乐观真值判「放不下」的，一定放不下。
fn placement_fits(state: &State) -> bool {
    (0..2).all(|device| state.unused[device] + state.pinned >= DATA_SLOTS_PER_COPY * state.request_units + state.cost)
}

fn reported_df(writing: DfWriting, arm: Arm, state: &State, allocated: AllocatedReading) -> i64 {
    let per_device_form = d28_per_copy(Arm::BingPrimePerDevice, state, allocated);
    let summed_form = d28_per_copy(Arm::YiPoolTimesTwo, state, allocated);
    match writing {
        DfWriting::D28RawAvailable => d28_available_raw(arm, state, allocated),
        DfWriting::D28PerCopyOwnArm => d28_per_copy(arm, state, allocated),
        DfWriting::D16SmallerDevice => df16(state, D16DeviceReading::SmallerDevice),
        DfWriting::D16HalfOfSum => df16(state, D16DeviceReading::HalfOfSum),
        DfWriting::MinOwnArmAndD16Smaller => d28_per_copy(arm, state, allocated).min(df16(state, D16DeviceReading::SmallerDevice)),
        DfWriting::MinPerDeviceFormAndD16Smaller => per_device_form.min(df16(state, D16DeviceReading::SmallerDevice)),
        DfWriting::MinSummedFormAndD16Half => summed_form.min(df16(state, D16DeviceReading::HalfOfSum)),
    }
}

/// 「df ≥ s 而写不成」：闸拒绝（gate），或闸放行而放不下（place）。
fn classify(writing: DfWriting, arm: Arm, state: &State, allocated: AllocatedReading, d16: D16DeviceReading) -> (bool, bool) {
    let user_bytes_in_slots = DATA_SLOTS_PER_COPY * state.request_units;
    if reported_df(writing, arm, state, allocated) < user_bytes_in_slots {
        return (false, false);
    }
    if !admitted(arm, state, allocated, d16) {
        return (true, false);
    }
    (false, !placement_fits(state))
}

fn self_tests() {
    let base = State { unused: [200, 200], pinned: 0, lag: 0, current: 0, live: 0, cost: 4, containers: 0, request_units: 70, prepay: 0 };
    // 两盘各 200、c = 4：切换预留 56 ⇒ 括号 144；乙的可用 288 − 8 = 280 盘字节；df16 = 200 − 38 − 29 = 133。
    assert_eq!(d28_available_raw(Arm::YiPoolTimesTwo, &base, AllocatedReading::LiveOnly), 280);
    assert_eq!(df16(&base, D16DeviceReading::SmallerDevice), 133);
    // 写 70 个单元（140 槽）：`df` 报 280 ≥ 140，D28 闸 280 ≥ 280 过，D16 闸 133 < 140 拒绝 ⇒ 按 D28 那一份的 `df` 是假性 ENOSPC。
    assert_eq!(classify(DfWriting::D28RawAvailable, Arm::YiPoolTimesTwo, &base, AllocatedReading::LiveOnly, D16DeviceReading::SmallerDevice), (true, false));
    // 两者取小：min(140, 133) = 133 < 140，不报这个空闲，就不是假性 ENOSPC。
    assert_eq!(classify(DfWriting::MinOwnArmAndD16Smaller, Arm::YiPoolTimesTwo, &base, AllocatedReading::LiveOnly, D16DeviceReading::SmallerDevice), (false, false));
    // checker 读法甲、两盘各 100、pinned 8、live 4、c = 4、写 21 个单元：丙′ 的逐设备 = 100 − 56 − 4 = 40 < 42 拒绝；
    // df16 = 100 + 8 + 4 − 67 = 45 ≥ 42 ⇒ 按 D16 那一份的 `df` 是假性 ENOSPC。
    let jia = State { unused: [100, 100], pinned: 8, live: 4, request_units: 21, ..base };
    assert_eq!(classify(DfWriting::D16SmallerDevice, Arm::BingPrimePerDevice, &jia, AllocatedReading::CheckerReadingJia, D16DeviceReading::SmallerDevice), (true, false));
    // 短盘：300 / 20、写 10 个单元、D16 按一半读：求和形态 min((244 − 36 − 8) / 2, (233 − 47) / 2) = min(100, 93) = 93 ≥ 20；
    // 甲的两道闸都过；盘 1 要 20 + 4 = 24 只有 20 ⇒ 放行而放不下。
    let short = State { unused: [300, 20], request_units: 10, ..base };
    assert_eq!(reported_df(DfWriting::MinSummedFormAndD16Half, Arm::JiaPoolOnce, &short, AllocatedReading::LiveOnly), 93);
    assert_eq!(classify(DfWriting::MinSummedFormAndD16Half, Arm::JiaPoolOnce, &short, AllocatedReading::LiveOnly, D16DeviceReading::HalfOfSum), (false, true));
    // 预付 2 槽：两盘各 120、c = 4，逐设备形态 120 − 56 − 4 = 60、df16 = 53 ⇒ 报 53；写 26 个单元（52 ≤ 53），需求 52 + 2 = 54 > 53 ⇒ D16 闸拒绝。
    let prepay = State { unused: [120, 120], request_units: 26, prepay: 2, ..base };
    assert_eq!(classify(DfWriting::MinPerDeviceFormAndD16Smaller, Arm::BingPrimePerDevice, &prepay, AllocatedReading::LiveOnly, D16DeviceReading::SmallerDevice), (true, false));
    println!("V4RESULT name=selftest passed=9");
}

#[derive(Default, Clone)]
struct Tally { gate: u64, gate_min: Option<(i64, String)>, place: u64, place_min: Option<(i64, String)> }

fn note(slot: &mut Option<(i64, String)>, weight: i64, text: &str) {
    if slot.as_ref().is_none_or(|(best, _)| weight < *best) { *slot = Some((weight, text.to_string())); }
}
fn show(slot: &Option<(i64, String)>) -> String { slot.as_ref().map_or_else(|| "none".to_string(), |(_, text)| text.clone()) }

fn main() {
    self_tests();
    for prepay in [0i64, 2] {
        for allocated in ALLOCATED_READINGS {
            for d16 in D16_READINGS {
                let mut tallies = vec![Tally::default(); ARMS.len() * WRITINGS.len()];
                let mut states = 0u64;
                for unused0 in (0..=320i64).step_by(4) {
                    for unused1 in (0..=320i64).step_by(4) {
                        for pinned in [0i64, 8] {
                            for (lag, current) in [(0i64, 0i64), (6, 0), (6, 2)] {
                                for cost in [4i64, 9] {
                                    for live in [0, cost] {
                                        for containers in [0i64, 1] {
                                            for request_units in 1..=4i64 {
                                                states += 1;
                                                let state = State { unused: [unused0, unused1], pinned, lag, current, live, cost, containers, request_units, prepay };
                                                let weight = (unused0 - unused1).abs() * 1_000_000 + (unused0 + unused1) * 100 + request_units;
                                                let text = format!("unused={unused0}/{unused1}:pinned={pinned}:lag={lag}:current={current}:live={live}:cost={cost}:containers={containers}:request_units={request_units}");
                                                for (arm_position, arm) in ARMS.iter().enumerate() {
                                                    for (writing_position, writing) in WRITINGS.iter().enumerate() {
                                                        let (gate, place) = classify(*writing, *arm, &state, allocated, d16);
                                                        let tally = &mut tallies[arm_position * WRITINGS.len() + writing_position];
                                                        if gate { tally.gate += 1; note(&mut tally.gate_min, weight, &text); }
                                                        if place { tally.place += 1; note(&mut tally.place_min, weight, &text); }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                for (writing_position, writing) in WRITINGS.iter().enumerate() {
                    let mut all_zero = true;
                    for (arm_position, arm) in ARMS.iter().enumerate() {
                        let tally = &tallies[arm_position * WRITINGS.len() + writing_position];
                        all_zero &= tally.gate == 0 && tally.place == 0;
                        println!(
                            "V4RESULT name=false_enospc prepay={prepay} allocated={allocated:?} d16_gate={d16:?} writing={writing:?} arm={arm:?} states={states} gate_rejects={} gate_min={} placement_fails={} placement_min={}",
                            tally.gate, show(&tally.gate_min), tally.place, show(&tally.place_min)
                        );
                    }
                    println!("V4RESULT name=writing_summary prepay={prepay} allocated={allocated:?} d16_gate={d16:?} writing={writing:?} zero_on_all_three_arms={all_zero}");
                }
            }
        }
    }
    println!("V4RESULT name=done");
}
