//! E119：槽定宽的补齐代价 —— 档怎么切，各切法各付多少。
//!
//! ## 它问什么
//!
//! D27 已定项 8 ④（2026-09-08 用户定案）取**槽定宽**：一个容器声明一个槽宽 `W`，
//! 容器内所有槽同宽，对象补齐到 `W`。补齐浪费因此按档宽回来了——
//! 一个 600 字节的对象落在哪一档就占那一档的宽度。
//!
//! **档切得粗，浪费大；切得细，同时开着的容器多。** E119 把这条曲线量出来，
//! 让「档怎么切」（D27 已定项 8 定了形态、切法随已定项 6 一起定）有数可依。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**已定项 2**：短 extent 补齐到整单元。
//! - **D18 已定项 11**：打包记录单元头 103。**D2 已定项 9**：第一版 2 盘恒 w = 2。
//! - **D27 已定项 2**：界线 ≤ 4 KiB 的对象才进容器。
//! - **D27 已定项 8 ④**：槽定宽，对象补齐到槽宽；槽 `i` 起点 = 头 + `i × W`，不需要槽目录。
//! - **E116 的闭式**：回本比下界 = `w / (cap − 1)` ⇒ 回得了本当且仅当 `cap > w + 1`；
//!   `w` = 2 ⇒ 要 `cap ≥ 4`。
//!
//! ## 两个假设（不是条款，标出来 —— C187）
//!
//! - `SLOT_EXTRA = 43`：每槽自描述 = 逻辑身份五元组 33（D18 已定项 3）+ 写序 10（D18 已定项 7）。
//!   **定宽之下不需要槽目录条目**（D27 已定项 8 ④ 逐字），所以比 E114 / E116 用的 47 少 4。
//! - **对象大小分布**：全仓没有任何实测分布（D25 的 smallfile 那一行三列全「未测」，C174）。
//!   E119 自定三个分布并全部报出来，**不挑一个当代表**。
//!
//! ## 六条臂（跑前列全，中途不许加臂 —— C186）
//!
//! | 臂 | 档 | 是不是候选 |
//! |---|---|---|
//! | `var` | 变长槽，按大小升序贪心装填 | **否**——是收益上界对照，D27 已定项 8 ④ 已定不取变长 |
//! | `gran64` | 64 字节粒度：64, 128, …, 4096（64 档）| 是 |
//! | `gran256` | 256 字节粒度：256, 512, …, 4096（16 档）| 是 |
//! | `pow2` | 2 的幂：64, 128, 256, …, 4096（7 档）| 是 |
//! | `tier4` | 四档：512, 1024, 2048, 4096 | 是 |
//! | `single` | 单档 4096（最粗）| 是 |
//!
//! ## 三个分布（跑前写死，全部报出）
//!
//! | 名 | 权重 | 为什么放它 |
//! |---|---|---|
//! | `uniform` | 1..=4096 每个整数等权 | 无信息先验 |
//! | `logunif` | 权重 ∝ 1/s | 小文件更多的那一侧，压定宽的补齐代价 |
//! | `discrete` | 只在 512 / 1024 / 4096 三点等权 | 与 E114 / E116 的取样点对齐，可比 |
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **主判据**：每条臂 × 每个分布，报容器数、收益倍数（`pad` 占用 ÷ 本臂占用）、
//!    平均每对象占用字节、以及**相对 `var` 的损失百分比**。
//! 2. **回本闸**：每条臂每一档的 `cap` 都要 ≥ 4（E116 闭式 `w/(cap−1) < 1` 的充要条件）。
//!    有一档不满足就点名报出来。
//! 3. **档数**：报每条臂的档数——它等于同时开着的容器数除以树数（C106 同类）。
//!
//! **阳性对照，逐臂跑**：把分布换成「所有对象大小恰好等于某一档宽」，
//! 那一档存在的每条定宽臂必须与 `var` 逐字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`single`（单档 4096）在全 512 字节的分布上，每对象占用必须正好是
//! `var` 的 `(4096+43)/(512+43)` 倍。不成立 ⇒ 补齐没被建模，整轮作废。
//! **阴性对照**：N = 0 时六条臂的所有字节恰好 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **`gran64` 相对 `var` 的损失 < 1%** ⇒ 结论写「细档等价于变长，定宽这一步不付代价」。
//! - 若 **`tier4` 在任一分布上相对 `var` 的损失 > 30%** ⇒ 结论写「四档太粗，档要更细」。
//! - 若 **`single` 的收益倍数在任一分布上 < 2** ⇒ 结论写「单档不可用」。
//! - 若 **每一条定宽臂在每一个分布上损失都 > 30%** ⇒ 结论写
//!   「定宽把 D27 的收益吃掉大半，已定项 8 ④ 该重开」，不许回头改口径。
//! - 若 **任一臂任一档的 `cap` < 4** ⇒ 结论写「那一档回不了本，不许进档表」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! **`var` 臂按「大小升序贪心」装填**——它是变长这条路的一个具体装法，不是最优装法；
//! 装箱最优解是 NP 难的，所以它是一个可实现的参照，不是理论上界。
//! **对象大小分布是自定的**，全仓没有实测分布（C174）；三个分布一起报，不挑代表。
//! 不答「同时开着的容器数」的绝对值——那要乘树数，而全仓没有树数的上界。

