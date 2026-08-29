//! E34：调小根环深度 K 到底能腾出多少空间
//!
//! 设计与依据见 `.claude/kb/experiments/34-两条钉块规则扣的不是同一批.md`。
//! **它攻的是** D22（单元原子性怎么合成）「根环深度 = 块重用延迟」一节里那句依据：
//! 「K 必须是运行时策略……那会把 ENOSPC 死锁的唯一逃生口焊死在格式里：
//! 盘快满时最该拿来救命的空间，正是被 K 扣住的那部分」。
//!
//! 仓里有两条互不相同的「刚释放的块不许重用」规则，它们扣的不是同一批块：
//!   根环规则     I-7.4：最近 K 代根引用的块不许重分配
//!                （K 代根 = 最近 K 次 fsync，D23 轴一取甲：每次 fsync 发根）
//!   重放窗口规则 D16 新规则 2 + D22 未定项 4：真正的约束是整个 journal 重放窗口，
//!                而 tail 只在 checkpoint 持久之后推进 ⇒ 窗口 = 距上次 checkpoint 的那些 fsync
//!
//! 判据：调小 K 能腾出的块数 = pinned(K) − pinned(2)。
//! 若它在一个 checkpoint 周期的绝大部分相位上恒为 0，那句「唯一逃生口」就是空的。
//!
//! 阳性对照：把 checkpoint 间隔设成 1（每次 fsync 都 checkpoint）⇒ 重放窗口塌成一格
//! ⇒ K 成为唯一约束 ⇒ 调小 K 必须腾出可观的量。
//! 少了它，「腾出 0」分不清是结论如此还是度量根本不动。

use e7_index_bench::Emitter;

/// 每次 fsync 释放多少块。取 D25（目标负载优先级）已定的粗粒度：一次 fsync 带 8 个叶子。
const FREED_PER_FSYNC: u64 = 8;

/// 根环规则扣住几次 fsync 的释放量——**仓里两处说法差一，两种都跑**。
///
/// | 读法 | 出处 | 值 |
/// |---|---|---|
/// | `Invariant` | I-7.4 的可判定形式「最近 K 代根**所引用的**块」 | **K − 1** |
/// | `Prose` | D22 散文「保留 K 代根 ⇒ 最近 K 个事务里释放的空间都不许重用」 | **K** |
///
/// `Invariant` 那一档的推导：块 b 在第 t 次 fsync 被释放 ⇒ 第 t 代根**不再引用它**、
/// 第 t−1 代根仍引用它。环里留着 `{now−K+1 … now}` 这 K 代
/// ⇒ b 被钉住 ⟺ `t−1 ≥ now−K+1` ⟺ `t ≥ now−K+2` ⇒ 恰好 **K−1** 个 t 值。
///
/// ⚠️ **两种读法都跑，是因为结论不许依赖这个差一**——若依赖，那本实验就是在
/// 量一处措辞不一致，而不是在量一个机制。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reading {
    Invariant,
    Prose,
}

impl Reading {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增读法不补这里就编译不过
            Reading::Invariant => "invariant_i74",
            Reading::Prose => "d22_prose",
        }
    }
    /// 根环规则扣住几次 fsync 的释放量。**与相位无关**——
    /// 环里留的是最近 K 代根，它跨越 checkpoint 边界，不因刚 checkpoint 过就变短。
    fn root_ring_events(self, k: u64) -> u64 {
        match self {
            Reading::Invariant => k.saturating_sub(1),
            Reading::Prose => k,
        }
    }
}

/// 相位 `p` = 距上次 checkpoint 已经过了几次 fsync（0 = 刚 checkpoint 完）。
///
/// 重放窗口规则扣住「本 checkpoint 窗口内释放的块」，即最近 `p + 1` 次 fsync；
/// tail 只在 checkpoint 持久之后推进，所以窗口在相位 0 处塌成一格。
fn pinned_events(reading: Reading, k: u64, phase: u64, ckpt_interval: u64) -> u64 {
    let root_ring = reading.root_ring_events(k);
    let replay_window = (phase + 1).min(ckpt_interval);
    root_ring.max(replay_window) // 并集：两条规则都要满足
}

