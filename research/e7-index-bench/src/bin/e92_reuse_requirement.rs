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
//! ## 补列重跑（2026-09-08，判据 1 的枚举挂闸触发）
//!
//! 判据 1 逐字「跑后发现清单外的消费者 ⇒ 补列重跑，不许悄悄漏」。第一轮之后查出三处：
//! I-4.8（近 K 代根校验和自洽）、记录核对器、管理员回退（D23 已定项 14，2026-09-05 用户定案，
//! 早于第一轮）。**同一轮还查出 c3 建错了形状**：第一轮把代际增量 scrub 建成「钉住一代根走到底」，
//! 而 D13 对它的逐字定义是「维护一个『上次 scrub 扫到的 generation 水位』，只遍历
//! `birth generation > 水位` 的块」——水位型，不持有旧根。全仓 grep「scrub…旧根」只命中
//! 第一轮自己写进 D22 与 C213 的那两处 ⇒ 那是稻草人形状，按 evidence-discipline 作废。
//! **判据 1-4 与失败条款一个字没改**，改的是消费者清单与 c3 的机制推导。

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
    /// 按外部事件计价——发生的时刻由本工程之外的东西决定（例如管理员什么时候决定回退）。
    /// 判据 2 的三选一本来就有这一类，第一轮没有任何消费者落进来。
    ExternalEvent,
    /// 未知——消费者本身还没实现，需求推不出来。**不许当成 0。**
    Blank,
}

impl Pricing {
    fn as_str(self) -> &'static str {
        match self {
            Pricing::Generations => "generations",
            Pricing::WallClock => "wallclock",
            Pricing::ExternalEvent => "external_event",
            Pricing::Blank => "blank",
        }
    }
}

struct Consumer {
    identifier: &'static str,
    source: &'static str,
    pricing: Pricing,
    /// 代数计价时的需求（多少代必须同时活着）。别的计价填 None。
    required_generations: Option<u64>,
}

/// 消费者清单（2026-09-08 补列重跑后的第二版）。判据 1：逐条给出处，一条都不许少列。
const CONSUMERS: [Consumer; 9] = [
    // c1 恢复择根回退：最新根撕裂 ⇒ 退一代。需求是 2 代不是 1 代——要判定最新那代坏了，
    // 上一代必须在它被验证的整段时间里都还活着（验证滞后）。
    Consumer { identifier: "c1_root_fallback", source: "D22-item2", pricing: Pricing::Generations, required_generations: Some(2) },
    // c2 水位重放：所选根之上的记录点名的单元，在施加前不许被重用。
    // 窗口是在飞记录范围，而「在飞记录数上限」本仓没有这个数 ⇒ 需求值留空，只判单位。
    Consumer { identifier: "c2_replay_watermark", source: "D23-item14", pricing: Pricing::Generations, required_generations: None },
    // c3 代际增量 scrub：D13 逐字「维护一个『上次 scrub 扫到的 generation 水位』，
    // 只遍历 `birth generation > 水位` 的块」⇒ **水位型，不持有旧根**，对旧根的需求恒 0。
    // ⚠️ 第一轮把它建成「钉住一代根走到底」，那个形状全仓没有一条条款要求。
    Consumer { identifier: "c3_scrub_watermark", source: "D13-scrub", pricing: Pricing::Generations, required_generations: Some(0) },
    // c4 时间线判别（设备失而复得）：C88 未实现，需求未知。**空白不是 0。**
    Consumer { identifier: "c4_timeline_discriminate", source: "C88", pricing: Pricing::Blank, required_generations: None },
    // c5 两条钉块规则的相位：E33 逐字「有帮助的相位数是 根环(K) − 1」⇒ 需求由 K 自己定义。
    Consumer { identifier: "c5_pinning_phase", source: "E33", pricing: Pricing::Generations, required_generations: None },
    // c6 I-4.8 近 K 代根校验和自洽：跑在崩溃点重放之后的**冻结镜像**上，没有并发分配器
    // 在跟它抢 ⇒ 对块重用延迟的需求退化为 0。第一轮漏列。
    Consumer { identifier: "c6_i48_frozen_image", source: "I-4.8", pricing: Pricing::Generations, required_generations: Some(0) },
    // c7 记录核对器：入参 D13 逐字 `(崩溃前镜像, 记录流, 崩溃后镜像)`，三个都是冻结的 ⇒ 同上。
    Consumer { identifier: "c7_record_verifier", source: "D13-verifier", pricing: Pricing::Generations, required_generations: Some(0) },
    // c8 管理员回退：D23 已定项 14（2026-09-05 用户定案）逐字「回退深度 ≤ 根环深度」，
    // 候选集逐字「根环里按实例表判仍然有效的根」——**判据里一个 K 字都没有**。
    // 触发**时刻**由管理员定 ⇒ 外部事件；但它的**需求**是「环里任何一个幸存槽都要能退得回去」
    // ⇒ 需求 = 根环深度 N，不是 2。
    // ⚠️ 第二轮给它判的「由根环深度定义、不反过来要求 K」是**循环论证**——
    // 那句话把待证的 `K = 深度` 当成了前提。需求由 `demand_is_ring_depth` 标出，
    // 逐 (N, K) 算假候选数，不再写成 None。
    Consumer { identifier: "c8_admin_rollback", source: "D23-item14", pricing: Pricing::ExternalEvent, required_generations: None },
    // c9 遍历跳：任何活树遍历读完父指针、再去读子块，这中间子块不许被释放并重用。
    // 这是清单里**唯一真正按墙钟计价**的一格，需求 = 一跳的耗时。第一轮没建。
    Consumer { identifier: "c9_traversal_hop", source: "D4-parent-checksum", pricing: Pricing::WallClock, required_generations: None },
];

