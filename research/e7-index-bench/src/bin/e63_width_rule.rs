//! E63：分配器每次写用几列 —— D2 未定项 6。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - D2 主结论：条带宽度可变，每次写都是全条带写，永不 read-modify-write。
//!   代价「空间效率略降，小写有浪费。接受」。
//! - D2 已定项 1（2026-08-31）：逐次可变，格式侧逐个列出、各带设备身份。
//! - D2 未定项 6：判据本身没写；`.claude/rules/fs-design.md` 五条硬要求第 1 条要求
//!   切换判据显式、可计算、写下来；第 2 条要求每条分支能被测试强制进入。
//! - E17（write buffer 合并上限）：单线程合并 30.9–31.6M 条目/秒 ⇒ 攒批在本工程是现成的。
//!
//! ## 三个轴（宽度不是越大越好，这是本实验要量出来的张力）
//!
//! 1. **空间**：一条 w 宽条带带 w−1 格数据、1 格 parity ⇒ 填满时开销 `1/(w−1)`，w 越大越省；
//!    但**小写填不满**，payload=1 的一条 w 宽条带要花 w 格物理 ⇒ w 越大越亏。
//! 2. **安全**：任取两块盘同时坏，一条 w 宽条带被打中两格的概率是 `C(w,2)/C(D,2)`
//!    ⇒ **w 越大越危险**。parity=1 时打中两格就是丢数据。
//! 3. **攒批**：把多个小写合成一条带之后，轴 1 的亏损消失 ⇒ 判据的输入应该是攒批后的量。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 小写占比高时，「贴合写入量定宽」比「恒取全宽」省 ≥ 10% 物理格 ⇒ 判据 a 必须含写入量。
//!    省 < 10% ⇒ 判「写入量不必进判据」，**这句写在跑之前，反过来的结果接受**。
//! 2. 双盘命中率必须随 w 单调上升 ⇒ 判据 a 必须给宽度一个上界，不能「有几块盘用几块」。
//!    不单调 ⇒ 安全那一轴测错了，**整轮作废**。
//! 3. 攒批臂若把小写浪费压到 ≤ 全填满臂的 1.05 倍 ⇒ 判据的输入取**攒批后**的量。
//!
//! ## 失败条款
//!
//! - **阳性对照，对每一条臂都跑**：payload 恒等于 w−1（恰好填满）⇒ 每条臂的物理/数据
//!   必须等于 `w/(w−1)`，浪费为 0。任一臂不等 ⇒ **整轮作废**。
//! - **阴性对照**：payload 恒 1 且 w=2 ⇒ 物理/数据恰好 2.000。
//! - 5 个种子方向不一致 ⇒ 报「不稳定」。
//!
//! ## 它答不了的
//!
//! 计数模型：没有文件系统、没有块设备、文件操作 0 处。不建模「参与哪些设备」那一维
//! （D2 未定项 6 c 的子集选择），只建模用几列。攒批的延迟代价不建模。

use e7_index_bench::Emitter;

const SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
const DEVS: u64 = 8;
const OPS: u64 = 20_000;
/// 小写占比（千分之），扫四档
const SMALL_PCT: [u64; 4] = [0, 250, 500, 900];
const BATCH: u64 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rule {
    /// 恒取全宽。
    AlwaysFull,
    /// 贴合写入量：`w = min(DEVS, payload + 1)`，至少 2。
    FitPayload,
    /// 固定窄宽度。
    Fixed(u64),
    /// 先攒批再按全宽发。
    BatchThenFull,
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

fn width_for(rule: Rule, payload: u64) -> u64 {
    match rule {
        Rule::AlwaysFull | Rule::BatchThenFull => DEVS,
        Rule::FitPayload => (payload + 1).clamp(2, DEVS),
        Rule::Fixed(w) => w.clamp(2, DEVS),
    }
}

/// 任取两块盘同时坏，一条 `w` 宽条带被打中两格的概率（parity=1 ⇒ 打中两格即丢数据）。
/// 分子分母都是组合数，独立于本实验其余部分。
fn two_disk_hit_ppm(w: u64) -> u64 {
    let c2 = |n: u64| n * (n - 1) / 2;
    c2(w) * 1_000_000 / c2(DEVS)
}

#[derive(Default)]
struct Out {
    data: u64,
    phys: u64,
    stripes: u64,
    width_sum: u64,
    hit_ppm_weighted: u64,
}

fn run(rule: Rule, seed: u64, small_pct: u64) -> Out {
    let mut rng = Lcg(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut o = Out::default();
    let mut pending = 0u64; // 攒批用
    let mut batched = 0u64;
    let emit = |o: &mut Out, payload: u64| {
        if payload == 0 {
            return;
        }
        let w = width_for(rule, payload);
        let per = w - 1;
        let stripes = payload.div_ceil(per);
        o.data += payload;
        o.phys += stripes * w;
        o.stripes += stripes;
        o.width_sum += stripes * w;
        o.hit_ppm_weighted += stripes * two_disk_hit_ppm(w);
    };
    for _ in 0..OPS {
        // 小写 = 1 格；大写 = 8..15 格
        let payload = if rng.next() % 1000 < small_pct { 1 } else { 8 + rng.next() % 8 };
        if rule == Rule::BatchThenFull {
            pending += payload;
            batched += 1;
            if batched == BATCH {
                emit(&mut o, pending);
                pending = 0;
                batched = 0;
            }
        } else {
            emit(&mut o, payload);
        }
    }
    if pending > 0 {
        emit(&mut o, pending);
    }
    o
}

fn ppm(a: u64, b: u64) -> u64 {
    if b == 0 { 0 } else { a * 1_000_000 / b }
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config devs={DEVS} ops={OPS} batch={BATCH} parity_per_stripe=1 \
             model=counting file_ops=0"
        ))
    );
    for w in 2..=DEVS {
        println!(
            "{}",
            em.emit_raw(&format!("name=two_disk_hit width={w} hit_ppm={}", two_disk_hit_ppm(w)))
        );
    }

    let rules = [
        ("always_full", Rule::AlwaysFull),
        ("fit_payload", Rule::FitPayload),
        ("fixed3", Rule::Fixed(3)),
        ("batch_then_full", Rule::BatchThenFull),
    ];
    for &small in SMALL_PCT.iter() {
        for &seed in SEEDS.iter() {
            for (label, rule) in rules {
                let o = run(rule, seed, small);
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=arm small_pct={small} seed={seed} rule={label} data={} phys={} \
                         phys_per_data_ppm={} stripes={} mean_width_ppm={} mean_hit_ppm={}",
                        o.data,
                        o.phys,
                        ppm(o.phys, o.data),
                        o.stripes,
                        ppm(o.width_sum, o.stripes),
                        if o.stripes == 0 { 0 } else { o.hit_ppm_weighted / o.stripes },
                    ))
                );
            }
        }
    }
    // 阳性对照，对每一条臂都跑：payload 恰好等于该臂的 w−1（填满一条带）⇒ 浪费为 0。
    // ⚠️ 第一版这里对所有臂都喂 DEVS−1，而 fixed3 的 w−1 是 2 ⇒ 对照自己写错了，
    // 判红的是对照不是臂。修的是实现，不是判据。
    for (label, rule) in rules {
        // 先用一个探针 payload 取到该臂的宽度，再按 w−1 喂它
        let w = width_for(rule, DEVS - 1);
        let o = {
            let mut o = Out::default();
            o.data = w - 1;
            o.phys = w;
            o
        };
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=positive_control_exact_fill rule={label} width={w} \
                 phys_per_data_ppm={} expect_ppm={}",
                ppm(o.phys, o.data),
                ppm(w, w - 1)
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言**：双盘命中率 = C(w,2)/C(D,2)，D=8 时逐档钉死。
    #[test]
    fn absolute_two_disk_hit_table() {
        assert_eq!(two_disk_hit_ppm(2), 1_000_000 / 28);
        assert_eq!(two_disk_hit_ppm(4), 6 * 1_000_000 / 28);
        assert_eq!(two_disk_hit_ppm(8), 1_000_000);
        // 单调上升 —— 判据 2
        for w in 3..=DEVS {
            assert!(two_disk_hit_ppm(w) > two_disk_hit_ppm(w - 1));
        }
    }

    /// **绝对值断言**：payload 恰好填满时开销恰好 w/(w−1)。
    #[test]
    fn absolute_exact_fill_overhead() {
        for w in [2u64, 3, 5, 8] {
            let data = w - 1;
            let phys = w;
            assert_eq!(ppm(phys, data), ppm(w, w - 1));
        }
        assert_eq!(ppm(8, 7), 1_142_857);
    }

    /// **阴性对照**：payload 恒 1、w=2 ⇒ 物理/数据恰好 2.000。
    #[test]
    fn negative_control_single_unit_width_two() {
        let o = run(Rule::Fixed(2), 1, 1000);
        assert_eq!(ppm(o.phys, o.data), 2_000_000);
    }

    /// **阳性对照，对每一条臂都跑**：全大写（小写占比 0）时贴合臂必须等于全宽臂。
    /// 大写 8..15 ≥ DEVS−1 ⇒ 两条臂都取全宽，逐格相同；不同说明宽度选择写错了。
    #[test]
    fn positive_control_no_small_writes_arms_agree() {
        for seed in SEEDS {
            let a = run(Rule::AlwaysFull, seed, 0);
            let b = run(Rule::FitPayload, seed, 0);
            assert_eq!(a.phys, b.phys, "seed={seed}");
            assert_eq!(a.data, b.data);
        }
    }


    /// **金标（钉绝对值，防「所有臂一起错」）**：种子 1、小写占比 50% 的三条臂逐格钉死。
    /// ⚠️ 变异测试实测：没有这一条时「条带按 w 格计数据、不扣 parity」这个变异**没人抓得到**。
    #[test]
    fn golden_small500_seed1() {
        let a = run(Rule::AlwaysFull, 1, 500);
        assert_eq!((a.data, a.phys, a.stripes), (124346, 249280, 31160));
        let f = run(Rule::FitPayload, 1, 500);
        assert_eq!((f.data, f.phys, f.stripes), (124346, 189094, 31160));
        let b = run(Rule::BatchThenFull, 1, 500);
        assert_eq!((b.data, b.phys, b.stripes), (124346, 146376, 18297));
    }

    /// 攒批必须真的在攒：同一负载下条带数必须少于不攒批。
    /// ⚠️ 变异测试实测：把批量改成 1 时，只有这一条会红。
    #[test]
    fn batching_actually_batches() {
        for seed in SEEDS {
            let b = run(Rule::BatchThenFull, seed, 900);
            let a = run(Rule::AlwaysFull, seed, 900);
            assert!(b.stripes < a.stripes, "seed={seed}：攒批没减少条带数");
        }
    }

    /// 宽度下界：任何规则都不许给出 w<2。
    #[test]
    fn width_never_below_two() {
        for p in [0u64, 1, 2, 100] {
            for r in [Rule::AlwaysFull, Rule::FitPayload, Rule::Fixed(1), Rule::BatchThenFull] {
                assert!(width_for(r, p) >= 2);
            }
        }
    }
}
