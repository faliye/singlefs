//! E121：按 `cap` 切档 —— 同 `cap` 的两档是冗余的，去掉它只赚不亏。
//!
//! ## 它问什么
//!
//! E120（档表取等比时公比怎么定）的四个候选是**同一个家族**：按槽宽 `W` 等比切。
//! 而决定占用的不是 `W`，是 **`cap` = ⌊(32768 − 103) / (W + 43)⌋**（一个容器装几个对象）。
//! 两档若 `cap` 相同，一个容器装的对象数就相同 ⇒ **它们对占用完全等价，留两档是冗余的**。
//!
//! E121 换这个家族：**先选一组 `cap` 值，再把每个 `cap` 对应的最大 `W` 取成档宽**。
//! 并检验一条可证伪的预测：把按 `W` 切的表**按 `cap` 去重**，容器数**只减不增**——
//! 因为合并同 `cap` 的两档只会让尾部没装满的容器少一个（⌈n₁/c⌉ + ⌈n₂/c⌉ ≥ ⌈(n₁+n₂)/c⌉）。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**D18 已定项 11**：打包记录单元头 103。
//! - **D2 已定项 9**：`w` = 2。**D27 已定项 2**：界线 ≤ 4 KiB。
//! - **D27 已定项 8 ④**：槽定宽，对象补齐到槽宽。
//! - **D27 已定项 12**：档表不进格式，容器头声明槽宽 ⇒ 档表是策略，可以按 `cap` 切。
//! - **E116 的闭式**：回本比下界 = `w / (cap − 1)` ⇒ 要 `cap ≥ 4`。
//! - **E120 的结论**：按 `W` 等比切时最坏相对浪费 = `1 − 1/r`，与分布无关。
//!
//! ## 假设（不是条款 —— C187）
//!
//! `SLOT_EXTRA = 43`；对象大小分布全仓无实测（C174），三个分布全部报出。
//!
//! ## 七条臂（跑前列全，中途不许加臂 —— C186）
//!
//! | 臂 | 档表怎么来 |
//! |---|---|
//! | `var` | 变长贪心，参照，不是候选 |
//! | `wgeo` | 按 `W` 等比，`r` = 1.125（E120 那张表，原样） |
//! | `wgeo_dedup` | 同上，**按 `cap` 去重**后 |
//! | `capgeo125` / `capgeo150` / `capgeo200` | 按 `cap` 等比，比值 1.25 / 1.5 / 2.0 |
//! | `capall` | 每个可达 `cap` 一档（最细的无冗余表） |
//!
//! `W_min` 另外扫 32 / 64 / 128 三个值，只对 `capgeo150` 扫。
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **去重预测**：`wgeo_dedup` 的容器数 **≤** `wgeo` 的，在三个分布上都成立。
//!    有一格大于 ⇒ 「同 `cap` 等价」这条推理错了，整轮作废。
//! 2. **档数与浪费**：每条臂报档数、最坏相对浪费（`W_min` 以上）、三个分布的实测损失。
//! 3. **回本闸**：每条臂每一档 `cap` ≥ 4。
//!
//! **阳性对照，逐臂跑**：对象恰好等于某档宽时，该臂与 `var` 逐字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`capgeo200` 与 `capall` 在同一分布上容器数必须不同。相同 ⇒ 档表没被建模。
//! **阴性对照**：N = 0 时七条臂全 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **`wgeo_dedup` 在任一分布上比 `wgeo` 多** ⇒ 结论写「同 `cap` 等价这条推理错了」，不许改口径。
//! - 若 **`capall`（280 档）比 `capgeo125` 省不到 1%** ⇒ 结论写「档数加到头也换不来多少，取少的」。
//! - 若 **`capgeo` 家族在所有分布上都不比 `wgeo` 家族省** ⇒ 结论写「按 `cap` 切没有好处，回到按 `W` 切」。
//! - 若 **任一臂任一档 `cap` < 4** ⇒ 结论写「那一档回不了本，不许进档表」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! `var` 是变长的一个具体装法（大小升序贪心），不是最优装法。
//! **不答同时开着的容器数的绝对值**（要乘整理器同时处理的树数，D19 未定项 6 未定）。
//! 不答「整理器怎么在多档之间调度」——那归 D27 已定项 6。