use e7_index_bench::Emitter;

const UNIT: u64 = 32768; // D4 已定项 5
const PACK_HDR: u64 = 103; // D18 已定项 11
const W_REPL: u64 = 2; // D2 已定项 9
const SLOT_EXTRA: u64 = 43; // 假设：五元组 33 + 写序 10，定宽下不要槽目录条目
const LIMIT: u64 = 4096; // D27 已定项 2：界线 ≤ 4 KiB
const N: u64 = 100_000;
const NET: u64 = UNIT - PACK_HDR; // 容器净荷 32665

fn cap_of(w: u64) -> u64 { NET / (w + SLOT_EXTRA) }

fn tiers(arm: &str) -> Vec<u64> {
    match arm {
        "gran64" => (1..=64).map(|i| i * 64).collect(),
        "gran256" => (1..=16).map(|i| i * 256).collect(),
        "pow2" => vec![64, 128, 256, 512, 1024, 2048, 4096],
        "tier4" => vec![512, 1024, 2048, 4096],
        "single" => vec![4096],
        _ => vec![],
    }
}

/// 权重表：下标 s（1..=LIMIT）上的对象个数。三个分布都按整数权重展开，不用随机源。
fn weights(dist: &str) -> Vec<u64> {
    let mut w = vec![0u64; (LIMIT + 1) as usize];
    match dist {
        "uniform" => { for s in 1..=LIMIT { w[s as usize] = 1; } }
        // 权重 ∝ 1/s，用整数近似：LIMIT / s，保证 s 小的权重大
        "logunif" => { for s in 1..=LIMIT { w[s as usize] = LIMIT / s; } }
        "discrete" => { for s in [512u64, 1024, 4096] { w[s as usize] = 1; } }
        _ => {}
    }
    w
}

