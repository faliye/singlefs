//! E49：根环几何的可行点 —— 接着 E48（根环失败域的损失）往下走。
//!
//! E48 量出候选几何（2 区 × 4 槽、盘首盘尾）在 6 档设备数里 4 档挂不上，
//! 并指出「要定的不是区域数，是**区域位置的选取规则**加**槽序**」。本实验把那两维做成自变量，
//! **搜一个满足全部判据的点**。
//!
//! ⚠️ **这是新实验不是 E48 的重跑**：问的问题变了（E48 问损失多大，本实验问有没有一个点没有这个损失）。
//!
//! ## 判据（跑前写死）：一个组合叫「可行」，当且仅当
//!
//! a. `devs >= R` 的格子上，丢任意一块盘之后 mountable 恒为 1；
//! b. 同样那些格子上 K=2 时 usable 恒为 1（最坏回退 ≤ 1）；
//! c. 任意两区间距 >= 65536（最大 io_min 档）；
//! d. `devs < R` 的格子**不作要求**——鸽笼原理，几何上的不可能，不是规则的失败。
//!
//! ## 失败条款
//!
//! - 阳性对照：devs=1 时任何规则 mountable 必须为 0。非 0 ⇒ 整轮作废。
//! - 阴性对照：`adjacent` 规则的间距必须 < io_min ⇒ 条件 c 判否。判是 ⇒ 整轮作废。
//! - 三条放置规则给出相同的落盘归属 ⇒ 这一维是死的，整轮作废。
//! - `prime_stride` 在某个 devs >= R 上仍同盘 ⇒ **如实记录，不许换 P 再跑**。
//!
//! ## 它答不了的
//!
//! 物理失败域；**设备集合变化之后归属会变**（`(r×P) mod devs` 依赖 devs，而区域位置是 mkfs 时的既成事实
//! ⇒ D2 未定项 3 的一个实例）；P 取 8191 只是「素数且大于任何合理设备数」，不是搜出来的。

use e7_index_bench::Emitter;

/// 条带布局下 LBA 落在哪块盘。与 E47 / E48 同形，本文件独立重写并单测。
fn device_of(lba: u64, chunk: u64, devs: u64) -> u64 {
    (lba / chunk) % devs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    /// 区域 r 在 `r × (容量 − 区域字节)/(R−1)`，按 chunk 向下对齐。E48 用的就是这条。
    EvenSpan,
    /// 区域 r 在 `r × P × chunk`，P = 8191（素数）。
    PrimeStride,
    /// 区域 r 在 `r × 区域字节`（紧挨着）。阴性对照。
    Adjacent,
}

const PRIME: u64 = 8191;

impl Placement {
    fn name(self) -> &'static str {
        match self {
            Placement::EvenSpan => "even_span",
            Placement::PrimeStride => "prime_stride",
            Placement::Adjacent => "adjacent",
        }
    }
    fn position(self, r: u64, regions: u64, capacity: u64, region_bytes: u64, chunk: u64) -> u64 {
        match self {
            Placement::EvenSpan => {
                if regions <= 1 || r == 0 {
                    0
                } else {
                    let raw = (capacity - region_bytes) * r / (regions - 1);
                    (raw / chunk) * chunk
                }
            }
            Placement::PrimeStride => r * PRIME * chunk,
            Placement::Adjacent => r * region_bytes,
        }
    }
    /// 相邻两区的最小间距（区域按位置升序）。R = 1 时无意义，返回 u64::MAX。
    fn min_gap(self, regions: u64, capacity: u64, region_bytes: u64, chunk: u64) -> u64 {
        if regions <= 1 {
            return u64::MAX;
        }
        let mut pos: Vec<u64> = (0..regions)
            .map(|r| self.position(r, regions, capacity, region_bytes, chunk))
            .collect();
        pos.sort_unstable();
        pos.windows(2)
            .map(|w| w[1].saturating_sub(w[0] + region_bytes))
            .min()
            .unwrap_or(u64::MAX)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotOrder {
    WithinRegion,
    AcrossRegions,
}

impl SlotOrder {
    fn name(self) -> &'static str {
        match self {
            SlotOrder::WithinRegion => "within_region",
            SlotOrder::AcrossRegions => "across_regions",
        }
    }
    fn region_of(self, txg: u64, regions: u64, slots: u64) -> u64 {
        match self {
            SlotOrder::WithinRegion => (txg % (regions * slots)) / slots,
            SlotOrder::AcrossRegions => txg % regions,
        }
    }
}