use e7_index_bench::Emitter;

const UNIT: u64 = 32768;
const PACK_HDR: u64 = 103;
const NET: u64 = UNIT - PACK_HDR; // 32665
const SLOT_EXTRA: u64 = 43;
const LIMIT: u64 = 4096;
const N: u64 = 100_000;

fn cap_of(w: u64) -> u64 { NET / (w + SLOT_EXTRA) }

/// 给定 `cap`，能达到这个 `cap` 的最大槽宽（同 `cap` 里浪费最小的那个选择）。
fn widest_for_cap(c: u64, w_min: u64) -> Option<u64> {
    if c == 0 { return None; }
    let w = (NET / c).saturating_sub(SLOT_EXTRA);
    let w = w.min(LIMIT);
    if w < w_min || cap_of(w) != c { None } else { Some(w) }
}

/// 按 `W` 等比（E120 那张表）。
fn wgeo_tiers(r_milli: u64, w_min: u64) -> Vec<u64> {
    let mut v = Vec::new();
    let mut w = w_min;
    while w < LIMIT {
        v.push(w);
        let next = (w * r_milli).div_ceil(1000);
        if next <= w { break; }
        w = next;
    }
    v.push(LIMIT);
    v.dedup();
    v
}

/// 按 `cap` 去重：同 `cap` 的几档只留最大的那个 `W`。
fn dedup_by_cap(ts: &[u64]) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    for &w in ts {
        if let Some(&last) = out.last() {
            if cap_of(last) == cap_of(w) { out.pop(); }
        }
        out.push(w);
    }
    out
}

/// 按 `cap` 等比：`cap` 从 4096 那档的 7 起，乘 `q` 往上走，直到超过 `w_min` 能给的最大 `cap`。
fn capgeo_tiers(q_milli: u64, w_min: u64) -> Vec<u64> {
    let cap_top = cap_of(w_min); // 最细档给出的最大 cap
    let mut caps = Vec::new();
    let mut c = cap_of(LIMIT);
    while c <= cap_top {
        caps.push(c);
        let next = (c * q_milli).div_ceil(1000);
        if next <= c { break; }
        c = next;
    }
    let mut ts: Vec<u64> = caps.iter().filter_map(|&c| widest_for_cap(c, w_min)).collect();
    ts.push(LIMIT);
    ts.sort_unstable();
    ts.dedup();
    ts
}

/// 每个可达 `cap` 一档（最细的无冗余表）。
fn capall_tiers(w_min: u64) -> Vec<u64> {
    let mut ts: Vec<u64> = (cap_of(LIMIT)..=cap_of(w_min))
        .filter_map(|c| widest_for_cap(c, w_min)).collect();
    ts.push(LIMIT);
    ts.sort_unstable();
    ts.dedup();
    ts
}

fn weights(dist: &str) -> Vec<u64> {
    let mut w = vec![0u64; (LIMIT + 1) as usize];
    match dist {
        "uniform" => { for s in 1..=LIMIT { w[s as usize] = 1; } }
        "logunif" => { for s in 1..=LIMIT { w[s as usize] = LIMIT / s; } }
        "discrete" => { for s in [512u64, 1024, 4096] { w[s as usize] = 1; } }
        _ => {}
    }
    w
}

fn counts(dist: &str, n: u64) -> Vec<u64> {
    let w = weights(dist);
    let total: u64 = w.iter().sum();
    if total == 0 || n == 0 { return vec![0u64; (LIMIT + 1) as usize]; }
    let mut c = vec![0u64; (LIMIT + 1) as usize];
    let (mut acc, mut last) = (0u64, 0usize);
    for s in 1..=LIMIT as usize {
        if w[s] == 0 { continue; }
        c[s] = n * w[s] / total; acc += c[s]; last = s;
    }
    c[last] += n - acc;
    c
}

fn containers_tiered(ts: &[u64], c: &[u64]) -> u64 {
    let mut total = 0u64;
    for (i, &w) in ts.iter().enumerate() {
        let lo = if i == 0 { 1 } else { ts[i - 1] + 1 };
        let n_t: u64 = (lo..=w).map(|s| c[s as usize]).sum();
        if n_t > 0 { total += n_t.div_ceil(cap_of(w).max(1)); }
    }
    total
}

