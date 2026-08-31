//! E47：根环失败域的损失 —— D22（单元原子性怎么合成）已定项 2 的最后一维。
//!
//! ## 它补的是什么洞
//!
//! E46（根环区域间距与失败域）证伪了「盘首与盘尾失败域独立」（18 格里 7 格同盘），
//! 但它只答了「区域会不会一起死」，**没答「一起死了会损失什么」**——
//! 而 D22 已定项 2 卡住的那个数（「要容忍几个失败域」）恰恰要靠损失的大小来定。
//!
//! ## 核心自变量：槽序，而 D22 从没定过这一维
//!
//! ① **区内连续**：`slot = txg mod (R×S)`，区域 = `slot / S` ⇒ 连续 S 次发布落在同一个区域。
//! ② **跨区轮转**：区域 = `txg mod R`，区内槽 = `(txg / R) mod S` ⇒ 相邻两次发布落在不同区域。
//!
//! 丢一个区域时，①丢的是**连续 S 代**，②丢的是**每 R 代里的一代**。
//! **这不是差一点，是差一个量纲。**
//!
//! ## 判据（跑前写死）
//!
//! 1. 幸存槽数 = `R×S − S × 落在失效设备上的区域数`（绝对值钉死）。
//! 2. 两种槽序的最坏回退**必须不同**：区内连续最坏 = S，跨区轮转最坏 = 1（R ≥ 2）。
//!    相等 ⇒ 槽序这一维没被实现，整轮作废。
//! 3. 容忍 F 个失败域要 R ≥ F+1，**且 R 个区域落在 R 个互不相同的设备上**。
//! 4. `usable = rollback ≤ K−1`（I-7.4 只保证最近 K 代块未被复用）
//!    ⇒ **可挂载 ≠ 可用**，两者分开量。
//!
//! ## 失败条款
//!
//! - 阳性对照：R=1 且承载它的设备失效 ⇒ 幸存必须为 0。非 0 ⇒ 整轮作废。
//! - 阴性对照：设备数 = 1 时任何几何全丢。有幸存 ⇒ 整轮作废。
//! - 两种槽序在所有格上 rollback 相同 ⇒ 整轮作废。
//!
//! ## 它答不了的
//!
//! 物理失败域的真实形状（要真硬件）；`rollback` 是**代数不是字节**，换算要 D25 的负载模型。

use e7_index_bench::Emitter;

/// 条带布局下 LBA 落在哪块盘。形态取自 Linux md（raid0/raid5 按 chunk 轮转）。
/// ⚠️ **故意与 E46 各写一份并各自单测**：两处共用同一段代码，两个实验就不再是两条独立路径。
fn device_of(lba: u64, chunk: u64, devs: u64) -> u64 {
    (lba / chunk) % devs
}

/// 槽序。两种都合法，D22 已定项 2 从没点名过。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotOrder {
    /// `slot = txg mod (R×S)`，区域 = `slot / S`。
    WithinRegion,
    /// 区域 = `txg mod R`，区内槽 = `(txg / R) mod S`。
    AcrossRegions,
}

impl SlotOrder {
    fn name(self) -> &'static str {
        match self {
            SlotOrder::WithinRegion => "within_region",
            SlotOrder::AcrossRegions => "across_regions",
        }
    }
    /// 第 `txg` 次发布落在哪个区域。
    fn region_of(self, txg: u64, regions: u64, slots: u64) -> u64 {
        match self {
            SlotOrder::WithinRegion => (txg % (regions * slots)) / slots,
            SlotOrder::AcrossRegions => txg % regions,
        }
    }
}

/// 区域 `r` 的起始 LBA：区域 0 在盘首，其余按 `(容量 − 区域字节) × r / (R − 1)` 均分到盘尾，
/// 并按 chunk 向下对齐。R = 1 时只有盘首那一个。
fn region_position(r: u64, regions: u64, capacity: u64, region_bytes: u64, chunk: u64) -> u64 {
    if regions <= 1 || r == 0 {
        return 0;
    }
    let span = capacity - region_bytes;
    let raw = span * r / (regions - 1);
    (raw / chunk) * chunk
}

