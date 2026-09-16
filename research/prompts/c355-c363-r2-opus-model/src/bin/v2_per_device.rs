//! C355（checkpoint 保留池池级扣而固定点每块要落两列）第二轮攻方腿，判据 V2：丙′（逐设备合取）自己的死角。只用 std，纯算术、确定性。
//!
//! 单位：16 KiB 槽。数据单元每份 2 槽（D18（块里携带什么信息） 已定项 9 的 32 KiB），固定点节点每份 1 槽，
//! 墓碑容器每份 2 槽（码 3 打包单元 32 KiB，D18（块里携带什么信息） 已定项 8 / 已定项 10）；每个单元 w = 2 份、落两块不同的盘。
//! a_d = 盘 d 已扣括号里六项之后的空闲槽；c = ckpt_cost（固定点每个节点两份）；
//! k = 池级承诺折成的墓碑容器个数（已承诺预留里的墓碑、待删占用都按「将来要在两块盘上各占 2 槽」建）；u = 这次请求的数据单元数。
//! 真值只看槽数够不够（落点政策、段、对齐都不建）；「分得到」= 数据、固定点、墓碑容器按 D2（RAID 条带策略） 已定项 8 的
//! 「取最空的 w 块」依次放下（贪心），另报「存在任何一种放法」作对照。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LedgerUnit {
    /// 统计量第 4 / 6 项按一份副本记字节（两块盘各一份时 = 每块盘要的量）。
    PerCopy,
    /// 统计量按全部副本之和记字节（与括号里按设备求和的那几项同一个「盘字节」口径）。
    BothCopies,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TwoDeviceVariant {
    /// ∀d：a_d − c − S ≥ 需求，S 是池级标量原值（每块盘都扣整份）。
    EachDeviceWholeScalar,
    /// ∀d：a_d − c − S / 设备数 ≥ 需求。
    EachDeviceScalarOverDevices,
    /// ∀d：a_d − c ≥ 需求 ∧ Σa − 2c − S ≥ 2 × 需求（池级项留在式子外面，照今天的形状从 Σ 里减）。
    PoolTermsKeptInSum,
}

const TWO_DEVICE_VARIANTS: [TwoDeviceVariant; 3] = [
    TwoDeviceVariant::EachDeviceWholeScalar,
    TwoDeviceVariant::EachDeviceScalarOverDevices,
    TwoDeviceVariant::PoolTermsKeptInSum,
];
const LEDGER_UNITS: [LedgerUnit; 2] = [LedgerUnit::PerCopy, LedgerUnit::BothCopies];

const DATA_SLOTS_PER_COPY: i64 = 2;
const TOMBSTONE_SLOTS_PER_COPY: i64 = 2;

fn ledger_scalar_slots(unit: LedgerUnit, containers: i64) -> i64 {
    match unit {
        LedgerUnit::PerCopy => TOMBSTONE_SLOTS_PER_COPY * containers,
        LedgerUnit::BothCopies => 2 * TOMBSTONE_SLOTS_PER_COPY * containers,
    }
}

/// 两块盘时真值：每块盘都要放下自己那一份数据、固定点、墓碑容器。
fn two_device_truth(free: [i64; 2], cost: i64, containers: i64, request_units: i64) -> bool {
    free.iter().all(|slots| *slots >= DATA_SLOTS_PER_COPY * request_units + cost + TOMBSTONE_SLOTS_PER_COPY * containers)
}

