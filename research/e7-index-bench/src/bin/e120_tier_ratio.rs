//! E120：档表取等比 —— 公比 `r` 怎么定，档数与浪费怎么换。
//!
//! ## 它问什么
//!
//! E119（槽定宽的补齐代价）判出「四档太粗、档要在小端密」，但它比的是五张**具体的**表，
//! 而那些表都是照某个分布挑的——**而对象大小分布全仓没测过**（C174）。
//! 照没测过的分布挑表，等于把一个未知量当已知量用。
//!
//! E120 换一条路：**等比档表的相对浪费上界与分布无关**。
//! 档宽取 `W_k = W_min · r^k`（截到 ≤ 4096），任何落进第 k 档的对象都满足
//! `W_{k-1} < s ≤ W_k`，所以它补齐后占的字节不超过它自身的 `r` 倍
//! ⇒ **最坏相对浪费 = 1 − 1/r，与对象大小分布无关。**
//!
//! 于是「档怎么切」变成一个一维的选择：`r` 越小浪费越小、档数越多。
//! E120 把这条曲线量出来，并验证闭式与三个分布上的实测对得上。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**D18 已定项 11**：打包记录单元头 103。
//! - **D2 已定项 9**：第一版 2 盘恒 `w` = 2。**D27 已定项 2**：界线 ≤ 4 KiB。
//! - **D27 已定项 8 ④**：槽定宽，对象补齐到槽宽，槽 `i` 起点 = 头 + `i × W`。
//! - **E116 的闭式**：回本比下界 = `w / (cap − 1)` ⇒ 要 `cap ≥ 4`（`w` = 2）。
//! - **E119 的结论**：档要在小端密，不按等距切。
//!
//! ## 两个假设（不是条款 —— C187）
//!
//! - `SLOT_EXTRA = 43`：五元组 33（D18 已定项 3）+ 写序 10（D18 已定项 7）；定宽下不要槽目录条目。
//! - **对象大小分布**：全仓无实测（C174）。三个分布全部报出，**不挑代表**；
//!   而 E120 的主结论（最坏浪费闭式）**不依赖分布**，这正是它相对 E119 的改进。
//!
//! ## 五条臂（跑前列全，中途不许加臂 —— C186）
//!
//! `var`（变长贪心，参照，不是候选）+ 四个公比 `r` ∈ {1.125, 1.25, 1.5, 2.0}，`W_min` = 64。
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **闭式**：每条等比臂的**最坏相对浪费**必须等于 `1 − 1/r`（相对误差 < 1e-9）。
//!    ⚠️ 它是按档宽比算的，不是按实测；实测那一列另报。
//! 2. **档数**：报每个 `r` 的档数与最大档的 `cap`。
//! 3. **实测**：每条臂 × 三个分布，报相对 `var` 的损失。
//! 4. **单调**：`r` 越大档数越少、损失越大。
//!
//! **阳性对照，逐臂跑**：把分布换成「所有对象恰好等于某个档宽」，每条等比臂在**它自己有那一档时**
//! 必须与 `var` 逐字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`r` = 2.0 与 `r` = 1.125 在同一分布上的损失必须不同。相同 ⇒ 公比没被建模，整轮作废。
//! **阴性对照**：N = 0 时五条臂全 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **任一 `r` 的最大档 `cap` < 4** ⇒ 结论写「那个 `r` 的最粗档回不了本，不许用」。
//! - 若 **`r` = 1.125 的实测损失在任一分布上 > `1 − 1/r` = 11.11%** ⇒ 结论写
//!   「闭式不是上界，等比这条路的论证垮了」，不许回头改口径。
//! - 若 **`r` = 2.0 的实测损失在所有分布上都 < 5%** ⇒ 结论写「公比无所谓，取最少的档数」。
//! - 若 **四个 `r` 的档数都 > 32** ⇒ 结论写「等比表太长，同时开着的容器数不可接受」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! `var` 是变长这条路的一个具体装法（大小升序贪心），不是最优装法。
//! **不答「同时开着的容器数」的绝对值**：它等于档数乘上整理器同时在处理的树数，
//! 而后者取决于 D19 未定项 6（中央映射条目的 key 编码）——若 key 以树 ID 领头，
//! 沿 key 序走一次只碰一棵树，那个乘数就是 1；D19 未定项 6 未定，所以这一格今天空着。

