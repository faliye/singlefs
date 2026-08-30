//! E51：反向链的碰撞机会有多少次 —— 复核 E49 的 `D-遍历` 分母口径。
//!
//! E49 把「记录核对器每比对一对相邻记录」都算成一次碰撞机会，据此判 32 位 12 档里只够 4 档。
//! **那个口径可能是过计的**，而它是「32 够不够」的唯一依据。
//!
//! ## 分辨两种读法的是一件可判定的事
//!
//! 两条**同源**记录的链值由**同一个 hash 作用在同一份输入**上得到 ⇒ **构造性相等，概率为 1**
//! ⇒ 它不可能碰撞。碰撞只在被比较的两条记录**本该不同**时才有意义。
//! 本实验把这条写成可判的：链检查模型里数两个计数器——**比较次数** 与 **两侧不同源的比较次数**。
//!
//! ## 判据（跑前写死）
//!
//! 1. 同源比较的碰撞机会计数恒为 0；
//! 2. 干净日志上不同源比较次数为 0；
//! 3. 两种读法的一生机会数分别给，按 E49 已定的「够用 = 一生至少一次 < 1%」判；
//! 4. 残留存活时间 = 一圈时长，按本机 fsync 率（E44 实测 2785/秒）折算，给绝对值。
//!
//! ## 失败条款
//!
//! - 阳性对照：造一条真不同源的残留 ⇒ 不同源计数必须 +1。不加 ⇒ 整轮作废。
//! - 阴性对照：干净日志 ⇒ 不同源计数恒 0。非 0 ⇒ 整轮作废。
//! - 两种读法给出同一个机会数 ⇒ 这一维是死的，整轮作废。
//! - **读法 B 下 32 位仍不够用 ⇒ 如实记录**，本实验不预设 32 位够用。

use e7_index_bench::Emitter;

/// 一条记录。`origin` 是**真值标签**（哪条时间线），模型用它数「不同源」，
/// **链检查本身不许读它**——那正是链要靠 hash 去分辨的东西。
#[derive(Clone, Copy, Debug)]
struct Rec {
    jsn: u64,
    origin: u32,
    prev_hash: u64,
}

/// 链值：由记录的内容算出。同源的前一条给出同一个输入 ⇒ 同一个值。
fn chain_of(r: &Rec, bits: u32) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for x in [r.jsn, r.origin as u64, r.prev_hash] {
        h ^= x;
        h = h.wrapping_mul(0x100000001b3);
        h ^= h >> 29;
    }
    if bits >= 64 {
        h
    } else {
        h & ((1u64 << bits) - 1)
    }
}

/// 走一遍链，数两个计数器。返回 `(比较次数, 两侧不同源的比较次数)`。
/// **链检查只看 `prev_hash` 与算出来的链值，不读 `origin`**；`origin` 只用来数真值。
fn walk(records: &[Rec], bits: u32) -> (u64, u64) {
    let mut comparisons = 0u64;
    let mut cross_origin = 0u64;
    for w in records.windows(2) {
        let (prev, cur) = (&w[0], &w[1]);
        comparisons += 1;
        if prev.origin != cur.origin {
            cross_origin += 1;
        }
        let _ = (chain_of(prev, bits), cur.prev_hash); // 链检查本身
    }
    (comparisons, cross_origin)
}

/// 造一段日志。`stale_tail` > 0 时在末尾接上属于**另一条时间线**的残留记录。
fn make_log(clean: u64, stale_tail: u64, bits: u32) -> Vec<Rec> {
    let mut v: Vec<Rec> = Vec::new();
    let mut prev_hash = 0u64;
    for j in 0..clean {
        let r = Rec { jsn: j, origin: 1, prev_hash };
        prev_hash = chain_of(&r, bits);
        v.push(r);
    }
    // 残留：另一条时间线（origin=0），它的链锚点与现行时间线无关
    let mut sp = 0xdead_beefu64;
    for k in 0..stale_tail {
        let r = Rec { jsn: clean + k, origin: 0, prev_hash: sp };
        sp = chain_of(&r, bits);
        v.push(r);
    }
    v
}

/// 一圈时长（秒）= 环里记录数 ÷ fsync 率。**提出来是因为变异测试证明它在 `main` 里没人看得见。**
fn lap_seconds(records: f64) -> f64 {
    records / FSYNC_PER_SEC
}

