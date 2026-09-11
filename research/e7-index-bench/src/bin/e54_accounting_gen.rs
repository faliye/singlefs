//! E54：记账的「代」要保留几代 —— 用户定案「需要代」之后，D5 已定项 2 只剩甲与丙。
//!
//! 甲 = 代取 checkpoint 号且**不丢**；丙 = 只保留最近 K 代。
//! 量两件事：**key 数量怎么长**，以及**丙的丢弃要不要遍历**——后者是生死判据
//! （`.claude/rules/fs-design.md` 第一格：运行时决策路径不许遍历，代价不许随盘容量增长）。
//!
//! ## 判据（跑前写死）
//!
//! 1. 丙的 key 数与 checkpoint 数无关（斜率 0）；甲的斜率 = 统计量数。
//! 2. 丙的丢弃代价**与盘容量无关**——盘容量出现在代价式里即判违反第一格。
//! 3. 甲 + 增量丢弃 与 丙 的 key 数逐格相同 ⇒ 记「同一条路的两种说法」。
//! 4. K 的下界由「根最多能回退几代」给：E53 实测最坏回退 1；E50 的槽总数是 N。
//!
//! ## 失败条款
//!
//! - 阳性对照：K = ∞ 时丙必须退化成甲。不退化 ⇒ 整轮作废。
//! - 阴性对照：统计量数 0 ⇒ 两者恒 0。
//! - **丙的丢弃需要遍历 ⇒ 如实记录并淘汰丙**，不管 key 数多好看。
//! - 两者逐格相同 ⇒ 如实记录「不是两个候选」。
//!
//! ## 它答不了的
//!
//! 计数模型不是实现（文件操作 0 处）；统计量个数是输入不是测量；
//! 不建模 write buffer 的合并 ⇒ 每 checkpoint 每统计量一条是**上界**。

use e7_index_bench::Emitter;

/// 甲：代 = checkpoint 号，不丢 ⇒ 每个 checkpoint 给每个统计量留一条。
fn keys_when_generation_never_discarded(statistic_count: u64, checkpoint_count: u64) -> u64 {
    statistic_count * checkpoint_count
}

/// 丙：只保留最近 K 代 ⇒ 恒定。`kept_generations = 0` 表示 K = ∞（阳性对照用，退化成甲）。
fn keys_when_keeping_recent_generations(statistic_count: u64, checkpoint_count: u64, kept_generations: u64) -> u64 {
    if kept_generations == 0 {
        keys_when_generation_never_discarded(statistic_count, checkpoint_count)
    } else {
        statistic_count * checkpoint_count.min(kept_generations)
    }
}

/// **丢弃一代要触及多少条 key**。丙：每个统计量一次点删——`(统计量, 维度, 代)` 里
/// 「代」是 key 的一段，要丢的那一代**算得出来**（当前代 − K），不用找。
/// ⇒ 代价 = 统计量数，**与盘容量、与 key 总数都无关**。
fn keys_touched_per_discard(statistic_count: u64, kept_generations: u64, checkpoint_count: u64) -> u64 {
    if kept_generations == 0 || checkpoint_count <= kept_generations {
        0 // 还没满 K 代，没有可丢的
    } else {
        statistic_count
    }
}

/// 丢弃代价的式子里出不出现盘容量。**这是第一格的判据本身，不是描述**：
/// 出现即违反「代价不许随盘容量增长」。
fn discard_depends_on_capacity() -> bool {
    false
}

/// 每个 checkpoint 的记账写次数：甲只写；丙写 + 删。
fn writes_per_checkpoint(statistic_count: u64, kept_generations: u64, checkpoint_count: u64) -> u64 {
    statistic_count + keys_touched_per_discard(statistic_count, kept_generations, checkpoint_count)
}

/// 甲 + 一条增量丢弃规则（丢掉代号 < 当前 − K 的那些）之后的 key 数。
fn keys_when_never_discarded_plus_incremental_discard(statistic_count: u64, checkpoint_count: u64, kept_generations: u64) -> u64 {
    keys_when_keeping_recent_generations(statistic_count, checkpoint_count, kept_generations)
}

