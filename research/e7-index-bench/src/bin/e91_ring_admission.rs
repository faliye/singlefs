//! E91：已定格式下的环账与准入账 —— E16 维护代价三方论证挂出的账，全部纯算术。
//!
//! 设计与判据见 `.claude/kb/experiments/91-已定格式下的环账与准入账.md`（跑前写死）。
//!
//! 三笔账：
//!   一、环下界的和解：三个已发表的数（8 KiB / 6992 KiB / 4→8 MiB）各按其口径复现，
//!       再按已定格式口径（D23 已定项 12：4 KiB 记录、78 B 头、56 B/项）算 I-8.1 的约束值。
//!   二、保留时长：水位语义（D23 已定项 14）下甲每次 fsync 发布、延后形态只在
//!       checkpoint 发布（T_time 与 T_dirty 取先到，D16 已定项 5），模拟环内活记录峰值。
//!   三、准入水位：搅动量 4.0（祖先不延后）与 0.0567（延后，多流 ckpt=1000）
//!       各代入 D23 的准入公式，配 E19 形态的 pending 队列模拟对起点。

use e7_index_bench::Emitter;

// ── 已定常量（各自的出处在 kb 正文引用条款一节）─────────────────────────────
const RECORD_BYTES: u64 = 4096; // D23 已定项 12
const REC_HDR: u64 = 78; // D23 已定项 4（现行头宽）
const ENTRY_BYTES: u64 = 56; // 点名项含校验和
const OLD_ENTRY_BYTES: u64 = 32; // D16 已定项 5 推导用的旧口径（E70 的 ENTRY）
const NODE_BYTES: u64 = 16384;
const T_DIRTY: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB
const T_TIME_MS: u64 = 5000; // D16 已定项 5 取区间上端
const FSYNC_PER_SEC: u64 = 2785; // E44 本机实测
const CKPT_COST_BLOCKS: u64 = 64; // E19 口径
/// 目标负载：一次 fsync 8 叶 + 1 脊柱 = 12 项（D25 / E75 同一格）
const TARGET_ITEMS_PER_FSYNC: u64 = 12;
/// 搅动量，微块/操作（E16 实测：4.0000 与 0.0567 块/操作）
const CHURN_JIA_MICRO: u64 = 4_000_000;
const CHURN_DEFER_MICRO: u64 = 56_700;

fn items_per_record() -> u64 {
    (RECORD_BYTES - REC_HDR) / ENTRY_BYTES
}

fn records_for_items(items: u64) -> u64 {
    items.div_ceil(items_per_record())
}

/// 一个事务的 journal 占用（字节）：记录数 × 定长记录。
fn txn_occupancy_bytes(items: u64) -> u64 {
    records_for_items(items) * RECORD_BYTES
}

/// I-8.1 的环下界：F × 最坏事务占用。
fn ring_bound_bytes(items: u64, f: u64) -> u64 {
    f * txn_occupancy_bytes(items)
}

/// D16 已定项 5 的旧口径复现：每个脏 16 KiB 节点占一个 32 B 条目 ⇒ 占用 = T_dirty / 512。
fn old_caliber_occupancy_bytes() -> u64 {
    T_DIRTY / (NODE_BYTES / OLD_ENTRY_BYTES)
}

/// T_dirty 满窗的点名项数：每个脏节点一项。
fn t_dirty_items() -> u64 {
    T_DIRTY / NODE_BYTES
}

// ── 账二：保留时长模拟（水位语义）────────────────────────────────────────────
//
// 时间按 fsync 拍打点。每拍：写一条记录（目标负载 12 项 ⇒ 1 条），脏量 +12 节点。
// 甲：每拍发布（根持久 ⇒ 水位推进 ⇒ 之前的记录全死）。
// 延后：只在 checkpoint 发布——「时间 ≥ T_time ∨ 脏量 ≥ T_dirty」取先到（D16 已定项 2/5）。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Retention {
    peak_live_records: u64,
    peak_live_bytes: u64,
    /// 单条记录最长存活（拍数；一拍 = 1/FSYNC_PER_SEC 秒）
    max_lifetime_ticks: u64,
    publishes: u64,
}

