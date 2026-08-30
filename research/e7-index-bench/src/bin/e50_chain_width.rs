//! E50：反向链宽度 32 还是 64 —— D23（journal 的角色与格式）未定项 8 剩下的那半个问题。
//!
//! 三方论证已判「16 位不可接受」，依据是**一生至少出一次误接受**的概率。
//! 同一把尺子往上量一格：**64 位比 32 位买到什么。**
//!
//! ## 两侧各扫一遍定义域，不取样本点（C49 要拦的正是取样本点）
//!
//! - **代价**：记录头总字节 × 链宽 {2,4,8} × 点名项数 **0..2000** × 单元 {512,4096}，逐格判占几个单元。
//! - **收益**：两个**不同的分母**各算一遍——
//!   `D-事故` = 36 500（每天崩 10 次 × 十年，D23 未定项 9 已采纳过的场景点）；
//!   `D-遍历` = 记录核对器每遍比对的记录对数 × 一生遍数。
//!
//! ## 判据（跑前写死）
//!
//! 1. 「n 位够用」：该分母下「一生至少出一次」< 1%。
//! 2. 「n 位比 m 位免费」：扫描到的**全部**格上盘上占用相同。
//! 3. 两个分母的结论**分开给**，不许合成一句。
//!
//! ## 失败条款
//!
//! - 阳性对照一：8 位在 D-事故 下必须 ≈ 100%。不是 ⇒ 整轮作废。
//! - 阳性对照二：2 与 4 字节在 base=93 时必须分岔（512 单元下 19 处，最小 items=44）。判不分岔 ⇒ 整轮作废。
//! - 32 与 64 都够用且代价相同 ⇒ **如实记录「两者都够」**，不许改判据凑方向。
//!
//! ## 它答不了的
//!
//! `prev_hash` 覆盖哪些字段（**不满足时误接受 100%，与位宽无关**）；两个分母都是场景点不是测量；
//! hash 算法未定 ⇒ 「误接受 = 2⁻ⁿ」条件于链值均匀（E41 已实测截断 CRC 不满足）。

use e7_index_bench::Emitter;

const ITEM_BYTES: u64 = 56;

/// 一条记录在盘上占几个原子单元。`base` 是**不含链**的记录头总字节。
fn units_on_disk(base: u64, chain: u64, items: u64, unit: u64) -> u64 {
    let sz = base + chain + items * ITEM_BYTES;
    sz.div_ceil(unit)
}

/// 两个链宽在给定 `base` / `unit` 下，0..=`max_items` 里有多少个项数上占的单元数不同。
fn divergences(base: u64, a: u64, b: u64, unit: u64, max_items: u64) -> (u64, Option<u64>) {
    let mut n = 0u64;
    let mut first = None;
    for items in 0..=max_items {
        if units_on_disk(base, a, items, unit) != units_on_disk(base, b, items, unit) {
            n += 1;
            if first.is_none() {
                first = Some(items);
            }
        }
    }
    (n, first)
}

/// 误接受期望次数 = 机会数 × 2⁻ⁿ。
fn expected_false_accepts(chances: f64, bits: u32) -> f64 {
    chances / 2f64.powi(bits as i32)
}

/// 「一生至少出一次」的概率。泊松近似：1 − e^(−λ)。
fn at_least_once(expected: f64) -> f64 {
    1.0 - (-expected).exp()
}

/// 判据 1：「n 位够用」= 一生至少出一次的概率 < 1%。**阈值写死在这里，跑前定的。**
fn enough(at_least_once_p: f64) -> bool {
    at_least_once_p < 0.01
}

/// 仓里出现过的记录头候选（不含链）：
/// 78 = 84 − tail_lsn 8 + 2；84 = E24 原表；86 = jsn 加宽后；
/// 93 / 95 = 加上未定项 7 的事务字段 9；99 = 再加 4 字节。
const BASES: [u64; 6] = [78, 84, 86, 93, 95, 99];
const CHAINS: [u64; 3] = [2, 4, 8];
const UNITS: [u64; 2] = [512, 4096];
const MAX_ITEMS: u64 = 2000;
const BITS: [u32; 4] = [8, 16, 32, 64];

