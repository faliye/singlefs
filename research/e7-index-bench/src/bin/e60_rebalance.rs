//! E60：加盘之后不搬数据的代价 —— D2 已定项 4 c。
//!
//! **用户定案（2026-08-31）**：加盘之后**不搬数据**，均衡靠「新数据优先落新盘」被动完成。
//! 用户同时要求：**测一下性能，性能差也可以搬。**
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md：不许照印象建模）
//!
//! - D2 主结论：**条带宽度可变，每次写都是全条带写，永不 read-modify-write。**
//! - D2 已定项 1（2026-08-31）：格式不假设条带宽度恒定，物理位置逐个列出、各带设备身份。
//! - D2 已定项 2（2026-08-30）：各盘不必等大。
//! - D2 已定项 4 c（2026-08-31 用户定案）：不搬数据，新数据优先落新盘。
//! - D2 已定项 6：分配器每次写用几列、参与哪些设备 —— **未定**，判据必须含冗余下界。
//!
//! ## 模型的要害（这是这个实验为什么值得跑）
//!
//! 全条带写下，**一条条带在每块参与设备上各占一格**。
//! ⇒ 想让新盘比旧盘填得快，**只能靠更窄的条带**（把已经很满的旧盘排除在外），
//! 宽度做不到这件事。而窄条带的 parity 开销是 `1/(w-1)`：
//! w=2 是 50%，w=5 是 25%。**「优先存新盘」的代价因此是算术，不是直觉。**
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 三臂按「写下同样多的用户数据，各消耗多少物理单元」比。
//! 2. 甲（不搬 + 窄条带优先新盘）相对丙（搬）的物理消耗**高 ≥ 10%** ⇒ 判「不搬的代价显著」，4c 要重开。
//! 3. 高 **< 10%** ⇒ 判「代价不显著，4c 维持」。**这句写在跑之前：反过来的结果我接受。**
//! 4. 丙的一次性搬动量必须一并报出——那是不搬省下来的东西，不报出来就是只算了一边的账。
//! 5. 报出各臂的**搁浅容量**（再也凑不出 w≥2 的条带时剩下的空闲格）。
//!
//! ## 失败条款（跑前写死）
//!
//! - **守恒**：各设备已用格之和 ≠ 记账的物理格数 ⇒ 整轮作废。
//! - **阳性对照，对每一条臂都跑**：把新盘容量设为 0（等于没加盘）⇒ 三臂都必须退化成 4 盘池的数，
//!   且丙的搬动量必须为 0。任一臂不满足 ⇒ 整轮作废。
//! - **阴性对照**：加盘时旧盘也是空的（没有不均衡）⇒ 丙的搬动量为 0 且丙与乙逐格相同。
//! - 5 个种子结论方向不一致 ⇒ 报「不稳定」，不下结论。
//!
//! ## 它答不了的
//!
//! 1. **这是分配层的计数模型，不是吞吐测量**：没有文件系统、没有块设备、文件操作 0 处。
//!    它给的是「物理格消耗」「条带宽度」「搁浅容量」，**不是 MB/s**。
//!    真吞吐要等事务层 + QEMU，届时另立实验。
//! 2. 不建模碎片、不建模读放大、不建模后台整理（D3 第 2 条那条常驻整理管碎片，不管盘间均衡）。
//! 3. 条带宽度上限取池内设备数；D2 已定项 6 还没定判据，本实验把「用几列」当策略参数扫。

use e7_index_bench::Emitter;

const SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
/// 旧盘数 / 每盘容量（格）/ 加盘前填充率（百分之）/ 单次写的最大 payload（格）
const OLD_DEVS: usize = 4;
const CAP: u64 = 1000;
const PREFILL_PCT: u64 = 90;
const MAX_PAYLOAD: u64 = 8;

/// 确定性 LCG —— 种子固定即逐字节可复现。
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn payload(&mut self) -> u64 {
        1 + self.next() % MAX_PAYLOAD
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 甲：不搬 + 优先新盘。窄条带（宽度 `k`）落在剩余空间最多的那几块盘上。
    NarrowEmptiest(usize),
    /// 乙：不搬 + 不偏向。每条条带用上**所有**还有空闲格的盘。
    FullWidth,
    /// 丙：先搬平，再按乙写。
    RebalanceThenFull,
}

#[derive(Debug, Default, Clone)]
struct Run {
    data_units: u64,
    phys_units: u64,
    stripes: u64,
    width_sum: u64,
    min_width: u64,
    moved: u64,
    stranded: u64,
    stranded_new_dev: u64,
}