/// 丢一块盘之后：幸存槽数、**对相位取最坏**的回退。与 E48 同一条口径，本文件独立重写。
fn worst_case(
    order: SlotOrder,
    regions: u64,
    slots: u64,
    dev_of_region: &[u64],
    failed_dev: u64,
) -> (u64, Option<u64>) {
    let ring = regions * slots;
    let mut survivors = 0u64;
    let mut worst: Option<u64> = None;
    for phase in 0..ring {
        let latest = ring * 1_000_000 + phase;
        let mut alive = 0u64;
        let mut newest: Option<u64> = None;
        for back in 0..ring {
            let t = latest - back;
            let r = order.region_of(t, regions, slots);
            if dev_of_region[r as usize] == failed_dev {
                continue;
            }
            alive += 1;
            if newest.is_none() {
                newest = Some(t);
            }
        }
        survivors = alive;
        if let Some(t) = newest {
            let rb = latest - t;
            worst = Some(worst.map_or(rb, |w: u64| w.max(rb)));
        }
    }
    (survivors, worst)
}

/// 一个组合的可行性判定，**从 `main` 里提出来的纯函数**——
/// 变异测试第一轮把它留在 `main` 里，`M5_可行判据不看可用性` 与 `M7_鸽笼判据反了`
/// **两条都没有任何测试看得见**。提出来之后两条都被钉住。
///
/// 返回 `(devs>=R 的格子数, 全挂得上的格子数, 全可用的格子数, 归属两两不同的格子数, 最坏回退, 间距够不够, 可行)`。
fn judge(
    place: Placement,
    order: SlotOrder,
    regions: u64,
    slots: u64,
) -> (u64, u64, u64, u64, u64, bool, bool) {
    let gap_ok = CHUNKS
        .iter()
        .all(|&c| place.min_gap(regions, CAPACITY, REGION_BYTES, c) >= MAX_IO_MIN);
    let mut cells_ge_r = 0u64;
    let mut mount_ok = 0u64;
    let mut usable_ok = 0u64;
    let mut worst_rb = 0u64;
    let mut distinct_cells = 0u64;
    for chunk in CHUNKS {
        for devs in DEVS {
            if devs < regions {
                continue; // 判据 d：鸽笼，不作要求
            }
            let dev_of_region: Vec<u64> = (0..regions)
                .map(|r| {
                    device_of(place.position(r, regions, CAPACITY, REGION_BYTES, chunk), chunk, devs)
                })
                .collect();
            let mut uniq = dev_of_region.clone();
            uniq.sort_unstable();
            uniq.dedup();
            cells_ge_r += 1;
            if uniq.len() as u64 == regions {
                distinct_cells += 1;
            }
            let mut all_mount = true;
            let mut all_usable = true;
            for failed in 0..devs {
                let (surv, rb) = worst_case(order, regions, slots, &dev_of_region, failed);
                if surv == 0 {
                    all_mount = false;
                    all_usable = false;
                } else {
                    let r = rb.unwrap_or(0);
                    worst_rb = worst_rb.max(r);
                    if r > K - 1 {
                        all_usable = false;
                    }
                }
            }
            if all_mount {
                mount_ok += 1;
            }
            if all_usable {
                usable_ok += 1;
            }
        }
    }
    let feasible =
        gap_ok && cells_ge_r > 0 && mount_ok == cells_ge_r && usable_ok == cells_ge_r;
    (cells_ge_r, mount_ok, usable_ok, distinct_cells, worst_rb, gap_ok, feasible)
}