fn simulate_retention(publish_every_fsync: bool, ticks: u64) -> Retention {
    let mut live: u64 = 0;
    let mut peak: u64 = 0;
    let mut dirty_bytes: u64 = 0;
    let mut ticks_since_pub: u64 = 0;
    let mut max_life: u64 = 0;
    let mut publishes: u64 = 0;
    for _ in 0..ticks {
        live += 1; // 本拍的记录进环（记录数按目标负载恰 1 条）
        dirty_bytes += TARGET_ITEMS_PER_FSYNC * NODE_BYTES;
        ticks_since_pub += 1;
        peak = peak.max(live);
        // 触发判据：时间那支按拍换算（T_time 秒 × 拍/秒），脏量那支按字节。取先到。
        let time_ticks = T_TIME_MS * FSYNC_PER_SEC / 1000;
        let should_publish = publish_every_fsync
            || ticks_since_pub >= time_ticks
            || dirty_bytes >= T_DIRTY;
        if should_publish {
            max_life = max_life.max(ticks_since_pub);
            live = 0; // 水位推进：所选根之下的记录全死（D23 已定项 14）
            dirty_bytes = 0;
            ticks_since_pub = 0;
            publishes += 1;
        }
    }
    Retention {
        peak_live_records: peak,
        peak_live_bytes: peak * RECORD_BYTES,
        max_lifetime_ticks: max_life,
        publishes,
    }
}

/// 闭式：延后形态的窗口拍数 = min(T_time 拍数, ceil(T_dirty / 每拍脏量))。
fn deferred_window_ticks() -> u64 {
    let time_ticks = T_TIME_MS * FSYNC_PER_SEC / 1000;
    let dirty_ticks = T_DIRTY.div_ceil(TARGET_ITEMS_PER_FSYNC * NODE_BYTES);
    time_ticks.min(dirty_ticks)
}

// ── 账三：准入水位 ──────────────────────────────────────────────────────────
//
// D23 准入规则逐字：「剩余空间必须 > 每 checkpoint 搅动量 × (延迟代数 + 1) + 一次
// checkpoint 的开销」。搅动量 = 每窗操作数 × 每操作搅动块。

fn admission_bound_blocks(churn_micro: u64, ops_per_ckpt: u64, delay: u64) -> u64 {
    churn_micro * ops_per_ckpt / 1_000_000 * (delay + 1) + CKPT_COST_BLOCKS
}

/// E19 形态的 pending 队列模拟：常数搅动下稳态被扣块数的峰值。
/// 独立路线——不用上面的闭式，逐窗推队列。
fn simulate_held_peak(churn_micro: u64, ops_per_ckpt: u64, delay: u64, windows: u64) -> u64 {
    let per_window = churn_micro * ops_per_ckpt / 1_000_000;
    let mut pending: Vec<u64> = vec![0; delay as usize];
    let mut freeing_now: u64;
    let mut peak: u64 = 0;
    for _ in 0..windows {
        freeing_now = per_window; // 本窗内逐操作释放，窗末达到 per_window
        let held: u64 = pending.iter().sum::<u64>() + freeing_now;
        peak = peak.max(held);
        // checkpoint：本窗释放入队，队首到期
        pending.push(freeing_now);
        pending.remove(0);
    }
    peak
}

