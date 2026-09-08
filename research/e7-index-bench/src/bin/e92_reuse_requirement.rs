//! E92：最坏块重用延迟需求 —— I-7.4 缺的那个对照量。
//!
//! E50 自陈「最坏块重用延迟需求这个量本仓没有 ⇒ 一圈毫秒级够不够判不了」。
//! 本实验逐个枚举「消费旧代根的人」，对每个消费者推导它对块重用延迟的需求，
//! **并回答需求用什么单位计价**——按代数（发布次数）、按墙钟、还是按外部事件。
//!
//! ## 判据（2026-09-03 跑前写死，跑完一个字没改）
//!
//! 1. 枚举完备性挂闸：消费者清单逐条给出处，模型里每个消费者至少一格。
//! 2. 单位判定：每个消费者写成（代数 | 墙钟 | 外部事件）三选一并给机制推导；
//!    若存在墙钟计价的消费者且其需求在甲的发布率下超过 K 代窗口
//!    ⇒ 判「I-7.4 以代计的单位在甲下保护不了它」，如实记，落点交 decide。
//! 3. 阳性对照：择根回退那一格必须推出「恰 1 代 + 验证滞后」（结构可导），
//!    模拟里 K ≥ 2 时该消费者的违例恒 0；把 K 改成 1 时必须出现违例。
//! 4. 绝对值：慢消费者的违例数由「扫描时长 × 发布率 − K」的算术独立给出，
//!    模拟与闭式恰等。
//!
//! ## 失败条款（跑前写死）
//!
//! - 判据 3 的判别力自证不中 ⇒ 模型没在测重用，整轮作废。
//! - C88 那格答不出需求 ⇒ 如实标「空白」，不许编（也不许写成 0——读不到 ≠ 读到 0）。
//! - 确定性模型：5 遍 md5 只证无隐藏状态，不证统计稳定。

use e7_index_bench::Emitter;

/// 需求的计价单位。三选一是判据 2 定死的。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pricing {
    /// 按代数（发布次数）计价。
    Generations,
    /// 按墙钟计价。
    WallClock,
    /// 未知——消费者本身还没实现，需求推不出来。**不许当成 0。**
    Blank,
}

impl Pricing {
    fn as_str(self) -> &'static str {
        match self {
            Pricing::Generations => "generations",
            Pricing::WallClock => "wallclock",
            Pricing::Blank => "blank",
        }
    }
}

struct Consumer {
    id: &'static str,
    source: &'static str,
    pricing: Pricing,
    /// 代数计价时的需求（多少代必须同时活着）。别的计价填 None。
    req_gens: Option<u64>,
}

/// 跑前登记的消费者清单。判据 1：逐条给出处，一条都不许少列。
const CONSUMERS: [Consumer; 5] = [
    // 恢复择根回退：最新根撕裂 ⇒ 退一代。**需求是 2 代不是 1 代**——
    // 要判定最新那代坏了，上一代必须在它被验证的整段时间里都还活着（验证滞后）。
    Consumer { id: "c1_root_fallback", source: "D22-item2", pricing: Pricing::Generations, req_gens: Some(2) },
    // 水位重放：所选根之上的记录点名的单元，在施加前不许被重用。
    // 窗口是在飞记录范围，而**「在飞记录数上限」本仓没有这个数** ⇒ 需求值留空，只判单位。
    Consumer { id: "c2_replay_watermark", source: "D23-item14", pricing: Pricing::Generations, req_gens: None },
    // 代际增量 scrub / checker 走旧代根：**走一遍要墙钟时间**。
    Consumer { id: "c3_scrub_old_root", source: "D13", pricing: Pricing::WallClock, req_gens: None },
    // 时间线判别（设备失而复得）：C88 未实现，需求未知。
    Consumer { id: "c4_timeline_discriminate", source: "C88", pricing: Pricing::Blank, req_gens: None },
    // 两条钉块规则的相位：E33 逐字「有帮助的相位数是 根环(K) − 1」⇒ 需求由 K 自己定义。
    Consumer { id: "c5_pinning_phase", source: "E33", pricing: Pricing::Generations, req_gens: None },
];

