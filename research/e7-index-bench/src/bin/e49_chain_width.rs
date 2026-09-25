//! E49：反向链宽度 32 还是 64 —— D23（journal 的角色与格式）已定项 8 剩下的那半个问题。
//!
//! 三方论证已判「16 位不可接受」，依据是**一生至少出一次误接受**的概率。
//! 同一把尺子往上量一格：**64 位比 32 位买到什么。**
//!
//! ## 两侧各扫一遍定义域，不取样本点（C49 要拦的正是取样本点）
//!
//! - **代价**：记录头总字节 × 链宽 {2,4,8} × 点名项数 **0..2000** × 单元 {512,4096}，逐格判占几个单元。
//! - **收益**：两个**不同的分母**各算一遍——
//!   `D-事故` = 36 500（每天崩 10 次 × 十年，D23 已定项 9 已采纳过的场景点）；
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
//! `previous_hash` 覆盖哪些字段（**不满足时误接受 100%，与位宽无关**）；两个分母都是场景点不是测量；
//! hash 算法未定 ⇒ 「误接受 = 2⁻ⁿ」条件于链值均匀（E40 已实测截断 CRC 不满足）。

use e7_index_bench::Emitter;

const ITEM_BYTES: u64 = 56;

/// 一条记录在盘上占几个原子单元。`base` 是**不含链**的记录头总字节。
fn units_on_disk(base: u64, chain_bytes: u64, items: u64, unit: u64) -> u64 {
    let record_bytes_before_rounding = base + chain_bytes + items * ITEM_BYTES;
    record_bytes_before_rounding.div_ceil(unit)
}

/// 两个链宽在给定 `base` / `unit` 下，0..=`largest_item_count` 里有多少个项数上占的单元数不同。
fn divergences(base: u64, narrow_chain_bytes: u64, wide_chain_bytes: u64, unit: u64, largest_item_count: u64) -> (u64, Option<u64>) {
    let mut divergent_item_count = 0u64;
    let mut first_divergent_items = None;
    for items in 0..=largest_item_count {
        if units_on_disk(base, narrow_chain_bytes, items, unit) != units_on_disk(base, wide_chain_bytes, items, unit) {
            divergent_item_count += 1;
            if first_divergent_items.is_none() {
                first_divergent_items = Some(items);
            }
        }
    }
    (divergent_item_count, first_divergent_items)
}

/// 读法乙：记录定长 `RECORD_BYTES_FIXED`（D23 已定项 12），一条装
/// ⌊(定长 − base − 链宽) ÷ 56⌋ 项（D23 已定项 17：一个点名项 56 字节）。
fn items_per_fixed_record(base: u64, chain_bytes: u64) -> u64 {
    (RECORD_BYTES_FIXED - base - chain_bytes) / ITEM_BYTES
}

/// 读法乙：`items` 项要几条定长记录。`max(1, ⌈N ÷ 每条项数⌉)`——
/// N = 0 时也要一条空发布记录（D23 已定项 19 ①）。
fn records_needed(base: u64, chain_bytes: u64, items: u64) -> u64 {
    let per_record = items_per_fixed_record(base, chain_bytes);
    items.div_ceil(per_record).max(1)
}

/// 读法乙版的 `divergences`：两个链宽在给定 `base` 下，0..=`largest_item_count`
/// 里有多少个项数上要的记录条数不同。
fn divergences_fixed_record(base: u64, narrow_chain_bytes: u64, wide_chain_bytes: u64, largest_item_count: u64) -> (u64, Option<u64>) {
    let mut divergent_item_count = 0u64;
    let mut first_divergent_items = None;
    for items in 0..=largest_item_count {
        if records_needed(base, narrow_chain_bytes, items) != records_needed(base, wide_chain_bytes, items) {
            divergent_item_count += 1;
            if first_divergent_items.is_none() {
                first_divergent_items = Some(items);
            }
        }
    }
    (divergent_item_count, first_divergent_items)
}

/// 「免费」判据：分岔数不超过门槛就算免费。原判据 2 的门槛 `tolerance = 0`；
/// 第八节的判别力自证把门槛挪到 `d`（该格自己的分岔数），判定必须翻面（V5）。
fn is_free(divergent_count: u64, tolerance: u64) -> bool {
    divergent_count <= tolerance
}