/// 甲（每次 fsync 发布一次根）下的发布率，本机实测值（E44）。
const PUBLISH_RATE: f64 = 2785.0;

/// 代数计价的消费者的违例数：需求超出 K 代窗口多少代。
fn generation_violations(required_generations: u64, window_generations: u64) -> u64 {
    required_generations.saturating_sub(window_generations)
}

/// K 代窗口折成墙钟有多宽（秒）。
fn window_seconds(window_generations: u64, publish_rate: f64) -> f64 {
    window_generations as f64 / publish_rate
}

/// 闭式：违例数 = 扫描时长 × 发布率 − K（判据 4 逐字）。
fn closed_form_violations(scan_seconds: f64, publish_rate: f64, window_generations: u64) -> u64 {
    let publishes = (scan_seconds * publish_rate).floor() as i64;
    (publishes - window_generations as i64).max(0) as u64
}

/// 模拟：一步一次发布。消费者在 t=0 钉住当时最新的那一代，
/// 该代在 K 次发布之后离开 K 代窗口，此后每一次发布记一次违例。
/// **与闭式各写各的**，判据 4 要求两者恰等。
fn simulate_violations(scan_seconds: f64, publish_rate: f64, window_generations: u64) -> u64 {
    let publishes = (scan_seconds * publish_rate).floor() as u64;
    let mut pinned_generation_age = 0u64; // 钉住的那一代已经老了几代
    let mut violations = 0u64;
    for _ in 0..publishes {
        pinned_generation_age += 1;
        if pinned_generation_age > window_generations {
            violations += 1;
        }
    }
    violations
}

/// 管理员回退的假候选数：环里有 N 个槽，而只有最近 K 代的块没被复用
/// ⇒ 其余 N − K 个虽然自证合法、却指向已被覆盖的块（D22 已定项 2 逐字「它们作为回退候选是假的」）。
fn fake_rollback_candidates(ring_depth: u64, window_generations: u64) -> u64 {
    ring_depth.saturating_sub(window_generations)
}