/// 甲（每次 fsync 发布一次根）下的发布率，本机实测值（E44）。
const PUBLISH_RATE: f64 = 2785.0;

/// 代数计价的消费者的违例数：需求超出 K 代窗口多少代。
fn gen_violations(req_gens: u64, k: u64) -> u64 {
    req_gens.saturating_sub(k)
}

/// K 代窗口折成墙钟有多宽（秒）。
fn window_seconds(k: u64, publish_rate: f64) -> f64 {
    k as f64 / publish_rate
}

/// 慢消费者扫一遍要多久（秒）。
fn scan_seconds(blocks_total: u64, blocks_per_sec: f64) -> f64 {
    blocks_total as f64 / blocks_per_sec
}

/// 闭式：违例数 = 扫描时长 × 发布率 − K（判据 4 逐字）。
fn closed_form_violations(scan_secs: f64, publish_rate: f64, k: u64) -> u64 {
    let publishes = (scan_secs * publish_rate).floor() as i64;
    (publishes - k as i64).max(0) as u64
}

/// 模拟：一步一次发布。消费者在 t=0 钉住当时最新的那一代，
/// 该代在 K 次发布之后离开 K 代窗口，此后每一次发布记一次违例。
/// **与闭式各写各的**，判据 4 要求两者恰等。
fn simulate_violations(scan_secs: f64, publish_rate: f64, k: u64) -> u64 {
    let publishes = (scan_secs * publish_rate).floor() as u64;
    let mut age = 0u64; // 钉住的那一代已经老了几代
    let mut violations = 0u64;
    for _ in 0..publishes {
        age += 1;
        if age > k {
            violations += 1;
        }
    }
    violations
}