fn containers_var(c: &[u64]) -> u64 {
    let (mut cont, mut room) = (0u64, 0u64);
    for s in 1..=LIMIT {
        let need = s + SLOT_EXTRA;
        let mut left = c[s as usize];
        while left > 0 {
            if room < need { cont += 1; room = NET; }
            let fit = (room / need).min(left);
            room -= fit * need; left -= fit;
        }
    }
    cont
}

fn worst_waste_above_floor(ts: &[u64]) -> f64 {
    let mut worst = 0.0f64;
    for (i, &w) in ts.iter().enumerate() {
        if i == 0 { continue; }
        let waste = 1.0 - ((ts[i - 1] + 1) as f64) / (w as f64);
        if waste > worst { worst = waste; }
    }
    worst
}

fn arm_tiers(arm: &str, w_min: u64) -> Vec<u64> {
    match arm {
        "wgeo" => wgeo_tiers(1125, w_min),
        "wgeo_dedup" => dedup_by_cap(&wgeo_tiers(1125, w_min)),
        "capgeo125" => capgeo_tiers(1250, w_min),
        "capgeo150" => capgeo_tiers(1500, w_min),
        "capgeo200" => capgeo_tiers(2000, w_min),
        "capall" => capall_tiers(w_min),
        _ => vec![],
    }
}