/// 误接受期望次数 = 机会数 × 2⁻ⁿ。
fn expected_false_accepts(chances: f64, bits: u32) -> f64 {
    chances / 2f64.powi(bits as i32)
}

/// 「一生至少出一次」的概率。泊松近似：1 − e^(−λ)。
fn at_least_once(expected_false_accept_count: f64) -> f64 {
    1.0 - (-expected_false_accept_count).exp()
}

/// 判据 1：「n 位够用」= 一生至少出一次的概率 < 1%。**阈值写死在这里，跑前定的。**
fn enough(at_least_once_probability: f64) -> bool {
    at_least_once_probability < 0.01
}

/// 仓里出现过的记录头候选（不含链）：
/// 78 = 84 − tail_lsn 8 + 2；84 = E23 原表；86 = jsn 加宽后；
/// 93 / 95 = 加上已定项 7 的事务字段 9；99 = 再加 4 字节。
const BASES: [u64; 6] = [78, 84, 86, 93, 95, 99];
const CHAINS: [u64; 3] = [2, 4, 8];
const UNITS: [u64; 2] = [512, 4096];
const LARGEST_ITEM_COUNT: u64 = 2000;
const BITS: [u32; 4] = [8, 16, 32, 64];

const INCIDENT_CHANCES: f64 = 36_500.0; // D-事故：每天崩 10 次 × 十年
const RINGS: [u64; 3] = [10 * 1024 * 1024, 100 * 1024 * 1024, 2 * 1024 * 1024 * 1024];
const RECORD_ON_DISK_BYTES: [u64; 2] = [512, 4096];
const PASSES: [(&str, f64); 2] = [("weekly", 522.0), ("daily", 3650.0)];

// ── E49 重跑登记（记录头 311）：`base` 这个旋钮上的两个被判取值 ──
// base 是「不含链」的记录头总字节（见 `units_on_disk` 文档注释）。