fn main() {
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| {
        out.push_str(&s);
        out.push('\n');
    };
    say(em.emit_raw(&format!(
        "name=config record={RECORD_BYTES} hdr={REC_HDR} entry={ENTRY_BYTES} old_entry={OLD_ENTRY_BYTES} \
         node={NODE_BYTES} t_dirty={T_DIRTY} t_time_ms={T_TIME_MS} fsync_per_sec={FSYNC_PER_SEC} \
         items_per_record={}", items_per_record())));

    // ── 账一：三个已发表数字各按其口径复现 + 已定口径的约束值 ──
    say(em.emit_raw(&format!(
        "name=bound caliber=e75_target items=12 records={} occupancy_bytes={} f2_bound_kib={}",
        records_for_items(12), txn_occupancy_bytes(12), ring_bound_bytes(12, 2) / 1024)));
    say(em.emit_raw(&format!(
        "name=bound caliber=e75_worst items=62000 records={} occupancy_bytes={} f2_bound_kib={}",
        records_for_items(62_000), txn_occupancy_bytes(62_000), ring_bound_bytes(62_000, 2) / 1024)));
    say(em.emit_raw(&format!(
        "name=bound caliber=d16_item5_old_entry occupancy_mib={} f2_bound_mib={}",
        old_caliber_occupancy_bytes() / (1024 * 1024),
        2 * old_caliber_occupancy_bytes() / (1024 * 1024))));
    say(em.emit_raw(&format!(
        "name=bound caliber=settled_t_dirty items={} records={} occupancy_kib={} \
         f2_bound_kib={} f3_bound_kib={}",
        t_dirty_items(), records_for_items(t_dirty_items()),
        txn_occupancy_bytes(t_dirty_items()) / 1024,
        ring_bound_bytes(t_dirty_items(), 2) / 1024,
        ring_bound_bytes(t_dirty_items(), 3) / 1024)));

    // ── 账二：保留时长 ──
    let ticks = 40_000;
    let jia = simulate_retention(true, ticks);
    let def = simulate_retention(false, ticks);
    say(em.emit_raw(&format!(
        "name=retention arm=jia peak_records={} peak_bytes={} max_lifetime_ticks={} publishes={}",
        jia.peak_live_records, jia.peak_live_bytes, jia.max_lifetime_ticks, jia.publishes)));
    say(em.emit_raw(&format!(
        "name=retention arm=deferred peak_records={} peak_mib={} max_lifetime_ticks={} \
         lifetime_ms={} publishes={} closed_form_window={}",
        def.peak_live_records, def.peak_live_bytes / (1024 * 1024),
        def.max_lifetime_ticks,
        def.max_lifetime_ticks * 1000 / FSYNC_PER_SEC,
        def.publishes, deferred_window_ticks())));

    // ── 账三：准入水位 ──
    for (arm, churn) in [("jia_4.0", CHURN_JIA_MICRO), ("defer_0.0567", CHURN_DEFER_MICRO)] {
        for ops in [547u64, 2785, 13925] {
            for delay in [2u64, 4, 8, 16] {
                let bound = admission_bound_blocks(churn, ops, delay);
                let sim = simulate_held_peak(churn, ops, delay, 200) + CKPT_COST_BLOCKS;
                say(em.emit_raw(&format!(
                    "name=admission arm={arm} ops_per_ckpt={ops} delay={delay} \
                     bound_blocks={bound} bound_mib={} sim_blocks={sim} agree={}",
                    bound * 4096 / (1024 * 1024), bound == sim)));
            }
        }
    }
    say(em.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 1：每条记录 71 项，由 (4096−78)/56 独立算出（E75 四种头读法同值）。
    #[test]
    fn items_per_record_is_pinned_by_independent_arithmetic() {
        assert_eq!(items_per_record(), 71);
        assert_eq!((4096 - 78) / 56, 71);
    }

    /// 判据 2 之一：E75 目标负载那格逐字复现——12 项恰 1 条记录，F=2 环下界恰 8 KiB。
    #[test]
    fn e75_target_load_cell_reproduces() {
        assert_eq!(records_for_items(12), 1);
        assert_eq!(ring_bound_bytes(12, 2), 8 * 1024);
    }

    /// 判据 2 之二：E75 的 62 000 项那格逐字复现——874 条记录、F=2 环下界 6992 KiB。
    #[test]
    fn e75_worst_cell_reproduces() {
        assert_eq!(records_for_items(62_000), 874);
        assert_eq!(ring_bound_bytes(62_000, 2) / 1024, 6992);
    }

    /// 判据 2 之三：D16 已定项 5 的旧口径逐字复现——T_dirty/512 = 4 MiB，F=2 ⇒ 8 MiB。
    #[test]
    fn d16_item5_old_caliber_reproduces() {
        assert_eq!(old_caliber_occupancy_bytes(), 4 * 1024 * 1024);
        assert_eq!(2 * old_caliber_occupancy_bytes(), 8 * 1024 * 1024);
        // 旧口径的放大倍数确实是 512（16 KiB 节点 / 32 B 条目）
        assert_eq!(NODE_BYTES / OLD_ENTRY_BYTES, 512);
    }

    /// 已定口径的约束值，逐步独立钉死：131072 项、1847 条记录（71×1846 < 131072 ≤ 71×1847）、
    /// 占用 7388 KiB、F=2 环下界 14776 KiB ≈ 14.4 MiB。
    #[test]
    fn settled_caliber_bound_is_pinned() {
        assert_eq!(t_dirty_items(), 131_072);
        assert!(71 * 1846 < 131_072 && 131_072 <= 71 * 1847, "记录数该恰为 1847");
        assert_eq!(records_for_items(131_072), 1847);
        assert_eq!(txn_occupancy_bytes(131_072) / 1024, 1847 * 4);
        assert_eq!(ring_bound_bytes(131_072, 2) / 1024, 14_776);
        assert_eq!(ring_bound_bytes(131_072, 3) / 1024, 22_164);
    }

    /// 已定口径与旧口径**确实不同**——这是入题的那条待验命题的判定：
    /// 同一个 T_dirty 满窗，56 B/项 + 4 KiB 定长记录的占用比 32 B/条目多。
    #[test]
    fn settled_and_old_calibers_disagree() {
        let settled = txn_occupancy_bytes(t_dirty_items());
        let old = old_caliber_occupancy_bytes();
        assert!(settled > old,
            "已定口径 {settled} 该大于旧口径 {old}——记录头与 56 B/项都比 32 B/条目贵");
        // 差异幅度也钉住：7388 KiB vs 4096 KiB
        assert_eq!(settled / 1024, 7388);
        assert_eq!(old / 1024, 4096);
    }

    /// 判据 3（账二）：延后形态的模拟峰值与闭式窗口恰等；取先到的是脏量那支
    /// （T_dirty/每拍 192 KiB = 10923 拍 < T_time 的 13925 拍）。
    #[test]
    fn deferred_retention_matches_closed_form_and_t_dirty_wins() {
        let time_ticks = T_TIME_MS * FSYNC_PER_SEC / 1000;
        assert_eq!(time_ticks, 13_925);
        let dirty_ticks = T_DIRTY.div_ceil(TARGET_ITEMS_PER_FSYNC * NODE_BYTES);
        assert_eq!(dirty_ticks, 10_923, "2 GiB / 192 KiB 向上取整");
        assert_eq!(deferred_window_ticks(), 10_923, "取先到该是脏量那支");
        let r = simulate_retention(false, 40_000);
        assert_eq!(r.peak_live_records, deferred_window_ticks(), "模拟峰值与闭式窗口不等");
        assert_eq!(r.max_lifetime_ticks, deferred_window_ticks());
    }

    /// 判据 4 阳性对照（账二退化档）：每次发布即截断 ⇒ 峰值恰塌到单事务的 1 条记录。
    #[test]
    fn per_fsync_publish_collapses_peak_to_one_record() {
        let r = simulate_retention(true, 40_000);
        assert_eq!(r.peak_live_records, 1);
        assert_eq!(r.max_lifetime_ticks, 1);
        assert_eq!(r.publishes, 40_000, "每拍都该发布");
    }

    /// 判据 3（账三）：pending 队列模拟的稳态峰值 + 开销与闭式逐格恰等。
    #[test]
    fn admission_sim_agrees_with_closed_form_everywhere() {
        for churn in [CHURN_JIA_MICRO, CHURN_DEFER_MICRO] {
            for ops in [547u64, 2785, 13925] {
                for delay in [2u64, 4, 8, 16] {
                    let bound = admission_bound_blocks(churn, ops, delay);
                    let sim = simulate_held_peak(churn, ops, delay, 200) + CKPT_COST_BLOCKS;
                    assert_eq!(bound, sim, "churn={churn} ops={ops} delay={delay}");
                }
            }
        }
    }

    /// 账三的绝对值：甲在 T_time=5 s 满窗（13925 操作）、延迟 8 代时的不可填水位
    /// 恰为 4×13925×9+64 = 501364 块 ≈ 1.91 GiB；延后形态同格 7161 块 ≈ 27.97 MiB。
    #[test]
    fn admission_watermarks_are_pinned() {
        let jia = admission_bound_blocks(CHURN_JIA_MICRO, 13_925, 8);
        assert_eq!(jia, 4 * 13_925 * 9 + 64);
        assert_eq!(jia, 501_364);
        let def = admission_bound_blocks(CHURN_DEFER_MICRO, 13_925, 8);
        // 0.0567 块/操作 × 13925 = 789.55 → 微块整除后 789
        assert_eq!(def, 789 * 9 + 64);
    }

    /// E19 原参数形状复现（判据 4 阳性对照）：churn=4 块/操作 × 200 操作、延迟 d ⇒
    /// 稳态被扣峰值恰 800×(d+1)——与 E19 的 pending 队列同构。
    #[test]
    fn e19_shape_reproduces_under_its_own_parameters() {
        for d in [1u64, 2, 4, 8, 16] {
            let sim = simulate_held_peak(4_000_000, 200, d, 100);
            assert_eq!(sim, 800 * (d + 1), "delay={d}");
        }
    }

    /// 甲与延后的水位比值必须等于搅动量之比（同式两次代入的守恒，防止某一臂被单独改错）。
    #[test]
    fn watermark_ratio_tracks_the_churn_ratio() {
        let ops = 13_925u64;
        let d = 8u64;
        let jia = admission_bound_blocks(CHURN_JIA_MICRO, ops, d) - CKPT_COST_BLOCKS;
        let def = admission_bound_blocks(CHURN_DEFER_MICRO, ops, d) - CKPT_COST_BLOCKS;
        // 4.0/0.0567 = 70.55…；整数截断后按块数比对（55700/789 = 70.6）
        let ratio_x10 = jia * 10 / def;
        assert!((700..=712).contains(&ratio_x10), "比值 ×10 = {ratio_x10}，该在 70.0–71.2");
    }
}
