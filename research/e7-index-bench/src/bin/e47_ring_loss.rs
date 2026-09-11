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
//! ① **区内连续**：`slot = transaction_group mod (R×S)`，区域 = `slot / S` ⇒ 连续 S 次发布落在同一个区域。
//! ② **跨区轮转**：区域 = `transaction_group mod R`，区内槽 = `(transaction_group / R) mod S` ⇒ 相邻两次发布落在不同区域。
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
fn device_of(logical_block_address: u64, chunk: u64, device_count: u64) -> u64 {
    (logical_block_address / chunk) % device_count
}

/// 槽序。两种都合法，D22 已定项 2 从没点名过。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotOrder {
    /// `slot = transaction_group mod (R×S)`，区域 = `slot / S`。
    WithinRegion,
    /// 区域 = `transaction_group mod R`，区内槽 = `(transaction_group / R) mod S`。
    AcrossRegions,
}

impl SlotOrder {
    fn name(self) -> &'static str {
        match self {
            SlotOrder::WithinRegion => "within_region",
            SlotOrder::AcrossRegions => "across_regions",
        }
    }
    /// 第 `transaction_group` 次发布落在哪个区域。
    fn region_of(self, transaction_group: u64, regions: u64, slots: u64) -> u64 {
        match self {
            SlotOrder::WithinRegion => (transaction_group % (regions * slots)) / slots,
            SlotOrder::AcrossRegions => transaction_group % regions,
        }
    }
}

/// 区域 `region_index` 的起始 LBA：区域 0 在盘首，其余按 `(容量 − 区域字节) × region_index / (R − 1)` 均分到盘尾，
/// 并按 chunk 向下对齐。R = 1 时只有盘首那一个。
fn region_position(region_index: u64, regions: u64, capacity: u64, region_bytes: u64, chunk: u64) -> u64 {
    if regions <= 1 || region_index == 0 {
        return 0;
    }
    let span = capacity - region_bytes;
    let unaligned_offset_bytes = span * region_index / (regions - 1);
    (unaligned_offset_bytes / chunk) * chunk
}

/// 一次失效之后，最近 `ring_slot_count` 代里还有多少代是可读的，以及最新可读的那一代落后多少。
/// 返回 `(幸存槽数, Option<回退代数>)`；`None` 表示一个都不剩。
///
/// 口径：假定环已经写满至少一圈，最新一代记为 `latest`；
/// 第 `generation` 代住在 `order.region_of(generation, R, S)` 这个区域，该区域落在 `device_of_region[region_index]` 这块盘上。
fn survey(
    order: SlotOrder,
    regions: u64,
    slots: u64,
    device_of_region: &[u64],
    failed_device: u64,
    latest: u64,
) -> (u64, Option<u64>) {
    let ring_slot_count = regions * slots;
    let mut survivors = 0u64;
    let mut newest_alive: Option<u64> = None;
    // 环里活着的是最近 ring 代：latest, latest-1, ..., latest-ring+1
    for steps_back in 0..ring_slot_count {
        let generation = latest - steps_back;
        let region_index = order.region_of(generation, regions, slots);
        if device_of_region[region_index as usize] == failed_device {
            continue; // 这一代随区域一起没了
        }
        survivors += 1;
        if newest_alive.is_none() {
            newest_alive = Some(generation);
        }
    }
    (survivors, newest_alive.map(|newest_alive_generation| latest - newest_alive_generation))
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
    device_of_region: &[u64],
    failed_device: u64,
) -> (u64, Option<u64>, Option<u64>, u64) {
    let ring_slot_count = regions * slots;
    let mut survivors = 0u64;
    let mut worst: Option<u64> = None;
    let mut best: Option<u64> = None;
    let mut dead_phases = 0u64;
    for phase in 0..ring_slot_count {
        let latest = ring_slot_count * 1_000_000 + phase; // 环已写满多圈，相位取遍一圈
        let (phase_survivors, phase_rollback) = survey(order, regions, slots, device_of_region, failed_device, latest);
        survivors = phase_survivors; // 与相位无关，逐相位相同
        match phase_rollback {
            None => dead_phases += 1,
            Some(rollback_generations) => {
                worst = Some(worst.map_or(rollback_generations, |worst_so_far: u64| worst_so_far.max(rollback_generations)));
                best = Some(best.map_or(rollback_generations, |best_so_far: u64| best_so_far.min(rollback_generations)));
            }
        }
    }
    (survivors, worst, best, dead_phases)
}

/// 容忍 `failure_domains` 个失败域要几个区域。与 E46 同一条算术，本文件独立重写。
fn regions_needed(failure_domains: u64) -> u64 {
    failure_domains + 1
}