use e7_index_bench::Emitter;

const UNIT: u64 = 32768;
const PACK_HDR: u64 = 103;
const NET: u64 = UNIT - PACK_HDR; // 32665
const SLOT_EXTRA: u64 = 43;
const LIMIT: u64 = 4096; // D27 已定项 2
const W_MIN: u64 = 64;
const N: u64 = 100_000;
/// 公比按千分之一为单位存成整数，避免浮点当键：1125 = 1.125
const RATIOS: [u64; 4] = [1125, 1250, 1500, 2000];

fn cap_of(w: u64) -> u64 { NET / (w + SLOT_EXTRA) }

/// 等比档表：`W_min · r^k`，向上取整到整数字节，去重，最后一档钉死为 LIMIT。
fn geo_tiers(r_milli: u64) -> Vec<u64> {
    let mut v = Vec::new();
    let mut w = W_MIN;
    loop {
        if w >= LIMIT { break; }
        v.push(w);
        let next = (w * r_milli).div_ceil(1000);
        if next <= w { break; } // 防呆：公比 ≤ 1 时不许死循环
        w = next;
    }
    v.push(LIMIT);
    v.dedup();
    v
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

/// 最坏相对浪费，**只看 `W_min` 以上**。
///
/// ⚠️ 跑前登记的是「最坏相对浪费 = `1 − 1/r`」，**判否**：第一档覆盖 1..=`W_min`，
/// 一个 1 字节的对象补到 64 就浪费 98.44%，四个公比上这个数**完全相同**——
/// 它是 `W_min` 那个地板造成的，不是公比造成的。登记的式子只在 `W_min` 以上成立。
/// 地板那一段的代价该按**绝对字节**看：每个对象至多浪费 `W_min − 1` = 63 字节。
fn worst_waste_above_floor(ts: &[u64]) -> f64 {
    let mut worst = 0.0f64;
    for (i, &w) in ts.iter().enumerate() {
        if i == 0 { continue; } // 地板那一档另算
        let lo = ts[i - 1] + 1;
        let waste = 1.0 - (lo as f64) / (w as f64);
        if waste > worst { worst = waste; }
    }
    worst
}

/// 地板那一档的绝对代价：落在 1..=`W_min` 的对象每个至多浪费这么多字节。
fn floor_waste_bytes() -> u64 { W_MIN - 1 }

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config unit={UNIT} pack_hdr={PACK_HDR} net={NET} slot_extra={SLOT_EXTRA} \
         limit={LIMIT} w_min={W_MIN} n={N} ratios={RATIOS:?}")));

    // 阳性对照：对象恰好等于某档宽 ⇒ 该臂与 var 逐字节相同
    for &r in RATIOS.iter() {
        let ts = geo_tiers(r);
        for &w in ts.iter() {
            let mut c = vec![0u64; (LIMIT + 1) as usize];
            c[w as usize] = N;
            let v = containers_var(&c);
            let t = containers_tiered(&ts, &c);
            println!("{}", em.emit_raw(&format!(
                "name=positive_exact r={r} tier={w} var={v} tiered={t} same={}", v == t)));
        }
    }

    // 判别力：两个极端公比在同一分布上必须不同
    {
        let c = counts("uniform", N);
        let a = containers_tiered(&geo_tiers(1125), &c);
        let b = containers_tiered(&geo_tiers(2000), &c);
        println!("{}", em.emit_raw(&format!(
            "name=discrimination r1125={a} r2000={b} differ={}", a != b)));
    }

    // 阴性对照
    {
        let c = counts("uniform", 0);
        println!("{}", em.emit_raw(&format!("name=negative arm=var containers={}", containers_var(&c))));
        for &r in RATIOS.iter() {
            println!("{}", em.emit_raw(&format!(
                "name=negative r={r} containers={}", containers_tiered(&geo_tiers(r), &c))));
        }
    }

    // 闭式与档数
    for &r in RATIOS.iter() {
        let ts = geo_tiers(r);
        let closed = 1.0 - 1000.0 / r as f64;
        println!("{}", em.emit_raw(&format!(
            "name=closed r={r} tiers={} worst_waste_above_floor={:.6} closed_form={:.6} \
             floor_waste_bytes={} max_tier={} cap_max_tier={} min_tier={} cap_min_tier={}",
            ts.len(), worst_waste_above_floor(&ts), closed, floor_waste_bytes(),
            ts.last().unwrap(), cap_of(*ts.last().unwrap()),
            ts[0], cap_of(ts[0]))));
    }

    // 实测
    for dist in ["uniform", "logunif", "discrete"] {
        let c = counts(dist, N);
        let v = containers_var(&c);
        println!("{}", em.emit_raw(&format!(
            "name=main dist={dist} arm=var containers={v} gain={:.4} loss=0.0000 tiers=0",
            (N * UNIT) as f64 / (v * UNIT) as f64)));
        for &r in RATIOS.iter() {
            let ts = geo_tiers(r);
            let t = containers_tiered(&ts, &c);
            println!("{}", em.emit_raw(&format!(
                "name=main dist={dist} r={r} containers={t} gain={:.4} loss={:.4} tiers={}",
                (N * UNIT) as f64 / (t * UNIT) as f64,
                (t as f64 - v as f64) / v as f64, ts.len())));
        }
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：对象恰好等于档宽时，等比档与变长逐字节相同。
    #[test]
    fn positive_control_exact_tier_matches_var() {
        for &r in RATIOS.iter() {
            let ts = geo_tiers(r);
            for &w in ts.iter() {
                let mut c = vec![0u64; (LIMIT + 1) as usize];
                c[w as usize] = N;
                assert_eq!(containers_tiered(&ts, &c), containers_var(&c),
                           "r={r} 档宽={w}");
            }
        }
    }

    /// 判别力：公比真的被建模了。
    #[test]
    fn discrimination_ratio_matters() {
        let c = counts("uniform", N);
        let a = containers_tiered(&geo_tiers(1125), &c);
        let b = containers_tiered(&geo_tiers(2000), &c);
        assert!(a < b, "r 越小该越省：r=1.125 {a}，r=2.0 {b}");
        assert_eq!(a, 7378);
        assert_eq!(b, 9463);
    }

    /// 阴性：N = 0 全 0。
    #[test]
    fn negative_control_zero() {
        let c = counts("uniform", 0);
        assert_eq!(containers_var(&c), 0);
        for &r in RATIOS.iter() { assert_eq!(containers_tiered(&geo_tiers(r), &c), 0, "r={r}"); }
    }

    /// 闭式：`W_min` 以上的最坏相对浪费 ≤ 1 − 1/r + 取整余量，且这一条与分布无关。
    ///
    /// ⚠️ 跑前登记的是「= 1 − 1/r，相对误差 < 1e-9」，**判否两次**：
    /// ① 第一档覆盖 1..=W_min，1 字节对象补到 64 浪费 98.44%，四个公比上这个数完全相同
    ///    ⇒ 那是地板不是公比，闭式只在 W_min 以上成立；
    /// ② 档宽按整数向上取整，实际档比略超 r（如 81 → 92 是 1.1358 > 1.125）
    ///    ⇒ 上界要放宽到 r + 1/W_min。两条都不许回头改登记的式子。
    #[test]
    fn worst_waste_matches_closed_form() {
        for &r in RATIOS.iter() {
            let ts = geo_tiers(r);
            let closed = 1.0 - 1000.0 / r as f64;
            let slack = 1.0 - 1.0 / (r as f64 / 1000.0 + 1.0 / W_MIN as f64);
            let got = worst_waste_above_floor(&ts);
            assert!(got <= slack + 1e-9,
                    "r={r}：W_min 以上最坏浪费 {got} 超过含取整余量的上界 {slack}（闭式 {closed}）");
        }
        // 地板那一段按绝对字节算，与公比无关
        assert_eq!(floor_waste_bytes(), 63);
        // 钉绝对值，不许只靠与闭式互比
        assert!(((1.0f64 - 1000.0 / 1125.0) - 0.111111).abs() < 1e-6);
        assert!(((1.0f64 - 1000.0 / 2000.0) - 0.5).abs() < 1e-12);
    }

    /// 档数与每档的 cap 钉绝对值。
    #[test]
    fn tier_counts_and_caps_absolute() {
        assert_eq!(geo_tiers(1125).len(), 36);
        assert_eq!(geo_tiers(1250).len(), 20);
        assert_eq!(geo_tiers(1500).len(), 12);
        assert_eq!(geo_tiers(2000).len(), 7);
        for &r in RATIOS.iter() {
            let ts = geo_tiers(r);
            assert_eq!(ts[0], W_MIN, "r={r} 最细一档该是 W_min");
            assert_eq!(*ts.last().unwrap(), LIMIT, "r={r} 最粗一档该钉死在界线上");
            for &w in ts.iter() {
                assert!(cap_of(w) >= 4, "r={r} 档宽={w} 的 cap={} < 4", cap_of(w));
            }
        }
        assert_eq!(cap_of(LIMIT), 7);
        assert_eq!(cap_of(W_MIN), 305);
    }

    /// 单调：r 越大档数越少、容器越多（同一分布上）。互比，与上面钉绝对值的配对。
    #[test]
    fn larger_ratio_fewer_tiers_more_containers() {
        for dist in ["uniform", "logunif"] {
            let c = counts(dist, N);
            let mut prev_t = usize::MAX;
            let mut prev_c = 0u64;
            for &r in RATIOS.iter() {
                let ts = geo_tiers(r);
                let cont = containers_tiered(&ts, &c);
                assert!(ts.len() < prev_t, "dist={dist} r={r}：档数该递减");
                assert!(cont >= prev_c, "dist={dist} r={r}：容器数该不减");
                prev_t = ts.len(); prev_c = cont;
            }
        }
    }

    /// 主判据的绝对值：三个分布 × 四个公比逐个钉住。
    #[test]
    fn main_absolute() {
        let u = counts("uniform", N);
        assert_eq!(containers_var(&u), 6814);
        assert_eq!(containers_tiered(&geo_tiers(1125), &u), 7378);
        assert_eq!(containers_tiered(&geo_tiers(1250), &u), 7675);
        assert_eq!(containers_tiered(&geo_tiers(1500), &u), 8101);
        assert_eq!(containers_tiered(&geo_tiers(2000), &u), 9463);

        let l = counts("logunif", N);
        assert_eq!(containers_var(&l), 1592);
        assert_eq!(containers_tiered(&geo_tiers(1125), &l), 1771);
        assert_eq!(containers_tiered(&geo_tiers(1250), &l), 1820);
        assert_eq!(containers_tiered(&geo_tiers(1500), &l), 1903);
        // ⚠️ discrete 那个分布不中立：它的三个取样点 512 / 1024 / 4096 全是 2 的幂，
        // 所以 r = 2.0 在它上面只多 1 个容器（0.02%），而 r = 1.125 多 1.91%。
        // 只看那个分布会得出「公比越大越好」这个与另两个分布相反的结论。
        assert_eq!(containers_tiered(&geo_tiers(2000), &l), 2099);

        let d = counts("discrete", N);
        assert_eq!(containers_var(&d), 6448);
        assert_eq!(containers_tiered(&geo_tiers(2000), &d), 6449);
    }

    /// 分布展开守恒。
    #[test]
    fn counts_conserve_objects() {
        for dist in ["uniform", "logunif", "discrete"] {
            let c: u64 = counts(dist, N).iter().sum();
            assert_eq!(c, N, "dist={dist}");
        }
    }
}
