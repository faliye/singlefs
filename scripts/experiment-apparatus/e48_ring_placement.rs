//! E48：根环几何的可行点 —— 接着 E47（根环失败域的损失）往下走。
//!
//! E47 量出候选几何（2 区 × 4 槽、盘首盘尾）在 6 档设备数里 4 档挂不上，
//! 并指出「要定的不是区域数，是**区域位置的选取规则**加**槽序**」。本实验把那两维做成自变量，
//! **搜一个满足全部判据的点**。
//!
//! ⚠️ **这是新实验不是 E47 的重跑**：问的问题变了（E47 问损失多大，本实验问有没有一个点没有这个损失）。
//!
//! ## 判据（跑前写死）：一个组合叫「可行」，当且仅当
//!
//! a. `device_count >= R` 的格子上，丢任意一块盘之后 mountable 恒为 1；
//! b. 同样那些格子上 K=2 时 usable 恒为 1（最坏回退 ≤ 1）；
//! c. 任意两区间距 >= 65536（最大 io_min 档）；
//! d. `device_count < R` 的格子**不作要求**——鸽笼原理，几何上的不可能，不是规则的失败。
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
//! 物理失败域；**设备集合变化之后归属会变**（`(r×P) mod device_count` 依赖 devs，而区域位置是 mkfs 时的既成事实
//! ⇒ D2 已定项 3 的一个实例，D2 已定项 5 已定不重放置）；P 取 8191 只是「素数且大于任何合理设备数」，不是搜出来的。

use e7_index_bench::Emitter;