const INCIDENT_CHANCES: f64 = 36_500.0; // D-事故：每天崩 10 次 × 十年
const RINGS: [u64; 3] = [10 * 1024 * 1024, 100 * 1024 * 1024, 2 * 1024 * 1024 * 1024];
const REC_ON_DISK: [u64; 2] = [512, 4096];
const PASSES: [(&str, f64); 2] = [("weekly", 522.0), ("daily", 3650.0)];

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config item_bytes={ITEM_BYTES} max_items={MAX_ITEMS} \
             incident_chances={INCIDENT_CHANCES}"
        ))
    );

    // ── 代价：扫遍定义域 ──
    let mut free_24 = 0u64;
    let mut free_48 = 0u64;
    let mut cells = 0u64;
    for base in BASES {
        for unit in UNITS {
            let (d24, f24) = divergences(base, 2, 4, unit, MAX_ITEMS);
            let (d48, f48) = divergences(base, 4, 8, unit, MAX_ITEMS);
            cells += 1;
            if d24 == 0 {
                free_24 += 1;
            }
            if d48 == 0 {
                free_48 += 1;
            }
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=cost base={base} unit={unit} div_2v4={d24} first_2v4={} \
                     div_4v8={d48} first_4v8={} cap_items={}",
                    f24.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    f48.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                    unit.saturating_sub(base + 8) / ITEM_BYTES,
                ))
            );
        }
    }
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=cost_summary cells={cells} free_2v4={free_24} free_4v8={free_48}"
        ))
    );

    // ── 收益一：D-事故 ──
    for bits in BITS {
        let e = expected_false_accepts(INCIDENT_CHANCES, bits);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=risk_incident bits={bits} chances={INCIDENT_CHANCES} expected={e:.6e} \
                 at_least_once={:.6e} enough={}",
                at_least_once(e),
                u8::from(enough(at_least_once(e))),
            ))
        );
    }

    // ── 收益二：D-遍历 ──
    for ring in RINGS {
        for rec in REC_ON_DISK {
            for (freq, passes) in PASSES {
                let pairs_per_pass = (ring / rec) as f64;
                let chances = pairs_per_pass * passes;
                for bits in BITS {
                    let e = expected_false_accepts(chances, bits);
                    println!(
                        "{}",
                        em.emit_raw(&format!(
                            "name=risk_traversal ring={ring} rec_on_disk={rec} freq={freq} \
                             pairs_per_pass={pairs_per_pass:.0} chances={chances:.6e} bits={bits} \
                             expected={e:.6e} at_least_once={:.6e} enough={}",
                            at_least_once(e),
                            u8::from(enough(at_least_once(e))),
                        ))
                    );
                }
            }
        }
    }

    // ── 阳性对照 ──
    let e8 = expected_false_accepts(INCIDENT_CHANCES, 8);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_8bit_incident at_least_once={:.6} expect_near_one=1",
            at_least_once(e8)
        ))
    );
    let (d, f) = divergences(93, 2, 4, 512, MAX_ITEMS);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_base93_diverges div={d} first={} expect_first=44",
            f.map(|v| v.to_string()).unwrap_or_else(|| "NA".into())
        ))
    );

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **盘上占用的绝对值**：base=93、链 2 字节、点名 44 项 ⇒ 2559 B ⇒ 5 个 512 单元；
    /// 链 4 字节 ⇒ 2561 B ⇒ **6 个**。这就是 E40 抽样点漏掉的那个分岔。
    #[test]
    fn base_93_diverges_at_forty_four_items() {
        assert_eq!(93 + 2 + 44 * 56, 2559);
        assert_eq!(93 + 4 + 44 * 56, 2561);
        assert_eq!(units_on_disk(93, 2, 44, 512), 5);
        assert_eq!(units_on_disk(93, 4, 44, 512), 6);
        let (n, first) = divergences(93, 2, 4, 512, 2000);
        assert_eq!(first, Some(44));
        assert!(n >= 19, "0..2000 里至少 19 处（0..1200 内已数出 19）");
    }

    /// **`jsn` 加宽之后（base 95）2 与 4 不再分岔**——同一段算术，独立钉死。
    #[test]
    fn base_95_does_not_diverge_between_two_and_four() {
        assert_eq!(divergences(95, 2, 4, 512, 2000).0, 0);
        assert_eq!(divergences(95, 2, 4, 4096, 2000).0, 0);
    }

    /// **「4 与 8 字节免费」同样是 base 依赖的，不是结构性质**——本实验的核心代价结论。
    /// 扫遍 0..2000 项、两档单元，不取样本点；绝对值逐 base 钉死。
    /// ⚠️ 这条把「64 位也免费」从一句话变成一个**条件式**。
    #[test]
    fn whether_four_versus_eight_is_free_depends_on_the_base() {
        // 分岔的：base 84（首个 items=35）与 base 99（首个 items=53），各 31 处 / 512 单元
        for (base, first) in [(84u64, 35u64), (99, 53)] {
            let (n, f) = divergences(base, 4, 8, 512, 2000);
            assert_eq!(n, 31, "base={base}");
            assert_eq!(f, Some(first));
            assert_eq!(divergences(base, 4, 8, 4096, 2000).0, 4);
        }
        // 不分岔的：78 / 86 / 93 / 95
        for base in [78u64, 86, 93, 95] {
            assert_eq!(divergences(base, 4, 8, 512, 2000).0, 0, "base={base}");
            assert_eq!(divergences(base, 4, 8, 4096, 2000).0, 0, "base={base}");
        }
    }

    /// **只有 base=95（今天定案后的现行值）两侧都免费**——2v4 与 4v8 同时为 0 的唯一一个。
    #[test]
    fn only_the_current_base_is_free_on_both_sides() {
        let both_free: Vec<u64> = BASES
            .into_iter()
            .filter(|&b| {
                UNITS
                    .iter()
                    .all(|&u| divergences(b, 2, 4, u, 2000).0 == 0 && divergences(b, 4, 8, u, 2000).0 == 0)
            })
            .collect();
        assert_eq!(both_free, vec![95]);
    }

    /// **一个 512 单元装几项**：base 95 + 8 字节链 ⇒ (512−103)/56 = 7 项，与链 4 字节时相同。
    #[test]
    fn capacity_per_unit_is_the_same_for_four_and_eight() {
        for base in BASES {
            let c4 = (512u64).saturating_sub(base + 4) / ITEM_BYTES;
            let c8 = (512u64).saturating_sub(base + 8) / ITEM_BYTES;
            assert_eq!(c4, c8, "base={base}");
        }
        assert_eq!((512u64 - 95 - 8) / 56, 7);
    }

    /// **风险算术的绝对值**：36 500 次机会下 16 位期望 0.5569 次、至少一次 42.70%；
    /// 32 位期望 8.499e-6、至少一次 8.499e-6。
    #[test]
    fn incident_risk_absolute_values() {
        let e16 = expected_false_accepts(36_500.0, 16);
        assert!((e16 - 0.556946).abs() < 1e-5, "{e16}");
        assert!((at_least_once(e16) - 0.427).abs() < 1e-3);
        let e32 = expected_false_accepts(36_500.0, 32);
        assert!((e32 - 8.4983e-6).abs() < 1e-9, "{e32}");
        assert!(at_least_once(e32) < 0.01, "32 位在事故分母下够用");
        assert!(at_least_once(e16) > 0.01, "16 位不够用");
    }

    /// **阳性对照**：8 位在事故分母下必须几乎必然出事。
    #[test]
    fn positive_control_eight_bits_is_a_certainty() {
        let e8 = expected_false_accepts(36_500.0, 8);
        assert!(e8 > 100.0, "{e8}");
        assert!(at_least_once(e8) > 0.999_999);
    }

    /// **遍历分母的绝对值**：10 MiB 环 / 每条 512 B ⇒ 每遍 20 480 对；每周 × 十年 = 522 遍
    /// ⇒ 1.069e7 次机会 ⇒ 32 位期望 2.489e-3、至少一次 0.2486%（**够用但不是零**）；
    /// 64 位期望 5.79e-13。
    #[test]
    fn traversal_risk_absolute_values() {
        let pairs = (10 * 1024 * 1024u64 / 512) as f64;
        assert_eq!(pairs, 20480.0);
        let chances = pairs * 522.0;
        assert!((chances - 1.069056e7).abs() < 1.0, "{chances}");
        let e32 = expected_false_accepts(chances, 32);
        assert!((e32 - 2.4888e-3).abs() < 1e-6, "{e32}");
        assert!(at_least_once(e32) < 0.01, "32 位在这一档仍够用");
        let e64 = expected_false_accepts(chances, 64);
        assert!(e64 < 1e-12);
    }

    /// **最恶劣的那一档遍历分母**：2 GiB 环 / 每条 512 B / 每天 × 十年
    /// ⇒ 每遍 4 194 304 对 × 3650 遍 = 1.531e10 次机会
    /// ⇒ **32 位期望 3.56 次、至少一次 97.2%**（远超 1% 判据）；64 位 8.3e-10。
    /// **这一档就是 64 位买到东西的地方。**
    #[test]
    fn the_worst_traversal_bucket_breaks_thirty_two_bits() {
        let pairs = (2 * 1024 * 1024 * 1024u64 / 512) as f64;
        assert_eq!(pairs, 4_194_304.0);
        let chances = pairs * 3650.0;
        assert!((chances - 1.53092e10).abs() < 1e6, "{chances}");
        let e32 = expected_false_accepts(chances, 32);
        assert!((e32 - 3.5645).abs() < 1e-3, "{e32}");
        assert!(at_least_once(e32) > 0.97, "32 位在这一档几乎必然出事");
        let e64 = expected_false_accepts(chances, 64);
        assert!(at_least_once(e64) < 1e-8, "64 位够用");
    }

    /// **够用判据的阈值本身**：1% 那条线钉死，且两侧各取一点。
    #[test]
    fn the_threshold_is_one_percent() {
        assert!(enough(0.009_999));
        assert!(!enough(0.010_001));
        assert!(!enough(0.427), "16 位在事故分母下的 42.7% 必须判不够用");
        assert!(enough(2.4860e-3), "32 位在最轻那档遍历分母下的 0.249% 必须判够用");
        assert!(!enough(0.971_69), "32 位在最重那档遍历分母下的 97.2% 必须判不够用");
    }

    /// **泊松近似本身**：小 λ 时 1−e^(−λ) ≈ λ；λ 大时趋近 1。
    #[test]
    fn at_least_once_behaves() {
        assert!((at_least_once(1e-6) - 1e-6).abs() < 1e-12);
        assert!(at_least_once(100.0) > 0.999_999);
        assert_eq!(at_least_once(0.0), 0.0);
    }
}