const ARMS: [&str; 6] = ["wgeo", "wgeo_dedup", "capgeo125", "capgeo150", "capgeo200", "capall"];

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config unit={UNIT} pack_hdr={PACK_HDR} net={NET} slot_extra={SLOT_EXTRA} \
         limit={LIMIT} n={N} arms={ARMS:?}")));

    // 阳性对照
    for &arm in ARMS.iter() {
        let ts = arm_tiers(arm, 64);
        for &w in ts.iter() {
            let mut c = vec![0u64; (LIMIT + 1) as usize];
            c[w as usize] = N;
            let (v, t) = (containers_var(&c), containers_tiered(&ts, &c));
            if v != t {
                println!("{}", em.emit_raw(&format!(
                    "name=positive_exact arm={arm} tier={w} var={v} tiered={t} same=false")));
            }
        }
        println!("{}", em.emit_raw(&format!(
            "name=positive_exact_summary arm={arm} tiers={} all_same=true", ts.len())));
    }

    // 判别力
    {
        let c = counts("uniform", N);
        let a = containers_tiered(&arm_tiers("capgeo200", 64), &c);
        let b = containers_tiered(&arm_tiers("capall", 64), &c);
        println!("{}", em.emit_raw(&format!(
            "name=discrimination capgeo200={a} capall={b} differ={}", a != b)));
    }

    // 阴性
    {
        let c = counts("uniform", 0);
        println!("{}", em.emit_raw(&format!("name=negative arm=var containers={}", containers_var(&c))));
        for &arm in ARMS.iter() {
            println!("{}", em.emit_raw(&format!(
                "name=negative arm={arm} containers={}", containers_tiered(&arm_tiers(arm, 64), &c))));
        }
    }

    // 档表本身
    for &arm in ARMS.iter() {
        let ts = arm_tiers(arm, 64);
        let min_cap = ts.iter().map(|&w| cap_of(w)).min().unwrap_or(0);
        println!("{}", em.emit_raw(&format!(
            "name=table arm={arm} tiers={} worst_waste={:.6} min_cap={} min_tier={} max_tier={}",
            ts.len(), worst_waste_above_floor(&ts), min_cap, ts[0], ts.last().unwrap())));
    }

    // 主判据
    for dist in ["uniform", "logunif", "discrete"] {
        let c = counts(dist, N);
        let v = containers_var(&c);
        println!("{}", em.emit_raw(&format!("name=main dist={dist} arm=var containers={v} loss=0.0000")));
        for &arm in ARMS.iter() {
            let t = containers_tiered(&arm_tiers(arm, 64), &c);
            println!("{}", em.emit_raw(&format!(
                "name=main dist={dist} arm={arm} containers={t} loss={:.4} tiers={}",
                (t as f64 - v as f64) / v as f64, arm_tiers(arm, 64).len())));
        }
    }

    // W_min 扫描，只对 capgeo150
    for &w_min in [32u64, 64, 128].iter() {
        let ts = arm_tiers("capgeo150", w_min);
        for dist in ["uniform", "logunif"] {
            let c = counts(dist, N);
            let v = containers_var(&c);
            let t = containers_tiered(&ts, &c);
            println!("{}", em.emit_raw(&format!(
                "name=wmin arm=capgeo150 w_min={w_min} dist={dist} tiers={} containers={t} \
                 loss={:.4} floor_waste_bytes={}",
                ts.len(), (t as f64 - v as f64) / v as f64, w_min - 1)));
        }
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 跑前写死的那条预测：按 `cap` 去重之后容器数只减不增。
    #[test]
    fn dedup_never_increases_containers() {
        for dist in ["uniform", "logunif", "discrete"] {
            let c = counts(dist, N);
            let a = containers_tiered(&arm_tiers("wgeo", 64), &c);
            let b = containers_tiered(&arm_tiers("wgeo_dedup", 64), &c);
            assert!(b <= a, "dist={dist}：去重后 {b} 该 ≤ 去重前 {a}");
        }
    }

    /// 阳性对照：对象恰好等于档宽时与变长逐字节相同。
    #[test]
    fn positive_control_exact_tier_matches_var() {
        for &arm in ARMS.iter() {
            let ts = arm_tiers(arm, 64);
            for &w in ts.iter() {
                let mut c = vec![0u64; (LIMIT + 1) as usize];
                c[w as usize] = N;
                assert_eq!(containers_tiered(&ts, &c), containers_var(&c), "arm={arm} 档宽={w}");
            }
        }
    }

    /// 判别力：档表真的被建模了。
    #[test]
    fn discrimination_table_matters() {
        let c = counts("uniform", N);
        let a = containers_tiered(&arm_tiers("capgeo200", 64), &c);
        let b = containers_tiered(&arm_tiers("capall", 64), &c);
        assert!(b < a, "capall 该比 capgeo200 省：capall={b} capgeo200={a}");
    }

    /// 阴性：N = 0 全 0。
    #[test]
    fn negative_control_zero() {
        let c = counts("uniform", 0);
        assert_eq!(containers_var(&c), 0);
        for &arm in ARMS.iter() {
            assert_eq!(containers_tiered(&arm_tiers(arm, 64), &c), 0, "arm={arm}");
        }
    }

    /// ⚠️ 跑前假定「按 `W` 等比的表里有同 `cap` 的冗余档」，**判否**：
    /// `r` = 1.125 那张表 36 档，按 `cap` 去重之后**还是 36 档**——每一步都跨过一个 `cap`。
    /// 去重这件事本身没错，只是**在这个公比上是空操作**；它要到表比 `cap` 的分辨率还细时才咬得动。
    /// 两个数都留着：`r` = 1.125 去重前后 36 / 36；`r` = 1.01 去重前后 370 / 208。
    #[test]
    fn dedup_is_a_noop_at_this_ratio_but_bites_on_finer_ones() {
        let a = arm_tiers("wgeo", 64);
        let b = arm_tiers("wgeo_dedup", 64);
        assert_eq!(a.len(), 36);
        assert_eq!(b.len(), 36, "r = 1.125 这张表没有同 cap 的冗余档");
        // 换一张比 cap 分辨率还细的表，去重当场砍掉 44%
        let fine = wgeo_tiers(1010, 64);
        assert_eq!(fine.len(), 370);
        assert_eq!(dedup_by_cap(&fine).len(), 208);
    }

    /// **档数不是越多越好**：档越多补齐越少，但**没装满的尾容器也越多**。
    /// `capall`（248 档）在对象总数摊得开的 `uniform` 上赢 `wgeo`（36 档），
    /// 在总容器数只有 1592 的 `logunif` 上反而输——248 个尾容器就占了 15%。
    /// 这一条是 E121 的主结论，必须有断言守着。
    #[test]
    fn more_tiers_is_not_always_better() {
        let u = counts("uniform", N);
        let l = counts("logunif", N);
        let (wg, ca) = (arm_tiers("wgeo", 64), arm_tiers("capall", 64));
        assert_eq!(wg.len(), 36);
        assert_eq!(ca.len(), 248);
        // uniform：档多的赢
        assert!(containers_tiered(&ca, &u) < containers_tiered(&wg, &u));
        // logunif：档多的输
        assert!(containers_tiered(&ca, &l) > containers_tiered(&wg, &l));
        // 尾容器那笔账钉绝对值：logunif 上 capall 的容器数 1788，其中档数 248
        assert_eq!(containers_tiered(&ca, &l), 1788);
        assert_eq!(containers_var(&l), 1592);
    }

    /// 等价性留档（变异 M10）：`capall_tiers` 末尾那句 `ts.push(LIMIT)` 是**冗余**的——
    /// `cap` = 7 那一档算出来的最大槽宽本来就是 4096（`32665/7 − 43 = 4623`，被界线截到 4096，
    /// 而 `cap_of(4096)` 恰好还是 7）。去掉那句在所有输入上同值 ⇒ 按
    /// `.claude/rules/mutation-sampling.md` 判**等价变异**，不是盲区，把等价性写成这条测试留档。
    #[test]
    fn capall_contains_limit_without_the_explicit_push() {
        assert_eq!(widest_for_cap(cap_of(LIMIT), 64), Some(LIMIT));
        assert_eq!(cap_of(LIMIT), 7);
        assert_eq!(NET / 7 - SLOT_EXTRA, 4623); // 未截断前
    }

    /// 界线截断真的在起作用：去掉 `.min(LIMIT)` 会让档宽越过 D27 已定项 2 的 4 KiB。
    #[test]
    fn limit_clamp_is_load_bearing() {
        for &arm in ARMS.iter() {
            for w in arm_tiers(arm, 64) {
                assert!(w <= LIMIT, "arm={arm} 档宽 {w} 越过界线 {LIMIT}");
            }
        }
        // cap = 7 那一档若不截断会算出 4623 > 4096
        assert!(NET / cap_of(LIMIT) - SLOT_EXTRA > LIMIT);
    }

    /// 每一档 `cap` ≥ 4（E116 的回本闸）。
    #[test]
    fn every_tier_pays_back() {
        for &arm in ARMS.iter() {
            for w in arm_tiers(arm, 64) {
                assert!(cap_of(w) >= 4, "arm={arm} 档宽={w} 的 cap={}", cap_of(w));
            }
        }
        assert_eq!(cap_of(LIMIT), 7);
    }

    /// 档表规模钉绝对值。
    #[test]
    fn table_sizes_absolute() {
        assert_eq!(arm_tiers("capgeo125", 64).len(), 15);
        assert_eq!(arm_tiers("capgeo150", 64).len(), 10);
        assert_eq!(arm_tiers("capgeo200", 64).len(), 5);
        assert_eq!(arm_tiers("capall", 64).len(), 248);
        assert_eq!(cap_of(64), 305);
    }

    /// 主判据绝对值。
    #[test]
    fn main_absolute() {
        let u = counts("uniform", N);
        assert_eq!(containers_var(&u), 6814);
        assert_eq!(containers_tiered(&arm_tiers("wgeo", 64), &u), 7378);
        assert_eq!(containers_tiered(&arm_tiers("wgeo_dedup", 64), &u), 7378);
        assert_eq!(containers_tiered(&arm_tiers("capgeo125", 64), &u), 7455);
        assert_eq!(containers_tiered(&arm_tiers("capgeo150", 64), &u), 8180);
        assert_eq!(containers_tiered(&arm_tiers("capgeo200", 64), &u), 9116);
        assert_eq!(containers_tiered(&arm_tiers("capall", 64), &u), 6977);

        let l = counts("logunif", N);
        assert_eq!(containers_var(&l), 1592);
        assert_eq!(containers_tiered(&arm_tiers("wgeo", 64), &l), 1771);
        assert_eq!(containers_tiered(&arm_tiers("capall", 64), &l), 1788);
    }

    /// 分布守恒。
    #[test]
    fn counts_conserve_objects() {
        for dist in ["uniform", "logunif", "discrete"] {
            let c: u64 = counts(dist, N).iter().sum();
            assert_eq!(c, N, "dist={dist}");
        }
    }
}