/// 一次失效之后，最近 `ring` 代里还有多少代是可读的，以及最新可读的那一代落后多少。
/// 返回 `(幸存槽数, Option<回退代数>)`；`None` 表示一个都不剩。
///
/// 口径：假定环已经写满至少一圈，最新一代记为 `latest`；
/// 第 `t` 代住在 `order.region_of(t, R, S)` 这个区域，该区域落在 `dev_of_region[r]` 这块盘上。
fn survey(
    order: SlotOrder,
    regions: u64,
    slots: u64,
    dev_of_region: &[u64],
    failed_dev: u64,
    latest: u64,
) -> (u64, Option<u64>) {
    let ring = regions * slots;
    let mut survivors = 0u64;
    let mut newest_alive: Option<u64> = None;
    // 环里活着的是最近 ring 代：latest, latest-1, ..., latest-ring+1
    for back in 0..ring {
        let t = latest - back;
        let r = order.region_of(t, regions, slots);
        if dev_of_region[r as usize] == failed_dev {
            continue; // 这一代随区域一起没了
        }
        survivors += 1;
        if newest_alive.is_none() {
            newest_alive = Some(t);
        }
    }
    (survivors, newest_alive.map(|t| latest - t))
}

/// **对相位取最坏**。回退代数依赖崩溃发生在一轮里的哪个相位（`latest mod (R×S)`）——
/// 只取一个相位得到的是那个相位的值，不是最坏值。
/// 返回 `(幸存槽数, 最坏回退, 最好回退, 有多少个相位一个都不剩)`。
///
/// ⚠️ **这个函数是本实验的第一版实现漏掉的那一层**：第一版固定 `latest = 1_000_000`，
/// 而 `1_000_000 % 8 == 0` 恰好落在区域 0 那一段的**开头**，于是区内连续那条臂量出的回退是 1 而不是 4。
/// 单元测试当场判红（判据写的是「最坏回退」），修的是实现不是判据。
fn survey_worst(
    order: SlotOrder,
    regions: u64,
    slots: u64,
    dev_of_region: &[u64],
    failed_dev: u64,
) -> (u64, Option<u64>, Option<u64>, u64) {
    let ring = regions * slots;
    let mut survivors = 0u64;
    let mut worst: Option<u64> = None;
    let mut best: Option<u64> = None;
    let mut dead_phases = 0u64;
    for phase in 0..ring {
        let latest = ring * 1_000_000 + phase; // 环已写满多圈，相位取遍一圈
        let (s, rb) = survey(order, regions, slots, dev_of_region, failed_dev, latest);
        survivors = s; // 与相位无关，逐相位相同
        match rb {
            None => dead_phases += 1,
            Some(v) => {
                worst = Some(worst.map_or(v, |w: u64| w.max(v)));
                best = Some(best.map_or(v, |b: u64| b.min(v)));
            }
        }
    }
    (survivors, worst, best, dead_phases)
}

/// 容忍 `f` 个失败域要几个区域。与 E46 同一条算术，本文件独立重写。
fn regions_needed(f: u64) -> u64 {
    f + 1
}

