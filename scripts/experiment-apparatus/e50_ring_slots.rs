//! E50：根环槽数的上下界 —— D22 已定项 2 的最后一块。
//!
//! E48 把该项收窄成 R = F + 1（产品判断）与 S（每区槽数）。本实验测 S。
//! D22 对 S 只写过一句「一圈的时长 ≥ 最坏的块重用延迟需求」，而那句话从没被算过。
//!
//! ## 判据（跑前写死）
//!
//! 1. 下界：N ≥ 2（D22 已定 K 下限 2），且丢一个失败域后至少还剩 1 个可用槽；
//! 2. 上界：I-7.4 扣住的块数开始要紧的那个 N。**「要紧」的线跑前定死为 1000 块**；
//! 3. 一圈时长逐档给绝对值，并显式记「与最坏块重用延迟需求比不了——本仓没有那个量」；
//! 4. 两档 fsync 率 × 两档每次写块数分别给。
//!
//! ## 失败条款
//!
//! - 阳性对照：N 翻倍 ⇒ 扣住的块数翻倍。不线性 ⇒ 整轮作废。
//! - 阴性对照：每次写块数取 0 ⇒ 扣住恒 0。非 0 ⇒ 整轮作废。
//! - 两档 fsync 率给出同一个一圈时长 ⇒ 整轮作废。
//! - **全部 N 上都没超过 1000 块 ⇒ 如实记录「上界在扫描范围内不存在」**，不许把线往下调。

use e7_index_bench::Emitter;

/// 一圈时长（秒）= 槽总数 ÷ fsync 率。每次 fsync 发一次根（D23 轴一已定）。
fn lap_seconds(slots: u64, fsync_per_second: f64) -> f64 {
    slots as f64 / fsync_per_second
}

/// I-7.4 扣住的块数 = 槽总数 × 每次 fsync 写的块数。
fn pinned_blocks(slots: u64, blocks_per_fsync: u64) -> u64 {
    slots * blocks_per_fsync
}

/// 丢一个失败域之后还剩几个槽：跨区轮转 + 区域落在互不相同的盘上（E48 的可行点）
/// ⇒ 丢一个区域 = 丢 S 个槽。
fn surviving_slots(regions: u64, slots_per_region: u64) -> u64 {
    (regions - 1) * slots_per_region
}

/// 判据 2 的线：扣住的块数 ≥ 这个数才算「开始要紧」。**跑前定死。**
const PINNED_MATTERS: u64 = 1000;

/// 每次 fsync 写多少块。粗粒度取 D25 已定的形态：8 叶 + 1 条共享脊柱（深 3）+ 1 根 + 1 记录；
/// 散开作对照：8 叶各自一条脊柱。
const COARSE_BLOCKS: u64 = 8 + 3 + 1 + 1;
const SCATTERED_BLOCKS: u64 = 8 + 8 * 3 + 1 + 1;