const CHUNKS: [u64; 3] = [64 * 1024, 512 * 1024, 4 * 1024 * 1024];
const DEVS: [u64; 6] = [1, 2, 3, 4, 6, 8];
const REGIONS: [u64; 4] = [1, 2, 3, 4];
const SLOTS: [u64; 3] = [2, 4, 8];
const CAPACITY: u64 = 10 * 1000 * 1000 * 1000 * 1000;
const REGION_BYTES: u64 = 1024;
const MAX_IO_MIN: u64 = 65536;
const K: u64 = 2; // 判据 b 取根环深度的下限

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config capacity={CAPACITY} region_bytes={REGION_BYTES} prime={PRIME} \
             max_io_min={MAX_IO_MIN} k={K}"
        ))
    );

    let placements = [Placement::EvenSpan, Placement::PrimeStride, Placement::Adjacent];
    let orders = [SlotOrder::WithinRegion, SlotOrder::AcrossRegions];

    // ── 量一：逐组合判可行性 ──
    let mut feasible_points = 0u64;
    for place in placements {
        for &regions in REGIONS.iter() {
            for &slots in SLOTS.iter() {
                for order in orders {
                    let (cells_ge_r, mount_ok, usable_ok, distinct_cells, worst_rb, gap_ok, feasible) =
                        judge(place, order, regions, slots);
                    if feasible {
                        feasible_points += 1;
                    }
                    println!(
                        "{}",
                        em.emit_raw(&format!(
                            "name=point place={} regions={regions} slots={slots} order={} \
                             cells_ge_r={cells_ge_r} mount_ok={mount_ok} usable_ok={usable_ok} \
                             distinct_cells={distinct_cells} worst_rollback={worst_rb} \
                             gap_ok={} feasible={}",
                            place.name(),
                            order.name(),
                            u8::from(gap_ok),
                            u8::from(feasible),
                        ))
                    );
                }
            }
        }
    }
    println!(
        "{}",
        em.emit_raw(&format!("name=search_summary feasible_points={feasible_points}"))
    );

    // ── 量二：三条放置规则的落盘归属，逐格摊开（证明这一维不是死的）──
    for place in placements {
        for chunk in CHUNKS {
            for devs in DEVS {
                let devs_of: Vec<String> = (0..4u64)
                    .map(|r| {
                        device_of(place.position(r, 4, CAPACITY, REGION_BYTES, chunk), chunk, devs)
                            .to_string()
                    })
                    .collect();
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=mapping place={} chunk={chunk} devs={devs} r0_3={} \
                         min_gap={}",
                        place.name(),
                        devs_of.join(","),
                        place.min_gap(4, CAPACITY, REGION_BYTES, chunk),
                    ))
                );
            }
        }
    }

    // ── 阳性对照：devs = 1 ⇒ 任何规则、任何几何都挂不上 ──
    let mut pos_ok = true;
    for place in placements {
        for &regions in REGIONS.iter() {
            let dev_of_region: Vec<u64> = vec![0; regions as usize];
            let (surv, _) = worst_case(SlotOrder::AcrossRegions, regions, 4, &dev_of_region, 0);
            if surv != 0 {
                pos_ok = false;
            }
            let _ = place;
        }
    }
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_single_device all_zero={} expect=1",
            u8::from(pos_ok)
        ))
    );
    // ── 阴性对照：adjacent 的间距必须小于 io_min ──
    let adj_gap = Placement::Adjacent.min_gap(4, CAPACITY, REGION_BYTES, 65536);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=negcontrol_adjacent_gap gap={adj_gap} below_io_min={} expect=1",
            u8::from(adj_gap < MAX_IO_MIN)
        ))
    );

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **落盘归属的绝对值**（本文件独立重写的那一份）。
    #[test]
    fn device_of_walks_the_stripe_round_robin() {
        assert_eq!(device_of(0, 65536, 4), 0);
        assert_eq!(device_of(65536 * 3, 65536, 4), 3);
        assert_eq!(device_of(65536 * 4, 65536, 4), 0);
    }

    /// **`prime_stride` 的机制**：归属恰好是 `(r × P) mod devs`，逐个钉死。
    /// P = 8191 是素数 ⇒ 对 devs ≤ 8 各给一组绝对值。
    #[test]
    fn prime_stride_maps_region_r_to_device_r_times_p() {
        let chunk = 65536u64;
        for devs in [2u64, 3, 4, 6, 8] {
            for r in 0..4u64 {
                let pos = Placement::PrimeStride.position(r, 4, CAPACITY, REGION_BYTES, chunk);
                assert_eq!(pos, r * PRIME * chunk);
                assert_eq!(device_of(pos, chunk, devs), (r * PRIME) % devs, "devs={devs} r={r}");
            }
        }
        // 绝对值：devs=4 时四个区域分别落在 0 / 3 / 2 / 1
        let d: Vec<u64> = (0..4)
            .map(|r| device_of(Placement::PrimeStride.position(r, 4, CAPACITY, REGION_BYTES, chunk), chunk, 4))
            .collect();
        assert_eq!(d, vec![0, 3, 2, 1]);
        // devs=8 时是 0 / 7 / 6 / 5
        let d8: Vec<u64> = (0..4)
            .map(|r| device_of(Placement::PrimeStride.position(r, 4, CAPACITY, REGION_BYTES, chunk), chunk, 8))
            .collect();
        assert_eq!(d8, vec![0, 7, 6, 5]);
    }

    /// **P 是素数 ⇒ 乘 P 是模 devs 的双射 ⇒ R ≤ devs 时归属两两不同**，三档 chunk 都要成立。
    #[test]
    fn prime_stride_gives_distinct_devices_whenever_regions_fit() {
        for chunk in CHUNKS {
            for devs in DEVS {
                for regions in REGIONS {
                    if devs < regions {
                        continue;
                    }
                    let mut v: Vec<u64> = (0..regions)
                        .map(|r| {
                            device_of(
                                Placement::PrimeStride.position(r, regions, CAPACITY, REGION_BYTES, chunk),
                                chunk,
                                devs,
                            )
                        })
                        .collect();
                    v.sort_unstable();
                    v.dedup();
                    assert_eq!(v.len() as u64, regions, "chunk={chunk} devs={devs} R={regions}");
                }
            }
        }
    }

    /// **`even_span` 做不到这一点**——E48 已经量过，这里钉一个绝对值：
    /// 2 区、chunk 64 KiB、2 / 3 / 6 盘时两区同盘。
    #[test]
    fn even_span_collides_on_some_device_counts() {
        let chunk = 65536u64;
        for devs in [2u64, 3, 6] {
            let a = device_of(Placement::EvenSpan.position(0, 2, CAPACITY, REGION_BYTES, chunk), chunk, devs);
            let b = device_of(Placement::EvenSpan.position(1, 2, CAPACITY, REGION_BYTES, chunk), chunk, devs);
            assert_eq!(a, b, "devs={devs} 该同盘");
        }
        // 4 盘时不同盘
        let a = device_of(Placement::EvenSpan.position(0, 2, CAPACITY, REGION_BYTES, chunk), chunk, 4);
        let b = device_of(Placement::EvenSpan.position(1, 2, CAPACITY, REGION_BYTES, chunk), chunk, 4);
        assert_ne!(a, b);
    }

    /// **间距的绝对值**：`prime_stride` 在 64 KiB chunk 上是 `8191 × 65536 − 1024`。
    #[test]
    fn prime_stride_gap_is_far_above_any_io_min() {
        let g = Placement::PrimeStride.min_gap(4, CAPACITY, REGION_BYTES, 65536);
        assert_eq!(g, 8191 * 65536 - 1024);
        // 独立算术：8191 × 65536 = 536 805 376，减去区域自身 1024 字节
        assert_eq!(g, 536_804_352);
        assert!(g >= MAX_IO_MIN);
        for chunk in CHUNKS {
            assert!(Placement::PrimeStride.min_gap(4, CAPACITY, REGION_BYTES, chunk) >= MAX_IO_MIN);
        }
    }

    /// **阴性对照的绝对值**：`adjacent` 的间距恰好是 0，必须小于任何 io_min。
    #[test]
    fn negative_control_adjacent_has_zero_gap() {
        assert_eq!(Placement::Adjacent.min_gap(4, CAPACITY, REGION_BYTES, 65536), 0);
        assert!(Placement::Adjacent.min_gap(4, CAPACITY, REGION_BYTES, 65536) < MAX_IO_MIN);
    }

    /// **三条规则的归属确实不同**（这一维不是死的）：4 区、chunk 64 KiB、4 盘。
    #[test]
    fn the_three_placements_are_not_the_same_thing() {
        let chunk = 65536u64;
        let map = |p: Placement| -> Vec<u64> {
            (0..4)
                .map(|r| device_of(p.position(r, 4, CAPACITY, REGION_BYTES, chunk), chunk, 4))
                .collect()
        };
        assert_eq!(map(Placement::PrimeStride), vec![0, 3, 2, 1]);
        assert_eq!(map(Placement::Adjacent), vec![0, 0, 0, 0]); // 全挤在一个 chunk 里
        assert_ne!(map(Placement::EvenSpan), map(Placement::PrimeStride));
    }

    /// **跨区轮转 + 四区四盘，丢一块 ⇒ 回退恰好 1、幸存恰好 R×S − S。**
    #[test]
    fn across_regions_on_distinct_devices_loses_exactly_one_generation() {
        let dev = [0u64, 1, 2, 3];
        let (surv, rb) = worst_case(SlotOrder::AcrossRegions, 4, 4, &dev, 2);
        assert_eq!(surv, 12);
        assert_eq!(rb, Some(1));
    }

    /// **区内连续在同样的几何上最坏回退 = S**，与跨区轮转不同（沿用 E48 的判据 2）。
    #[test]
    fn within_region_still_loses_a_whole_run() {
        let dev = [0u64, 1, 2, 3];
        for s in [2u64, 4, 8] {
            let (_, rb) = worst_case(SlotOrder::WithinRegion, 4, s, &dev, 0);
            assert_eq!(rb, Some(s), "S={s}");
        }
    }

    /// **可行性判定的绝对值，钉住「可用性」那一项**（变异 M5 打的就是它）：
    /// 素数步长 + 跨区轮转 ⇒ 可行；素数步长 + 区内连续 ⇒ **挂得上但不可用** ⇒ 不可行。
    #[test]
    fn feasibility_requires_usable_not_just_mountable() {
        let (cells, mount, usable, _, rb, gap, feas) =
            judge(Placement::PrimeStride, SlotOrder::AcrossRegions, 2, 4);
        assert_eq!((cells, mount, usable), (15, 15, 15));
        assert_eq!(rb, 1);
        assert!(gap && feas);
        // 同样的放置规则，换成区内连续：**全部格子仍然挂得上，但一个都不可用**
        let (cells2, mount2, usable2, _, rb2, _, feas2) =
            judge(Placement::PrimeStride, SlotOrder::WithinRegion, 2, 4);
        assert_eq!((cells2, mount2), (15, 15), "挂得上");
        assert_eq!(usable2, 0, "K=2 时一个都不可用");
        assert_eq!(rb2, 4, "回退 = S");
        assert!(!feas2, "只看 mountable 会把它判成可行——那正是要拦的");
    }

    /// **鸽笼那条判据的绝对值**（变异 M7 打的就是它）：`devs >= R` 的格子数，
    /// 独立算术 = 3 档 chunk × {DEVS 里 ≥ R 的个数}：R=2 → 3×5=15、R=3 → 3×4=12、R=4 → 3×3=9。
    #[test]
    fn only_cells_with_enough_devices_are_judged() {
        for (regions, want) in [(1u64, 18u64), (2, 15), (3, 12), (4, 9)] {
            let (cells, _, _, _, _, _, _) =
                judge(Placement::PrimeStride, SlotOrder::AcrossRegions, regions, 4);
            assert_eq!(cells, want, "R={regions}");
        }
    }

    /// **阳性对照**：全部区域同盘 ⇒ 幸存 0。
    #[test]
    fn positive_control_all_regions_on_one_device() {
        for regions in [1u64, 2, 3, 4] {
            let dev = vec![0u64; regions as usize];
            let (surv, rb) = worst_case(SlotOrder::AcrossRegions, regions, 4, &dev, 0);
            assert_eq!(surv, 0);
            assert_eq!(rb, None);
        }
    }
}