/// 条带布局下 LBA 落在哪块盘。与 E46 / E47 同形，本文件独立重写并单测。
fn device_of(logical_block_address: u64, chunk: u64, device_count: u64) -> u64 {
    (logical_block_address / chunk) % device_count
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    /// 区域 r 在 `r × (容量 − 区域字节)/(R−1)`，按 chunk 向下对齐。E47 用的就是这条。
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
    fn position(self, region_index: u64, regions: u64, capacity: u64, region_bytes: u64, chunk: u64) -> u64 {
        match self {
            Placement::EvenSpan => {
                if regions <= 1 || region_index == 0 {
                    0
                } else {
                    let unaligned_offset_bytes = (capacity - region_bytes) * region_index / (regions - 1);
                    (unaligned_offset_bytes / chunk) * chunk
                }
            }
            Placement::PrimeStride => region_index * PRIME * chunk,
            Placement::Adjacent => region_index * region_bytes,
        }
    }
    /// 相邻两区的最小间距（区域按位置升序）。R = 1 时无意义，返回 u64::MAX。
    fn minimum_gap(self, regions: u64, capacity: u64, region_bytes: u64, chunk: u64) -> u64 {
        if regions <= 1 {
            return u64::MAX;
        }
        let mut region_positions: Vec<u64> = (0..regions)
            .map(|region_index| self.position(region_index, regions, capacity, region_bytes, chunk))
            .collect();
        region_positions.sort_unstable();
        region_positions.windows(2)
            .map(|adjacent_positions| adjacent_positions[1].saturating_sub(adjacent_positions[0] + region_bytes))
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

/// 丢一块盘之后：幸存槽数、**对相位取最坏**的回退。与 E47 同一条口径，本文件独立重写。
fn worst_case(
    order: SlotOrder,
    regions: u64,
    slots: u64,
    device_of_region: &[u64],
    failed_device: u64,
) -> (u64, Option<u64>) {
    let ring_slot_count = regions * slots;
    let mut survivors = 0u64;
    let mut worst_rollback: Option<u64> = None;
    for phase in 0..ring_slot_count {
        let latest_txg = ring_slot_count * 1_000_000 + phase;
        let mut alive_slots = 0u64;
        let mut newest_surviving_txg: Option<u64> = None;
        for steps_back in 0..ring_slot_count {
            let candidate_txg = latest_txg - steps_back;
            let region_index = order.region_of(candidate_txg, regions, slots);
            if device_of_region[region_index as usize] == failed_device {
                continue;
            }
            alive_slots += 1;
            if newest_surviving_txg.is_none() {
                newest_surviving_txg = Some(candidate_txg);
            }
        }
        survivors = alive_slots;
        if let Some(newest_txg) = newest_surviving_txg {
            let rollback = latest_txg - newest_txg;
            worst_rollback = Some(worst_rollback.map_or(rollback, |previous_worst: u64| previous_worst.max(rollback)));
        }
    }
    (survivors, worst_rollback)
}

/// 一个组合的可行性判定，**从 `main` 里提出来的纯函数**——
/// 变异测试第一轮把它留在 `main` 里，`M5_可行判据不看可用性` 与 `M7_鸽笼判据反了`
/// **两条都没有任何测试看得见**。提出来之后两条都被钉住。
///
/// 返回 `(device_count>=R 的格子数, 全挂得上的格子数, 全可用的格子数, 归属两两不同的格子数, 最坏回退, 间距够不够, 可行)`。
fn judge(
    placement: Placement,
    order: SlotOrder,
    regions: u64,
    slots: u64,
) -> (u64, u64, u64, u64, u64, bool, bool) {
    let gap_is_sufficient = CHUNKS
        .iter()
        .all(|&candidate_chunk| placement.minimum_gap(regions, CAPACITY, REGION_BYTES, candidate_chunk) >= LARGEST_MINIMUM_INPUT_OUTPUT_SIZE_BYTES);
    let mut cells_with_enough_devices = 0u64;
    let mut mountable_cells = 0u64;
    let mut usable_cells = 0u64;
    let mut worst_rollback = 0u64;
    let mut distinct_cells = 0u64;
    for chunk in CHUNKS {
        for device_count in DEVICE_COUNTS {
            if device_count < regions {
                continue; // 判据 d：鸽笼，不作要求
            }
            let device_of_region: Vec<u64> = (0..regions)
                .map(|region_index| {
                    device_of(placement.position(region_index, regions, CAPACITY, REGION_BYTES, chunk), chunk, device_count)
                })
                .collect();
            let mut distinct_devices = device_of_region.clone();
            distinct_devices.sort_unstable();
            distinct_devices.dedup();
            cells_with_enough_devices += 1;
            if distinct_devices.len() as u64 == regions {
                distinct_cells += 1;
            }
            let mut all_mountable = true;
            let mut all_usable = true;
            for failed_device in 0..device_count {
                let (surviving_slots, rollback) = worst_case(order, regions, slots, &device_of_region, failed_device);
                if surviving_slots == 0 {
                    all_mountable = false;
                    all_usable = false;
                } else {
                    let rollback_depth = rollback.unwrap_or(0);
                    worst_rollback = worst_rollback.max(rollback_depth);
                    if rollback_depth > MINIMUM_RING_DEPTH - 1 {
                        all_usable = false;
                    }
                }
            }
            if all_mountable {
                mountable_cells += 1;
            }
            if all_usable {
                usable_cells += 1;
            }
        }
    }
    let feasible =
        gap_is_sufficient && cells_with_enough_devices > 0 && mountable_cells == cells_with_enough_devices && usable_cells == cells_with_enough_devices;
    (cells_with_enough_devices, mountable_cells, usable_cells, distinct_cells, worst_rollback, gap_is_sufficient, feasible)
}

const CHUNKS: [u64; 3] = [64 * 1024, 512 * 1024, 4 * 1024 * 1024];
const DEVICE_COUNTS: [u64; 6] = [1, 2, 3, 4, 6, 8];
const REGIONS: [u64; 4] = [1, 2, 3, 4];
const SLOTS: [u64; 3] = [2, 4, 8];
const CAPACITY: u64 = 10 * 1000 * 1000 * 1000 * 1000;
const REGION_BYTES: u64 = 1024;
const LARGEST_MINIMUM_INPUT_OUTPUT_SIZE_BYTES: u64 = 65536;
const MINIMUM_RING_DEPTH: u64 = 2; // 判据 b 取根环深度的下限

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config capacity={CAPACITY} region_bytes={REGION_BYTES} prime={PRIME} \
             max_io_min={LARGEST_MINIMUM_INPUT_OUTPUT_SIZE_BYTES} k={MINIMUM_RING_DEPTH}"
        ))
    );

    let placements = [Placement::EvenSpan, Placement::PrimeStride, Placement::Adjacent];
    let orders = [SlotOrder::WithinRegion, SlotOrder::AcrossRegions];

    // ── 量一：逐组合判可行性 ──
    let mut feasible_points = 0u64;
    for placement in placements {
        for &regions in REGIONS.iter() {
            for &slots in SLOTS.iter() {
                for order in orders {
                    let (cells_with_enough_devices, mountable_cells, usable_cells, distinct_cells, worst_rollback, gap_is_sufficient, feasible) =
                        judge(placement, order, regions, slots);
                    if feasible {
                        feasible_points += 1;
                    }
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=point place={} regions={regions} slots={slots} order={} \
                             cells_ge_r={cells_with_enough_devices} mount_ok={mountable_cells} usable_ok={usable_cells} \
                             distinct_cells={distinct_cells} worst_rollback={worst_rollback} \
                             gap_ok={} feasible={}",
                            placement.name(),
                            order.name(),
                            u8::from(gap_is_sufficient),
                            u8::from(feasible),
                        ))
                    );
                }
            }
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!("name=search_summary feasible_points={feasible_points}"))
    );

    // ── 量二：三条放置规则的落盘归属，逐格摊开（证明这一维不是死的）──
    for placement in placements {
        for chunk in CHUNKS {
            for device_count in DEVICE_COUNTS {
                let devices_of_regions: Vec<String> = (0..4u64)
                    .map(|region_index| {
                        device_of(placement.position(region_index, 4, CAPACITY, REGION_BYTES, chunk), chunk, device_count)
                            .to_string()
                    })
                    .collect();
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=mapping place={} chunk={chunk} devs={device_count} r0_3={} \
                         min_gap={}",
                        placement.name(),
                        devices_of_regions.join(","),
                        placement.minimum_gap(4, CAPACITY, REGION_BYTES, chunk),
                    ))
                );
            }
        }
    }

    // ── 阳性对照：devs = 1 ⇒ 任何规则、任何几何都挂不上 ──
    let mut positive_control_holds = true;
    for placement in placements {
        for &regions in REGIONS.iter() {
            let device_of_region: Vec<u64> = vec![0; regions as usize];
            let (surviving_slots, _) = worst_case(SlotOrder::AcrossRegions, regions, 4, &device_of_region, 0);
            if surviving_slots != 0 {
                positive_control_holds = false;
            }
            let _ = placement;
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_single_device all_zero={} expect=1",
            u8::from(positive_control_holds)
        ))
    );
    // ── 阴性对照：adjacent 的间距必须小于 io_min ──
    let adjacent_gap_bytes = Placement::Adjacent.minimum_gap(4, CAPACITY, REGION_BYTES, 65536);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=negcontrol_adjacent_gap gap={adjacent_gap_bytes} below_io_min={} expect=1",
            u8::from(adjacent_gap_bytes < LARGEST_MINIMUM_INPUT_OUTPUT_SIZE_BYTES)
        ))
    );

    println!("{}", emitter.finish());
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

    /// **`prime_stride` 的机制**：归属恰好是 `(r × P) mod device_count`，逐个钉死。
    /// P = 8191 是素数 ⇒ 对 devs ≤ 8 各给一组绝对值。
    #[test]
    fn prime_stride_maps_region_index_to_region_index_times_prime_modulo_devices() {
        let chunk = 65536u64;
        for device_count in [2u64, 3, 4, 6, 8] {
            for region_index in 0..4u64 {
                let region_position = Placement::PrimeStride.position(region_index, 4, CAPACITY, REGION_BYTES, chunk);
                assert_eq!(region_position, region_index * PRIME * chunk);
                assert_eq!(device_of(region_position, chunk, device_count), (region_index * PRIME) % device_count, "devs={device_count} r={region_index}");
            }
        }
        // 绝对值：devs=4 时四个区域分别落在 0 / 3 / 2 / 1
        let devices_on_four_disks: Vec<u64> = (0..4)
            .map(|region_index| device_of(Placement::PrimeStride.position(region_index, 4, CAPACITY, REGION_BYTES, chunk), chunk, 4))
            .collect();
        assert_eq!(devices_on_four_disks, vec![0, 3, 2, 1]);
        // devs=8 时是 0 / 7 / 6 / 5
        let devices_on_eight_disks: Vec<u64> = (0..4)
            .map(|region_index| device_of(Placement::PrimeStride.position(region_index, 4, CAPACITY, REGION_BYTES, chunk), chunk, 8))
            .collect();
        assert_eq!(devices_on_eight_disks, vec![0, 7, 6, 5]);
    }

    /// **P 是素数 ⇒ 乘 P 是模 devs 的双射 ⇒ R ≤ devs 时归属两两不同**，三档 chunk 都要成立。
    #[test]
    fn prime_stride_gives_distinct_devices_whenever_regions_fit() {
        for chunk in CHUNKS {
            for device_count in DEVICE_COUNTS {
                for regions in REGIONS {
                    if device_count < regions {
                        continue;
                    }
                    let mut distinct_devices: Vec<u64> = (0..regions)
                        .map(|region_index| {
                            device_of(
                                Placement::PrimeStride.position(region_index, regions, CAPACITY, REGION_BYTES, chunk),
                                chunk,
                                device_count,
                            )
                        })
                        .collect();
                    distinct_devices.sort_unstable();
                    distinct_devices.dedup();
                    assert_eq!(distinct_devices.len() as u64, regions, "chunk={chunk} devs={device_count} R={regions}");
                }
            }
        }
    }

    /// **`even_span` 做不到这一点**——E47 已经量过，这里钉一个绝对值：
    /// 2 区、chunk 64 KiB、2 / 3 / 6 盘时两区同盘。
    #[test]
    fn even_span_collides_on_some_device_counts() {
        let chunk = 65536u64;
        for device_count in [2u64, 3, 6] {
            let region_0_device = device_of(Placement::EvenSpan.position(0, 2, CAPACITY, REGION_BYTES, chunk), chunk, device_count);
            let region_1_device = device_of(Placement::EvenSpan.position(1, 2, CAPACITY, REGION_BYTES, chunk), chunk, device_count);
            assert_eq!(region_0_device, region_1_device, "devs={device_count} 该同盘");
        }
        // 4 盘时不同盘
        let region_0_device = device_of(Placement::EvenSpan.position(0, 2, CAPACITY, REGION_BYTES, chunk), chunk, 4);
        let region_1_device = device_of(Placement::EvenSpan.position(1, 2, CAPACITY, REGION_BYTES, chunk), chunk, 4);
        assert_ne!(region_0_device, region_1_device);
    }

    /// **间距的绝对值**：`prime_stride` 在 64 KiB chunk 上是 `8191 × 65536 − 1024`。
    #[test]
    fn prime_stride_gap_is_far_above_any_minimum_input_output_size() {
        let prime_stride_gap_bytes = Placement::PrimeStride.minimum_gap(4, CAPACITY, REGION_BYTES, 65536);
        assert_eq!(prime_stride_gap_bytes, 8191 * 65536 - 1024);
        // 独立算术：8191 × 65536 = 536 805 376，减去区域自身 1024 字节
        assert_eq!(prime_stride_gap_bytes, 536_804_352);
        assert!(prime_stride_gap_bytes >= LARGEST_MINIMUM_INPUT_OUTPUT_SIZE_BYTES);
        for chunk in CHUNKS {
            assert!(Placement::PrimeStride.minimum_gap(4, CAPACITY, REGION_BYTES, chunk) >= LARGEST_MINIMUM_INPUT_OUTPUT_SIZE_BYTES);
        }
    }

    /// **阴性对照的绝对值**：`adjacent` 的间距恰好是 0，必须小于任何 io_min。
    #[test]
    fn negative_control_adjacent_has_zero_gap() {
        assert_eq!(Placement::Adjacent.minimum_gap(4, CAPACITY, REGION_BYTES, 65536), 0);
        assert!(Placement::Adjacent.minimum_gap(4, CAPACITY, REGION_BYTES, 65536) < LARGEST_MINIMUM_INPUT_OUTPUT_SIZE_BYTES);
    }

    /// **三条规则的归属确实不同**（这一维不是死的）：4 区、chunk 64 KiB、4 盘。
    #[test]
    fn the_three_placements_are_not_the_same_thing() {
        let chunk = 65536u64;
        let devices_for_placement = |placement: Placement| -> Vec<u64> {
            (0..4)
                .map(|region_index| device_of(placement.position(region_index, 4, CAPACITY, REGION_BYTES, chunk), chunk, 4))
                .collect()
        };
        assert_eq!(devices_for_placement(Placement::PrimeStride), vec![0, 3, 2, 1]);
        assert_eq!(devices_for_placement(Placement::Adjacent), vec![0, 0, 0, 0]); // 全挤在一个 chunk 里
        assert_ne!(devices_for_placement(Placement::EvenSpan), devices_for_placement(Placement::PrimeStride));
    }

    /// **跨区轮转 + 四区四盘，丢一块 ⇒ 回退恰好 1、幸存恰好 R×S − S。**
    #[test]
    fn across_regions_on_distinct_devices_loses_exactly_one_generation() {
        let device_of_region = [0u64, 1, 2, 3];
        let (surviving_slots, rollback) = worst_case(SlotOrder::AcrossRegions, 4, 4, &device_of_region, 2);
        assert_eq!(surviving_slots, 12);
        assert_eq!(rollback, Some(1));
    }

    /// **区内连续在同样的几何上最坏回退 = S**，与跨区轮转不同（沿用 E47 的判据 2）。
    #[test]
    fn within_region_still_loses_a_whole_run() {
        let device_of_region = [0u64, 1, 2, 3];
        for slots_per_region in [2u64, 4, 8] {
            let (_, rollback) = worst_case(SlotOrder::WithinRegion, 4, slots_per_region, &device_of_region, 0);
            assert_eq!(rollback, Some(slots_per_region), "S={slots_per_region}");
        }
    }

    /// **可行性判定的绝对值，钉住「可用性」那一项**（变异 M5 打的就是它）：
    /// 素数步长 + 跨区轮转 ⇒ 可行；素数步长 + 区内连续 ⇒ **挂得上但不可用** ⇒ 不可行。
    #[test]
    fn feasibility_requires_usable_not_just_mountable() {
        let (cells_with_enough_devices, mountable_cells, usable_cells, _, rollback, gap_is_sufficient, feasible) =
            judge(Placement::PrimeStride, SlotOrder::AcrossRegions, 2, 4);
        assert_eq!((cells_with_enough_devices, mountable_cells, usable_cells), (15, 15, 15));
        assert_eq!(rollback, 1);
        assert!(gap_is_sufficient && feasible);
        // 同样的放置规则，换成区内连续：**全部格子仍然挂得上，但一个都不可用**
        let (within_region_cells, within_region_mountable_cells, within_region_usable_cells, _, within_region_rollback, _, within_region_feasible) =
            judge(Placement::PrimeStride, SlotOrder::WithinRegion, 2, 4);
        assert_eq!((within_region_cells, within_region_mountable_cells), (15, 15), "挂得上");
        assert_eq!(within_region_usable_cells, 0, "K=2 时一个都不可用");
        assert_eq!(within_region_rollback, 4, "回退 = S");
        assert!(!within_region_feasible, "只看 mountable 会把它判成可行——那正是要拦的");
    }

    /// **鸽笼那条判据的绝对值**（变异 M7 打的就是它）：`device_count >= R` 的格子数，
    /// 独立算术 = 3 档 chunk × {DEVS 里 ≥ R 的个数}：R=2 → 3×5=15、R=3 → 3×4=12、R=4 → 3×3=9。
    #[test]
    fn only_cells_with_enough_devices_are_judged() {
        for (regions, expected_cells) in [(1u64, 18u64), (2, 15), (3, 12), (4, 9)] {
            let (cells_with_enough_devices, _, _, _, _, _, _) =
                judge(Placement::PrimeStride, SlotOrder::AcrossRegions, regions, 4);
            assert_eq!(cells_with_enough_devices, expected_cells, "R={regions}");
        }
    }

    /// **阳性对照**：全部区域同盘 ⇒ 幸存 0。
    #[test]
    fn positive_control_all_regions_on_one_device() {
        for regions in [1u64, 2, 3, 4] {
            let device_of_region = vec![0u64; regions as usize];
            let (surviving_slots, rollback) = worst_case(SlotOrder::AcrossRegions, regions, 4, &device_of_region, 0);
            assert_eq!(surviving_slots, 0);
            assert_eq!(rollback, None);
        }
    }
}