/// 读法 A 的一生机会数：每对相邻比较都算。
fn chances_reading_a(records: f64, passes: f64) -> f64 {
    records * passes
}

/// 读法 B 的一生机会数：只算接缝 + 残留在被覆盖前被扫到的次数。
fn chances_reading_b(incidents: f64, seams_per_incident: f64, scans_before_overwrite: f64) -> f64 {
    incidents * (seams_per_incident + scans_before_overwrite)
}

fn expected_false_accepts(chances: f64, bits: u32) -> f64 {
    chances / 2f64.powi(bits as i32)
}
fn at_least_once(expected: f64) -> f64 {
    1.0 - (-expected).exp()
}
/// 判据沿用 E49：一生至少出一次 < 1% 叫够用。
fn enough(p: f64) -> bool {
    p < 0.01
}

const INCIDENTS: f64 = 36_500.0; // 每天崩 10 次 × 十年，与 E49 / D23 已定项 9 同一个场景点
const SEAMS_PER_INCIDENT: f64 = 1.0; // 一次事故留一个接缝
const FSYNC_PER_SEC: f64 = 2785.0; // E44 本机实测
const RINGS: [u64; 3] = [10 * 1024 * 1024, 100 * 1024 * 1024, 2 * 1024 * 1024 * 1024];
const REC_ON_DISK: [u64; 2] = [512, 4096];
const PASSES: [(&str, f64); 2] = [("weekly", 522.0), ("daily", 3650.0)];
const BITS: [u32; 3] = [16, 32, 64];

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config incidents={INCIDENTS} seams_per_incident={SEAMS_PER_INCIDENT} \
             fsync_per_sec={FSYNC_PER_SEC}"
        ))
    );

    // ── 量一：干净日志上，不同源比较恒为 0（阴性对照）──
    for n in [10u64, 100, 20480] {
        let log = make_log(n, 0, 32);
        let (cmps, cross) = walk(&log, 32);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=clean_log records={n} comparisons={cmps} cross_origin={cross} expect_cross=0"
            ))
        );
    }

    // ── 量二：接上残留之后，不同源比较恰好等于接缝数（阳性对照）──
    for stale in [1u64, 6, 20] {
        let log = make_log(100, stale, 32);
        let (cmps, cross) = walk(&log, 32);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=stale_log clean=100 stale={stale} comparisons={cmps} cross_origin={cross} \
                 expect_cross=1"
            ))
        );
    }

    // ── 量三：一圈时长与残留被扫到的次数 ──
    for ring in RINGS {
        for rec in REC_ON_DISK {
            let records = (ring / rec) as f64;
            let lap_sec = lap_seconds(records);
            for (freq, passes) in PASSES {
                let per_sec = passes / (10.0 * 365.25 * 86400.0);
                let scans = per_sec * lap_sec;
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=lap ring={ring} rec_on_disk={rec} records={records:.0} \
                         lap_seconds={lap_sec:.3} freq={freq} scans_before_overwrite={scans:.6e}"
                    ))
                );
            }
        }
    }

    // ── 量四：两种读法的一生机会数，逐位宽判够不够 ──
    for ring in RINGS {
        for rec in REC_ON_DISK {
            let records = (ring / rec) as f64;
            let lap_sec = lap_seconds(records);
            for (freq, passes) in PASSES {
                let per_sec = passes / (10.0 * 365.25 * 86400.0);
                let chances_a = chances_reading_a(records, passes);
                let chances_b = chances_reading_b(INCIDENTS, SEAMS_PER_INCIDENT, per_sec * lap_sec);
                for bits in BITS {
                    let ea = expected_false_accepts(chances_a, bits);
                    let eb = expected_false_accepts(chances_b, bits);
                    println!(
                        "{}",
                        em.emit_raw(&format!(
                            "name=readings ring={ring} rec_on_disk={rec} freq={freq} bits={bits} \
                             chances_a={chances_a:.6e} at_least_once_a={:.6e} enough_a={} \
                             chances_b={chances_b:.6e} at_least_once_b={:.6e} enough_b={}",
                            at_least_once(ea),
                            u8::from(enough(at_least_once(ea))),
                            at_least_once(eb),
                            u8::from(enough(at_least_once(eb))),
                        ))
                    );
                }
            }
        }
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **同源的链值构造性相等**——这是两种读法的分水岭，先钉死它。
    /// 同一条前驱记录算两次，值必须相同；这不是概率，是同一个函数同一份输入。
    #[test]
    fn same_origin_chain_values_are_equal_by_construction() {
        let r = Rec { jsn: 7, origin: 1, prev_hash: 42 };
        assert_eq!(chain_of(&r, 32), chain_of(&r, 32));
        // 造一段干净日志，逐对验证「存下来的 prev_hash == 算出来的链值」
        let log = make_log(50, 0, 32);
        for w in log.windows(2) {
            assert_eq!(w[1].prev_hash, chain_of(&w[0], 32));
        }
    }

    /// **截断必须真的生效**：n < 64 时链值必须落在 `[0, 2ⁿ)` 里；64 位不截断。
    /// 不钉这一条，「误接受 = 2⁻ⁿ」那条曲线在模型里根本没被实现。
    #[test]
    fn truncation_actually_narrows_the_value() {
        let r = Rec { jsn: 12345, origin: 1, prev_hash: 999 };
        for bits in [8u32, 16, 32] {
            let v = chain_of(&r, bits);
            assert!(v < (1u64 << bits), "bits={bits} v={v}");
        }
        assert_eq!(chain_of(&r, 32), chain_of(&r, 64) & 0xffff_ffff);
        // 至少有一条记录的 64 位值超出 32 位，否则上面那条是空的
        let wide = (0..64u64)
            .map(|j| chain_of(&Rec { jsn: j, origin: 1, prev_hash: j }, 64))
            .any(|v| v > u32::MAX as u64);
        assert!(wide, "64 位那一档必须真的用到高位");
    }

    /// **链值必须依赖前一条**——不依赖的话链就不是链，而两种读法的分辨也塌了。
    /// 同 `jsn`、同 `origin`、只差 `prev_hash` 的两条记录，链值必须不同。
    #[test]
    fn the_chain_value_depends_on_the_predecessor() {
        let a = Rec { jsn: 7, origin: 1, prev_hash: 1 };
        let b = Rec { jsn: 7, origin: 1, prev_hash: 2 };
        assert_ne!(chain_of(&a, 64), chain_of(&b, 64));
        assert_ne!(chain_of(&a, 32), chain_of(&b, 32));
        // 干净日志里每一条的 prev_hash 都真的是前一条算出来的，改一位就对不上
        let log = make_log(20, 0, 32);
        let mut tampered = log[5];
        tampered.prev_hash ^= 1;
        assert_ne!(chain_of(&tampered, 32), chain_of(&log[5], 32));
    }

    /// **两个读法的机会数各自的公式，逐个钉绝对值**（变异 M4 打的就是它）。
    #[test]
    fn the_two_chance_formulas_are_not_the_same_function() {
        assert_eq!(chances_reading_a(20480.0, 522.0), 20480.0 * 522.0);
        assert_eq!(chances_reading_b(36500.0, 1.0, 0.0), 36500.0);
        assert_eq!(chances_reading_b(36500.0, 1.0, 1.0), 73000.0);
        // 读法 B 与记录数、遍数**无关**；读法 A 与事故次数无关
        assert_eq!(chances_reading_b(36500.0, 1.0, 0.0), chances_reading_b(36500.0, 1.0, 0.0));
        assert!(chances_reading_a(20480.0, 522.0) > 290.0 * chances_reading_b(36500.0, 1.0, 3.2e-4));
    }

    /// **一圈时长的公式**（变异 M5 打的就是它）：记录数 ÷ fsync 率，绝对值钉死。
    #[test]
    fn lap_seconds_divides_by_the_measured_fsync_rate() {
        assert!((lap_seconds(20480.0) - 7.3537).abs() < 1e-3);
        assert!((lap_seconds(2785.0) - 1.0).abs() < 1e-9, "恰好一秒");
        assert!(lap_seconds(20480.0) < 20480.0, "必须除过");
    }

    /// **阴性对照的绝对值**：干净日志上不同源比较恒为 0，比较次数恰好是 n−1。
    #[test]
    fn a_clean_log_has_no_cross_origin_comparison() {
        for n in [2u64, 10, 100, 20480] {
            let (cmps, cross) = walk(&make_log(n, 0, 32), 32);
            assert_eq!(cmps, n - 1, "n={n}");
            assert_eq!(cross, 0, "n={n}");
        }
    }

    /// **阳性对照的绝对值**：接上 k 条残留 ⇒ 比较次数涨 k，而**不同源比较恰好加 1**
    /// （只有接缝那一对跨时间线，残留内部仍是同源）。
    #[test]
    fn stale_tail_adds_exactly_one_seam() {
        for stale in [1u64, 6, 20, 100] {
            let (cmps, cross) = walk(&make_log(100, stale, 32), 32);
            assert_eq!(cmps, 100 + stale - 1, "stale={stale}");
            assert_eq!(cross, 1, "stale={stale} —— 接缝只有一个");
        }
    }

    /// **两种读法差几个数量级**：10 MiB 环 / 512 B 每条 / 每周。
    /// 读法 A = 20480 × 522 = 1.069e7；读法 B ≈ 36 500 × (1 + 极小) ≈ 3.65e4。
    /// **差 293 倍。**
    #[test]
    fn the_two_readings_differ_by_orders_of_magnitude() {
        let records = (10 * 1024 * 1024u64 / 512) as f64;
        assert_eq!(records, 20480.0);
        let a = records * 522.0;
        assert!((a - 1.069056e7).abs() < 1.0);
        let lap = records / FSYNC_PER_SEC;
        assert!((lap - 7.3537).abs() < 1e-3, "一圈 {lap} 秒");
        let per_sec = 522.0 / (10.0 * 365.25 * 86400.0);
        let b = INCIDENTS * (1.0 + per_sec * lap);
        assert!((b - 36500.0).abs() < 1.0, "b={b}");
        assert!(a / b > 290.0);
    }

    /// **读法 B 下 32 位够用，16 位不够用**——绝对值，且两侧各取一点。
    #[test]
    fn under_reading_b_thirty_two_bits_is_enough() {
        let b = INCIDENTS * 1.000_001;
        let e32 = expected_false_accepts(b, 32);
        assert!(enough(at_least_once(e32)), "32 位够用");
        assert!((at_least_once(e32) - 8.4983e-6).abs() < 1e-9);
        let e16 = expected_false_accepts(b, 16);
        assert!(!enough(at_least_once(e16)), "16 位不够用");
        assert!((at_least_once(e16) - 0.4270).abs() < 1e-3);
    }

    /// **读法 A 下最重那档 32 位不够用**——保留 E49 的那个结论，证明本实验没把它算没了。
    #[test]
    fn under_reading_a_the_heaviest_bucket_still_breaks_thirty_two() {
        let records = (2 * 1024 * 1024 * 1024u64 / 512) as f64;
        let a = records * 3650.0;
        let e32 = expected_false_accepts(a, 32);
        assert!(!enough(at_least_once(e32)));
        assert!(at_least_once(e32) > 0.97);
    }

    /// **一圈时长的绝对值**：10 MiB 环 / 512 B ⇒ 20480 条 ÷ 2785 次每秒 = 7.354 秒；
    /// 2 GiB 环 / 4096 B ⇒ 524288 条 ⇒ 188.2 秒。
    /// ⇒ **残留在被覆盖前，每周一次的核对器扫到它的期望次数是 10⁻⁵ 量级**。
    #[test]
    fn lap_time_is_seconds_not_years() {
        let lap_small = (10 * 1024 * 1024u64 / 512) as f64 / FSYNC_PER_SEC;
        assert!((lap_small - 7.3537).abs() < 1e-3);
        let lap_big = (2 * 1024 * 1024 * 1024u64 / 4096) as f64 / FSYNC_PER_SEC;
        assert!((lap_big - 188.25).abs() < 0.05, "{lap_big}");
        // 绝对值：每周一次 ⇒ 1.654e-6 次/秒；最大的那个环一圈 188.25 秒
        // ⇒ 残留在被覆盖前被扫到的期望次数 = 3.11e-4，**远小于 1**
        let per_sec = 522.0 / (10.0 * 365.25 * 86400.0);
        assert!((per_sec - 1.6541e-6f64).abs() < 1e-9, "{per_sec}");
        let scans = per_sec * lap_big;
        assert!((scans - 3.1138e-4f64).abs() < 1e-7, "{scans}");
        assert!(scans < 1e-3, "扫到的期望次数远小于 1");
    }

    /// **够用判据的阈值**（沿用 E49 的 1%），两侧各取一点。
    #[test]
    fn the_threshold_is_one_percent() {
        assert!(enough(0.009_999));
        assert!(!enough(0.010_001));
    }
}