/// 反向链宽度（今天）：D23 已定项 8「宽度 32 位」＝ 4 字节。
const BACK_CHAIN_BYTES_TODAY: u64 = 4;
/// 记录头总字节（今天）：D23 已定项 4「头 311 字节」。门禁 27 号把这个名字与 kb 标记绑住。
const JOURNAL_HEADER_BYTES: u64 = 311;
/// 改之前那一档（2026-09-14 到 2026-09-24 现行）的记录头总字节。
const HEADER_BYTES_BEFORE: u64 = 307;
/// 今天的 base = 今天的记录头总字节 − 反向链宽度。
const BASE_TODAY: u64 = JOURNAL_HEADER_BYTES - BACK_CHAIN_BYTES_TODAY;
/// 改之前那一档的 base，同一个减法。
const BASE_BEFORE: u64 = HEADER_BYTES_BEFORE - BACK_CHAIN_BYTES_TODAY;
/// 读法乙：记录尺寸定长（D23 已定项 12）。
const RECORD_BYTES_FIXED: u64 = 4096;

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config item_bytes={ITEM_BYTES} max_items={LARGEST_ITEM_COUNT} \
             incident_chances={INCIDENT_CHANCES}"
        ))
    );

    // ── 代价：扫遍定义域 ──
    let mut free_cell_count_2_versus_4 = 0u64;
    let mut free_cell_count_4_versus_8 = 0u64;
    let mut cell_count = 0u64;
    for base in BASES {
        for unit in UNITS {
            let (divergent_count_2_versus_4, first_divergent_items_2_versus_4) = divergences(base, 2, 4, unit, LARGEST_ITEM_COUNT);
            let (divergent_count_4_versus_8, first_divergent_items_4_versus_8) = divergences(base, 4, 8, unit, LARGEST_ITEM_COUNT);
            cell_count += 1;
            if divergent_count_2_versus_4 == 0 {
                free_cell_count_2_versus_4 += 1;
            }
            if divergent_count_4_versus_8 == 0 {
                free_cell_count_4_versus_8 += 1;
            }
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=cost base={base} unit={unit} div_2v4={divergent_count_2_versus_4} first_2v4={} \
                     div_4v8={divergent_count_4_versus_8} first_4v8={} cap_items={}",
                    first_divergent_items_2_versus_4.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                    first_divergent_items_4_versus_8.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                    unit.saturating_sub(base + 8) / ITEM_BYTES,
                ))
            );
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=cost_summary cells={cell_count} free_2v4={free_cell_count_2_versus_4} free_4v8={free_cell_count_4_versus_8}"
        ))
    );

    // ── E49 重跑登记（记录头 311）：Q49.2 / Q49.3，读法甲在两个被判 base 上 ──
    for base in [BASE_BEFORE, BASE_TODAY] {
        for unit in UNITS {
            let (divergent_count_2_versus_4, first_divergent_items_2_versus_4) = divergences(base, 2, 4, unit, LARGEST_ITEM_COUNT);
            let (divergent_count_4_versus_8, first_divergent_items_4_versus_8) = divergences(base, 4, 8, unit, LARGEST_ITEM_COUNT);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=cost_current base={base} unit={unit} div_2v4={divergent_count_2_versus_4} first_2v4={} \
                     div_4v8={divergent_count_4_versus_8} first_4v8={} cap_items={}",
                    first_divergent_items_2_versus_4.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                    first_divergent_items_4_versus_8.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                    unit.saturating_sub(base + 8) / ITEM_BYTES,
                ))
            );
        }
    }

    // ── E49 重跑登记：Q49.4，读法乙在两个被判 base 上（与原子单元无关，不带 unit=） ──
    for base in [BASE_BEFORE, BASE_TODAY] {
        let (divergent_count_2_versus_4, first_divergent_items_2_versus_4) = divergences_fixed_record(base, 2, 4, LARGEST_ITEM_COUNT);
        let (divergent_count_4_versus_8, first_divergent_items_4_versus_8) = divergences_fixed_record(base, 4, 8, LARGEST_ITEM_COUNT);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=cost_fixed_record base={base} div_2v4={divergent_count_2_versus_4} first_2v4={} \
                 div_4v8={divergent_count_4_versus_8} first_4v8={} items_per_record_2={} items_per_record_4={} items_per_record_8={}",
                first_divergent_items_2_versus_4.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                first_divergent_items_4_versus_8.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                items_per_fixed_record(base, 2),
                items_per_fixed_record(base, 4),
                items_per_fixed_record(base, 8),
            ))
        );
    }

    // ── 收益一：D-事故 ──
    for bits in BITS {
        let expected_false_accept_count = expected_false_accepts(INCIDENT_CHANCES, bits);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=risk_incident bits={bits} chances={INCIDENT_CHANCES} expected={expected_false_accept_count:.6e} \
                 at_least_once={:.6e} enough={}",
                at_least_once(expected_false_accept_count),
                u8::from(enough(at_least_once(expected_false_accept_count))),
            ))
        );
    }

    // ── 收益二：D-遍历 ──
    for ring_bytes in RINGS {
        for record_on_disk_bytes in RECORD_ON_DISK_BYTES {
            for (pass_frequency, lifetime_pass_count) in PASSES {
                let pairs_per_pass = (ring_bytes / record_on_disk_bytes) as f64;
                let chances = pairs_per_pass * lifetime_pass_count;
                for bits in BITS {
                    let expected_false_accept_count = expected_false_accepts(chances, bits);
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=risk_traversal ring={ring_bytes} rec_on_disk={record_on_disk_bytes} freq={pass_frequency} \
                             pairs_per_pass={pairs_per_pass:.0} chances={chances:.6e} bits={bits} \
                             expected={expected_false_accept_count:.6e} at_least_once={:.6e} enough={}",
                            at_least_once(expected_false_accept_count),
                            u8::from(enough(at_least_once(expected_false_accept_count))),
                        ))
                    );
                }
            }
        }
    }

    // ── 阳性对照 ──
    let expected_at_eight_bits = expected_false_accepts(INCIDENT_CHANCES, 8);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_8bit_incident at_least_once={:.6} expect_near_one=1",
            at_least_once(expected_at_eight_bits)
        ))
    );
    let (base93_divergent_count, base93_first_divergent_items) = divergences(93, 2, 4, 512, LARGEST_ITEM_COUNT);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_base93_diverges div={base93_divergent_count} first={} expect_first=44",
            base93_first_divergent_items.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into())
        ))
    );

    // ── P3：4 对 8 的阳性对照（读法甲，原实验缺的那半）──
    let (base84_divergent_count, base84_first_divergent_items) = divergences(84, 4, 8, 512, LARGEST_ITEM_COUNT);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_base84_four_versus_eight div={base84_divergent_count} first={} expect_divergent=1",
            base84_first_divergent_items.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into())
        ))
    );

    // ── P4：读法乙的两个阳性对照（第七节 B7 算出的两个 base：每条项数恰好在这一对链宽之间差一）──
    for (base, pair, narrow, wide) in [(342u64, "2v4", 2u64, 4u64), (340u64, "4v8", 4u64, 8u64)] {
        let (divergent_count, first_divergent_items) = divergences_fixed_record(base, narrow, wide, LARGEST_ITEM_COUNT);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=poscontrol_fixed_record base={base} pair={pair} div={divergent_count} first={} expect_divergent=1",
                first_divergent_items.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into())
            ))
        );
    }

    // ── 第八节：几何敏感性，读法甲，base ∈ 296..=319（覆盖 base mod 8 的每个余数各三次）──
    for base in 296u64..=319 {
        for unit in UNITS {
            let (divergent_count_2_versus_4, first_divergent_items_2_versus_4) = divergences(base, 2, 4, unit, LARGEST_ITEM_COUNT);
            let (divergent_count_4_versus_8, first_divergent_items_4_versus_8) = divergences(base, 4, 8, unit, LARGEST_ITEM_COUNT);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=base_sweep reading=甲 base={base} unit={unit} div_2v4={divergent_count_2_versus_4} first_2v4={} \
                     div_4v8={divergent_count_4_versus_8} first_4v8={}",
                    first_divergent_items_2_versus_4.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                    first_divergent_items_4_versus_8.map(|first_divergent_items| first_divergent_items.to_string()).unwrap_or_else(|| "NA".into()),
                ))
            );
        }
    }

    // ── 第八节：几何敏感性，读法乙，base ∈ 280..=345（307 两侧最近的翻面点隔二三十字节）──
    for base in 280u64..=345 {
        let divergent_count_2_versus_4 = divergences_fixed_record(base, 2, 4, LARGEST_ITEM_COUNT).0;
        let divergent_count_4_versus_8 = divergences_fixed_record(base, 4, 8, LARGEST_ITEM_COUNT).0;
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=base_sweep reading=乙 base={base} items_per_record_2={} items_per_record_4={} items_per_record_8={} \
                 div_2v4={divergent_count_2_versus_4} div_4v8={divergent_count_4_versus_8}",
                items_per_fixed_record(base, 2),
                items_per_fixed_record(base, 4),
                items_per_fixed_record(base, 8),
            ))
        );
    }

    // ── 第八节：反方向取样点，项数上界从 2000 换成 67（今天一条定长记录最多装的项数，D23 已定项 4）──
    const ITEM_UPPER_BOUND_TODAY: u64 = 67;
    for base in [BASE_BEFORE, BASE_TODAY] {
        for unit in UNITS {
            let divergent_count_2_versus_4 = divergences(base, 2, 4, unit, ITEM_UPPER_BOUND_TODAY).0;
            let divergent_count_4_versus_8 = divergences(base, 4, 8, unit, ITEM_UPPER_BOUND_TODAY).0;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=base_sweep_items reading=甲 max_items={ITEM_UPPER_BOUND_TODAY} base={base} unit={unit} \
                     div_2v4={divergent_count_2_versus_4} div_4v8={divergent_count_4_versus_8}"
                ))
            );
        }
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **盘上占用的绝对值**：base=93、链 2 字节、点名 44 项 ⇒ 2559 B ⇒ 5 个 512 单元；
    /// 链 4 字节 ⇒ 2561 B ⇒ **6 个**。这就是 E39 抽样点漏掉的那个分岔。
    #[test]
    fn base_93_diverges_at_forty_four_items() {
        assert_eq!(93 + 2 + 44 * 56, 2559);
        assert_eq!(93 + 4 + 44 * 56, 2561);
        assert_eq!(units_on_disk(93, 2, 44, 512), 5);
        assert_eq!(units_on_disk(93, 4, 44, 512), 6);
        let (divergent_count, first_divergent_items) = divergences(93, 2, 4, 512, 2000);
        assert_eq!(first_divergent_items, Some(44));
        assert!(divergent_count >= 19, "0..2000 里至少 19 处（0..1200 内已数出 19）");
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
        for (base, expected_first_divergent_items) in [(84u64, 35u64), (99, 53)] {
            let (divergent_count, first_divergent_items) = divergences(base, 4, 8, 512, 2000);
            assert_eq!(divergent_count, 31, "base={base}");
            assert_eq!(first_divergent_items, Some(expected_first_divergent_items));
            assert_eq!(divergences(base, 4, 8, 4096, 2000).0, 4);
        }
        // 不分岔的：78 / 86 / 93 / 95
        for base in [78u64, 86, 93, 95] {
            assert_eq!(divergences(base, 4, 8, 512, 2000).0, 0, "base={base}");
            assert_eq!(divergences(base, 4, 8, 4096, 2000).0, 0, "base={base}");
        }
    }

    /// **只有 base=95（跑这个实验那天的现行值，不是今天的）两侧都免费**——2v4 与 4v8 同时为 0 的唯一一个。
    /// E49 重跑登记（记录头 311）：这一档 `BASES` 原样不变（Q49.1），95 早已不是今天的 base——今天是 `BASE_TODAY`（307）。
    #[test]
    fn only_the_base_from_that_run_is_free_on_both_sides() {
        let both_free: Vec<u64> = BASES
            .into_iter()
            .filter(|&candidate_base| {
                UNITS
                    .iter()
                    .all(|&candidate_unit| divergences(candidate_base, 2, 4, candidate_unit, 2000).0 == 0 && divergences(candidate_base, 4, 8, candidate_unit, 2000).0 == 0)
            })
            .collect();
        assert_eq!(both_free, vec![95]);
    }

    /// **一个 512 单元装几项**：base 95 + 8 字节链 ⇒ (512−103)/56 = 7 项，与链 4 字节时相同。
    #[test]
    fn capacity_per_unit_is_the_same_for_four_and_eight() {
        for base in BASES {
            let items_per_unit_with_four_byte_chain = (512u64).saturating_sub(base + 4) / ITEM_BYTES;
            let items_per_unit_with_eight_byte_chain = (512u64).saturating_sub(base + 8) / ITEM_BYTES;
            assert_eq!(items_per_unit_with_four_byte_chain, items_per_unit_with_eight_byte_chain, "base={base}");
        }
        assert_eq!((512u64 - 95 - 8) / 56, 7);
    }

    /// **风险算术的绝对值**：36 500 次机会下 16 位期望 0.5569 次、至少一次 42.70%；
    /// 32 位期望 8.499e-6、至少一次 8.499e-6。
    #[test]
    fn incident_risk_absolute_values() {
        let expected_at_sixteen_bits = expected_false_accepts(36_500.0, 16);
        assert!((expected_at_sixteen_bits - 0.556946).abs() < 1e-5, "{expected_at_sixteen_bits}");
        assert!((at_least_once(expected_at_sixteen_bits) - 0.427).abs() < 1e-3);
        let expected_at_thirty_two_bits = expected_false_accepts(36_500.0, 32);
        assert!((expected_at_thirty_two_bits - 8.4983e-6).abs() < 1e-9, "{expected_at_thirty_two_bits}");
        assert!(at_least_once(expected_at_thirty_two_bits) < 0.01, "32 位在事故分母下够用");
        assert!(at_least_once(expected_at_sixteen_bits) > 0.01, "16 位不够用");
    }

    /// **阳性对照**：8 位在事故分母下必须几乎必然出事。
    #[test]
    fn positive_control_eight_bits_is_a_certainty() {
        let expected_at_eight_bits = expected_false_accepts(36_500.0, 8);
        assert!(expected_at_eight_bits > 100.0, "{expected_at_eight_bits}");
        assert!(at_least_once(expected_at_eight_bits) > 0.999_999);
    }

    /// **遍历分母的绝对值**：10 MiB 环 / 每条 512 B ⇒ 每遍 20 480 对；每周 × 十年 = 522 遍
    /// ⇒ 1.069e7 次机会 ⇒ 32 位期望 2.489e-3、至少一次 0.2486%（**够用但不是零**）；
    /// 64 位期望 5.79e-13。
    #[test]
    fn traversal_risk_absolute_values() {
        let pairs_per_pass = (10 * 1024 * 1024u64 / 512) as f64;
        assert_eq!(pairs_per_pass, 20480.0);
        let chances = pairs_per_pass * 522.0;
        assert!((chances - 1.069056e7).abs() < 1.0, "{chances}");
        let expected_at_thirty_two_bits = expected_false_accepts(chances, 32);
        assert!((expected_at_thirty_two_bits - 2.4888e-3).abs() < 1e-6, "{expected_at_thirty_two_bits}");
        assert!(at_least_once(expected_at_thirty_two_bits) < 0.01, "32 位在这一档仍够用");
        let expected_at_sixty_four_bits = expected_false_accepts(chances, 64);
        assert!(expected_at_sixty_four_bits < 1e-12);
    }

    /// **最恶劣的那一档遍历分母**：2 GiB 环 / 每条 512 B / 每天 × 十年
    /// ⇒ 每遍 4 194 304 对 × 3650 遍 = 1.531e10 次机会
    /// ⇒ **32 位期望 3.56 次、至少一次 97.2%**（远超 1% 判据）；64 位 8.3e-10。
    /// **这一档就是 64 位买到东西的地方。**
    #[test]
    fn the_worst_traversal_bucket_breaks_thirty_two_bits() {
        let pairs_per_pass = (2 * 1024 * 1024 * 1024u64 / 512) as f64;
        assert_eq!(pairs_per_pass, 4_194_304.0);
        let chances = pairs_per_pass * 3650.0;
        assert!((chances - 1.53092e10).abs() < 1e6, "{chances}");
        let expected_at_thirty_two_bits = expected_false_accepts(chances, 32);
        assert!((expected_at_thirty_two_bits - 3.5645).abs() < 1e-3, "{expected_at_thirty_two_bits}");
        assert!(at_least_once(expected_at_thirty_two_bits) > 0.97, "32 位在这一档几乎必然出事");
        let expected_at_sixty_four_bits = expected_false_accepts(chances, 64);
        assert!(at_least_once(expected_at_sixty_four_bits) < 1e-8, "64 位够用");
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

    // ═══ E49 重跑登记（记录头 311）：第七节 A1–A4、B1–B10 ═══

    /// **A1 / A2**：今天的 base = 头 311 − 链 4 = 307；改之前那一档 = 307 − 4 = 303。
    #[test]
    fn base_today_and_base_before_are_derived_correctly() {
        assert_eq!(JOURNAL_HEADER_BYTES, 311);
        assert_eq!(BACK_CHAIN_BYTES_TODAY, 4);
        assert_eq!(BASE_TODAY, 307);
        assert_eq!(BASE_BEFORE, 303);
    }

    /// **A3**：读法乙在 base=307、链 4 上，一条记录装 67 项（4096 − 311 = 3785，⌊3785 / 56⌋ = 67）。
    #[test]
    fn fixed_record_capacity_at_base_today_is_sixty_seven() {
        assert_eq!(RECORD_BYTES_FIXED - JOURNAL_HEADER_BYTES, 3785);
        assert_eq!(items_per_fixed_record(BASE_TODAY, BACK_CHAIN_BYTES_TODAY), 67);
    }

    /// **A4 / Q49.5**：D23 已定项 8 射程「只取决于记录头总字节 mod 8」——
    /// base ∈ 296..=319（每个余数各三次）里，同一个 `base % 8` 分组的四格判定必须一致。
    #[test]
    fn base_mod_eight_predicts_which_pair_diverges() {
        let mut groups: std::collections::HashMap<u64, Vec<(u64, u64, u64, u64)>> = std::collections::HashMap::new();
        for base in 296u64..=319 {
            let signature = (
                divergences(base, 2, 4, 512, LARGEST_ITEM_COUNT).0,
                divergences(base, 4, 8, 512, LARGEST_ITEM_COUNT).0,
                divergences(base, 2, 4, 4096, LARGEST_ITEM_COUNT).0,
                divergences(base, 4, 8, 4096, LARGEST_ITEM_COUNT).0,
            );
            groups.entry(base % 8).or_default().push(signature);
        }
        for (remainder, signatures) in &groups {
            let first = signatures[0];
            assert!(signatures.iter().all(|signature| *signature == first), "base mod 8 = {remainder} 组内不一致：{signatures:?}");
        }
    }

    /// **B1**：读法甲，base=303 四格全 0（改之前那一档，两对都免费）。
    #[test]
    fn base_before_is_free_on_all_four_cells() {
        for unit in UNITS {
            assert_eq!(divergences(BASE_BEFORE, 2, 4, unit, LARGEST_ITEM_COUNT).0, 0, "unit={unit}");
            assert_eq!(divergences(BASE_BEFORE, 4, 8, unit, LARGEST_ITEM_COUNT).0, 0, "unit={unit}");
        }
    }

    /// **B2**：读法甲，base=307（今天）四格——2v4 两档都免费；4v8 在 512 上 31 处（首个 31）、
    /// 4096 上 4 处（首个 287）；`cap_items` 512 上 3、4096 上 67。
    #[test]
    fn base_today_diverges_on_four_versus_eight_only() {
        assert_eq!(divergences(BASE_TODAY, 2, 4, 512, LARGEST_ITEM_COUNT), (0, None));
        assert_eq!(divergences(BASE_TODAY, 2, 4, 4096, LARGEST_ITEM_COUNT), (0, None));
        assert_eq!(divergences(BASE_TODAY, 4, 8, 512, LARGEST_ITEM_COUNT), (31, Some(31)));
        assert_eq!(divergences(BASE_TODAY, 4, 8, 4096, LARGEST_ITEM_COUNT), (4, Some(287)));
        assert_eq!(512u64.saturating_sub(BASE_TODAY + 8) / ITEM_BYTES, 3);
        assert_eq!(4096u64.saturating_sub(BASE_TODAY + 8) / ITEM_BYTES, 67);
    }

    /// **B3**：M8 的取样点——如果 base 被算成含链（307 变成 311），四格会全部塌成 0
    /// （这正是 M8 该被抓住的地方：B2 的 4v8 分岔在 base=311 上消失）。
    #[test]
    fn base_311_is_free_on_all_four_cells() {
        for unit in UNITS {
            assert_eq!(divergences(311, 2, 4, unit, LARGEST_ITEM_COUNT).0, 0, "unit={unit}");
            assert_eq!(divergences(311, 4, 8, unit, LARGEST_ITEM_COUNT).0, 0, "unit={unit}");
        }
    }

    /// **B4**：项数上界从 2000 换成 67（今天一条定长记录最多装的项数）——
    /// base=303 四格仍全 0；base=307 只有「512 单元、4 对 8」变成 1 处，其余三格仍 0。
    #[test]
    fn item_upper_bound_sixty_seven_changes_only_one_cell() {
        for unit in UNITS {
            assert_eq!(divergences(BASE_BEFORE, 2, 4, unit, 67).0, 0, "unit={unit}");
            assert_eq!(divergences(BASE_BEFORE, 4, 8, unit, 67).0, 0, "unit={unit}");
        }
        assert_eq!(divergences(BASE_TODAY, 2, 4, 512, 67).0, 0);
        assert_eq!(divergences(BASE_TODAY, 4, 8, 512, 67).0, 1);
        assert_eq!(divergences(BASE_TODAY, 2, 4, 4096, 67).0, 0);
        assert_eq!(divergences(BASE_TODAY, 4, 8, 4096, 67).0, 0);
    }

    /// **B5 / P3**：4 对 8 的阳性对照——base=84、单元 512：31 处，首个 35（原实验缺的那半，2 对 4 才有阳性对照）。
    #[test]
    fn positive_control_base_84_four_versus_eight_diverges() {
        assert_eq!(divergences(84, 4, 8, 512, LARGEST_ITEM_COUNT), (31, Some(35)));
    }

    /// **B6**：读法乙，base=303、307 两条都不分岔——每条项数（链 2/4/8）都是 67/67/67。
    #[test]
    fn fixed_record_reading_agrees_at_both_judged_bases() {
        for base in [BASE_BEFORE, BASE_TODAY] {
            assert_eq!(items_per_fixed_record(base, 2), 67, "base={base}");
            assert_eq!(items_per_fixed_record(base, 4), 67, "base={base}");
            assert_eq!(items_per_fixed_record(base, 8), 67, "base={base}");
            assert_eq!(divergences_fixed_record(base, 2, 4, LARGEST_ITEM_COUNT), (0, None), "base={base}");
            assert_eq!(divergences_fixed_record(base, 4, 8, LARGEST_ITEM_COUNT), (0, None), "base={base}");
        }
    }

    /// **B7 / P4**：读法乙的两个阳性对照——base=340（4 对 8 恰好差一项，455 处、首个 67）、
    /// base=342（2 对 4 恰好差一项，455 处、首个 67）。
    #[test]
    fn positive_control_fixed_record_diverges_at_these_two_bases() {
        assert_eq!(items_per_fixed_record(340, 2), 67);
        assert_eq!(items_per_fixed_record(340, 4), 67);
        assert_eq!(items_per_fixed_record(340, 8), 66);
        assert_eq!(divergences_fixed_record(340, 2, 4, LARGEST_ITEM_COUNT), (0, None));
        assert_eq!(divergences_fixed_record(340, 4, 8, LARGEST_ITEM_COUNT), (455, Some(67)));

        assert_eq!(items_per_fixed_record(342, 2), 67);
        assert_eq!(items_per_fixed_record(342, 4), 66);
        assert_eq!(items_per_fixed_record(342, 8), 66);
        assert_eq!(divergences_fixed_record(342, 2, 4, LARGEST_ITEM_COUNT), (455, Some(67)));
        assert_eq!(divergences_fixed_record(342, 4, 8, LARGEST_ITEM_COUNT), (0, None));
    }

    /// **B8**：base mod 8 分组的绝对值——余 0 与余 7 四格全 0；余 1–4 只有 4 对 8 分岔
    /// （512 上 31、4096 上 4）；余 5–6 只有 2 对 4 分岔（同样 31 / 4）。
    #[test]
    fn base_mod_eight_groups_have_these_exact_values() {
        for base in 296u64..=319 {
            let cells = (
                divergences(base, 2, 4, 512, LARGEST_ITEM_COUNT).0,
                divergences(base, 4, 8, 512, LARGEST_ITEM_COUNT).0,
                divergences(base, 2, 4, 4096, LARGEST_ITEM_COUNT).0,
                divergences(base, 4, 8, 4096, LARGEST_ITEM_COUNT).0,
            );
            match base % 8 {
                0 | 7 => assert_eq!(cells, (0, 0, 0, 0), "base={base}"),
                1..=4 => assert_eq!(cells, (0, 31, 0, 4), "base={base}"),
                5 | 6 => assert_eq!(cells, (31, 0, 4, 0), "base={base}"),
                _ => unreachable!(),
            }
        }
    }

    /// **B9**：读法乙，base ∈ 280..=345 里，2 对 4 只在 {285,286,341,342} 分岔，
    /// 4 对 8 只在 {281,282,283,284,337,338,339,340} 分岔，其余全 0。
    #[test]
    fn fixed_record_reading_diverges_only_at_these_bases_in_280_to_345() {
        let diverging_2_versus_4: Vec<u64> = (280u64..=345).filter(|&base| divergences_fixed_record(base, 2, 4, LARGEST_ITEM_COUNT).0 > 0).collect();
        let diverging_4_versus_8: Vec<u64> = (280u64..=345).filter(|&base| divergences_fixed_record(base, 4, 8, LARGEST_ITEM_COUNT).0 > 0).collect();
        assert_eq!(diverging_2_versus_4, vec![285, 286, 341, 342]);
        assert_eq!(diverging_4_versus_8, vec![281, 282, 283, 284, 337, 338, 339, 340]);
    }

    /// **B10 / 判别力自证（V5）**：Q49.2 那一格（base=307、单元 512、4 对 8）在原判据（`tolerance=0`）下
    /// 判「不免费」；把门槛挪到 `d`（这一格自己的分岔数，31）之后必须翻成「免费」——挪了不变就作废（V5）。
    /// M12 攻的正是 `is_free` 里让门槛起作用的那一步。
    #[test]
    fn discriminative_power_self_proof_flips_the_verdict() {
        let (divergent_count, _) = divergences(BASE_TODAY, 4, 8, 512, LARGEST_ITEM_COUNT);
        assert_eq!(divergent_count, 31, "B10：d 必须是 31");
        assert!(!is_free(divergent_count, 0), "原判据（tolerance=0）：不免费");
        assert!(is_free(divergent_count, divergent_count), "挪门槛到 d 之后必须翻成免费");
    }
}