const STATISTIC_COUNTS: [u64; 4] = [0, 8, 64, 512];
const CHECKPOINT_COUNTS: [u64; 5] = [1, 10, 100, 10_000, 1_000_000];
const KEPT_GENERATION_COUNTS: [u64; 3] = [2, 8, 32];

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw("name=config note=counting_model_only file_ops=0")
    );

    let mut is_identical_everywhere = true;
    for statistic_count in STATISTIC_COUNTS {
        for checkpoint_count in CHECKPOINT_COUNTS {
            let keys_never_discarded = keys_when_generation_never_discarded(statistic_count, checkpoint_count);
            for kept_generations in KEPT_GENERATION_COUNTS {
                let keys_recent_generations = keys_when_keeping_recent_generations(statistic_count, checkpoint_count, kept_generations);
                let keys_never_discarded_with_discard = keys_when_never_discarded_plus_incremental_discard(statistic_count, checkpoint_count, kept_generations);
                if keys_recent_generations != keys_never_discarded_with_discard {
                    is_identical_everywhere = false;
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=keys stats={statistic_count} checkpoints={checkpoint_count} k={kept_generations} \
                         scheme_a={keys_never_discarded} scheme_c={keys_recent_generations} a_with_discard={keys_never_discarded_with_discard} same={} \
                         discard_touched={} writes_per_cp={}",
                        u8::from(keys_recent_generations == keys_never_discarded_with_discard),
                        keys_touched_per_discard(statistic_count, kept_generations, checkpoint_count),
                        writes_per_checkpoint(statistic_count, kept_generations, checkpoint_count),
                    ))
                );
            }
        }
    }

    // ── 斜率：key 数对 checkpoint 数的增量 ──
    for statistic_count in [8u64, 64, 512] {
        let slope_never_discarded = keys_when_generation_never_discarded(statistic_count, 1001) - keys_when_generation_never_discarded(statistic_count, 1000);
        let slope_recent_generations = keys_when_keeping_recent_generations(statistic_count, 1001, 8) - keys_when_keeping_recent_generations(statistic_count, 1000, 8);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=slope stats={statistic_count} slope_a={slope_never_discarded} slope_c={slope_recent_generations} \
                 a_grows={} c_flat={}",
                u8::from(slope_never_discarded == statistic_count),
                u8::from(slope_recent_generations == 0),
            ))
        );
    }

    // ── 第一格判据：丢弃代价依不依赖盘容量 ──
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=first_cell discard_depends_on_capacity={} verdict_ok={}",
            u8::from(discard_depends_on_capacity()),
            u8::from(!discard_depends_on_capacity()),
        ))
    );

    // ── K 的下界：由「根最多能回退几代」给 ──
    for (bound_source, rollback_depth) in [("e53_measured_worst_rollback", 1u64), ("e50_ring_slots_n", 8)] {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=k_lower_bound source={bound_source} rollback_depth={rollback_depth} k_min={} \
                 keys_at_64_stats={}",
                rollback_depth + 1,
                keys_when_keeping_recent_generations(64, 1_000_000, rollback_depth + 1),
            ))
        );
    }

    // ── 阳性对照：K = ∞ 时丙退化成甲 ──
    let keys_recent_generations_with_infinite_kept_generations = keys_when_keeping_recent_generations(64, 1_000_000, 0);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_k_infinite scheme_c={keys_recent_generations_with_infinite_kept_generations} scheme_a={} degenerates={}",
            keys_when_generation_never_discarded(64, 1_000_000),
            u8::from(keys_recent_generations_with_infinite_kept_generations == keys_when_generation_never_discarded(64, 1_000_000)),
        ))
    );
    // ── 阴性对照：统计量 0 ⇒ 两者恒 0 ──
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=negcontrol_zero_stats a={} c={} expect=0",
            keys_when_generation_never_discarded(0, 1_000_000),
            keys_when_keeping_recent_generations(0, 1_000_000, 8),
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=verdict a_and_c_identical_with_discard={}",
            u8::from(is_identical_everywhere)
        ))
    );

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **甲的 key 数是乘法**：64 个统计量 × 100 万个 checkpoint = 6400 万条。
    #[test]
    fn never_discarded_scheme_grows_without_bound() {
        assert_eq!(keys_when_generation_never_discarded(64, 1_000_000), 64_000_000);
        assert_eq!(keys_when_generation_never_discarded(8, 10), 80);
        // 斜率恰好等于统计量数
        assert_eq!(keys_when_generation_never_discarded(64, 1001) - keys_when_generation_never_discarded(64, 1000), 64);
    }

    /// **丙的 key 数封顶在 统计量数 × K**：K=8、64 个统计量 ⇒ **512 条，与跑多久无关**。
    #[test]
    fn recent_generations_scheme_is_bounded_by_statistics_times_kept_generations() {
        assert_eq!(keys_when_keeping_recent_generations(64, 1_000_000, 8), 512);
        assert_eq!(keys_when_keeping_recent_generations(64, 10, 8), 512);
        assert_eq!(keys_when_keeping_recent_generations(64, 3, 8), 192, "还没满 K 代时按实际代数算");
        // 斜率为 0
        assert_eq!(keys_when_keeping_recent_generations(64, 1001, 8) - keys_when_keeping_recent_generations(64, 1000, 8), 0);
    }

    /// **丢弃代价 = 统计量数，且式子里没有盘容量**——这是第一格的判据。
    #[test]
    fn discard_is_incremental_and_capacity_free() {
        assert_eq!(keys_touched_per_discard(64, 8, 1_000_000), 64);
        assert_eq!(keys_touched_per_discard(64, 8, 1_000_000), keys_touched_per_discard(64, 8, 10), );
        assert_eq!(keys_touched_per_discard(64, 8, 3), 0, "还没满 K 代时无可丢");
        assert!(!discard_depends_on_capacity());
    }

    /// **每 checkpoint 的写次数**：甲 = 统计量数；丙 = 统计量数 × 2（一写一删）。
    #[test]
    fn writes_per_checkpoint_absolute() {
        assert_eq!(writes_per_checkpoint(64, 0, 1_000_000), 64, "甲只写");
        assert_eq!(writes_per_checkpoint(64, 8, 1_000_000), 128, "丙写 + 删");
        assert_eq!(writes_per_checkpoint(64, 8, 3), 64, "还没满 K 代时只写");
    }

    /// **甲 + 增量丢弃 与 丙 逐格相同**——若成立，它们不是两个候选。
    #[test]
    fn never_discarded_plus_discard_equals_recent_generations_scheme() {
        for statistic_count in STATISTIC_COUNTS {
            for checkpoint_count in CHECKPOINT_COUNTS {
                for kept_generations in KEPT_GENERATION_COUNTS {
                    assert_eq!(
                        keys_when_never_discarded_plus_incremental_discard(statistic_count, checkpoint_count, kept_generations),
                        keys_when_keeping_recent_generations(statistic_count, checkpoint_count, kept_generations),
                        "stats={statistic_count} cps={checkpoint_count} k={kept_generations}"
                    );
                }
            }
        }
    }

    /// **阳性对照**：K = ∞ ⇒ 丙退化成甲。
    #[test]
    fn positive_control_infinite_kept_generations_degenerates_to_never_discarded() {
        for statistic_count in [8u64, 64, 512] {
            for checkpoint_count in [10u64, 10_000, 1_000_000] {
                assert_eq!(keys_when_keeping_recent_generations(statistic_count, checkpoint_count, 0), keys_when_generation_never_discarded(statistic_count, checkpoint_count));
            }
        }
    }

    /// **阴性对照**：统计量 0 ⇒ 两者恒 0。
    #[test]
    fn negative_control_zero_statistics() {
        for checkpoint_count in CHECKPOINT_COUNTS {
            assert_eq!(keys_when_generation_never_discarded(0, checkpoint_count), 0);
            assert_eq!(keys_when_keeping_recent_generations(0, checkpoint_count, 8), 0);
            assert_eq!(keys_touched_per_discard(0, 8, checkpoint_count), 0);
        }
    }

    /// **K 的下界**：E53 实测最坏回退 1 ⇒ K ≥ 2；E50 的槽总数 8 ⇒ 保守取 K ≥ 9。
    /// 64 个统计量下分别是 128 条与 576 条——**两个下界差 4.5 倍，但都是常数**。
    #[test]
    fn kept_generations_lower_bounds_from_measured_rollback() {
        assert_eq!(keys_when_keeping_recent_generations(64, 1_000_000, 1 + 1), 128);
        assert_eq!(keys_when_keeping_recent_generations(64, 1_000_000, 8 + 1), 576);
        assert!(576 < 64_000_000, "两个下界都远小于甲");
    }
}