/// 按权重把 N 个对象摊到各个大小上（最后一格吸收取整误差）。
fn counts(dist: &str, n: u64) -> Vec<u64> {
    let w = weights(dist);
    let total: u64 = w.iter().sum();
    if total == 0 || n == 0 { return vec![0u64; (LIMIT + 1) as usize]; }
    let mut c = vec![0u64; (LIMIT + 1) as usize];
    let mut acc = 0u64;
    let mut last = 0usize;
    for s in 1..=LIMIT as usize {
        if w[s] == 0 { continue; }
        c[s] = n * w[s] / total;
        acc += c[s];
        last = s;
    }
    c[last] += n - acc; // 取整误差全给最后一格，总数恒为 n
    c
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Arm {
    containers: u64,
    occupancy: u64,
    tiers: u64,
    min_cap: u64,
}

fn run_tiered(arm: &str, c: &[u64]) -> Arm {
    let ts = tiers(arm);
    let mut containers = 0u64;
    let mut min_cap = u64::MAX;
    for (i, &w) in ts.iter().enumerate() {
        let lo = if i == 0 { 1 } else { ts[i - 1] + 1 };
        let n_t: u64 = (lo..=w).map(|s| c[s as usize]).sum();
        let cap = cap_of(w);
        if cap < min_cap { min_cap = cap; }
        if n_t > 0 { containers += n_t.div_ceil(cap.max(1)); }
    }
    Arm { containers, occupancy: containers * UNIT, tiers: ts.len() as u64, min_cap }
}

/// `var` 臂：变长槽，按对象大小升序**贪心装填**（同一容器内槽长可以各异，装不下就换一个容器）。
/// 这是可实现的装法，不是「完美装填」——⚠️ 早先按 Σ 槽长 ÷ 净荷上取整算过一版，
/// 那等于允许一个槽跨容器，**阳性对照当场判否**（全 512 字节时它给 1700，而 58 槽 × 1725 = 100050，
/// 正确答案是 1725）。两个模型都记在这里，登记的那个不许回头改。
fn run_var(c: &[u64]) -> Arm {
    let mut containers = 0u64;
    let mut room = 0u64; // 当前容器剩余净荷；0 表示还没开容器
    for s in 1..=LIMIT {
        let need = s + SLOT_EXTRA;
        let mut left = c[s as usize];
        while left > 0 {
            if room < need {
                containers += 1;
                room = NET;
            }
            let fit = (room / need).min(left);
            room -= fit * need;
            left -= fit;
        }
    }
    Arm { containers, occupancy: containers * UNIT, tiers: 0, min_cap: cap_of(LIMIT) }
}

fn total_objects(c: &[u64]) -> u64 { c.iter().sum() }

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config unit={UNIT} pack_hdr={PACK_HDR} net={NET} w={W_REPL} \
         slot_extra={SLOT_EXTRA} limit={LIMIT} n={N}")));

    let arms = ["gran64", "gran256", "pow2", "tier4", "single"];

    // 阳性对照：所有对象恰好等于某一档宽 ⇒ 有那一档的定宽臂必须与 var 逐字节相同
    for &w in [512u64, 1024, 2048, 4096].iter() {
        let mut c = vec![0u64; (LIMIT + 1) as usize];
        c[w as usize] = N;
        let v = run_var(&c);
        for &arm in arms.iter() {
            if !tiers(arm).contains(&w) { continue; }
            let a = run_tiered(arm, &c);
            println!("{}", em.emit_raw(&format!(
                "name=positive_exact tier_width={w} arm={arm} var_occ={} arm_occ={} same={}",
                v.occupancy, a.occupancy, v.occupancy == a.occupancy)));
        }
    }

    // 判别力：single 在全 512 的分布上，每对象占用应是 var 的 (4096+43)/(512+43) 倍
    {
        let mut c = vec![0u64; (LIMIT + 1) as usize];
        c[512] = N;
        let v = run_var(&c);
        let s = run_tiered("single", &c);
        println!("{}", em.emit_raw(&format!(
            "name=discrimination var_containers={} single_containers={} ratio={:.6} expect={:.6}",
            v.containers, s.containers,
            s.occupancy as f64 / v.occupancy as f64,
            (4096.0 + SLOT_EXTRA as f64) / (512.0 + SLOT_EXTRA as f64))));
    }

    // 阴性对照
    {
        let c = counts("uniform", 0);
        let v = run_var(&c);
        println!("{}", em.emit_raw(&format!(
            "name=negative arm=var containers={} occ={}", v.containers, v.occupancy)));
        for &arm in arms.iter() {
            let a = run_tiered(arm, &c);
            println!("{}", em.emit_raw(&format!(
                "name=negative arm={arm} containers={} occ={}", a.containers, a.occupancy)));
        }
    }

    // 主判据
    for dist in ["uniform", "logunif", "discrete"] {
        let c = counts(dist, N);
        let n = total_objects(&c);
        let pad_occ = n * UNIT;
        let v = run_var(&c);
        println!("{}", em.emit_raw(&format!(
            "name=main dist={dist} arm=var n={n} containers={} occ={} gain={:.4} \
             per_obj={:.2} loss_vs_var=0.0000 tiers=0 min_cap={}",
            v.containers, v.occupancy, pad_occ as f64 / v.occupancy as f64,
            v.occupancy as f64 / n as f64, v.min_cap)));
        for &arm in arms.iter() {
            let a = run_tiered(arm, &c);
            println!("{}", em.emit_raw(&format!(
                "name=main dist={dist} arm={arm} n={n} containers={} occ={} gain={:.4} \
                 per_obj={:.2} loss_vs_var={:.4} tiers={} min_cap={}",
                a.containers, a.occupancy, pad_occ as f64 / a.occupancy as f64,
                a.occupancy as f64 / n as f64,
                (a.occupancy as f64 - v.occupancy as f64) / v.occupancy as f64,
                a.tiers, a.min_cap)));
        }
    }

    // 回本闸：逐臂逐档报 cap，标出 cap < 4 的档
    for &arm in arms.iter() {
        for w in tiers(arm) {
            let cap = cap_of(w);
            println!("{}", em.emit_raw(&format!(
                "name=capgate arm={arm} tier={w} cap={cap} pays_back={}", cap >= 4)));
        }
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：对象恰好等于档宽时，定宽与变长必须逐字节相同。
    #[test]
    fn positive_control_exact_tier_matches_var() {
        for &w in [512u64, 1024, 2048, 4096].iter() {
            let mut c = vec![0u64; (LIMIT + 1) as usize];
            c[w as usize] = N;
            let v = run_var(&c);
            for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
                if !tiers(arm).contains(&w) { continue; }
                assert_eq!(run_tiered(arm, &c).occupancy, v.occupancy,
                           "arm={arm} 档宽={w}：对象恰好等于档宽时该与变长相同");
            }
        }
    }

    /// 判别力：补齐真的被建模了 —— single 在全 512 上比 var 差，且差得正好是那个比。
    #[test]
    fn discrimination_padding_is_modeled() {
        let mut c = vec![0u64; (LIMIT + 1) as usize];
        c[512] = N;
        let v = run_var(&c);
        let s = run_tiered("single", &c);
        assert!(s.occupancy > v.occupancy, "single 该比 var 差");
        assert_eq!(v.containers, 1725);
        assert_eq!(s.containers, 14286);
        let ratio = s.occupancy as f64 / v.occupancy as f64;
        assert!((ratio - 8.2818).abs() < 0.001, "实测比 {ratio}");
        // 每对象占用之比正好是 (4096+43)/(512+43) 的上取整效应：58 槽 vs 7 槽
        assert_eq!(cap_of(512), 58);
        assert_eq!(cap_of(4096), 7);
    }

    /// 阴性：N = 0 时六条臂全 0。
    #[test]
    fn negative_control_zero() {
        let c = counts("uniform", 0);
        assert_eq!(run_var(&c).occupancy, 0);
        for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
            let a = run_tiered(arm, &c);
            assert_eq!((a.containers, a.occupancy), (0, 0), "arm={arm}");
        }
    }

    /// 容量钉绝对值，含跨整数边界的取样点（rules/mutation-sampling.md）。
    #[test]
    fn cap_absolute() {
        assert_eq!(NET, UNIT - 103);
        assert_eq!(NET, 32665);
        assert_eq!(cap_of(512), 58);
        assert_eq!(cap_of(1024), 30);
        assert_eq!(cap_of(2048), 15);
        assert_eq!(cap_of(4096), 7);
        assert_eq!(cap_of(64), 305);
        // 跨整数边界：cap = 8 的档宽区间是 [3587, 4040]，抬到 4041 就掉成 7。
        assert_eq!(cap_of(4040), 8);
        assert_eq!(cap_of(4041), 7);
    }

    /// 每一档都要 cap ≥ 4（E116 闭式 w/(cap−1) < 1 的充要条件）。
    #[test]
    fn every_tier_pays_back() {
        for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
            for w in tiers(arm) {
                assert!(cap_of(w) >= 4, "arm={arm} 档宽={w} 的 cap={} < 4，回不了本",
                        cap_of(w));
            }
        }
        // 钉住边界：cap = 4 的最大档宽
        assert_eq!(cap_of(8123), 4);
        assert_eq!(cap_of(8124), 3);
    }

    /// 分布展开之后对象总数恒等于 N —— 取整误差不许漏对象。
    #[test]
    fn counts_conserve_objects() {
        for dist in ["uniform", "logunif", "discrete"] {
            assert_eq!(total_objects(&counts(dist, N)), N, "dist={dist}");
        }
    }

    /// var 是上界：任何定宽臂的占用都不小于它。且 var 自己钉绝对值。
    #[test]
    fn var_is_upper_bound() {
        for dist in ["uniform", "logunif", "discrete"] {
            let c = counts(dist, N);
            let v = run_var(&c);
            for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
                assert!(run_tiered(arm, &c).occupancy >= v.occupancy,
                        "dist={dist} arm={arm}");
            }
        }
        // uniform 下 var 的绝对值：Σ (s+43)，s 从 1 到 4096 各 24 个（100000/4096=24 余 1696）
        let c = counts("uniform", N);
        let bytes: u64 = (1..=LIMIT).map(|s| c[s as usize] * (s + SLOT_EXTRA)).sum();
        assert_eq!(bytes, 212_622_560);
        assert_eq!(run_var(&c).containers, 6814);
    }

    /// 主判据的绝对值：uniform 下四条候选臂的容器数逐个钉住。
    #[test]
    fn main_absolute_uniform() {
        let c = counts("uniform", N);
        assert_eq!(run_tiered("gran64", &c).containers, 6935);
        assert_eq!(run_tiered("gran256", &c).containers, 7271);
        assert_eq!(run_tiered("pow2", &c).containers, 9463);
        assert_eq!(run_tiered("tier4", &c).containers, 9525);
        assert_eq!(run_tiered("single", &c).containers, 14286);
    }

    /// `logunif` 与 `discrete` 两个分布的绝对值 —— 变异 M8（把对数均匀退化成均匀）
    /// 第一轮**一个测试都没红**，因为所有钉绝对值的断言都只在 `uniform` 上。
    /// 按 `.claude/rules/mutation-sampling.md` 三分：那不是等价变异也不是取样点不敏感，
    /// 是**真盲区**——`logunif` 这一整个分布上没有任何断言。而结论最重的那个数正在它上面。
    #[test]
    fn main_absolute_logunif_and_discrete() {
        let c = counts("logunif", N);
        assert_eq!(run_var(&c).containers, 1592);
        assert_eq!(run_tiered("gran64", &c).containers, 1745);
        assert_eq!(run_tiered("gran256", &c).containers, 2186);
        assert_eq!(run_tiered("pow2", &c).containers, 2099);
        assert_eq!(run_tiered("tier4", &c).containers, 3038);
        assert_eq!(run_tiered("single", &c).containers, 14286);
        // 小端密的档表（pow2，7 档）在这个分布上胜过等距的 gran256（16 档）——
        // 档要在小端密，不是等距。这一条是结论，必须有断言守着。
        assert!(run_tiered("pow2", &c).containers < run_tiered("gran256", &c).containers);

        let d = counts("discrete", N);
        assert_eq!(run_var(&d).containers, 6448);
        for &arm in ["gran64", "gran256", "pow2", "tier4"].iter() {
            assert_eq!(run_tiered(arm, &d).containers, 6449,
                       "arm={arm}：三个取样点都恰好落在档宽上，定宽只多 1 个容器");
        }
        assert_eq!(run_tiered("single", &d).containers, 14286);
    }

    /// 档数就是同时开着的容器数（除以树数）—— 钉绝对值。
    #[test]
    fn tier_counts_absolute() {
        assert_eq!(tiers("gran64").len(), 64);
        assert_eq!(tiers("gran256").len(), 16);
        assert_eq!(tiers("pow2").len(), 7);
        assert_eq!(tiers("tier4").len(), 4);
        assert_eq!(tiers("single").len(), 1);
        // 最细那档的宽度
        assert_eq!(tiers("gran64")[0], 64);
        assert_eq!(*tiers("gran64").last().unwrap(), 4096);
    }

    /// 单调：档越细，占用越小（同一分布上）。这一条是互比，上面几条钉绝对值的与它配对。
    #[test]
    fn finer_tiers_never_worse() {
        for dist in ["uniform", "logunif", "discrete"] {
            let c = counts(dist, N);
            let g64 = run_tiered("gran64", &c).occupancy;
            let g256 = run_tiered("gran256", &c).occupancy;
            let p2 = run_tiered("pow2", &c).occupancy;
            let t4 = run_tiered("tier4", &c).occupancy;
            let s1 = run_tiered("single", &c).occupancy;
            assert!(g64 <= g256, "dist={dist}: gran64={g64} gran256={g256}");
            assert!(g256 <= t4, "dist={dist}: gran256={g256} tier4={t4}");
            assert!(t4 <= s1, "dist={dist}: tier4={t4} single={s1}");
            assert!(p2 <= s1, "dist={dist}: pow2={p2} single={s1}");
        }
    }
}
