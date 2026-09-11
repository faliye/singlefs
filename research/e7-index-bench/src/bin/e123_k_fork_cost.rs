//! E123：K 与回退深度的二选一各要付什么 —— C215 那个岔路的代价表。
//!
//! 臂① K ≡ 根环深度（改 I-7.4 的「运行时策略、下限 2」）
//! 臂② 回退深度 ≤ K（改 D23 已定项 14 的用户定案「回退深度 ≤ 根环深度」）
//!
//! ## 判据（2026-09-09 跑前写死）
//!
//! 1. 四个量两条臂都要给，同一套口径。
//! 2. 每个量逐 S 给出，不许只在一个 S 上取值。
//! 3. 阳性对照：S=1（N=3）时两条臂的「可退代数」必须不同（3 vs 2）；相同则整轮作废。
//! 4. 阴性对照：臂②的 K 设成 N 之后，四个量逐格相等；不等则整轮作废。
//! 5. 每个量要有钉绝对值的断言，不许只做臂间互比。
//!
//! ## 失败条款（跑前写死）
//!
//! - 判据 3 或 4 不中 ⇒ 整轮作废。
//! - 任何一个量在两条臂上恒等 ⇒ 如实记「这个量分不出两条臂」，不许调参数凑差异。
//! - 确定性模型：跑 N 轮只证无隐藏状态。

use e7_index_bench::Emitter;

/// R = 3 已定（D22 已定项 2：R = F + 1，F = 2 用户定案）。
const REGION_COUNT: u64 = 3;
/// 臂②的 K 取 I-7.4 的下限。
const ARM2_PROTECTED_GENERATIONS: u64 = 2;
/// 每次 fsync 写多少块：D25 粗粒度定案（8 叶 + 深 3 共享脊柱 + 1 根 + 1 记录）。
const BLOCKS_PER_FSYNC: u64 = 8 + 3 + 1 + 1;
/// 统计量数：E54 的示例值，不是定案。
const STATISTIC_COUNT: u64 = 64;

const SLOTS_PER_REGION_SWEEP: [u64; 5] = [1, 2, 4, 8, 16];

fn ring_depth(slots_per_region: u64) -> u64 {
    REGION_COUNT * slots_per_region
}

/// 量一：I-7.4 扣住的块数 = 保护代数 × 每次 fsync 写的块数（E50 的式子）。
fn pinned_blocks(protected_generations: u64) -> u64 {
    protected_generations * BLOCKS_PER_FSYNC
}

/// 量二：记账的 key 数 = 统计量数 × 保留代数；保留代数按 D5 已定项 2 的保守口径 = 保护代数 + 1。
fn accounting_keys(protected_generations: u64) -> u64 {
    STATISTIC_COUNT * (protected_generations + 1)
}

/// 量三：可退代数——臂①是环深，臂②是 K。两条臂都直接等于它的保护代数。
fn rollback_depth(protected_generations: u64) -> u64 {
    protected_generations
}

/// 量四：ENOSPC 逃生口的相位数 = 保护代数 − 1（E33 逐字「有帮助的相位数是 根环(K) − 1」）。
fn escape_phases(protected_generations: u64) -> u64 {
    protected_generations.saturating_sub(1)
}

/// 一条臂的四个量打包。判据 4 的阴性对照要拿它做差，不是拿一个值跟自己比。
fn metrics(protected_generations: u64) -> (u64, u64, u64, u64) {
    (
        pinned_blocks(protected_generations),
        accounting_keys(protected_generations),
        rollback_depth(protected_generations),
        escape_phases(protected_generations),
    )
}