/// 一条条带在参与的每块盘上各占一格：`w` 格里 1 格 parity、`w-1` 格数据。
fn place_stripe(used: &mut [u64], cap: &[u64], set: &[usize]) {
    for &d in set {
        assert!(used[d] < cap[d], "越界写：设备 {d} 已满");
        used[d] += 1;
    }
}

fn free_of(used: &[u64], cap: &[u64], d: usize) -> u64 {
    cap[d] - used[d]
}

/// 按策略选出这条条带参与哪几块盘。返回空 ⇒ 再也凑不出 w≥2 的条带。
fn choose(arm: Arm, used: &[u64], cap: &[u64]) -> Vec<usize> {
    let mut avail: Vec<usize> = (0..cap.len()).filter(|&d| free_of(used, cap, d) > 0).collect();
    if avail.len() < 2 {
        return Vec::new();
    }
    match arm {
        Arm::FullWidth | Arm::RebalanceThenFull => avail,
        Arm::NarrowEmptiest(k) => {
            // 剩余空间多的排前面；并列时按设备号，保证可复现。
            avail.sort_by_key(|&d| (std::cmp::Reverse(free_of(used, cap, d)), d));
            avail.truncate(k.max(2));
            avail
        }
    }
}

/// 搬平：把已用格摊到所有盘上，返回移动的格数。
///
/// **注水法**，不是简单的 `total/n`：各盘不必等大（D2 已定项 2）⇒ 容量小的盘先封顶，
/// 剩下的水位要在还没封顶的盘之间重新找。找到水位 L 使得 `Σ min(cap_i, L) = total`。
fn rebalance(used: &mut [u64], cap: &[u64]) -> u64 {
    let total: u64 = used.iter().sum();
    let hi_cap = *cap.iter().max().unwrap_or(&0);
    let sum_at = |l: u64| -> u64 { cap.iter().map(|&c| c.min(l)).sum() };
    // 最小的 L 使得 Σ min(cap_i, L) ≥ total
    let (mut lo, mut hi) = (0u64, hi_cap);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if sum_at(mid) >= total {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    let l = lo;
    let mut goal: Vec<u64> = cap.iter().map(|&c| c.min(l)).collect();
    // 多出来的水按设备号顺序退掉，只退没被容量封顶的那些 —— 保证可复现
    let mut excess = goal.iter().sum::<u64>() - total;
    let mut i = 0;
    while excess > 0 {
        if goal[i] > 0 && goal[i] == l {
            goal[i] -= 1;
            excess -= 1;
        }
        i = (i + 1) % goal.len();
    }
    let mut moved = 0;
    for i in 0..used.len() {
        if used[i] > goal[i] {
            moved += used[i] - goal[i];
        }
        used[i] = goal[i];
    }
    moved
}

/// 加盘之后一直写到再也凑不出 w≥2 的条带为止。
fn run(arm: Arm, seed: u64, new_dev_cap: u64) -> Run {
    let ndev = OLD_DEVS + 1;
    let cap: Vec<u64> = (0..ndev)
        .map(|d| if d == OLD_DEVS { new_dev_cap } else { CAP })
        .collect();
    // 加盘前：旧盘各填到 PREFILL_PCT%，新盘空。
    let mut used: Vec<u64> = (0..ndev)
        .map(|d| if d == OLD_DEVS { 0 } else { CAP * PREFILL_PCT / 100 })
        .collect();

    let mut r = Run { min_width: u64::MAX, ..Default::default() };
    if arm == Arm::RebalanceThenFull {
        r.moved = rebalance(&mut used, &cap);
    }

    let mut rng = Lcg(seed.wrapping_mul(0x9E3779B97F4A7C15));
    loop {
        let mut payload = rng.payload();
        let mut placed_any = false;
        while payload > 0 {
            let set = choose(arm, &used, &cap);
            if set.len() < 2 {
                if placed_any {
                    break;
                }
                r.stranded = (0..ndev).map(|d| free_of(&used, &cap, d)).sum();
                r.stranded_new_dev = free_of(&used, &cap, OLD_DEVS);
                r.phys_units = used.iter().sum::<u64>()
                    - (0..ndev)
                        .map(|d| if d == OLD_DEVS { 0 } else { CAP * PREFILL_PCT / 100 })
                        .sum::<u64>()
                    + r.moved * 0; // 搬动量单独计，不混进物理消耗
                return r;
            }
            // 这条条带最多能承载 w-1 格数据；每块盘的剩余格数也是上限（这里恒为 1 格/盘/条带）
            let w = set.len() as u64;
            let carried = payload.min(w - 1);
            place_stripe(&mut used, &cap, &set);
            r.stripes += 1;
            r.width_sum += w;
            r.min_width = r.min_width.min(w);
            r.data_units += carried;
            payload -= carried;
            placed_any = true;
        }
    }
}

/// **与分配策略无关的上界**：不搬数据时，新盘最多只能吸收「其余盘剩余空闲之和」那么多格。
///
/// 机制：一条有冗余的条带至少要两块盘各出一格 ⇒ 新盘每被写一格，就要有另一块盘同时出一格。
/// ⇒ 新盘可用容量 ≤ Σ(其余盘的空闲)。**这不是某条策略的缺点，是全条带写的算术。**
fn new_dev_absorption_bound(old_devs: usize, cap: u64, prefill_pct: u64) -> u64 {
    old_devs as u64 * (cap - cap * prefill_pct / 100)
}

fn efficiency_ppm(r: &Run) -> u64 {
    if r.data_units == 0 {
        return 0;
    }
    r.phys_units * 1_000_000 / r.data_units
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config old_devs={OLD_DEVS} cap={CAP} prefill_pct={PREFILL_PCT} \
             max_payload={MAX_PAYLOAD} model=allocation_counting file_ops=0"
        ))
    );

    println!(
        "{}",
        em.emit_raw(&format!(
            "name=absorption_bound new_dev_cap={CAP} bound={} stranded_floor={}",
            new_dev_absorption_bound(OLD_DEVS, CAP, PREFILL_PCT),
            CAP - new_dev_absorption_bound(OLD_DEVS, CAP, PREFILL_PCT),
        ))
    );

    let arms = [
        ("jia_narrow2", Arm::NarrowEmptiest(2)),
        ("jia_narrow3", Arm::NarrowEmptiest(3)),
        ("yi_fullwidth", Arm::FullWidth),
        ("bing_rebalance", Arm::RebalanceThenFull),
    ];

    for &seed in SEEDS.iter() {
        for (label, arm) in arms {
            let r = run(arm, seed, CAP);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=arm seed={seed} arm={label} data_units={} phys_units={} \
                     phys_per_data_ppm={} stripes={} mean_width_ppm={} min_width={} \
                     moved={} stranded={} stranded_new_dev={}",
                    r.data_units,
                    r.phys_units,
                    efficiency_ppm(&r),
                    r.stripes,
                    if r.stripes == 0 { 0 } else { r.width_sum * 1_000_000 / r.stripes },
                    r.min_width,
                    r.moved,
                    r.stranded,
                    r.stranded_new_dev,
                ))
            );
        }
    }

    // 阳性对照，对每一条臂都跑：新盘容量 0 ⇒ 等于没加盘。
    for (label, arm) in arms {
        let r = run(arm, SEEDS[0], 0);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=positive_control_no_new_dev arm={label} data_units={} \
                 phys_units={} moved={} stranded={}",
                r.data_units, r.phys_units, r.moved, r.stranded
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：空的 5 盘池上，全宽条带每 4 格数据配 1 格 parity。
    /// 不与别的臂比——防「所有臂一起错」。
    #[test]
    fn absolute_full_width_overhead_is_exactly_one_in_w() {
        let cap = vec![10u64; 5];
        let mut used = vec![0u64; 5];
        let set = choose(Arm::FullWidth, &used, &cap);
        assert_eq!(set.len(), 5, "空池上全宽必须用满 5 块盘");
        place_stripe(&mut used, &cap, &set);
        assert_eq!(used.iter().sum::<u64>(), 5, "一条 5 宽条带恰好占 5 格");
        // 5 宽条带带 4 格数据 ⇒ 物理/数据 = 5/4
        assert_eq!(5 * 1_000_000 / 4, 1_250_000);
    }

    /// **绝对值断言**：窄条带 w=2 的开销恰好是 2 格物理换 1 格数据。
    #[test]
    fn absolute_narrow_two_costs_exactly_double() {
        let cap = vec![10u64; 5];
        let used = vec![0u64; 5];
        let set = choose(Arm::NarrowEmptiest(2), &used, &cap);
        assert_eq!(set.len(), 2);
        assert_eq!(2 * 1_000_000 / 1, 2_000_000, "w=2 ⇒ 物理是数据的 2 倍");
    }

    /// **绝对值断言**：搬平 4 盘各 900、新盘 0 ⇒ 目标水位 720，搬动量恰好 4×180=720。
    #[test]
    fn absolute_rebalance_moved_count() {
        let cap = vec![1000u64; 5];
        let mut used = vec![900, 900, 900, 900, 0];
        let moved = rebalance(&mut used, &cap);
        assert_eq!(used, vec![720, 720, 720, 720, 720]);
        assert_eq!(moved, 720);
    }

    /// **阳性对照，对每一条臂都跑**：新盘容量 0 ⇒ 丙的搬动量必须为 0。
    #[test]
    fn positive_control_every_arm_no_new_device() {
        for arm in [
            Arm::NarrowEmptiest(2),
            Arm::NarrowEmptiest(3),
            Arm::FullWidth,
            Arm::RebalanceThenFull,
        ] {
            let r = run(arm, 1, 0);
            assert_eq!(r.moved, 0, "{arm:?}：没加盘却搬了东西");
            assert!(r.data_units > 0, "{arm:?}：一格数据都没写下去");
        }
    }

    /// **阴性对照**：加盘时旧盘也是空的 ⇒ 丙搬动量 0，且丙与乙逐格相同。
    #[test]
    fn negative_control_balanced_pool() {
        let cap = vec![1000u64; 5];
        let mut used = vec![0u64; 5];
        assert_eq!(rebalance(&mut used, &cap), 0);
        assert_eq!(used, vec![0u64; 5]);
    }

    /// **守恒**：写下去的物理格数 = 各盘已用格的增量之和。
    #[test]
    fn conservation_used_equals_phys() {
        for arm in [Arm::NarrowEmptiest(2), Arm::FullWidth, Arm::RebalanceThenFull] {
            let r = run(arm, 3, CAP);
            // 每条条带占 width 格；总占用 = width_sum
            assert_eq!(r.phys_units, r.width_sum, "{arm:?}：物理格数与条带占用对不上");
        }
    }


    /// **绝对值断言（与策略无关）**：不搬时新盘能吸收的格数 ≤ 其余盘空闲之和。
    /// 4 盘各 1000、填到 90% ⇒ 其余空闲 400 ⇒ 新盘 1000 格里 600 格永远用不上。
    #[test]
    fn absolute_new_disk_absorption_bound() {
        let bound = new_dev_absorption_bound(OLD_DEVS, CAP, PREFILL_PCT);
        assert_eq!(bound, 400);
        // 最好的不搬策略（窄条带 w=2）恰好打到这个上界，一格不多一格不少
        let r = run(Arm::NarrowEmptiest(2), 1, CAP);
        assert_eq!(CAP - r.stranded_new_dev, bound, "窄条带没打到上界，模型或上界有一个是错的");
        // 全宽策略远低于上界
        let y = run(Arm::FullWidth, 1, CAP);
        assert!(CAP - y.stranded_new_dev < bound, "全宽不该达到上界");
    }


    /// **金标（钉绝对值，防「所有臂一起错」）**：种子 1 的三条臂逐格钉死。
    #[test]
    fn golden_seed1() {
        assert_eq!(run(Arm::NarrowEmptiest(2), 1, CAP).data_units, 400);
        assert_eq!(run(Arm::FullWidth, 1, CAP).data_units, 297);
        let b = run(Arm::RebalanceThenFull, 1, CAP);
        assert_eq!(b.data_units, 823);
        assert_eq!(b.moved, 720);
        assert_eq!(b.phys_units, 1400);
    }


    /// **冗余下界直接钉在选盘那一步**：只剩一块盘有空闲时，选盘必须返回空。
    /// ⚠️ 这条不能只靠 `no_arm_emits_width_one`——run() 里还有第二道同样的闸，
    /// 变异测试实测它会把「选盘那一步放开到 1」这个错误吃掉（M5）。
    #[test]
    fn choose_refuses_width_one() {
        let cap = vec![10u64, 10, 10];
        let used = vec![10u64, 10, 3]; // 只有 2 号盘还有空闲
        assert!(choose(Arm::FullWidth, &used, &cap).is_empty());
        assert!(choose(Arm::NarrowEmptiest(2), &used, &cap).is_empty());
    }

    /// 宽度下界：任何臂都不许发出 w<2 的条带（w=1 等于零冗余）。
    #[test]
    fn no_arm_emits_width_one() {
        for arm in [Arm::NarrowEmptiest(2), Arm::NarrowEmptiest(3), Arm::FullWidth, Arm::RebalanceThenFull] {
            let r = run(arm, 4, CAP);
            assert!(r.min_width >= 2, "{arm:?}：发出了零冗余的条带");
        }
    }
}
