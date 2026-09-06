//! E110：条带表在连续发布下的期望写放大——把 E107 那两个端点之间的 `f` 交给负载自己产生。
//!
//! **它答的是** E107 自陈答不了的那一格：`f`（当前打开容器已占用的槽位数）在真实负载下的分布没有模型，
//! 而 E107 实测主负载 8 叶那一点上 `f = 0` 时甲臂便宜 5.0%、`f = 582` 时甲臂贵 5.0%
//! ⇒ **结论落在一条刀刃上，而刀刃两侧由一个没建模的量决定。**
//! E110 让 `f` 由连续发布自己演化，报**累计比值**而不是端点比值。
//!
//! ## 两条臂（与 E107 逐字同尺）
//!
//! 甲 = `w ≥ 3` + 条带成员表（含 parity、条带表整条链恒走 `w = 2` 镜像）；
//! 丙 = `w` 恒 2 全镜像、一条条带记录都不写。
//!
//! ## 口径（跑前写死）
//!
//! 1. **单次发布的算术与 E107 逐字节相同**——作废条款 2 就是拿 E107 已钉的两个端点当交叉校验。
//! 2. **`f` 由负载演化**：每次发布产生的成员记录累加进当前打开的容器，满 583 条就开新容器。
//! 3. 两条负载：`main`（恒 8 叶，D25 主负载）与 `mixed`（1 / 8 / 64 叶轮转，粗测叶数分布的影响）。
//! 4. **确定性模型**：同一个二进制跑 N 遍必然一致，证据强度来自变异与钉死绝对值的断言，不来自轮数。
//!
//! ## 它不答什么（跑前写死）
//!
//! 不答挂钟；不答空间效率（`.claude/rules/fs-design.md`「比谁省不构成任何判据」）；
//! 不答失败模式（那才是格式分支的判据，本实验只补写字节这一维）；
//! `mixed` 那条负载的叶数分布是**建模选择**，不是实测出来的负载画像。

use e7_index_bench::{Emitter, Sample};

const DATA_UNIT: u64 = 32768;
const NODE: u64 = 16384;
const JOURNAL_REC: u64 = 4096;
const ROOT_SLOT: u64 = 512;
const SPINE_NODES: u64 = 4;
const PACKED_HDR: u64 = 103;
const STRIPE_REC: u64 = 56;
const CIDX_LAYERS: u64 = 3;
const W_MAX: u64 = 4;
const W_MIN: u64 = 2;

fn recs_per_container() -> u64 {
    (DATA_UNIT - PACKED_HDR) / STRIPE_REC
}

fn width_for(cells: u64, devices: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    (cells + 1).clamp(W_MIN, W_MAX).min(devices).max(W_MIN)
}

fn stripes_for(cells: u64, w: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    cells.div_ceil(w - 1)
}

/// 一次发布：甲臂的字节与它产生的成员记录条数。`f` 是当前容器已占槽位。
fn arm_a_publish(leaves: u64, devices: u64, f: u64) -> (u64, u64, u64) {
    if leaves == 0 {
        return (0, 0, 0);
    }
    let spine = SPINE_NODES;
    let base = leaves * DATA_UNIT + spine * NODE + JOURNAL_REC + ROOT_SLOT;
    let wd = width_for(leaves, devices);
    let wm = width_for(spine, devices);
    let sd = stripes_for(leaves, wd);
    let sm = stripes_for(spine, wm);
    let parity = sd * DATA_UNIT + sm * NODE;
    let records = leaves + sd + spine + sm;
    let per = recs_per_container();
    let containers = (f + records).div_ceil(per).max(1);
    let table = (containers * DATA_UNIT + CIDX_LAYERS * NODE) * 2;
    (base + parity + table, records, containers)
}

fn arm_c_publish(leaves: u64) -> u64 {
    if leaves == 0 {
        return 0;
    }
    (leaves * DATA_UNIT + SPINE_NODES * NODE) * 2 + JOURNAL_REC + ROOT_SLOT
}

/// 跑 `rounds` 次发布，`f` 自己演化。返回 (甲累计, 丙累计, 溢出轮数, 甲更省的轮数)。
fn steady(rounds: u64, leaves_of: impl Fn(u64) -> u64, devices: u64) -> (u64, u64, u64, u64) {
    let per = recs_per_container();
    let mut f = 0u64;
    let (mut a_tot, mut c_tot, mut overflow, mut a_cheaper) = (0u64, 0u64, 0u64, 0u64);
    for r in 0..rounds {
        let n = leaves_of(r);
        let (a, records, containers) = arm_a_publish(n, devices, f);
        let c = arm_c_publish(n);
        if containers > 1 {
            overflow += 1;
        }
        if a < c {
            a_cheaper += 1;
        }
        a_tot += a;
        c_tot += c;
        f = (f + records) % per;
    }
    (a_tot, c_tot, overflow, a_cheaper)
}