fn two_device_admits(variant: TwoDeviceVariant, unit: LedgerUnit, free: [i64; 2], cost: i64, containers: i64, request_units: i64) -> bool {
    let scalar = ledger_scalar_slots(unit, containers);
    let per_device_need = DATA_SLOTS_PER_COPY * request_units;
    match variant {
        TwoDeviceVariant::EachDeviceWholeScalar => free.iter().all(|slots| slots - cost - scalar >= per_device_need),
        // 除以设备数用乘法写，避免取整：2 × (a_d − c − 需求) ≥ S。
        TwoDeviceVariant::EachDeviceScalarOverDevices => free.iter().all(|slots| 2 * (slots - cost - per_device_need) >= scalar),
        TwoDeviceVariant::PoolTermsKeptInSum => {
            free.iter().all(|slots| slots - cost >= per_device_need) && free[0] + free[1] - 2 * cost - scalar >= 2 * per_device_need
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreeDeviceVariant {
    /// ∀d（三块全判）：a_d − c − 2k ≥ 2u。
    AllDevices,
    /// 只判准入那一刻最空的两块（D2（RAID 条带策略） 已定项 8 会选的那两块）：a_d − c − 2k ≥ 2u。
    SelectedDevices,
    /// Σ 形（乙推广到三块盘）：Σa − 2c − 4k ≥ 4u。
    Summed,
    /// 被选中的两块判数据与固定点，池级项留在 Σ 里：a_d − c ≥ 2u ∧ Σa − 2c − 4k ≥ 4u。
    SelectedWithPoolTermsInSum,
}

const THREE_DEVICE_VARIANTS: [ThreeDeviceVariant; 4] = [
    ThreeDeviceVariant::AllDevices,
    ThreeDeviceVariant::SelectedDevices,
    ThreeDeviceVariant::Summed,
    ThreeDeviceVariant::SelectedWithPoolTermsInSum,
];

/// 准入那一刻最空的两块（并列取编号小的）。
fn two_emptiest(free: [i64; 3]) -> [usize; 2] {
    let mut order = [0usize, 1, 2];
    order.sort_by(|left, right| free[*right].cmp(&free[*left]).then(left.cmp(right)));
    [order[0], order[1]]
}

fn three_device_admits(variant: ThreeDeviceVariant, free: [i64; 3], cost: i64, containers: i64, request_units: i64) -> bool {
    let need = DATA_SLOTS_PER_COPY * request_units;
    let pool_per_copy = TOMBSTONE_SLOTS_PER_COPY * containers;
    let summed: i64 = free.iter().sum();
    match variant {
        ThreeDeviceVariant::AllDevices => free.iter().all(|slots| slots - cost - pool_per_copy >= need),
        ThreeDeviceVariant::SelectedDevices => two_emptiest(free).iter().all(|device| free[*device] - cost - pool_per_copy >= need),
        ThreeDeviceVariant::Summed => summed - 2 * cost - 2 * pool_per_copy >= 2 * need,
        ThreeDeviceVariant::SelectedWithPoolTermsInSum => {
            two_emptiest(free).iter().all(|device| free[*device] - cost >= need) && summed - 2 * cost - 2 * pool_per_copy >= 2 * need
        }
    }
}

/// 按 D2（RAID 条带策略） 已定项 8 依次放：每个单元取当时剩得最多、且放得下的两块盘。
fn place_greedily(free: [i64; 3], items_in_order: &[i64]) -> bool {
    let mut remaining = free;
    for size in items_in_order {
        let mut order = [0usize, 1, 2];
        order.sort_by(|left, right| remaining[*right].cmp(&remaining[*left]).then(left.cmp(right)));
        let chosen: Vec<usize> = order.iter().copied().filter(|device| remaining[*device] >= *size).take(2).collect();
        if chosen.len() < 2 {
            return false;
        }
        for device in chosen {
            remaining[device] -= size;
        }
    }
    true
}

/// 存在任何一种放法：每个单元「跳过」一块盘，盘 d 的负载 = 总量 − 跳过 d 的那批之和。
fn placement_exists(free: [i64; 3], twos: i64, ones: i64) -> bool {
    let total = 2 * twos + ones;
    let lower = free.map(|slots| (total - slots).max(0));
    for twos_skipping_0 in 0..=twos {
        for twos_skipping_1 in 0..=(twos - twos_skipping_0) {
            let twos_skipping_2 = twos - twos_skipping_0 - twos_skipping_1;
            for ones_skipping_0 in 0..=ones {
                for ones_skipping_1 in 0..=(ones - ones_skipping_0) {
                    let ones_skipping_2 = ones - ones_skipping_0 - ones_skipping_1;
                    let skipped = [
                        2 * twos_skipping_0 + ones_skipping_0,
                        2 * twos_skipping_1 + ones_skipping_1,
                        2 * twos_skipping_2 + ones_skipping_2,
                    ];
                    if (0..3).all(|device| skipped[device] >= lower[device]) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn three_device_greedy_truth(free: [i64; 3], cost: i64, containers: i64, request_units: i64) -> bool {
    let mut items = Vec::new();
    items.extend(std::iter::repeat_n(DATA_SLOTS_PER_COPY, usize::try_from(request_units).expect("u")));
    items.extend(std::iter::repeat_n(1, usize::try_from(cost).expect("c")));
    items.extend(std::iter::repeat_n(TOMBSTONE_SLOTS_PER_COPY, usize::try_from(containers).expect("k")));
    place_greedily(free, &items)
}

#[derive(Default)]
struct Tally {
    unsafe_admits: u64,
    unsafe_minimum: Option<(i64, String)>,
    false_rejects: u64,
    false_reject_minimum: Option<(i64, String)>,
}

impl Tally {
    fn note(slot: &mut Option<(i64, String)>, weight: i64, text: String) {
        if slot.as_ref().is_none_or(|(best, _)| weight < *best) {
            *slot = Some((weight, text));
        }
    }
    fn text(slot: &Option<(i64, String)>) -> String {
        slot.as_ref().map_or_else(|| "none".to_string(), |(_, text)| text.clone())
    }
}

fn two_device_sweep() {
    for cost in [1i64, 4, 9] {
        for unit in LEDGER_UNITS {
            for variant in TWO_DEVICE_VARIANTS {
                let mut tally = Tally::default();
                let mut states = 0u64;
                for free0 in 0..=40i64 {
                    for free1 in 0..=40i64 {
                        for containers in 0..=4i64 {
                            for request_units in 1..=8i64 {
                                states += 1;
                                let free = [free0, free1];
                                let admitted = two_device_admits(variant, unit, free, cost, containers, request_units);
                                let truth = two_device_truth(free, cost, containers, request_units);
                                // 例子按 (两盘差, 两盘和, u, k) 取最小。
                                let weight = ((free0 - free1).abs() * 1_000_000) + (free0 + free1) * 1000 + request_units * 10 + containers;
                                let text = format!("free0={free0}:free1={free1}:containers={containers}:request_units={request_units}");
                                if admitted && !truth {
                                    tally.unsafe_admits += 1;
                                    Tally::note(&mut tally.unsafe_minimum, weight, text);
                                } else if !admitted && truth {
                                    tally.false_rejects += 1;
                                    Tally::note(&mut tally.false_reject_minimum, weight, text);
                                }
                            }
                        }
                    }
                }
                println!(
                    "V2RESULT name=two_devices cost={cost} ledger={unit:?} variant={variant:?} states={states} unsafe_admits={} unsafe_min={} false_rejects={} false_reject_min={}",
                    tally.unsafe_admits, Tally::text(&tally.unsafe_minimum), tally.false_rejects, Tally::text(&tally.false_reject_minimum)
                );
            }
        }
    }
}

fn three_device_sweep() {
    for cost in [1i64, 3] {
        let mut tallies: Vec<(Tally, Tally)> = THREE_DEVICE_VARIANTS.iter().map(|_| (Tally::default(), Tally::default())).collect();
        let mut states = 0u64;
        let mut greedy_loses = 0u64;
        let mut greedy_loses_minimum: Option<(i64, String)> = None;
        for free0 in 0..=20i64 {
            for free1 in 0..=20i64 {
                for free2 in 0..=20i64 {
                    for containers in 0..=2i64 {
                        for request_units in 1..=5i64 {
                            states += 1;
                            let free = [free0, free1, free2];
                            let greedy = three_device_greedy_truth(free, cost, containers, request_units);
                            let exists = placement_exists(free, request_units + containers, cost);
                            assert!(!greedy || exists, "贪心放得下就一定存在放法");
                            let weight = (free0 + free1 + free2) * 1000 + request_units * 10 + containers;
                            let text = format!("free={free0}/{free1}/{free2}:containers={containers}:request_units={request_units}");
                            if exists && !greedy {
                                greedy_loses += 1;
                                Tally::note(&mut greedy_loses_minimum, weight, text.clone());
                            }
                            for (position, variant) in THREE_DEVICE_VARIANTS.iter().enumerate() {
                                let admitted = three_device_admits(*variant, free, cost, containers, request_units);
                                let (greedy_tally, exists_tally) = &mut tallies[position];
                                for (truth, tally) in [(greedy, greedy_tally), (exists, exists_tally)] {
                                    if admitted && !truth {
                                        tally.unsafe_admits += 1;
                                        Tally::note(&mut tally.unsafe_minimum, weight, text.clone());
                                    } else if !admitted && truth {
                                        tally.false_rejects += 1;
                                        Tally::note(&mut tally.false_reject_minimum, weight, text.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("V2RESULT name=three_devices_policy cost={cost} states={states} greedy_loses_to_some_placement={greedy_loses} min={}", Tally::text(&greedy_loses_minimum));
        for (position, variant) in THREE_DEVICE_VARIANTS.iter().enumerate() {
            for (truth_name, tally) in [("greedy", &tallies[position].0), ("any_placement", &tallies[position].1)] {
                println!(
                    "V2RESULT name=three_devices cost={cost} truth={truth_name} variant={variant:?} unsafe_admits={} unsafe_min={} false_rejects={} false_reject_min={}",
                    tally.unsafe_admits, Tally::text(&tally.unsafe_minimum), tally.false_rejects, Tally::text(&tally.false_reject_minimum)
                );
            }
        }
    }
}

/// 手算锚点：每一格的数都先手算写死，模型必须给同一个判定。
fn self_tests() {
    // 两盘各 9 槽、c = 4、一个墓碑容器、请求 1 个数据单元：真值 9 ≥ 2 + 4 + 2 = 8，放得下。
    assert!(two_device_truth([9, 9], 4, 1, 1));
    // 统计量按两份副本之和记（4 槽）、每块盘扣整份：9 − 4 − 4 = 1 < 2 ⇒ 拒绝，是假性拒绝。
    assert!(!two_device_admits(TwoDeviceVariant::EachDeviceWholeScalar, LedgerUnit::BothCopies, [9, 9], 4, 1, 1));
    // 同一格按设备数平分：2 × (9 − 4 − 2) = 6 ≥ 4 ⇒ 放行。
    assert!(two_device_admits(TwoDeviceVariant::EachDeviceScalarOverDevices, LedgerUnit::BothCopies, [9, 9], 4, 1, 1));
    // 统计量按一份副本记（2 槽）却按设备数平分：两盘各 7 槽，2 × (7 − 4 − 2) = 2 ≥ 2 放行，而真值 7 < 8 ⇒ 放行而分不到。
    assert!(two_device_admits(TwoDeviceVariant::EachDeviceScalarOverDevices, LedgerUnit::PerCopy, [7, 7], 4, 1, 1));
    assert!(!two_device_truth([7, 7], 4, 1, 1));
    // 池级项留在 Σ 里：盘 0 20、盘 1 7，∀d 过（16、3 ≥ 2），Σ 27 − 8 − 4 = 15 ≥ 4 ⇒ 放行；盘 1 要 8 只有 7 ⇒ 墓碑容器的第二份落不下。
    assert!(two_device_admits(TwoDeviceVariant::PoolTermsKeptInSum, LedgerUnit::BothCopies, [20, 7], 4, 1, 1));
    assert!(!two_device_truth([20, 7], 4, 1, 1));
    // 三块盘：6 / 6 / 0、c = 1、k = 0、u = 1：选中两块各 6 − 1 ≥ 2 放行；贪心两份落盘 0、1 各剩 4，固定点落 0、1，放得下。
    assert!(three_device_greedy_truth([6, 6, 0], 1, 0, 1));
    assert!(three_device_admits(ThreeDeviceVariant::SelectedDevices, [6, 6, 0], 1, 0, 1));
    // 同一格按三块全判：盘 2 是 0 − 1 < 2 ⇒ 拒绝，而放得下 ⇒ 假性拒绝。
    assert!(!three_device_admits(ThreeDeviceVariant::AllDevices, [6, 6, 0], 1, 0, 1));
    // 放法存在性：4 / 4 / 4 放 3 个每份 2 槽的单元（每份总量 T = 6，盘 d 负载 = T − 跳过 d 的那批 ≤ 4 ⇒ 每块跳过 ≥ 2，合计 6 = T）⇒ 存在（三个单元各跳一块）；
    // 放 4 个（T = 8，每块跳过 ≥ 4，合计 12 > 8）⇒ 不存在；3 个单元再加 1 个 1 槽节点（T = 7，每块跳过 ≥ 3，合计 9 > 7）⇒ 不存在。
    assert!(placement_exists([4, 4, 4], 3, 0));
    assert!(!placement_exists([4, 4, 4], 4, 0));
    assert!(!placement_exists([4, 4, 4], 3, 1));
    println!("V2RESULT name=selftest passed=11");
}

fn main() {
    self_tests();
    two_device_sweep();
    three_device_sweep();
    println!("V2RESULT name=done");
}