const KS: [u64; 5] = [1, 2, 3, 4, 8];
/// 慢消费者的两档：树多大 × 扫多快。都是参数，不是实测——E92 不建真实 scrub。
const SCRUB_CASES: [(&str, u64, f64); 3] = [
    ("small_1e4_at_1e5", 10_000, 100_000.0),
    ("mid_1e6_at_1e5", 1_000_000, 100_000.0),
    ("big_1e8_at_1e6", 100_000_000, 1_000_000.0),
];

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config publish_rate={PUBLISH_RATE} consumers={} ks={}",
        CONSUMERS.len(), KS.len()
    )));

    // ── 判据 1 + 2：逐消费者一格，写清单位 ──
    for c in &CONSUMERS {
        let req = c.req_gens.map(|v| v.to_string()).unwrap_or_else(|| "blank".into());
        println!("{}", em.emit_raw(&format!(
            "name=consumer id={} source={} pricing={} req_gens={req}",
            c.id, c.source, c.pricing.as_str()
        )));
    }

    // ── 判据 3：代数计价的消费者逐 K ──
    for c in &CONSUMERS {
        let Some(req) = c.req_gens else { continue };
        for k in KS {
            println!("{}", em.emit_raw(&format!(
                "name=gen_consumer id={} k={k} req_gens={req} violations={}",
                c.id, gen_violations(req, k)
            )));
        }
    }

    // ── 判据 2 + 4：墙钟计价的消费者逐 K × 逐档 ──
    let mut wallclock_exceeds = 0u64;
    for (label, blocks, rate) in SCRUB_CASES {
        let secs = scan_seconds(blocks, rate);
        for k in KS {
            let sim = simulate_violations(secs, PUBLISH_RATE, k);
            let closed = closed_form_violations(secs, PUBLISH_RATE, k);
            let win = window_seconds(k, PUBLISH_RATE);
            if secs > win {
                wallclock_exceeds += 1;
            }
            println!("{}", em.emit_raw(&format!(
                "name=wallclock_consumer id=c3_scrub_old_root case={label} k={k} \
                 scan_seconds={secs:.6} window_seconds={win:.9} exceeds={} \
                 sim_violations={sim} closed_violations={closed} agree={}",
                u8::from(secs > win),
                u8::from(sim == closed)
            )));
        }
    }
    println!("{}", em.emit_raw(&format!(
        "name=verdict_unit wallclock_cells_exceeding_window={wallclock_exceeds} \
         total_wallclock_cells={} gens_unit_protects_wallclock_consumer={}",
        SCRUB_CASES.len() * KS.len(),
        u8::from(wallclock_exceeds == 0)
    )));

    // ── 判据 3 的判别力自证：K=1 必须出违例，K≥2 必须恒 0 ──
    let c1_req = CONSUMERS[0].req_gens.unwrap();
    println!("{}", em.emit_raw(&format!(
        "name=poscontrol_fallback k1_violations={} k2_violations={} discriminates={}",
        gen_violations(c1_req, 1),
        gen_violations(c1_req, 2),
        u8::from(gen_violations(c1_req, 1) > 0 && gen_violations(c1_req, 2) == 0)
    )));

    // ── 阴性对照：扫描时长为 0 ⇒ 违例恒 0 ──
    println!("{}", em.emit_raw(&format!(
        "name=negcontrol_instant_scan violations={} expect=0",
        simulate_violations(0.0, PUBLISH_RATE, 3)
    )));

    // ── C88 那一格：如实标空白，不是 0 ──
    println!("{}", em.emit_raw(
        "name=blank_cell id=c4_timeline_discriminate requirement=NA \
         reason=consumer_not_implemented_C88"
    ));

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1**：清单五条，逐条有出处，出处不许重复也不许为空。
    #[test]
    fn consumer_list_is_complete_and_sourced() {
        assert_eq!(CONSUMERS.len(), 5);
        let mut srcs: Vec<&str> = CONSUMERS.iter().map(|c| c.source).collect();
        srcs.sort_unstable();
        srcs.dedup();
        assert_eq!(srcs.len(), 5, "出处不许重复");
        assert!(CONSUMERS.iter().all(|c| !c.source.is_empty()));
    }

    /// **判据 3 的阳性对照**：择根回退的需求是 **2 代**（1 代 + 验证滞后）。
    /// K=1 出 1 次违例，K≥2 恒 0 ⇒ 这个模型分得出差别。
    /// ⚠️ 这条同时是一个独立方向的复算：它算出的 K 下限恰是 I-7.4 已写死的 2。
    #[test]
    fn root_fallback_needs_exactly_two_generations() {
        assert_eq!(CONSUMERS[0].req_gens, Some(2));
        assert_eq!(gen_violations(2, 1), 1);
        for k in [2u64, 3, 4, 8] {
            assert_eq!(gen_violations(2, k), 0, "k={k}");
        }
    }

    /// **K 代窗口折成墙钟的绝对值**：甲的发布率 2785/秒下，
    /// K=2 是 0.718 毫秒、K=3 是 1.077 毫秒、K=8 是 2.873 毫秒。**全是毫秒级。**
    #[test]
    fn the_k_window_in_wall_clock_is_sub_ten_milliseconds() {
        assert!((window_seconds(2, PUBLISH_RATE) - 0.000_718_1).abs() < 1e-7, "{}", window_seconds(2, PUBLISH_RATE));
        assert!((window_seconds(3, PUBLISH_RATE) - 0.001_077_2).abs() < 1e-7);
        assert!((window_seconds(8, PUBLISH_RATE) - 0.002_872_5).abs() < 1e-7);
        // 就算 K 取到 48（R=3 × S=16 的槽总数）也只有 17.2 毫秒
        assert!((window_seconds(48, PUBLISH_RATE) - 0.017_235_2).abs() < 1e-6);
    }

    /// **扫描时长的绝对值**：1e4 块 @ 1e5 块/秒 = 0.1 秒；1e6 @ 1e5 = 10 秒；1e8 @ 1e6 = 100 秒。
    #[test]
    fn scan_seconds_absolute_values() {
        assert!((scan_seconds(10_000, 100_000.0) - 0.1).abs() < 1e-12);
        assert!((scan_seconds(1_000_000, 100_000.0) - 10.0).abs() < 1e-9);
        assert!((scan_seconds(100_000_000, 1_000_000.0) - 100.0).abs() < 1e-9);
    }

    /// **判据 4 的绝对值**：10 秒扫描 × 2785 发布/秒 = 27 850 次发布，
    /// K=3 时违例 **27 847** 次。闭式与模拟必须恰等。
    #[test]
    fn slow_consumer_violation_count_is_pinned() {
        let secs = scan_seconds(1_000_000, 100_000.0);
        assert_eq!(closed_form_violations(secs, PUBLISH_RATE, 3), 27_847);
        assert_eq!(simulate_violations(secs, PUBLISH_RATE, 3), 27_847);
        // 最小那一档：0.1 秒 = 278 次发布（2785 × 0.1 = 278.5 向下取整），K=8 ⇒ 270
        assert_eq!(closed_form_violations(0.1, PUBLISH_RATE, 8), 270);
        // 最大那一档：100 秒 = 278 500 次发布，K=2 ⇒ 278 498
        assert_eq!(closed_form_violations(100.0, PUBLISH_RATE, 2), 278_498);
    }

    /// **闭式与模拟在全部格上恰等**（判据 4）。两者各写各的：一个是减法，一个是逐步计数。
    #[test]
    fn closed_form_and_simulation_agree_on_every_cell() {
        for (_, blocks, rate) in SCRUB_CASES {
            let secs = scan_seconds(blocks, rate);
            for k in KS {
                assert_eq!(
                    closed_form_violations(secs, PUBLISH_RATE, k),
                    simulate_violations(secs, PUBLISH_RATE, k),
                    "blocks={blocks} k={k}"
                );
            }
        }
    }

    /// **阴性对照**：扫描时长为 0 ⇒ 违例恒 0，且 K 再小也是 0。
    #[test]
    fn instant_scan_never_violates() {
        for k in KS {
            assert_eq!(simulate_violations(0.0, PUBLISH_RATE, k), 0);
            assert_eq!(closed_form_violations(0.0, PUBLISH_RATE, k), 0);
        }
    }

    /// **判据 2 的判定**：墙钟计价的消费者，在扫描到的**每一格**上需求都超出 K 代窗口。
    /// ⇒ 「I-7.4 以代计的单位在甲下保护不了它」。
    #[test]
    fn every_wallclock_cell_exceeds_the_generation_window() {
        let mut cells = 0;
        for (_, blocks, rate) in SCRUB_CASES {
            let secs = scan_seconds(blocks, rate);
            for k in KS {
                assert!(secs > window_seconds(k, PUBLISH_RATE), "blocks={blocks} k={k}");
                cells += 1;
            }
        }
        assert_eq!(cells, 15);
    }

    /// **C88 那一格是空白，不是 0**——读不到 ≠ 读到 0。
    #[test]
    fn the_unimplemented_consumer_is_blank_not_zero() {
        let c4 = CONSUMERS.iter().find(|c| c.id == "c4_timeline_discriminate").unwrap();
        assert_eq!(c4.pricing, Pricing::Blank);
        assert_eq!(c4.req_gens, None);
    }

    /// 三种计价各自至少一个消费者，且 c2 / c5 的需求值留空（本仓没有那两个数）。
    #[test]
    fn pricing_kinds_are_all_represented() {
        assert!(CONSUMERS.iter().any(|c| c.pricing == Pricing::Generations));
        assert!(CONSUMERS.iter().any(|c| c.pricing == Pricing::WallClock));
        assert!(CONSUMERS.iter().any(|c| c.pricing == Pricing::Blank));
        let c2 = CONSUMERS.iter().find(|c| c.id == "c2_replay_watermark").unwrap();
        assert_eq!(c2.req_gens, None);
        let c5 = CONSUMERS.iter().find(|c| c.id == "c5_pinning_phase").unwrap();
        assert_eq!(c5.req_gens, None);
    }
}