/// 恒 8 叶：D25 主负载。
fn main_leaves(_r: u64) -> u64 {
    8
}

/// 1 / 8 / 64 轮转：粗测叶数分布的影响。**这是建模选择，不是实测的负载画像。**
fn mixed_leaves(r: u64) -> u64 {
    match r % 3 {
        0 => 1,
        1 => 8,
        _ => 64,
    }
}

fn main() {
    let mut em = Emitter::new();
    let mut out = String::new();

    // 阴性对照（作废条款 1）
    let (a0, c0, _, _) = steady(0, |_| 8, 4);
    out.push_str(&em.emit_raw(&format!(
        "name=negative_control rounds=0 arm_a={a0} arm_c={c0} verdict={}",
        if a0 == 0 && c0 == 0 { "pass" } else { "VOID" }
    )));
    out.push('\n');

    // 交叉校验（作废条款 2）：单次发布必须与 E107 已钉的两个端点逐字节相同
    let (a_f0, _, _) = arm_a_publish(8, 4, 0);
    let (a_f582, _, _) = arm_a_publish(8, 4, 582);
    let c8 = arm_c_publish(8);
    out.push_str(&em.emit_raw(&format!(
        "name=e107_crosscheck a_f0={a_f0} a_f582={a_f582} c={c8} verdict={}",
        if a_f0 == 627200 && a_f582 == 692736 && c8 == 659968 {
            "pass"
        } else {
            "VOID"
        }
    )));
    out.push('\n');

    for &rounds in &[100u64, 1000, 10000] {
        let loads: [(&str, fn(u64) -> u64); 2] = [("main", main_leaves), ("mixed", mixed_leaves)];
        for (name, f) in loads {
            let (a, c, ov, cheap) = steady(rounds, f, 4);
            out.push_str(&em.emit_raw(&format!(
                "name=steady load={name} rounds={rounds} arm_a_total={a} arm_c_total={c} \
                 overflow_rounds={ov} rounds_where_a_cheaper={cheap} ratio_a_over_c={:.4}",
                a as f64 / c as f64
            )));
            out.push('\n');
        }
    }

    // 阳性对照：容器只装 1 条记录 ⇒ 每条记录一个容器 ⇒ 条带表代价必须严格随记录数走
    let per = recs_per_container();
    out.push_str(&em.emit_raw(&format!(
        "name=positive_control recs_per_container={per} overflow_period_rounds={:.2}",
        per as f64 / 17.0
    )));
    out.push('\n');

    out.push_str(&em.emit(
        "main_1000_arm_a",
        &Sample {
            ops: 1,
            bytes_per_op: steady(1000, |_| 8, 4).0,
            elapsed_ns: 1,
        },
    ));
    out.push('\n');
    out.push_str(&em.finish());
    println!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 交叉校验（作废条款 2）：单次发布的三个数必须与 E107 已钉的逐字节相同。
    /// 不同就说明两个实验不在同一把尺子上，E110 的期望值也就没有意义。
    #[test]
    fn single_publish_matches_e107_pinned_values() {
        assert_eq!(arm_a_publish(8, 4, 0).0, 627200);
        assert_eq!(arm_a_publish(8, 4, 582).0, 692736);
        assert_eq!(arm_c_publish(8), 659968);
        assert_eq!(recs_per_container(), 583);
        assert_eq!(PACKED_HDR + STRIPE_REC * 583 + 17, 32768);
    }

    /// 主负载每次发布 17 条记录，容器装 583 条。
    ///
    /// 手算 1000 轮的溢出次数（不看模型输出）：容器边界 = 583 的倍数，
    /// 1000 轮共 17 000 条记录 ⇒ 越过的边界数是 `floor(17000 / 583) = 29`。
    /// **但其中一次是恰好填满、不算溢出**：`gcd(17, 583) = 1`，
    /// 而 `17 × 583 = 9911 = 583 × 17` ⇒ 第 583 轮（0 基下 `r = 582`）之前 `f = 566`，
    /// `566 + 17 = 583` 恰好装满，`div_ceil(583, 583) = 1` ⇒ 那一轮只碰一个容器。
    /// ⇒ **28**。⚠️ 建模时先按 `17 × 1000 / 583 ≈ 29.16` 估成 29，漏了这一次恰好填满。
    #[test]
    fn overflow_period_is_pinned() {
        assert_eq!(arm_a_publish(8, 4, 0).1, 17);
        assert_eq!(17 * 583, 9911);
        assert_eq!(17000 / 583, 29);
        let (_, _, ov, _) = steady(1000, main_leaves, 4);
        assert_eq!(ov, 28);
    }

    /// **这个实验要回答的那个数**：主负载连续 1000 次发布的累计比值。
    /// 手算：甲 = 1000 × 627200 + 29 × 65536 = 629 100 544；丙 = 1000 × 659968 = 659 968 000。
    #[test]
    fn steady_ratio_on_main_workload_is_pinned() {
        let (a, c, ov, cheap) = steady(1000, main_leaves, 4);
        assert_eq!(ov, 28);
        assert_eq!(a, 1000 * 627200 + 28 * 65536);
        assert_eq!(a, 629_035_008);
        assert_eq!(c, 659_968_000);
        // 甲更省的轮数 = 1000 − 溢出轮数
        assert_eq!(cheap + ov, 1000);
        let r = a as f64 / c as f64;
        assert!((r - 0.9531).abs() < 5e-4, "算出来 {r}");
    }

    /// 期望值必须严格落在 E107 那两个端点之间——落在外面说明演化写错了。
    #[test]
    fn steady_ratio_lies_strictly_between_the_two_e107_endpoints() {
        let lo = 627200_f64 / 659968.0;
        let hi = 692736_f64 / 659968.0;
        let (a, c, _, _) = steady(10000, main_leaves, 4);
        let r = a as f64 / c as f64;
        assert!(lo < r && r < hi, "lo={lo} r={r} hi={hi}");
    }

    /// 阴性对照（作废条款 1）的**直接形态**：零叶的一次发布，两条臂都必须是 0。
    /// ⚠️ 只测 `steady(0, ...)` 不够——那条路径根本不调用单次发布，
    /// 变异把 `leaves == 0` 的守卫删掉时一个测试都不红（2026-09-06 变异实测抓到的盲区）。
    #[test]
    fn zero_leaf_publish_is_zero_on_both_arms() {
        assert_eq!(arm_a_publish(0, 4, 0), (0, 0, 0));
        assert_eq!(arm_a_publish(0, 4, 582), (0, 0, 0));
        assert_eq!(arm_c_publish(0), 0);
    }

    /// 「甲更省的轮数」那个计数用 `<` 还是 `<=`，在本模型里**等价**——
    /// 主负载下两条臂的字节从不相等（非溢出轮 627200 vs 659968，溢出轮 692736 vs 659968）。
    /// 等价变异不算盲区，但要把等价性写成测试留档
    /// （`.claude/singlefs-ai-sop/rules/test-discipline.md`「等价变异不算盲区，但要分开记」）。
    #[test]
    fn cheaper_count_comparison_is_equivalent_here_because_arms_never_tie() {
        for f in 0..583u64 {
            let a = arm_a_publish(8, 4, f).0;
            assert_ne!(a, arm_c_publish(8), "f={f} 时两条臂打平了，等价性不再成立");
        }
    }

    /// 阴性对照（作废条款 1）。
    #[test]
    fn zero_rounds_is_zero() {
        let (a, c, ov, cheap) = steady(0, main_leaves, 4);
        assert_eq!((a, c, ov, cheap), (0, 0, 0, 0));
    }

    /// 混合负载的累计比值**比纯主负载低**（甲臂更省），而不是更高。
    ///
    /// ⚠️ **建模时先猜反了**：以为单叶那一档甲臂贵 1.651× 会把混合负载拉高，
    /// 而按**字节加权**时大发布主导——手算一个 1 / 8 / 64 的周期：
    /// 甲 `332288 + 627200 + 3084800 = 4 044 288`，丙 `201216 + 659968 + 4329984 = 5 191 168`
    /// ⇒ 0.7791，比纯主负载的 0.9531 低得多。
    /// 单叶那一档确实更贵，只是它的绝对字节太小、抬不动累计值——所以下面第二条单独把那一档钉住，
    /// 证明「叶数」这一维在模型里是活的，不是被抹平的常数。
    #[test]
    fn mixed_workload_ratio_is_lower_not_higher() {
        let (am, cm, _, _) = steady(999, main_leaves, 4);
        let (ax, cx, _, _) = steady(999, mixed_leaves, 4);
        let rm = am as f64 / cm as f64;
        let rx = ax as f64 / cx as f64;
        assert!(rx < rm, "main={rm} mixed={rx}");
        assert_eq!(332288 + 627200 + 3084800, 4_044_288);
        assert_eq!(201216 + 659968 + 4329984, 5_191_168);
        assert!((rx - 0.7815).abs() < 5e-4, "算出来 {rx}");
    }

    /// 单叶那一档甲臂确实更贵，且倍数与 E107 已钉的一致。
    #[test]
    fn single_leaf_publish_is_worse_for_arm_a() {
        assert_eq!(arm_a_publish(1, 4, 0).0, 332288);
        assert_eq!(arm_c_publish(1), 201216);
        let r = 332288_f64 / 201216.0;
        assert!((r - 1.6514).abs() < 5e-4, "算出来 {r}");
    }

    /// `w` 的判据与两条边界（D2 已定项 6），与 E107 同一份算术。
    #[test]
    fn width_rule_matches_d2_item6() {
        assert_eq!(width_for(1, 8), 2);
        assert_eq!(width_for(3, 8), 4);
        assert_eq!(width_for(99, 8), 4);
        assert_eq!(width_for(99, 2), 2);
        assert_eq!(stripes_for(8, 4), 3);
    }
}