const CHUNKS: [u64; 3] = [64 * 1024, 512 * 1024, 4 * 1024 * 1024];
const DEVS: [u64; 6] = [1, 2, 3, 4, 6, 8];
const REGIONS: [u64; 4] = [1, 2, 3, 4];
const SLOTS: [u64; 3] = [2, 4, 8];
const KS: [u64; 3] = [2, 4, 8];
const CAPACITY: u64 = 10 * 1000 * 1000 * 1000 * 1000; // 10 TB，与 E40 / E46 同口径
const REGION_BYTES: u64 = 1024; // E41 候选：每区 4 槽 × 256 字节。槽宽已被 E41 判红，这里只用它定位
const LATEST: u64 = 1_000_000; // 已经跑了很久，环写满过多圈

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config capacity={CAPACITY} region_bytes={REGION_BYTES} latest={LATEST}"
        ))
    );

    // ── 量一：逐格的幸存与回退 ──
    let mut worst_within = 0u64;
    let mut worst_across = 0u64;
    let mut unmountable = 0u64;
    let mut mountable_but_unusable = 0u64;
    let mut total = 0u64;
    for &regions in REGIONS.iter() {
        for &slots in SLOTS.iter() {
            for chunk in CHUNKS {
                for devs in DEVS {
                    let dev_of_region: Vec<u64> = (0..regions)
                        .map(|r| {
                            device_of(
                                region_position(r, regions, CAPACITY, REGION_BYTES, chunk),
                                chunk,
                                devs,
                            )
                        })
                        .collect();
                    let distinct = {
                        let mut v = dev_of_region.clone();
                        v.sort_unstable();
                        v.dedup();
                        v.len() as u64
                    };
                    for failed in 0..devs {
                        for order in [SlotOrder::WithinRegion, SlotOrder::AcrossRegions] {
                            let (survivors, rollback, best, dead_phases) =
                                survey_worst(order, regions, slots, &dev_of_region, failed);
                            total += 1;
                            let mountable = survivors > 0;
                            if !mountable {
                                unmountable += 1;
                            }
                            if let Some(rb) = rollback {
                                match order {
                                    SlotOrder::WithinRegion => worst_within = worst_within.max(rb),
                                    SlotOrder::AcrossRegions => worst_across = worst_across.max(rb),
                                }
                            }
                            // usable 按三档 K 各判一次
                            let mut usable_bits = String::new();
                            for k in KS {
                                let u = rollback.map(|rb| rb <= k - 1).unwrap_or(false);
                                usable_bits.push_str(&format!(" usable_k{k}={}", u8::from(u)));
                                if mountable && !u {
                                    mountable_but_unusable += 1; // 计的是「格 × K 档」，分母 cases × 3
                                }
                            }
                            println!(
                                "{}",
                                em.emit_raw(&format!(
                                    "name=loss order={} regions={regions} slots={slots} \
                                     chunk={chunk} devs={devs} failed_dev={failed} \
                                     distinct_devs={distinct} survivors={survivors} \
                                     worst_rollback={} best_rollback={} dead_phases={dead_phases} \
                                     mountable={}{}",
                                    order.name(),
                                    rollback
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|| "NA".into()),
                                    best.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
                                    u8::from(mountable),
                                    usable_bits,
                                ))
                            );
                        }
                    }
                }
            }
        }
    }
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=loss_summary cases={total} k_cells={} unmountable={unmountable} \
             mountable_but_unusable_kcells={mountable_but_unusable} \
             worst_rollback_within_region={worst_within} \
             worst_rollback_across_regions={worst_across} \
             orders_differ={}",
            total * KS.len() as u64,
            u8::from(worst_within != worst_across),
        ))
    );

    // ── 量二：R 个区域能不能落在 R 块互不相同的盘上 ──
    for &regions in REGIONS.iter() {
        let mut ok = 0u64;
        let mut cells = 0u64;
        for chunk in CHUNKS {
            for devs in DEVS {
                let mut v: Vec<u64> = (0..regions)
                    .map(|r| {
                        device_of(
                            region_position(r, regions, CAPACITY, REGION_BYTES, chunk),
                            chunk,
                            devs,
                        )
                    })
                    .collect();
                v.sort_unstable();
                v.dedup();
                cells += 1;
                if v.len() as u64 == regions {
                    ok += 1;
                }
            }
        }
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=distinct_domains regions={regions} cells={cells} all_distinct={ok} \
                 tolerates_domains={}",
                regions.saturating_sub(1),
            ))
        );
    }

    // ── 量三：容忍 F 个失败域要几个区域 ──
    for f in 0u64..=3 {
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=regions_needed tolerate={f} regions={} candidate_regions=2 enough={}",
                regions_needed(f),
                u8::from(2 >= regions_needed(f)),
            ))
        );
    }

    // ── 阳性对照：R=1，承载它的盘失效 ⇒ 幸存必须为 0 ──
    let single = vec![0u64];
    let (s1, rb1, _, dead1) = survey_worst(SlotOrder::WithinRegion, 1, 4, &single, 0);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_single_region survivors={s1} rollback={} dead_phases={dead1} \
             expect_survivors=0",
            rb1.map(|v| v.to_string()).unwrap_or_else(|| "NA".into()),
        ))
    );
    // ── 阴性对照：设备数 1 ⇒ 全部区域同盘 ⇒ 全丢 ──
    let one_dev: Vec<u64> = vec![0, 0];
    let (s2, _, _, _) = survey_worst(SlotOrder::AcrossRegions, 2, 4, &one_dev, 0);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=negcontrol_single_device survivors={s2} expect_survivors=0"
        ))
    );

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **落盘归属的绝对值**（本文件独立重写的那一份）：chunk 64 KiB、4 盘。
    #[test]
    fn device_of_walks_the_stripe_round_robin() {
        assert_eq!(device_of(0, 65536, 4), 0);
        assert_eq!(device_of(65536, 65536, 4), 1);
        assert_eq!(device_of(65536 * 4, 65536, 4), 0);
        assert_eq!(device_of(65535, 65536, 4), 0);
    }

    /// **两种槽序落的区域不同，且各自的绝对值钉死**（R=2, S=4）。
    /// 区内连续：txg 0..3 → 区域 0，txg 4..7 → 区域 1。
    /// 跨区轮转：txg 偶数 → 区域 0，奇数 → 区域 1。
    #[test]
    fn the_two_slot_orders_place_generations_differently() {
        for t in 0..4 {
            assert_eq!(SlotOrder::WithinRegion.region_of(t, 2, 4), 0);
        }
        for t in 4..8 {
            assert_eq!(SlotOrder::WithinRegion.region_of(t, 2, 4), 1);
        }
        assert_eq!(SlotOrder::WithinRegion.region_of(8, 2, 4), 0); // 绕回
        for t in 0..8 {
            assert_eq!(SlotOrder::AcrossRegions.region_of(t, 2, 4), t % 2);
        }
    }

    /// **判据 1 的绝对值**：R=2 / S=4，两个区域分别在盘 0 与盘 1，丢盘 0
    /// ⇒ 幸存恰好 `R×S − S = 4`，两种槽序都一样（幸存数与槽序无关）。
    #[test]
    fn survivors_equal_ring_minus_the_dead_region() {
        let dev = [0u64, 1];
        for order in [SlotOrder::WithinRegion, SlotOrder::AcrossRegions] {
            let (s, _) = survey(order, 2, 4, &dev, 0, LATEST);
            assert_eq!(s, 4, "{}", order.name());
            let (s1, _) = survey(order, 2, 4, &dev, 1, LATEST);
            assert_eq!(s1, 4, "{}", order.name());
        }
        // R=4 / S=2、四个区域四块盘，丢一块 ⇒ 8 − 2 = 6
        let dev4 = [0u64, 1, 2, 3];
        let (s, _) = survey(SlotOrder::AcrossRegions, 4, 2, &dev4, 2, LATEST);
        assert_eq!(s, 6);
    }

    /// **判据 2：两种槽序的回退代数必须不同，且各自的绝对值由独立算术给出。**
    /// R=2 / S=4，最新一代 latest 落在区域 `region_of(latest)`；丢掉那个区域时——
    /// 区内连续：latest, latest-1, latest-2, latest-3 全在同一区 ⇒ 回退 **4**；
    /// 跨区轮转：latest-1 就在另一个区 ⇒ 回退 **1**。
    #[test]
    fn the_worst_rollback_differs_by_an_order_of_magnitude() {
        let dev = [0u64, 1];
        // latest 选成让「最新那一代落在区域 0」的值，然后丢盘 0
        let latest = 1_000_000u64;
        assert_eq!(SlotOrder::WithinRegion.region_of(latest, 2, 4), 0);
        assert_eq!(SlotOrder::AcrossRegions.region_of(latest, 2, 4), 0);
        let _ = latest;
        let (_, rb_within, best_within, _) = survey_worst(SlotOrder::WithinRegion, 2, 4, &dev, 0);
        let (_, rb_across, best_across, _) = survey_worst(SlotOrder::AcrossRegions, 2, 4, &dev, 0);
        // **最好那个相位两条臂都是 0**（最新一代恰好落在活着的区域，一代都不丢）
        // ⇒ **只取一个相位量不出两条臂的差别，这正是第一版实现的错。**
        assert_eq!(best_within, Some(0));
        assert_eq!(best_across, Some(0));
        assert_eq!(rb_within, Some(4), "区内连续丢的是连续 S 代");
        assert_eq!(rb_across, Some(1), "跨区轮转丢的是每 R 代里的一代");
        assert_ne!(rb_within, rb_across, "两种槽序必须给出不同的回退，否则这一维是死的");
    }

    /// **区内连续的最坏回退恰好等于 S**，三档 S 各钉一次（绝对值，不是互比）。
    #[test]
    fn within_region_worst_rollback_equals_slots() {
        let dev = [0u64, 1];
        for s in [2u64, 4, 8] {
            let (_, rb, _, _) = survey_worst(SlotOrder::WithinRegion, 2, s, &dev, 0);
            assert_eq!(rb, Some(s), "S={s}");
        }
    }

    /// **跨区轮转的最坏回退恒为 R−1 之内**：R 个区域轮转，丢一个 ⇒ 最多回退 1 代
    /// （R=2）或更少……绝对值：R=2 → 1；R=3 → 1；R=4 → 1。
    #[test]
    fn across_regions_worst_rollback_is_one() {
        for r in [2u64, 3, 4] {
            let dev: Vec<u64> = (0..r).collect();
            let (_, rb, _, _) = survey_worst(SlotOrder::AcrossRegions, r, 4, &dev, 0);
            assert_eq!(rb, Some(1), "R={r}");
        }
    }

    /// **判据 4：区内连续 + S > K 时必然出现「挂得上但不可用」。**
    /// S=8、K=4 ⇒ 回退 8 > K−1=3 ⇒ mountable 但 usable=0。
    #[test]
    fn mountable_is_not_usable_when_the_rollback_exceeds_k() {
        let dev = [0u64, 1];
        let (survivors, rb, _, _) = survey_worst(SlotOrder::WithinRegion, 2, 8, &dev, 0);
        assert_eq!(survivors, 8);
        assert_eq!(rb, Some(8));
        assert!(rb.unwrap() > 4 - 1, "K=4 时它不可用");
        assert!(rb.unwrap() > 2 - 1, "K=2 时它同样不可用");
        assert!(rb.unwrap() <= 8 - 1 || rb.unwrap() == 8); // K=8 时恰好卡在边界外
    }

    /// **阳性对照**：R=1，承载它的盘失效 ⇒ 一个都不剩，回退记 None 而不是 0。
    #[test]
    fn positive_control_single_region_loses_everything() {
        let dev = [0u64];
        let (s, rb, _, dead) = survey_worst(SlotOrder::WithinRegion, 1, 4, &dev, 0);
        assert_eq!(s, 0);
        assert_eq!(rb, None, "读不到 ≠ 回退 0");
        assert_eq!(dead, 4, "四个相位全军覆没");
    }

    /// **阴性对照**：设备数 1 ⇒ 全部区域同盘 ⇒ 全丢。
    #[test]
    fn negative_control_single_device_loses_everything() {
        let dev = [0u64, 0, 0, 0];
        for order in [SlotOrder::WithinRegion, SlotOrder::AcrossRegions] {
            let (s, rb, _, dead) = survey_worst(order, 4, 4, &dev, 0);
            assert_eq!(s, 0, "{}", order.name());
            assert_eq!(rb, None);
            assert_eq!(dead, 16, "16 个相位全军覆没");
        }
    }

    /// **区域位置的绝对值**：R=2 时区域 1 落在盘尾（与 E46 的 tail 同型）。
    #[test]
    fn region_positions_span_the_device() {
        assert_eq!(region_position(0, 2, CAPACITY, REGION_BYTES, 65536), 0);
        let tail = region_position(1, 2, CAPACITY, REGION_BYTES, 65536);
        assert_eq!(tail, 9_999_999_959_040);
        // R=3 ⇒ 中间那个落在约一半处
        let mid = region_position(1, 3, CAPACITY, REGION_BYTES, 65536);
        assert_eq!(mid, 4_999_999_979_520);
        assert_eq!(region_position(2, 3, CAPACITY, REGION_BYTES, 65536), tail);
    }

    /// **容忍 F 个失败域要 F+1 个区域**，绝对值。
    #[test]
    fn tolerating_f_domains_needs_f_plus_one_regions() {
        assert_eq!(regions_needed(0), 1);
        assert_eq!(regions_needed(1), 2);
        assert_eq!(regions_needed(2), 3);
        assert!(2 < regions_needed(2));
    }
}