const CHUNKS: [u64; 3] = [64 * 1024, 512 * 1024, 4 * 1024 * 1024];
const DEVICE_COUNTS: [u64; 6] = [1, 2, 3, 4, 6, 8];
const REGIONS: [u64; 4] = [1, 2, 3, 4];
const SLOTS: [u64; 3] = [2, 4, 8];
const ROOT_RING_DEPTHS: [u64; 3] = [2, 4, 8];
const CAPACITY: u64 = 10 * 1000 * 1000 * 1000 * 1000; // 10 TB，与 E40 / E46 同口径
const REGION_BYTES: u64 = 1024; // E41 候选：每区 4 槽 × 256 字节。槽宽已被 E41 判红，这里只用它定位
const LATEST: u64 = 1_000_000; // 已经跑了很久，环写满过多圈

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
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
                for device_count in DEVICE_COUNTS {
                    let device_of_region: Vec<u64> = (0..regions)
                        .map(|region_index| {
                            device_of(
                                region_position(region_index, regions, CAPACITY, REGION_BYTES, chunk),
                                chunk,
                                device_count,
                            )
                        })
                        .collect();
                    let distinct = {
                        let mut distinct_devices = device_of_region.clone();
                        distinct_devices.sort_unstable();
                        distinct_devices.dedup();
                        distinct_devices.len() as u64
                    };
                    for failed_device in 0..device_count {
                        for order in [SlotOrder::WithinRegion, SlotOrder::AcrossRegions] {
                            let (survivors, rollback, best, dead_phases) =
                                survey_worst(order, regions, slots, &device_of_region, failed_device);
                            total += 1;
                            let mountable = survivors > 0;
                            if !mountable {
                                unmountable += 1;
                            }
                            if let Some(rollback_generations) = rollback {
                                match order {
                                    SlotOrder::WithinRegion => worst_within = worst_within.max(rollback_generations),
                                    SlotOrder::AcrossRegions => worst_across = worst_across.max(rollback_generations),
                                }
                            }
                            // usable 按三档 K 各判一次
                            let mut usable_bits = String::new();
                            for root_ring_depth in ROOT_RING_DEPTHS {
                                let is_usable = rollback.map(|rollback_generations| rollback_generations <= root_ring_depth - 1).unwrap_or(false);
                                usable_bits.push_str(&format!(" usable_k{root_ring_depth}={}", u8::from(is_usable)));
                                if mountable && !is_usable {
                                    mountable_but_unusable += 1; // 计的是「格 × K 档」，分母 cases × 3
                                }
                            }
                            println!(
                                "{}",
                                emitter.emit_raw(&format!(
                                    "name=loss order={} regions={regions} slots={slots} \
                                     chunk={chunk} devs={device_count} failed_dev={failed_device} \
                                     distinct_devs={distinct} survivors={survivors} \
                                     worst_rollback={} best_rollback={} dead_phases={dead_phases} \
                                     mountable={}{}",
                                    order.name(),
                                    rollback
                                        .map(|rollback_generations| rollback_generations.to_string())
                                        .unwrap_or_else(|| "NA".into()),
                                    best.map(|rollback_generations| rollback_generations.to_string()).unwrap_or_else(|| "NA".into()),
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
        emitter.emit_raw(&format!(
            "name=loss_summary cases={total} k_cells={} unmountable={unmountable} \
             mountable_but_unusable_kcells={mountable_but_unusable} \
             worst_rollback_within_region={worst_within} \
             worst_rollback_across_regions={worst_across} \
             orders_differ={}",
            total * ROOT_RING_DEPTHS.len() as u64,
            u8::from(worst_within != worst_across),
        ))
    );

    // ── 量二：R 个区域能不能落在 R 块互不相同的盘上 ──
    for &regions in REGIONS.iter() {
        let mut all_distinct_cells = 0u64;
        let mut cells = 0u64;
        for chunk in CHUNKS {
            for device_count in DEVICE_COUNTS {
                let mut devices_hosting_regions: Vec<u64> = (0..regions)
                    .map(|region_index| {
                        device_of(
                            region_position(region_index, regions, CAPACITY, REGION_BYTES, chunk),
                            chunk,
                            device_count,
                        )
                    })
                    .collect();
                devices_hosting_regions.sort_unstable();
                devices_hosting_regions.dedup();
                cells += 1;
                if devices_hosting_regions.len() as u64 == regions {
                    all_distinct_cells += 1;
                }
            }
        }
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=distinct_domains regions={regions} cells={cells} all_distinct={all_distinct_cells} \
                 tolerates_domains={}",
                regions.saturating_sub(1),
            ))
        );
    }

    // ── 量三：容忍 F 个失败域要几个区域 ──
    for tolerated_domains in 0u64..=3 {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=regions_needed tolerate={tolerated_domains} regions={} candidate_regions=2 enough={}",
                regions_needed(tolerated_domains),
                u8::from(2 >= regions_needed(tolerated_domains)),
            ))
        );
    }

    // ── 阳性对照：R=1，承载它的盘失效 ⇒ 幸存必须为 0 ──
    let one_region_on_device_zero = vec![0u64];
    let (single_region_survivors, single_region_rollback, _, single_region_dead_phases) = survey_worst(SlotOrder::WithinRegion, 1, 4, &one_region_on_device_zero, 0);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=poscontrol_single_region survivors={single_region_survivors} rollback={} dead_phases={single_region_dead_phases} \
             expect_survivors=0",
            single_region_rollback.map(|rollback_generations| rollback_generations.to_string()).unwrap_or_else(|| "NA".into()),
        ))
    );
    // ── 阴性对照：设备数 1 ⇒ 全部区域同盘 ⇒ 全丢 ──
    let one_device_for_both_regions: Vec<u64> = vec![0, 0];
    let (single_device_survivors, _, _, _) = survey_worst(SlotOrder::AcrossRegions, 2, 4, &one_device_for_both_regions, 0);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=negcontrol_single_device survivors={single_device_survivors} expect_survivors=0"
        ))
    );

    println!("{}", emitter.finish());
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
        for generation in 0..4 {
            assert_eq!(SlotOrder::WithinRegion.region_of(generation, 2, 4), 0);
        }
        for generation in 4..8 {
            assert_eq!(SlotOrder::WithinRegion.region_of(generation, 2, 4), 1);
        }
        assert_eq!(SlotOrder::WithinRegion.region_of(8, 2, 4), 0); // 绕回
        for generation in 0..8 {
            assert_eq!(SlotOrder::AcrossRegions.region_of(generation, 2, 4), generation % 2);
        }
    }

    /// **判据 1 的绝对值**：R=2 / S=4，两个区域分别在盘 0 与盘 1，丢盘 0
    /// ⇒ 幸存恰好 `R×S − S = 4`，两种槽序都一样（幸存数与槽序无关）。
    #[test]
    fn survivors_equal_ring_minus_the_dead_region() {
        let device_of_region = [0u64, 1];
        for order in [SlotOrder::WithinRegion, SlotOrder::AcrossRegions] {
            let (survivors, _) = survey(order, 2, 4, &device_of_region, 0, LATEST);
            assert_eq!(survivors, 4, "{}", order.name());
            let (survivors_losing_device_1, _) = survey(order, 2, 4, &device_of_region, 1, LATEST);
            assert_eq!(survivors_losing_device_1, 4, "{}", order.name());
        }
        // R=4 / S=2、四个区域四块盘，丢一块 ⇒ 8 − 2 = 6
        let device_of_four_regions = [0u64, 1, 2, 3];
        let (survivors, _) = survey(SlotOrder::AcrossRegions, 4, 2, &device_of_four_regions, 2, LATEST);
        assert_eq!(survivors, 6);
    }

    /// **判据 2：两种槽序的回退代数必须不同，且各自的绝对值由独立算术给出。**
    /// R=2 / S=4，最新一代 latest 落在区域 `region_of(latest)`；丢掉那个区域时——
    /// 区内连续：latest, latest-1, latest-2, latest-3 全在同一区 ⇒ 回退 **4**；
    /// 跨区轮转：latest-1 就在另一个区 ⇒ 回退 **1**。
    #[test]
    fn the_worst_rollback_differs_by_an_order_of_magnitude() {
        let device_of_region = [0u64, 1];
        // latest 选成让「最新那一代落在区域 0」的值，然后丢盘 0
        let latest = 1_000_000u64;
        assert_eq!(SlotOrder::WithinRegion.region_of(latest, 2, 4), 0);
        assert_eq!(SlotOrder::AcrossRegions.region_of(latest, 2, 4), 0);
        let _ = latest;
        let (_, rollback_within, best_within, _) = survey_worst(SlotOrder::WithinRegion, 2, 4, &device_of_region, 0);
        let (_, rollback_across, best_across, _) = survey_worst(SlotOrder::AcrossRegions, 2, 4, &device_of_region, 0);
        // **最好那个相位两条臂都是 0**（最新一代恰好落在活着的区域，一代都不丢）
        // ⇒ **只取一个相位量不出两条臂的差别，这正是第一版实现的错。**
        assert_eq!(best_within, Some(0));
        assert_eq!(best_across, Some(0));
        assert_eq!(rollback_within, Some(4), "区内连续丢的是连续 S 代");
        assert_eq!(rollback_across, Some(1), "跨区轮转丢的是每 R 代里的一代");
        assert_ne!(rollback_within, rollback_across, "两种槽序必须给出不同的回退，否则这一维是死的");
    }

    /// **区内连续的最坏回退恰好等于 S**，三档 S 各钉一次（绝对值，不是互比）。
    #[test]
    fn within_region_worst_rollback_equals_slots() {
        let device_of_region = [0u64, 1];
        for slots in [2u64, 4, 8] {
            let (_, rollback, _, _) = survey_worst(SlotOrder::WithinRegion, 2, slots, &device_of_region, 0);
            assert_eq!(rollback, Some(slots), "S={slots}");
        }
    }

    /// **跨区轮转的最坏回退恒为 R−1 之内**：R 个区域轮转，丢一个 ⇒ 最多回退 1 代
    /// （R=2）或更少……绝对值：R=2 → 1；R=3 → 1；R=4 → 1。
    #[test]
    fn across_regions_worst_rollback_is_one() {
        for regions in [2u64, 3, 4] {
            let device_of_region: Vec<u64> = (0..regions).collect();
            let (_, rollback, _, _) = survey_worst(SlotOrder::AcrossRegions, regions, 4, &device_of_region, 0);
            assert_eq!(rollback, Some(1), "R={regions}");
        }
    }

    /// **判据 4：区内连续 + S > K 时必然出现「挂得上但不可用」。**
    /// S=8、K=4 ⇒ 回退 8 > K−1=3 ⇒ mountable 但 usable=0。
    #[test]
    fn mountable_is_not_usable_when_the_rollback_exceeds_root_ring_depth() {
        let device_of_region = [0u64, 1];
        let (survivors, rollback, _, _) = survey_worst(SlotOrder::WithinRegion, 2, 8, &device_of_region, 0);
        assert_eq!(survivors, 8);
        assert_eq!(rollback, Some(8));
        assert!(rollback.unwrap() > 4 - 1, "K=4 时它不可用");
        assert!(rollback.unwrap() > 2 - 1, "K=2 时它同样不可用");
        assert!(rollback.unwrap() <= 8 - 1 || rollback.unwrap() == 8); // K=8 时恰好卡在边界外
    }

    /// **阳性对照**：R=1，承载它的盘失效 ⇒ 一个都不剩，回退记 None 而不是 0。
    #[test]
    fn positive_control_single_region_loses_everything() {
        let device_of_region = [0u64];
        let (survivors, rollback, _, dead) = survey_worst(SlotOrder::WithinRegion, 1, 4, &device_of_region, 0);
        assert_eq!(survivors, 0);
        assert_eq!(rollback, None, "读不到 ≠ 回退 0");
        assert_eq!(dead, 4, "四个相位全军覆没");
    }

    /// **阴性对照**：设备数 1 ⇒ 全部区域同盘 ⇒ 全丢。
    #[test]
    fn negative_control_single_device_loses_everything() {
        let device_of_region = [0u64, 0, 0, 0];
        for order in [SlotOrder::WithinRegion, SlotOrder::AcrossRegions] {
            let (survivors, rollback, _, dead) = survey_worst(order, 4, 4, &device_of_region, 0);
            assert_eq!(survivors, 0, "{}", order.name());
            assert_eq!(rollback, None);
            assert_eq!(dead, 16, "16 个相位全军覆没");
        }
    }

    /// **区域位置的绝对值**：R=2 时区域 1 落在盘尾（与 E46 的 tail 同型）。
    #[test]
    fn region_positions_span_the_device() {
        assert_eq!(region_position(0, 2, CAPACITY, REGION_BYTES, 65536), 0);
        let last_region_position = region_position(1, 2, CAPACITY, REGION_BYTES, 65536);
        assert_eq!(last_region_position, 9_999_999_959_040);
        // R=3 ⇒ 中间那个落在约一半处
        let middle_region_position = region_position(1, 3, CAPACITY, REGION_BYTES, 65536);
        assert_eq!(middle_region_position, 4_999_999_979_520);
        assert_eq!(region_position(2, 3, CAPACITY, REGION_BYTES, 65536), last_region_position);
    }

    /// **容忍 F 个失败域要 F+1 个区域**，绝对值。
    #[test]
    fn tolerating_failure_domains_needs_one_region_more_than_domains() {
        assert_eq!(regions_needed(0), 1);
        assert_eq!(regions_needed(1), 2);
        assert_eq!(regions_needed(2), 3);
        assert!(2 < regions_needed(2));
    }
}