const WINDOW_GENERATION_VALUES: [u64; 5] = [1, 2, 3, 4, 8];
/// 根环深度 N = R × S，R = 3 已定（D22 已定项 2），S ∈ {1,2,4,8,16}（D22 那一格自陈不碰任何一条边）。
const RING_DEPTHS: [(u64, u64); 5] = [(1, 3), (2, 6), (4, 12), (8, 24), (16, 48)];
/// 假想的墙钟消费者持续多久（秒）。**这是定价表的自变量，不是任何条款要求的形状**——
/// 第一轮把 scrub 当成这种形状是错的（D13 的 scrub 是水位型）。留着它是为了回答
/// 「若将来出现一个持续 D 秒的墙钟消费者，要多少代才罩得住」。
const DURATIONS: [(&str, f64); 4] = [
    ("hop_100us", 0.000_1),
    ("hop_1ms", 0.001),
    ("scan_100ms", 0.1),
    ("scan_10s", 10.0),
];

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config publish_rate={PUBLISH_RATE} consumers={} ks={} durations={}",
        CONSUMERS.len(), WINDOW_GENERATION_VALUES.len(), DURATIONS.len()
    )));

    // ── 判据 1 + 2：逐消费者一格，写清单位 ──
    let mut consumer_count_by_pricing = (0u32, 0u32, 0u32, 0u32); // 代数 / 墙钟 / 外部事件 / 空白
    for consumer in &CONSUMERS {
        match consumer.pricing {
            Pricing::Generations => consumer_count_by_pricing.0 += 1,
            Pricing::WallClock => consumer_count_by_pricing.1 += 1,
            Pricing::ExternalEvent => consumer_count_by_pricing.2 += 1,
            Pricing::Blank => consumer_count_by_pricing.3 += 1,
        }
        let required_generations_text = consumer.required_generations.map(|value| value.to_string()).unwrap_or_else(|| "blank".into());
        println!("{}", emitter.emit_raw(&format!(
            "name=consumer id={} source={} pricing={} req_gens={required_generations_text}",
            consumer.identifier, consumer.source, consumer.pricing.as_str()
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=pricing_census generations={} wallclock={} external_event={} blank={} total={}",
        consumer_count_by_pricing.0, consumer_count_by_pricing.1, consumer_count_by_pricing.2, consumer_count_by_pricing.3, CONSUMERS.len()
    )));

    // ── 判据 3：代数计价且需求值已知的消费者，逐 K ──
    for consumer in &CONSUMERS {
        let Some(required_generations) = consumer.required_generations else { continue };
        for window_generations in WINDOW_GENERATION_VALUES {
            println!("{}", emitter.emit_raw(&format!(
                "name=gen_consumer id={} k={window_generations} req_gens={required_generations} violations={}",
                consumer.identifier, generation_violations(required_generations, window_generations)
            )));
        }
    }

    // ── 判据 2 的核心：K 代窗口罩得住多长的墙钟需求 ──
    // 这是交叉点，不含任何假设的设备取值：窗口 = K / 发布率。
    for window_generations in WINDOW_GENERATION_VALUES {
        println!("{}", emitter.emit_raw(&format!(
            "name=window k={window_generations} window_seconds={:.9} covers_up_to_seconds={:.9}",
            window_seconds(window_generations, PUBLISH_RATE),
            window_seconds(window_generations, PUBLISH_RATE)
        )));
    }

    // ── 判据 4：假想墙钟消费者的定价表，闭式与模拟必须恰等 ──
    let mut cells_exceeding_window = 0u32;
    let total_cells = (DURATIONS.len() * WINDOW_GENERATION_VALUES.len()) as u32;
    for (label, duration_seconds) in DURATIONS {
        for window_generations in WINDOW_GENERATION_VALUES {
            let simulated_violations = simulate_violations(duration_seconds, PUBLISH_RATE, window_generations);
            let closed_form_violation_count = closed_form_violations(duration_seconds, PUBLISH_RATE, window_generations);
            let window_duration_seconds = window_seconds(window_generations, PUBLISH_RATE);
            if duration_seconds > window_duration_seconds {
                cells_exceeding_window += 1;
            }
            println!("{}", emitter.emit_raw(&format!(
                "name=hypothetical_wallclock case={label} k={window_generations} duration_seconds={duration_seconds:.6} \
                 window_seconds={window_duration_seconds:.9} exceeds={} sim_violations={simulated_violations} closed_violations={closed_form_violation_count} agree={}",
                u8::from(duration_seconds > window_duration_seconds),
                u8::from(simulated_violations == closed_form_violation_count)
            )));
        }
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_unit real_wallclock_consumers={} hypothetical_cells_exceeding={cells_exceeding_window} \
         hypothetical_cells_total={total_cells}",
        consumer_count_by_pricing.1
    )));

    // ── c8：逐 (S, N, K) 算假回退候选数 ──
    let mut cells_with_fakes = 0u32;
    let mut total_pairs = 0u32;
    for (slots_per_region, ring_depth) in RING_DEPTHS {
        for window_generations in WINDOW_GENERATION_VALUES {
            if window_generations > ring_depth { continue; }
            total_pairs += 1;
            let fake_candidates = fake_rollback_candidates(ring_depth, window_generations);
            if fake_candidates > 0 { cells_with_fakes += 1; }
            println!("{}", emitter.emit_raw(&format!(
                "name=admin_rollback s_per_region={slots_per_region} ring_depth={ring_depth} k={window_generations} \
                 promised_depth={ring_depth} usable_depth={window_generations} fake_candidates={fake_candidates}"
            )));
        }
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_rollback pairs={total_pairs} pairs_with_fake_candidates={cells_with_fakes} \
         k_must_equal_ring_depth={}",
        u8::from(cells_with_fakes > 0)
    )));

    // ── 判据 3 的判别力自证：K=1 必须出违例，K≥2 必须恒 0 ──
    let root_fallback_required_generations = CONSUMERS[0].required_generations.unwrap();
    println!("{}", emitter.emit_raw(&format!(
        "name=poscontrol_fallback k1_violations={} k2_violations={} discriminates={}",
        generation_violations(root_fallback_required_generations, 1),
        generation_violations(root_fallback_required_generations, 2),
        u8::from(generation_violations(root_fallback_required_generations, 1) > 0 && generation_violations(root_fallback_required_generations, 2) == 0)
    )));

    // ── 阴性对照：持续时间为 0 ⇒ 违例恒 0 ──
    println!("{}", emitter.emit_raw(&format!(
        "name=negcontrol_instant violations={} expect=0",
        simulate_violations(0.0, PUBLISH_RATE, 3)
    )));

    // ── C88 那一格：如实标空白，不是 0 ──
    println!("{}", emitter.emit_raw(
        "name=blank_cell id=c4_timeline_discriminate requirement=NA \
         reason=consumer_not_implemented_C88"
    ));

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1**：清单九条，逐条有出处，出处不许为空。
    /// 补列重跑之后必须比第一轮多出四条（I-4.8、记录核对器、管理员回退、遍历跳）。
    #[test]
    fn consumer_list_is_complete_and_sourced() {
        assert_eq!(CONSUMERS.len(), 9);
        assert!(CONSUMERS.iter().all(|consumer| !consumer.source.is_empty()));
        for expected_identifier in ["c6_i48_frozen_image", "c7_record_verifier", "c8_admin_rollback", "c9_traversal_hop"] {
            assert!(CONSUMERS.iter().any(|consumer| consumer.identifier == expected_identifier), "补列漏了 {expected_identifier}");
        }
    }

    /// **判据 2 的三选一必须都有对象**：第一轮「外部事件」是空的，补列之后不空。
    /// **而墙钟那一类只剩一个成员：遍历跳。** scrub 不在里面。
    #[test]
    fn pricing_census_absolute_values() {
        let generations_count = CONSUMERS.iter().filter(|consumer| consumer.pricing == Pricing::Generations).count();
        let wallclock_count = CONSUMERS.iter().filter(|consumer| consumer.pricing == Pricing::WallClock).count();
        let external_event_count = CONSUMERS.iter().filter(|consumer| consumer.pricing == Pricing::ExternalEvent).count();
        let blank_count = CONSUMERS.iter().filter(|consumer| consumer.pricing == Pricing::Blank).count();
        assert_eq!((generations_count, wallclock_count, external_event_count, blank_count), (6, 1, 1, 1));
        assert_eq!(generations_count + wallclock_count + external_event_count + blank_count, 9);
        let only_wallclock_consumer = CONSUMERS.iter().find(|consumer| consumer.pricing == Pricing::WallClock).unwrap();
        assert_eq!(only_wallclock_consumer.identifier, "c9_traversal_hop");
    }

    /// **c3 按 D13 的原话是水位型，对旧根的需求恒 0。**
    /// 这条钉住补列重跑改掉的那个稻草人：需求若回到「钉住一代根走到底」，它就红。
    #[test]
    fn scrub_is_a_watermark_consumer_not_an_old_root_consumer() {
        let scrub_consumer = CONSUMERS.iter().find(|consumer| consumer.identifier == "c3_scrub_watermark").unwrap();
        assert_eq!(scrub_consumer.required_generations, Some(0));
        assert_eq!(scrub_consumer.pricing, Pricing::Generations);
        for window_generations in WINDOW_GENERATION_VALUES {
            assert_eq!(generation_violations(0, window_generations), 0, "k={window_generations}");
        }
    }

    /// **冻结镜像上的两个消费者需求恒 0**：没有并发分配器跟它们抢。
    #[test]
    fn frozen_image_consumers_need_nothing() {
        for expected_identifier in ["c6_i48_frozen_image", "c7_record_verifier"] {
            let consumer = CONSUMERS.iter().find(|consumer| consumer.identifier == expected_identifier).unwrap();
            assert_eq!(consumer.required_generations, Some(0), "{expected_identifier}");
        }
    }

    /// **判据 3 的阳性对照**：择根回退的需求是 2 代（1 代 + 验证滞后）。
    /// K=1 出 1 次违例，K≥2 恒 0 ⇒ 模型分得出差别。
    /// ⚠️ 它同时是一次独立方向的复算：算出的 K 下限恰是 I-7.4 已写死的 2。
    #[test]
    fn root_fallback_needs_exactly_two_generations() {
        assert_eq!(CONSUMERS[0].required_generations, Some(2));
        assert_eq!(generation_violations(2, 1), 1);
        for window_generations in [2u64, 3, 4, 8] {
            assert_eq!(generation_violations(2, window_generations), 0, "k={window_generations}");
        }
    }

    /// **K 代窗口的绝对值**：2785 发布/秒下 K=2 是 0.718 毫秒、K=3 是 1.077 毫秒、
    /// K=8 是 2.873 毫秒。**这就是墙钟需求的交叉点**——一跳耗时超过它就不再被罩住。
    #[test]
    fn the_window_is_the_crossover_and_it_is_sub_ten_milliseconds() {
        assert!((window_seconds(2, PUBLISH_RATE) - 0.000_718_1).abs() < 1e-7);
        assert!((window_seconds(3, PUBLISH_RATE) - 0.001_077_2).abs() < 1e-7);
        assert!((window_seconds(8, PUBLISH_RATE) - 0.002_872_5).abs() < 1e-7);
    }

    /// **唯一那个墙钟消费者今天被罩住，且余量可算**：一跳 100 微秒时，
    /// K=2 的窗口 718 微秒是它的 7.18 倍；K=1 的 359 微秒仍是 3.59 倍。
    /// ⇒ 判据 2 **不触发**。
    #[test]
    fn the_only_wallclock_consumer_is_covered_with_margin() {
        let hop_seconds = 0.000_1_f64;
        assert!(hop_seconds < window_seconds(1, PUBLISH_RATE));
        let margin = window_seconds(2, PUBLISH_RATE) / hop_seconds;
        assert!((margin - 7.181).abs() < 0.01, "{margin}");
        assert_eq!(closed_form_violations(hop_seconds, PUBLISH_RATE, 2), 0);
    }

    /// **判据 4 的绝对值**（假想消费者的定价表）：10 秒 × 2785 = 27 850 次发布，
    /// K=3 时违例 27 847 次；0.1 秒那档 K=8 时 270 次。
    #[test]
    fn hypothetical_consumer_pricing_is_pinned() {
        assert_eq!(closed_form_violations(10.0, PUBLISH_RATE, 3), 27_847);
        assert_eq!(simulate_violations(10.0, PUBLISH_RATE, 3), 27_847);
        assert_eq!(closed_form_violations(0.1, PUBLISH_RATE, 8), 270);
        assert_eq!(closed_form_violations(0.001, PUBLISH_RATE, 2), 0);
    }

    /// **闭式与模拟在全部格上恰等**（判据 4）。两者各写各的。
    #[test]
    fn closed_form_and_simulation_agree_on_every_cell() {
        for (_, duration_seconds) in DURATIONS {
            for window_generations in WINDOW_GENERATION_VALUES {
                assert_eq!(
                    closed_form_violations(duration_seconds, PUBLISH_RATE, window_generations),
                    simulate_violations(duration_seconds, PUBLISH_RATE, window_generations),
                    "secs={duration_seconds} k={window_generations}"
                );
            }
        }
    }

    /// **阴性对照**：持续时间为 0 ⇒ 违例恒 0。
    #[test]
    fn instant_consumer_never_violates() {
        for window_generations in WINDOW_GENERATION_VALUES {
            assert_eq!(simulate_violations(0.0, PUBLISH_RATE, window_generations), 0);
            assert_eq!(closed_form_violations(0.0, PUBLISH_RATE, window_generations), 0);
        }
    }

    /// **C88 那一格是空白，不是 0**——读不到 ≠ 读到 0。
    #[test]
    fn the_unimplemented_consumer_is_blank_not_zero() {
        let timeline_consumer = CONSUMERS.iter().find(|consumer| consumer.identifier == "c4_timeline_discriminate").unwrap();
        assert_eq!(timeline_consumer.pricing, Pricing::Blank);
        assert_eq!(timeline_consumer.required_generations, None);
    }

    /// **管理员回退的触发按外部事件计价，但它的需求是根环深度，不是 2。**
    /// ⚠️ 这条替下第二轮那个只断言表里字面值的测试——那个测试锁死了一个错的分类，
    /// c8 判错时十条变异照样全绿（`.claude/singlefs-ai-sop/rules/show-me-test.md`
    /// 「测试测的是实现，不是契约」那一格）。这一条断的是机制：**假候选数**。
    #[test]
    fn admin_rollback_demands_the_whole_ring_depth() {
        let admin_rollback_consumer = CONSUMERS.iter().find(|consumer| consumer.identifier == "c8_admin_rollback").unwrap();
        assert_eq!(admin_rollback_consumer.pricing, Pricing::ExternalEvent);
        // R = 3 已定 ⇒ 最小的环也有 3 个槽；K 取下限 2 时就已经有 1 个假候选。
        assert_eq!(fake_rollback_candidates(3, 2), 1);
        // S = 16 那一档：环 48 槽，K=2 时 46 个候选是假的。
        assert_eq!(fake_rollback_candidates(48, 2), 46);
        // 只有 K 等于环深时假候选才归零。
        for (_, ring_depth) in RING_DEPTHS {
            assert_eq!(fake_rollback_candidates(ring_depth, ring_depth), 0, "depth={ring_depth}");
            assert!(fake_rollback_candidates(ring_depth, 2) > 0, "depth={ring_depth}");
        }
    }

    /// **环深表必须满足 N = R × S，R = 3 已定**（D22 已定项 2）。
    /// ⚠️ 这条是补出来的敏感取样点：把最大那档从 48 改成 24 时，
    /// 「k < depth 的对数」不变（两者都大于 WINDOW_GENERATION_VALUES 的最大值 8）⇒ 那条断言看不见
    /// （`.claude/rules/mutation-sampling.md` 的第三类：取样点不敏感，不是等价变异）。
    #[test]
    fn ring_depth_table_is_three_times_slots() {
        for (slots_per_region, ring_depth) in RING_DEPTHS {
            assert_eq!(ring_depth, 3 * slots_per_region, "S={slots_per_region}");
        }
        assert_eq!(RING_DEPTHS[RING_DEPTHS.len() - 1], (16, 48));
        assert_eq!(RING_DEPTHS[0], (1, 3));
    }

    /// **假候选这一格必须在扫描到的每一对 (N, K &lt; N) 上都非零**——
    /// 这是「K = 2 与 D23 已定项 14 的回退承诺不能同时成立」的可判形式。
    #[test]
    fn every_window_below_ring_depth_leaves_fake_candidates() {
        let mut pairs = 0;
        for (_, ring_depth) in RING_DEPTHS {
            for window_generations in WINDOW_GENERATION_VALUES {
                if window_generations >= ring_depth { continue; }
                assert!(fake_rollback_candidates(ring_depth, window_generations) > 0, "depth={ring_depth} k={window_generations}");
                pairs += 1;
            }
        }
        assert_eq!(pairs, 21);
    }
}