const TOTAL_SLOT_COUNTS_SCANNED: [u64; 8] = [2, 4, 8, 16, 32, 64, 256, 1024];
const FSYNC_RATES: [(&str, f64); 2] = [("local_2785", 2785.0), ("enterprise_1e5", 100_000.0)];
const REGION_GEOMETRIES: [(u64, u64); 6] = [(2, 1), (2, 2), (2, 4), (3, 2), (3, 4), (4, 2)];

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config pinned_matters={PINNED_MATTERS} coarse_blocks={COARSE_BLOCKS} \
             scattered_blocks={SCATTERED_BLOCKS}"
        ))
    );

    // ── 量一 + 量二：逐 N ──
    let mut first_matter_coarse: Option<u64> = None;
    let mut first_matter_scattered: Option<u64> = None;
    for total_slot_count in TOTAL_SLOT_COUNTS_SCANNED {
        let pinned_coarse_blocks = pinned_blocks(total_slot_count, COARSE_BLOCKS);
        let pinned_scattered_blocks = pinned_blocks(total_slot_count, SCATTERED_BLOCKS);
        if pinned_coarse_blocks >= PINNED_MATTERS && first_matter_coarse.is_none() {
            first_matter_coarse = Some(total_slot_count);
        }
        if pinned_scattered_blocks >= PINNED_MATTERS && first_matter_scattered.is_none() {
            first_matter_scattered = Some(total_slot_count);
        }
        for (fsync_rate_label, fsync_per_second) in FSYNC_RATES {
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=slots n={total_slot_count} fsync={fsync_rate_label} lap_seconds={:.6} pinned_coarse={pinned_coarse_blocks} \
                     pinned_scattered={pinned_scattered_blocks} coarse_matters={} scattered_matters={}",
                    lap_seconds(total_slot_count, fsync_per_second),
                    u8::from(pinned_coarse_blocks >= PINNED_MATTERS),
                    u8::from(pinned_scattered_blocks >= PINNED_MATTERS),
                ))
            );
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=upper_bound first_matter_coarse={} first_matter_scattered={} \
             max_n_scanned={}",
            first_matter_coarse.map(|total_slot_count| total_slot_count.to_string()).unwrap_or_else(|| "none".into()),
            first_matter_scattered.map(|total_slot_count| total_slot_count.to_string()).unwrap_or_else(|| "none".into()),
            TOTAL_SLOT_COUNTS_SCANNED[TOTAL_SLOT_COUNTS_SCANNED.len() - 1],
        ))
    );

    // ── 量三：下界 —— 丢一个失败域之后还剩几个槽 ──
    for (region_count, slots_per_region) in REGION_GEOMETRIES {
        let surviving_slot_count = surviving_slots(region_count, slots_per_region);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=lower_bound regions={region_count} slots_per_region={slots_per_region} total={} \
                 surviving_after_one_domain={surviving_slot_count} meets_k2={}",
                region_count * slots_per_region,
                u8::from(region_count * slots_per_region >= 2 && surviving_slot_count >= 1),
            ))
        );
    }

    // ── 对照 ──
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_linear n8={} n16={} doubled={}",
            pinned_blocks(8, COARSE_BLOCKS),
            pinned_blocks(16, COARSE_BLOCKS),
            u8::from(pinned_blocks(16, COARSE_BLOCKS) == 2 * pinned_blocks(8, COARSE_BLOCKS)),
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=negcontrol_zero_blocks pinned={} expect=0",
            pinned_blocks(1024, 0)
        ))
    );

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **一圈时长的绝对值**：8 槽 ÷ 2785 次每秒 = 2.873 毫秒；企业盘那一档 80 微秒。
    /// ⇒ **根环一圈是毫秒级，不是秒级。**
    #[test]
    fn one_lap_is_milliseconds_not_seconds() {
        let local_disk_lap_seconds = lap_seconds(8, 2785.0);
        assert!((local_disk_lap_seconds - 0.0028725).abs() < 1e-6, "{local_disk_lap_seconds}");
        let enterprise_disk_lap_seconds = lap_seconds(8, 100_000.0);
        assert!((enterprise_disk_lap_seconds - 0.00008).abs() < 1e-9, "{enterprise_disk_lap_seconds}");
        // 1024 槽也只有 0.368 秒
        assert!((lap_seconds(1024, 2785.0) - 0.36769).abs() < 1e-4);
    }

    /// **两档 fsync 率必须给出不同的一圈时长**（这一维不是死的）。
    #[test]
    fn the_two_rates_are_not_the_same_thing() {
        for total_slot_count in TOTAL_SLOT_COUNTS_SCANNED {
            assert_ne!(lap_seconds(total_slot_count, 2785.0), lap_seconds(total_slot_count, 100_000.0), "n={total_slot_count}");
        }
        assert!((lap_seconds(8, 2785.0) / lap_seconds(8, 100_000.0) - 35.91).abs() < 0.01);
    }

    /// **扣住的块数的绝对值**：粗粒度每次 fsync 13 块 ⇒ 8 槽扣 104 块、1024 槽扣 13 312 块；
    /// 散开每次 34 块 ⇒ 8 槽扣 272 块。
    #[test]
    fn pinned_blocks_absolute_values() {
        assert_eq!(COARSE_BLOCKS, 13);
        assert_eq!(SCATTERED_BLOCKS, 34);
        assert_eq!(pinned_blocks(8, COARSE_BLOCKS), 104);
        assert_eq!(pinned_blocks(1024, COARSE_BLOCKS), 13_312);
        assert_eq!(pinned_blocks(8, SCATTERED_BLOCKS), 272);
    }

    /// **上界：粗粒度下要 77 槽才够到 1000 块这条线** ⇒ 扫描到的 N 里首个是 **256**。
    /// 散开下 30 槽就够到 ⇒ 首个是 **32**。
    #[test]
    fn the_upper_bound_is_far_above_any_sane_slot_count() {
        assert!(pinned_blocks(64, COARSE_BLOCKS) < PINNED_MATTERS, "64 槽只扣 832 块");
        assert!(pinned_blocks(256, COARSE_BLOCKS) >= PINNED_MATTERS);
        let first_coarse = TOTAL_SLOT_COUNTS_SCANNED.into_iter().find(|&total_slot_count| pinned_blocks(total_slot_count, COARSE_BLOCKS) >= PINNED_MATTERS);
        assert_eq!(first_coarse, Some(256));
        let first_scattered =
            TOTAL_SLOT_COUNTS_SCANNED.into_iter().find(|&total_slot_count| pinned_blocks(total_slot_count, SCATTERED_BLOCKS) >= PINNED_MATTERS);
        assert_eq!(first_scattered, Some(32));
    }

    /// **下界：丢一个失败域之后还剩几个槽**，逐几何钉死。
    /// 2 区 × 1 槽 ⇒ 剩 1（恰好够）；2 区 × 4 槽 ⇒ 剩 4；4 区 × 2 槽 ⇒ 剩 6。
    #[test]
    fn lower_bound_survivors_per_geometry() {
        assert_eq!(surviving_slots(2, 1), 1);
        assert_eq!(surviving_slots(2, 2), 2);
        assert_eq!(surviving_slots(2, 4), 4);
        assert_eq!(surviving_slots(3, 2), 4);
        assert_eq!(surviving_slots(4, 2), 6);
        // R=1 一个都不剩——与 E47 / E48 的阳性对照同一条
        assert_eq!(surviving_slots(1, 8), 0);
    }

    /// **判据 1 的合取**：N ≥ 2 且丢一个域后至少剩 1 个 ⇒ **S = 1 就已经满足**（只要 R ≥ 2）。
    #[test]
    fn one_slot_per_region_already_satisfies_the_lower_bound() {
        for region_count in [2u64, 3, 4] {
            let total_slot_count = region_count * 1;
            assert!(total_slot_count >= 2);
            assert!(surviving_slots(region_count, 1) >= 1, "R={region_count}");
        }
        // R=1 无论 S 多大都不满足
        for slots_per_region in [1u64, 4, 8] {
            assert_eq!(surviving_slots(1, slots_per_region), 0);
        }
    }

    /// **阳性对照**：N 翻倍 ⇒ 扣住的块数翻倍。
    #[test]
    fn positive_control_pinned_is_linear_in_slots() {
        for total_slot_count in [2u64, 8, 64] {
            assert_eq!(pinned_blocks(2 * total_slot_count, COARSE_BLOCKS), 2 * pinned_blocks(total_slot_count, COARSE_BLOCKS));
        }
    }

    /// **阴性对照**：每次写 0 块 ⇒ 扣住恒 0。
    #[test]
    fn negative_control_zero_blocks_pins_nothing() {
        for total_slot_count in TOTAL_SLOT_COUNTS_SCANNED {
            assert_eq!(pinned_blocks(total_slot_count, 0), 0, "n={total_slot_count}");
        }
    }
}