/// 只被根环规则扣住、重放窗口规则扣不住的部分——**这才是调小 K 能动的那些块**。
fn k_only_events(reading: Reading, k: u64, phase: u64, ckpt_interval: u64) -> u64 {
    let replay_window = (phase + 1).min(ckpt_interval);
    reading.root_ring_events(k).saturating_sub(replay_window)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    /// 一个周期里，调小 K 到下限 2 能腾出的块数：最大值
    max_freed: u64,
    /// 同上：整周期总和
    total_freed: u64,
    /// 有多少个相位上「调小 K」腾得出哪怕一块
    phases_where_k_helps: u64,
    /// 周期长度（相位数）
    phases: u64,
    /// 对照量：同一周期里重放窗口规则平均扣住多少块
    mean_pinned_by_window: u64,
    /// 两条规则并集平均扣住多少块——「盘快满时被扣住的空间」总量就是它
    mean_pinned_total: u64,
    /// ⚠️ **同相位对照**：在逃生口真正存在的那个相位（p = 0）上，此刻合起来扣住多少块。
    /// 拿它跟 `max_freed` 比才是同一时刻的比较；拿整周期均值去比是**跨相位比较**，
    /// 会把「逃生口存在时也杯水车薪」说得比实际强。
    pinned_at_phase0: u64,
}