/// 两条臂的差价，逐量相减。
fn arm_cost_difference(arm1_protected_generations: u64, arm2_protected_generations: u64) -> (u64, u64, u64, u64) {
    let arm1_metrics = metrics(arm1_protected_generations);
    let arm2_metrics = metrics(arm2_protected_generations);
    (arm1_metrics.0 - arm2_metrics.0, arm1_metrics.1 - arm2_metrics.1, arm1_metrics.2 - arm2_metrics.2, arm1_metrics.3 - arm2_metrics.3)
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config r={REGION_COUNT} arm2_k={ARM2_PROTECTED_GENERATIONS} blocks_per_fsync={BLOCKS_PER_FSYNC} \
         stat_count={STATISTIC_COUNT} s_values={}",
        SLOTS_PER_REGION_SWEEP.len()
    )));

    let mut differing = (0u32, 0u32, 0u32, 0u32); // 四个量各在几档 S 上分得出两条臂
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        let root_ring_depth = ring_depth(slots_per_region);
        // 臂①：保护代数 = 环深；臂②：保护代数 = K（下限 2）
        for (arm, protected_generations) in [("arm1_k_eq_depth", root_ring_depth), ("arm2_rollback_le_k", ARM2_PROTECTED_GENERATIONS)] {
            println!("{}", emitter.emit_raw(&format!(
                "name=cost arm={arm} s_per_region={slots_per_region} ring_depth={root_ring_depth} protect_gens={protected_generations} \
                 pinned_blocks={} accounting_keys={} rollback_depth={} escape_phases={}",
                pinned_blocks(protected_generations), accounting_keys(protected_generations),
                rollback_depth(protected_generations), escape_phases(protected_generations)
            )));
        }
        if pinned_blocks(root_ring_depth) != pinned_blocks(ARM2_PROTECTED_GENERATIONS) { differing.0 += 1; }
        if accounting_keys(root_ring_depth) != accounting_keys(ARM2_PROTECTED_GENERATIONS) { differing.1 += 1; }
        if rollback_depth(root_ring_depth) != rollback_depth(ARM2_PROTECTED_GENERATIONS) { differing.2 += 1; }
        if escape_phases(root_ring_depth) != escape_phases(ARM2_PROTECTED_GENERATIONS) { differing.3 += 1; }
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=discriminating s_total={} pinned={} accounting={} rollback={} phases={}",
        SLOTS_PER_REGION_SWEEP.len(), differing.0, differing.1, differing.2, differing.3
    )));

    // 判据 3：阳性对照——S=1 时两条臂的可退代数必须不同
    let smallest_root_ring_depth = ring_depth(1);
    println!("{}", emitter.emit_raw(&format!(
        "name=poscontrol_s1 ring_depth={smallest_root_ring_depth} arm1_rollback={} arm2_rollback={} differs={}",
        rollback_depth(smallest_root_ring_depth), rollback_depth(ARM2_PROTECTED_GENERATIONS),
        u8::from(rollback_depth(smallest_root_ring_depth) != rollback_depth(ARM2_PROTECTED_GENERATIONS))
    )));

    // 判据 4：阴性对照——臂②的 K 取成 N 之后，两条臂的差价必须逐量归零
    let mut merged_equal = true;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        let root_ring_depth = ring_depth(slots_per_region);
        merged_equal &= arm_cost_difference(root_ring_depth, root_ring_depth) == (0, 0, 0, 0);
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=negcontrol_merged all_equal={} expect=1", u8::from(merged_equal)
    )));

    // 两条臂的差价，逐 S 给绝对值
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        let root_ring_depth = ring_depth(slots_per_region);
        println!("{}", emitter.emit_raw(&format!(
            "name=delta s_per_region={slots_per_region} ring_depth={root_ring_depth} \
             extra_pinned_blocks={} extra_accounting_keys={} extra_rollback_gens={} \
             extra_escape_phases={}",
            arm_cost_difference(root_ring_depth, ARM2_PROTECTED_GENERATIONS).0, arm_cost_difference(root_ring_depth, ARM2_PROTECTED_GENERATIONS).1, arm_cost_difference(root_ring_depth, ARM2_PROTECTED_GENERATIONS).2, arm_cost_difference(root_ring_depth, ARM2_PROTECTED_GENERATIONS).3
        )));
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **环深的绝对值**：R = 3 已定 ⇒ S ∈ {1,2,4,8,16} 给出 N ∈ {3,6,12,24,48}。
    #[test]
    fn ring_depth_absolute_values() {
        assert_eq!(REGION_COUNT, 3);
        let ring_depths: Vec<u64> = SLOTS_PER_REGION_SWEEP.iter().map(|&slots_per_region| ring_depth(slots_per_region)).collect();
        assert_eq!(ring_depths, vec![3, 6, 12, 24, 48]);
    }

    /// **量一的绝对值**：粗粒度每次 fsync 13 块。
    /// 臂② 恒扣 2 × 13 = 26 块；臂① 在 S=16 上扣 48 × 13 = 624 块。
    #[test]
    fn pinned_blocks_absolute_values() {
        assert_eq!(BLOCKS_PER_FSYNC, 13);
        assert_eq!(pinned_blocks(ARM2_PROTECTED_GENERATIONS), 26);
        assert_eq!(pinned_blocks(3), 39);
        assert_eq!(pinned_blocks(48), 624);
    }

    /// **量二的绝对值**：统计量 64，保留代数 = 保护代数 + 1（D5 保守口径）。
    /// 臂② 是 64 × 3 = 192 条 key；臂① 在 S=16 上是 64 × 49 = 3136 条。
    #[test]
    fn accounting_keys_absolute_values() {
        assert_eq!(STATISTIC_COUNT, 64);
        assert_eq!(accounting_keys(ARM2_PROTECTED_GENERATIONS), 192);
        assert_eq!(accounting_keys(3), 256);
        assert_eq!(accounting_keys(48), 3136);
    }

    /// **量四的绝对值**：逃生口相位数 = 保护代数 − 1。
    /// 臂② 恒为 1；臂① 在 S=1 上是 2、S=16 上是 47。
    #[test]
    fn escape_phases_absolute_values() {
        assert_eq!(escape_phases(ARM2_PROTECTED_GENERATIONS), 1);
        assert_eq!(escape_phases(3), 2);
        assert_eq!(escape_phases(48), 47);
        // 保护代数为 0 时不许下溢
        assert_eq!(escape_phases(0), 0);
    }

    /// **判据 3 的阳性对照**：S=1（最小的环）时两条臂的可退代数就已经不同（3 vs 2）。
    #[test]
    fn positive_control_arms_differ_at_smallest_ring() {
        assert_eq!(ring_depth(1), 3);
        assert_eq!(rollback_depth(ring_depth(1)), 3);
        assert_eq!(rollback_depth(ARM2_PROTECTED_GENERATIONS), 2);
        assert_ne!(rollback_depth(ring_depth(1)), rollback_depth(ARM2_PROTECTED_GENERATIONS));
    }

    /// **判据 4 的阴性对照**：两条臂的保护代数取成同一个值时，差价逐量归零。
    /// ⚠️ 第一版把它写成了 `pinned_blocks(n) == pinned_blocks(n)`——拿一个值跟自己比，
    /// 永远成立，那条对照测不了任何东西
    /// （`.claude/singlefs-ai-sop/rules/show-me-test.md`「测试测的是实现，不是契约」）。
    /// 改成走 `arm_cost_difference`：差价函数算错时这条会红。
    #[test]
    fn negative_control_arms_merge_when_protection_is_equal() {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            let root_ring_depth = ring_depth(slots_per_region);
            assert_eq!(arm_cost_difference(root_ring_depth, root_ring_depth), (0, 0, 0, 0), "S={slots_per_region}");
        }
        // 差价非零的那一侧也要钉住，否则把 delta 写成恒 0 照样绿
        assert_ne!(arm_cost_difference(ring_depth(1), ARM2_PROTECTED_GENERATIONS), (0, 0, 0, 0));
    }

    /// **四个量在扫描到的每一档 S 上都分得出两条臂**——没有一个是恒等的。
    #[test]
    fn all_four_metrics_discriminate_on_every_slots_per_region() {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            let root_ring_depth = ring_depth(slots_per_region);
            assert_ne!(pinned_blocks(root_ring_depth), pinned_blocks(ARM2_PROTECTED_GENERATIONS), "S={slots_per_region}");
            assert_ne!(accounting_keys(root_ring_depth), accounting_keys(ARM2_PROTECTED_GENERATIONS), "S={slots_per_region}");
            assert_ne!(rollback_depth(root_ring_depth), rollback_depth(ARM2_PROTECTED_GENERATIONS), "S={slots_per_region}");
            assert_ne!(escape_phases(root_ring_depth), escape_phases(ARM2_PROTECTED_GENERATIONS), "S={slots_per_region}");
        }
    }

    /// **差价的绝对值**：S=16 那一档，臂① 比臂② 多扣 598 块、多 2944 条 key，
    /// 换来多退 46 代与多 46 个逃生相位。
    #[test]
    fn delta_absolute_values_at_the_largest_ring() {
        let root_ring_depth = ring_depth(16);
        assert_eq!(arm_cost_difference(root_ring_depth, ARM2_PROTECTED_GENERATIONS), (598, 2944, 46, 46));
    }

    /// **最小的环上差价也不为零**：S=1 时多扣 13 块、多 64 条 key，换来多退 1 代。
    #[test]
    fn delta_absolute_values_at_the_smallest_ring() {
        let root_ring_depth = ring_depth(1);
        assert_eq!(arm_cost_difference(root_ring_depth, ARM2_PROTECTED_GENERATIONS), (13, 64, 1, 1));
    }
}