fn measure(reading: Reading, k: u64, ckpt_interval: u64) -> Out {
    let mut o = Out { phases: ckpt_interval, ..Default::default() };
    let mut win_sum = 0u64;
    let mut tot_sum = 0u64;
    for phase in 0..ckpt_interval {
        let freed = (k_only_events(reading, k, phase, ckpt_interval)
            .saturating_sub(k_only_events(reading, 2, phase, ckpt_interval)))
            * FREED_PER_FSYNC;
        o.max_freed = o.max_freed.max(freed);
        o.total_freed += freed;
        if freed > 0 {
            o.phases_where_k_helps += 1;
        }
        win_sum += (phase + 1).min(ckpt_interval) * FREED_PER_FSYNC;
        tot_sum += pinned_events(reading, k, phase, ckpt_interval) * FREED_PER_FSYNC;
    }
    o.mean_pinned_by_window = win_sum / ckpt_interval;
    o.mean_pinned_total = tot_sum / ckpt_interval;
    o.pinned_at_phase0 = pinned_events(reading, k, 0, ckpt_interval) * FREED_PER_FSYNC;
    o
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config note=调小K能腾出多少 freed_per_fsync={FREED_PER_FSYNC} \
             rule1=最近K次fsync rule2=本checkpoint窗口"
        ))
    );
    for reading in [Reading::Invariant, Reading::Prose] {
        for ckpt in [1u64, 8, 100, 1000] {
            for k in [2u64, 4, 8, 16] {
                let o = measure(reading, k, ckpt);
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=cell reading={} ckpt_interval={ckpt} k={k} max_freed={} \
                         total_freed={} phases_where_k_helps={} phases={} \
                         mean_pinned_by_window={} mean_pinned_total={} pinned_at_phase0={}",
                        reading.name(),
                        o.max_freed,
                        o.total_freed,
                        o.phases_where_k_helps,
                        o.phases,
                        o.mean_pinned_by_window,
                        o.mean_pinned_total,
                        o.pinned_at_phase0
                    ))
                );
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    const READINGS: [Reading; 2] = [Reading::Invariant, Reading::Prose];

    /// **主判据（绝对值，且与 checkpoint 间隔无关）：调小 K 有帮助的相位数，
    /// 在 I-7.4 读法下是 K−2，在 D22 散文读法下是 K−1。**
    ///
    /// 相位数由算术独立给出：`k_only(K,p) > k_only(2,p)` ⟺ `p + 1 < root_ring(K)`
    /// ⇒ 有帮助的相位是 `p ∈ [0, root_ring(K) − 2]`。
    /// **两种读法只差一个相位，结论不依赖选哪一种**——这正是两种都跑的理由。
    #[test]
    fn lowering_k_helps_in_a_phase_count_that_does_not_depend_on_the_interval() {
        for reading in READINGS {
            for ckpt in [100u64, 1000] {
                for k in [4u64, 8, 16] {
                    let want = reading.root_ring_events(k) - 1;
                    let o = measure(reading, k, ckpt);
                    assert_eq!(
                        o.phases_where_k_helps, want,
                        "{} ckpt={ckpt} k={k}：有帮助的相位数该是 {want}",
                        reading.name()
                    );
                }
            }
        }
    }

    /// **同相位对照：在逃生口真正存在的那个相位上，调小 K 腾出的量不是杯水车薪。**
    ///
    /// ⚠️ 这条测试是本实验最容易被写错的地方。拿「相位 0 才存在的 `max_freed`」
    /// 去比「整周期均值 `mean_pinned_by_window`」是**跨相位比较**，会得出一个
    /// 好看但不诚实的倍数（间隔 1000 时 250 倍，间隔 8 时只有约 2 倍）。
    /// 同一时刻的比较结果相反：相位 0 上根环恰恰是被扣空间的大头。
    #[test]
    fn at_the_phase_where_the_escape_hatch_exists_it_is_not_negligible() {
        for reading in READINGS {
            let o = measure(reading, 8, 1000);
            assert!(o.max_freed > 0, "{}：相位 0 上该腾得出东西", reading.name());
            assert!(
                o.max_freed * 2 >= o.pinned_at_phase0,
                "{}：相位 0 上腾出 {} 块、此刻共扣 {} 块——它不是杯水车薪，\
                 「250 倍」那种说法是跨相位比较，不许用",
                reading.name(),
                o.max_freed,
                o.pinned_at_phase0
            );
        }
    }

    /// **倍数随 checkpoint 间隔线性变，所以它不能当结论**——相位数才能。
    /// 间隔 1000 与间隔 8 上，同一个 K 的「均值比 max」差两个数量级。
    #[test]
    fn the_ratio_argument_is_interval_dependent_and_the_phase_count_is_not() {
        for reading in READINGS {
            let wide = measure(reading, 4, 1000);
            let narrow = measure(reading, 4, 8);
            assert_eq!(
                wide.phases_where_k_helps, narrow.phases_where_k_helps,
                "{}：相位数不该随间隔变", reading.name()
            );
            if narrow.max_freed > 0 && wide.max_freed > 0 {
                let r_wide = wide.mean_pinned_by_window / wide.max_freed;
                let r_narrow = narrow.mean_pinned_by_window / narrow.max_freed;
                assert!(
                    r_wide > r_narrow * 10,
                    "{}：倍数该随间隔显著变（宽 {} vs 窄 {}）",
                    reading.name(), r_wide, r_narrow
                );
            }
        }
    }

    /// **阳性对照：checkpoint 间隔 = 1 时 K 成为唯一约束，调小 K 必须腾出可观的量。**
    /// 少了它，「腾出 0」分不清是结论如此还是度量根本不动。
    #[test]
    fn with_a_checkpoint_every_fsync_lowering_k_really_does_free_space() {
        for reading in READINGS {
            for k in [4u64, 8, 16] {
                let o = measure(reading, k, 1);
                // 间隔 1 ⇒ 只有相位 0，窗口恒扣 1 次 fsync
                // ⇒ 腾出 = (根环(K) − 根环(2)) 次 fsync。**两种读法在这一格给同一个数。**
                let want = (reading.root_ring_events(k) - reading.root_ring_events(2))
                    * FREED_PER_FSYNC;
                assert_eq!(
                    o.max_freed, want,
                    "{} k={k}：每次 fsync 都 checkpoint 时该腾出 {want} 块",
                    reading.name()
                );
                assert_eq!(o.phases_where_k_helps, 1);
            }
        }
    }

    /// **K = 2 是下限，调不动**（D22 已定「下限 ≥ 2」）⇒ 腾出恒为 0。
    #[test]
    fn k_at_its_floor_frees_nothing_by_construction() {
        for reading in READINGS {
            for ckpt in [1u64, 8, 100, 1000] {
                assert_eq!(measure(reading, 2, ckpt).total_freed, 0);
            }
        }
    }

    /// **并集是真的并集**：K 大到超过窗口时总扣住量必须跟着 K 涨。
    #[test]
    fn the_total_pinned_set_is_the_union_of_both_rules() {
        for reading in READINGS {
            for k in [2u64, 4, 8, 16] {
                let want = reading.root_ring_events(k).max(1) * FREED_PER_FSYNC;
                assert_eq!(measure(reading, k, 1).mean_pinned_total, want,
                           "{} k={k}", reading.name());
            }
        }
    }

    /// **判别力：两条规则确实扣的不是同一批块。**
    /// 若把重放窗口也当成「最近 K 次」，本实验的全部结论都是假的。
    #[test]
    fn the_two_rules_pin_different_sets() {
        for reading in READINGS {
            let rr = reading.root_ring_events(8);
            assert_eq!(pinned_events(reading, 8, 0, 1000), rr, "刚 checkpoint 完，根环规则更大");
            assert_eq!(pinned_events(reading, 8, 99, 1000), 100, "相位 99 时重放窗口规则更大");
            assert_eq!(k_only_events(reading, 8, 0, 1000), rr - 1);
            assert_eq!(k_only_events(reading, 8, 99, 1000), 0, "相位 99 时根环规则一点也不多扣");
        }
    }

    /// **两种读法必须给出不同的数，且各自是它自己那条推导的结果。**
    ///
    /// 绝对值，不从被测代码反解：I-7.4 那一支是 K−1（块在第 t 次 fsync 被释放
    /// ⇒ 第 t 代根不再引用它 ⇒ 环里 K 代只覆盖 K−1 个 t 值）；
    /// D22 散文那一支逐字是「最近 K 个事务」。
    /// ⚠️ 少了这条，把两支合成一支（变异 M2）一个测试都不红——
    /// 而「结论不依赖选哪一支」这句话就没有被任何检查看着。
    #[test]
    fn the_two_readings_really_differ_and_by_exactly_one() {
        assert_eq!(Reading::Invariant.root_ring_events(4), 3);
        assert_eq!(Reading::Prose.root_ring_events(4), 4);
        assert_eq!(Reading::Invariant.root_ring_events(16), 15);
        assert_eq!(Reading::Prose.root_ring_events(16), 16);
        for k in [4u64, 8, 16] {
            let a = measure(Reading::Invariant, k, 1000).phases_where_k_helps;
            let b = measure(Reading::Prose, k, 1000).phases_where_k_helps;
            assert_eq!(a + 1, b, "k={k}：两支的相位数该恰好差一，实测 {a} 与 {b}");
        }
    }

    /// **根环扣住的量与相位无关**——环里留的是最近 K 代根，
    /// 它跨越 checkpoint 边界，不因刚 checkpoint 过就变短。
    /// ⚠️ 本地腿在这一点上判错过（它给的是 `min(相位, K)`），所以钉一条。
    #[test]
    fn the_root_ring_reaches_back_across_the_checkpoint_boundary() {
        for reading in READINGS {
            let rr = reading.root_ring_events(8);
            for phase in [0u64, 1, 7, 50, 999] {
                assert_eq!(
                    pinned_events(reading, 8, phase, 1000).max(rr),
                    pinned_events(reading, 8, phase, 1000),
                    "{} 相位 {phase}：根环扣住的量不该被相位截短", reading.name()
                );
            }
            assert_eq!(pinned_events(reading, 8, 0, 1000), rr,
                       "{}：相位 0 上扣住的就是根环那一段，不是 1", reading.name());
        }
    }
}
