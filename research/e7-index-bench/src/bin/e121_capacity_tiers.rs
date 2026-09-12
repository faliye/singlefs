//! E121：按 `cap` 切档 —— 同 `cap` 的两档是冗余的，去掉它只赚不亏。
//!
//! ## 它问什么
//!
//! E120（档表取等比时公比怎么定）的四个候选是**同一个家族**：按槽宽 `W` 等比切。
//! 而决定占用的不是 `W`，是 **`cap` = ⌊(32768 − 103) / (W + 43)⌋**（一个容器装几个对象）。
//! 两档若 `cap` 相同，一个容器装的对象数就相同 ⇒ **它们对占用完全等价，留两档是冗余的**。
//!
//! E121 换这个家族：**先选一组 `cap` 值，再把每个 `cap` 对应的最大 `W` 取成档宽**。
//! 并检验一条可证伪的预测：把按 `W` 切的表**按 `cap` 去重**，容器数**只减不增**——
//! 因为合并同 `cap` 的两档只会让尾部没装满的容器少一个（⌈n₁/c⌉ + ⌈n₂/c⌉ ≥ ⌈(n₁+n₂)/c⌉）。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**D18 已定项 11**：打包记录单元头 103。
//! - **D2 已定项 9**：`w` = 2。**D27 已定项 2**：界线 ≤ 4 KiB。
//! - **D27 已定项 8 ④**：槽定宽，对象补齐到槽宽。
//! - **D27 已定项 12**：档表不进格式，容器头声明槽宽 ⇒ 档表是策略，可以按 `cap` 切。
//! - **E116 的闭式**：回本比下界 = `w / (cap − 1)` ⇒ 要 `cap ≥ 4`。
//! - **E120 的结论**：按 `W` 等比切时最坏相对浪费 = `1 − 1/r`，与分布无关。
//!
//! ## 假设（不是条款 —— C187）
//!
//! `SLOT_EXTRA = 43`；对象大小分布全仓无实测（C174），三个分布全部报出。
//!
//! ## 七条臂（跑前列全，中途不许加臂 —— C186）
//!
//! | 臂 | 档表怎么来 |
//! |---|---|
//! | `var` | 变长贪心，参照，不是候选 |
//! | `wgeo` | 按 `W` 等比，`r` = 1.125（E120 那张表，原样） |
//! | `wgeo_dedup` | 同上，**按 `cap` 去重**后 |
//! | `capgeo125` / `capgeo150` / `capgeo200` | 按 `cap` 等比，比值 1.25 / 1.5 / 2.0 |
//! | `capall` | 每个可达 `cap` 一档（最细的无冗余表） |
//!
//! `W_min` 另外扫 32 / 64 / 128 三个值，只对 `capgeo150` 扫。
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **去重预测**：`wgeo_dedup` 的容器数 **≤** `wgeo` 的，在三个分布上都成立。
//!    有一格大于 ⇒ 「同 `cap` 等价」这条推理错了，整轮作废。
//! 2. **档数与浪费**：每条臂报档数、最坏相对浪费（`W_min` 以上）、三个分布的实测损失。
//! 3. **回本闸**：每条臂每一档 `cap` ≥ 4。
//!
//! **阳性对照，逐臂跑**：对象恰好等于某档宽时，该臂与 `var` 逐字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`capgeo200` 与 `capall` 在同一分布上容器数必须不同。相同 ⇒ 档表没被建模。
//! **阴性对照**：N = 0 时七条臂全 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **`wgeo_dedup` 在任一分布上比 `wgeo` 多** ⇒ 结论写「同 `cap` 等价这条推理错了」，不许改口径。
//! - 若 **`capall`（280 档）比 `capgeo125` 省不到 1%** ⇒ 结论写「档数加到头也换不来多少，取少的」。
//! - 若 **`capgeo` 家族在所有分布上都不比 `wgeo` 家族省** ⇒ 结论写「按 `cap` 切没有好处，回到按 `W` 切」。
//! - 若 **任一臂任一档 `cap` < 4** ⇒ 结论写「那一档回不了本，不许进档表」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! `var` 是变长的一个具体装法（大小升序贪心），不是最优装法。
//! **不答同时开着的容器数的绝对值**（要乘整理器同时处理的树数；D19 已定项 6 在 2026-09-11 定了 key，按它重算没做）。
//! 不答「整理器怎么在多档之间调度」——那归 D27 已定项 6。

use e7_index_bench::Emitter;

const UNIT_BYTES: u64 = 32768;
const PACKED_UNIT_HEADER_BYTES: u64 = 103;
const UNIT_PAYLOAD_BYTES: u64 = UNIT_BYTES - PACKED_UNIT_HEADER_BYTES; // 32665
const SLOT_EXTRA: u64 = 43;
const LIMIT: u64 = 4096;
const OBJECT_COUNT: u64 = 100_000;

fn objects_per_container_for_slot_width(slot_width: u64) -> u64 { UNIT_PAYLOAD_BYTES / (slot_width + SLOT_EXTRA) }

/// 给定 `cap`，能达到这个 `cap` 的最大槽宽（同 `cap` 里浪费最小的那个选择）。
fn widest_slot_width_for_objects_per_container(objects_per_container: u64, minimum_slot_width: u64) -> Option<u64> {
    if objects_per_container == 0 { return None; }
    let slot_width = (UNIT_PAYLOAD_BYTES / objects_per_container).saturating_sub(SLOT_EXTRA);
    let slot_width = slot_width.min(LIMIT);
    if slot_width < minimum_slot_width || objects_per_container_for_slot_width(slot_width) != objects_per_container { None } else { Some(slot_width) }
}

/// 按 `W` 等比（E120 那张表）。
fn wgeo_tiers(ratio_in_thousandths: u64, minimum_slot_width: u64) -> Vec<u64> {
    let mut tier_widths = Vec::new();
    let mut slot_width = minimum_slot_width;
    while slot_width < LIMIT {
        tier_widths.push(slot_width);
        let next = (slot_width * ratio_in_thousandths).div_ceil(1000);
        if next <= slot_width { break; }
        slot_width = next;
    }
    tier_widths.push(LIMIT);
    tier_widths.dedup();
    tier_widths
}

/// 按 `cap` 去重：同 `cap` 的几档只留最大的那个 `W`。
fn deduplicate_by_objects_per_container(tier_widths: &[u64]) -> Vec<u64> {
    let mut deduplicated_tier_widths: Vec<u64> = Vec::new();
    for &slot_width in tier_widths {
        if let Some(&last) = deduplicated_tier_widths.last() {
            if objects_per_container_for_slot_width(last) == objects_per_container_for_slot_width(slot_width) { deduplicated_tier_widths.pop(); }
        }
        deduplicated_tier_widths.push(slot_width);
    }
    deduplicated_tier_widths
}

/// 按 `cap` 等比：`cap` 从 4096 那档的 7 起，乘 `q` 往上走，直到超过 `minimum_slot_width` 能给的最大 `cap`。
fn capgeo_tiers(ratio_in_thousandths: u64, minimum_slot_width: u64) -> Vec<u64> {
    let largest_objects_per_container = objects_per_container_for_slot_width(minimum_slot_width); // 最细档给出的最大 cap
    let mut objects_per_container_steps = Vec::new();
    let mut objects_per_container = objects_per_container_for_slot_width(LIMIT);
    while objects_per_container <= largest_objects_per_container {
        objects_per_container_steps.push(objects_per_container);
        let next = (objects_per_container * ratio_in_thousandths).div_ceil(1000);
        if next <= objects_per_container { break; }
        objects_per_container = next;
    }
    let mut tier_widths: Vec<u64> = objects_per_container_steps.iter().filter_map(|&objects_per_container| widest_slot_width_for_objects_per_container(objects_per_container, minimum_slot_width)).collect();
    tier_widths.push(LIMIT);
    tier_widths.sort_unstable();
    tier_widths.dedup();
    tier_widths
}

/// 每个可达 `cap` 一档（最细的无冗余表）。
fn capall_tiers(minimum_slot_width: u64) -> Vec<u64> {
    let mut tier_widths: Vec<u64> = (objects_per_container_for_slot_width(LIMIT)..=objects_per_container_for_slot_width(minimum_slot_width))
        .filter_map(|objects_per_container| widest_slot_width_for_objects_per_container(objects_per_container, minimum_slot_width)).collect();
    tier_widths.push(LIMIT);
    tier_widths.sort_unstable();
    tier_widths.dedup();
    tier_widths
}

fn weights(distribution: &str) -> Vec<u64> {
    let mut weight_by_object_size = vec![0u64; (LIMIT + 1) as usize];
    match distribution {
        "uniform" => { for object_size in 1..=LIMIT { weight_by_object_size[object_size as usize] = 1; } }
        "logunif" => { for object_size in 1..=LIMIT { weight_by_object_size[object_size as usize] = LIMIT / object_size; } }
        "discrete" => { for object_size in [512u64, 1024, 4096] { weight_by_object_size[object_size as usize] = 1; } }
        _ => {}
    }
    weight_by_object_size
}

fn counts(distribution: &str, object_count: u64) -> Vec<u64> {
    let weight_by_object_size = weights(distribution);
    let total: u64 = weight_by_object_size.iter().sum();
    if total == 0 || object_count == 0 { return vec![0u64; (LIMIT + 1) as usize]; }
    let mut count_by_object_size = vec![0u64; (LIMIT + 1) as usize];
    let (mut assigned_object_count, mut last_nonzero_object_size) = (0u64, 0usize);
    for object_size in 1..=LIMIT as usize {
        if weight_by_object_size[object_size] == 0 { continue; }
        count_by_object_size[object_size] = object_count * weight_by_object_size[object_size] / total; assigned_object_count += count_by_object_size[object_size]; last_nonzero_object_size = object_size;
    }
    count_by_object_size[last_nonzero_object_size] += object_count - assigned_object_count;
    count_by_object_size
}

fn containers_tiered(tier_widths: &[u64], count_by_object_size: &[u64]) -> u64 {
    let mut total = 0u64;
    for (tier_index, &slot_width) in tier_widths.iter().enumerate() {
        let tier_smallest_object_size = if tier_index == 0 { 1 } else { tier_widths[tier_index - 1] + 1 };
        let objects_in_tier: u64 = (tier_smallest_object_size..=slot_width).map(|object_size| count_by_object_size[object_size as usize]).sum();
        if objects_in_tier > 0 { total += objects_in_tier.div_ceil(objects_per_container_for_slot_width(slot_width).max(1)); }
    }
    total
}

fn containers_variable_length(count_by_object_size: &[u64]) -> u64 {
    let (mut container_count, mut room) = (0u64, 0u64);
    for object_size in 1..=LIMIT {
        let need = object_size + SLOT_EXTRA;
        let mut left = count_by_object_size[object_size as usize];
        while left > 0 {
            if room < need { container_count += 1; room = UNIT_PAYLOAD_BYTES; }
            let fit = (room / need).min(left);
            room -= fit * need; left -= fit;
        }
    }
    container_count
}

fn worst_waste_above_floor(tier_widths: &[u64]) -> f64 {
    let mut worst = 0.0f64;
    for (tier_index, &slot_width) in tier_widths.iter().enumerate() {
        if tier_index == 0 { continue; }
        let waste = 1.0 - ((tier_widths[tier_index - 1] + 1) as f64) / (slot_width as f64);
        if waste > worst { worst = waste; }
    }
    worst
}

fn arm_tiers(arm: &str, minimum_slot_width: u64) -> Vec<u64> {
    match arm {
        "wgeo" => wgeo_tiers(1125, minimum_slot_width),
        "wgeo_dedup" => deduplicate_by_objects_per_container(&wgeo_tiers(1125, minimum_slot_width)),
        "capgeo125" => capgeo_tiers(1250, minimum_slot_width),
        "capgeo150" => capgeo_tiers(1500, minimum_slot_width),
        "capgeo200" => capgeo_tiers(2000, minimum_slot_width),
        "capall" => capall_tiers(minimum_slot_width),
        _ => vec![],
    }
}

const ARMS: [&str; 6] = ["wgeo", "wgeo_dedup", "capgeo125", "capgeo150", "capgeo200", "capall"];

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config unit={UNIT_BYTES} pack_hdr={PACKED_UNIT_HEADER_BYTES} net={UNIT_PAYLOAD_BYTES} slot_extra={SLOT_EXTRA} \
         limit={LIMIT} n={OBJECT_COUNT} arms={ARMS:?}")));

    // 阳性对照
    for &arm in ARMS.iter() {
        let tier_widths = arm_tiers(arm, 64);
        for &slot_width in tier_widths.iter() {
            let mut count_by_object_size = vec![0u64; (LIMIT + 1) as usize];
            count_by_object_size[slot_width as usize] = OBJECT_COUNT;
            let (variable_length_containers, tiered_containers) = (containers_variable_length(&count_by_object_size), containers_tiered(&tier_widths, &count_by_object_size));
            if variable_length_containers != tiered_containers {
                println!("{}", emitter.emit_raw(&format!(
                    "name=positive_exact arm={arm} tier={slot_width} var={variable_length_containers} tiered={tiered_containers} same=false")));
            }
        }
        println!("{}", emitter.emit_raw(&format!(
            "name=positive_exact_summary arm={arm} tiers={} all_same=true", tier_widths.len())));
    }

    // 判别力
    {
        let count_by_object_size = counts("uniform", OBJECT_COUNT);
        let capgeo200_containers = containers_tiered(&arm_tiers("capgeo200", 64), &count_by_object_size);
        let capall_containers = containers_tiered(&arm_tiers("capall", 64), &count_by_object_size);
        println!("{}", emitter.emit_raw(&format!(
            "name=discrimination capgeo200={capgeo200_containers} capall={capall_containers} differ={}", capgeo200_containers != capall_containers)));
    }

    // 阴性
    {
        let count_by_object_size = counts("uniform", 0);
        println!("{}", emitter.emit_raw(&format!("name=negative arm=var containers={}", containers_variable_length(&count_by_object_size))));
        for &arm in ARMS.iter() {
            println!("{}", emitter.emit_raw(&format!(
                "name=negative arm={arm} containers={}", containers_tiered(&arm_tiers(arm, 64), &count_by_object_size))));
        }
    }

    // 档表本身
    for &arm in ARMS.iter() {
        let tier_widths = arm_tiers(arm, 64);
        let smallest_objects_per_container = tier_widths.iter().map(|&slot_width| objects_per_container_for_slot_width(slot_width)).min().unwrap_or(0);
        println!("{}", emitter.emit_raw(&format!(
            "name=table arm={arm} tiers={} worst_waste={:.6} min_cap={} min_tier={} max_tier={}",
            tier_widths.len(), worst_waste_above_floor(&tier_widths), smallest_objects_per_container, tier_widths[0], tier_widths.last().unwrap())));
    }

    // 主判据
    for distribution in ["uniform", "logunif", "discrete"] {
        let count_by_object_size = counts(distribution, OBJECT_COUNT);
        let variable_length_containers = containers_variable_length(&count_by_object_size);
        println!("{}", emitter.emit_raw(&format!("name=main dist={distribution} arm=var containers={variable_length_containers} loss=0.0000")));
        for &arm in ARMS.iter() {
            let tiered_containers = containers_tiered(&arm_tiers(arm, 64), &count_by_object_size);
            println!("{}", emitter.emit_raw(&format!(
                "name=main dist={distribution} arm={arm} containers={tiered_containers} loss={:.4} tiers={}",
                (tiered_containers as f64 - variable_length_containers as f64) / variable_length_containers as f64, arm_tiers(arm, 64).len())));
        }
    }

    // W_min 扫描，只对 capgeo150
    for &minimum_slot_width in [32u64, 64, 128].iter() {
        let tier_widths = arm_tiers("capgeo150", minimum_slot_width);
        for distribution in ["uniform", "logunif"] {
            let count_by_object_size = counts(distribution, OBJECT_COUNT);
            let variable_length_containers = containers_variable_length(&count_by_object_size);
            let tiered_containers = containers_tiered(&tier_widths, &count_by_object_size);
            println!("{}", emitter.emit_raw(&format!(
                "name=wmin arm=capgeo150 w_min={minimum_slot_width} dist={distribution} tiers={} containers={tiered_containers} \
                 loss={:.4} floor_waste_bytes={}",
                tier_widths.len(), (tiered_containers as f64 - variable_length_containers as f64) / variable_length_containers as f64, minimum_slot_width - 1)));
        }
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 跑前写死的那条预测：按 `cap` 去重之后容器数只减不增。
    #[test]
    fn dedup_never_increases_containers() {
        for distribution in ["uniform", "logunif", "discrete"] {
            let count_by_object_size = counts(distribution, OBJECT_COUNT);
            let wgeo_containers = containers_tiered(&arm_tiers("wgeo", 64), &count_by_object_size);
            let deduplicated_containers = containers_tiered(&arm_tiers("wgeo_dedup", 64), &count_by_object_size);
            assert!(deduplicated_containers <= wgeo_containers, "dist={distribution}：去重后 {deduplicated_containers} 该 ≤ 去重前 {wgeo_containers}");
        }
    }

    /// 阳性对照：对象恰好等于档宽时与变长逐字节相同。
    #[test]
    fn positive_control_exact_tier_matches_variable_length() {
        for &arm in ARMS.iter() {
            let tier_widths = arm_tiers(arm, 64);
            for &slot_width in tier_widths.iter() {
                let mut count_by_object_size = vec![0u64; (LIMIT + 1) as usize];
                count_by_object_size[slot_width as usize] = OBJECT_COUNT;
                assert_eq!(containers_tiered(&tier_widths, &count_by_object_size), containers_variable_length(&count_by_object_size), "arm={arm} 档宽={slot_width}");
            }
        }
    }

    /// 判别力：档表真的被建模了。
    #[test]
    fn discrimination_table_matters() {
        let count_by_object_size = counts("uniform", OBJECT_COUNT);
        let capgeo200_containers = containers_tiered(&arm_tiers("capgeo200", 64), &count_by_object_size);
        let capall_containers = containers_tiered(&arm_tiers("capall", 64), &count_by_object_size);
        assert!(capall_containers < capgeo200_containers, "capall 该比 capgeo200 省：capall={capall_containers} capgeo200={capgeo200_containers}");
    }

    /// 阴性：N = 0 全 0。
    #[test]
    fn negative_control_zero() {
        let count_by_object_size = counts("uniform", 0);
        assert_eq!(containers_variable_length(&count_by_object_size), 0);
        for &arm in ARMS.iter() {
            assert_eq!(containers_tiered(&arm_tiers(arm, 64), &count_by_object_size), 0, "arm={arm}");
        }
    }

    /// ⚠️ 跑前假定「按 `W` 等比的表里有同 `cap` 的冗余档」，**判否**：
    /// `r` = 1.125 那张表 36 档，按 `cap` 去重之后**还是 36 档**——每一步都跨过一个 `cap`。
    /// 去重这件事本身没错，只是**在这个公比上是空操作**；它要到表比 `cap` 的分辨率还细时才咬得动。
    /// 两个数都留着：`r` = 1.125 去重前后 36 / 36；`r` = 1.01 去重前后 370 / 208。
    #[test]
    fn dedup_is_a_noop_at_this_ratio_but_bites_on_finer_ones() {
        let wgeo_tier_widths = arm_tiers("wgeo", 64);
        let wgeo_dedup_tier_widths = arm_tiers("wgeo_dedup", 64);
        assert_eq!(wgeo_tier_widths.len(), 36);
        assert_eq!(wgeo_dedup_tier_widths.len(), 36, "r = 1.125 这张表没有同 cap 的冗余档");
        // 换一张比 cap 分辨率还细的表，去重当场砍掉 44%
        let fine = wgeo_tiers(1010, 64);
        assert_eq!(fine.len(), 370);
        assert_eq!(deduplicate_by_objects_per_container(&fine).len(), 208);
    }

    /// **档数不是越多越好**：档越多补齐越少，但**没装满的尾容器也越多**。
    /// `capall`（248 档）在对象总数摊得开的 `uniform` 上赢 `wgeo`（36 档），
    /// 在总容器数只有 1592 的 `logunif` 上反而输——248 个尾容器就占了 15%。
    /// 这一条是 E121 的主结论，必须有断言守着。
    #[test]
    fn more_tiers_is_not_always_better() {
        let uniform_counts = counts("uniform", OBJECT_COUNT);
        let log_uniform_counts = counts("logunif", OBJECT_COUNT);
        let (wgeo_tier_widths, capall_tier_widths) = (arm_tiers("wgeo", 64), arm_tiers("capall", 64));
        assert_eq!(wgeo_tier_widths.len(), 36);
        assert_eq!(capall_tier_widths.len(), 248);
        // uniform：档多的赢
        assert!(containers_tiered(&capall_tier_widths, &uniform_counts) < containers_tiered(&wgeo_tier_widths, &uniform_counts));
        // logunif：档多的输
        assert!(containers_tiered(&capall_tier_widths, &log_uniform_counts) > containers_tiered(&wgeo_tier_widths, &log_uniform_counts));
        // 尾容器那笔账钉绝对值：logunif 上 capall 的容器数 1788，其中档数 248
        assert_eq!(containers_tiered(&capall_tier_widths, &log_uniform_counts), 1788);
        assert_eq!(containers_variable_length(&log_uniform_counts), 1592);
    }

    /// 等价性留档（变异 M10）：`capall_tiers` 末尾那句 `tier_widths.push(LIMIT)` 是**冗余**的——
    /// `cap` = 7 那一档算出来的最大槽宽本来就是 4096（`32665/7 − 43 = 4623`，被界线截到 4096，
    /// 而 `objects_per_container_for_slot_width(4096)` 恰好还是 7）。去掉那句在所有输入上同值 ⇒ 按
    /// `.claude/rules/mutation-sampling.md` 判**等价变异**，不是盲区，把等价性写成这条测试留档。
    #[test]
    fn capall_contains_limit_without_the_explicit_push() {
        assert_eq!(widest_slot_width_for_objects_per_container(objects_per_container_for_slot_width(LIMIT), 64), Some(LIMIT));
        assert_eq!(objects_per_container_for_slot_width(LIMIT), 7);
        assert_eq!(UNIT_PAYLOAD_BYTES / 7 - SLOT_EXTRA, 4623); // 未截断前
    }

    /// 界线截断真的在起作用：去掉 `.min(LIMIT)` 会让档宽越过 D27 已定项 2 的 4 KiB。
    #[test]
    fn limit_clamp_is_load_bearing() {
        for &arm in ARMS.iter() {
            for slot_width in arm_tiers(arm, 64) {
                assert!(slot_width <= LIMIT, "arm={arm} 档宽 {slot_width} 越过界线 {LIMIT}");
            }
        }
        // cap = 7 那一档若不截断会算出 4623 > 4096
        assert!(UNIT_PAYLOAD_BYTES / objects_per_container_for_slot_width(LIMIT) - SLOT_EXTRA > LIMIT);
    }

    /// 每一档 `cap` ≥ 4（E116 的回本闸）。
    #[test]
    fn every_tier_pays_back() {
        for &arm in ARMS.iter() {
            for slot_width in arm_tiers(arm, 64) {
                assert!(objects_per_container_for_slot_width(slot_width) >= 4, "arm={arm} 档宽={slot_width} 的 cap={}", objects_per_container_for_slot_width(slot_width));
            }
        }
        assert_eq!(objects_per_container_for_slot_width(LIMIT), 7);
    }

    /// 档表规模钉绝对值。
    #[test]
    fn table_sizes_absolute() {
        assert_eq!(arm_tiers("capgeo125", 64).len(), 15);
        assert_eq!(arm_tiers("capgeo150", 64).len(), 10);
        assert_eq!(arm_tiers("capgeo200", 64).len(), 5);
        assert_eq!(arm_tiers("capall", 64).len(), 248);
        assert_eq!(objects_per_container_for_slot_width(64), 305);
    }

    /// 主判据绝对值。
    #[test]
    fn main_absolute() {
        let uniform_counts = counts("uniform", OBJECT_COUNT);
        assert_eq!(containers_variable_length(&uniform_counts), 6814);
        assert_eq!(containers_tiered(&arm_tiers("wgeo", 64), &uniform_counts), 7378);
        assert_eq!(containers_tiered(&arm_tiers("wgeo_dedup", 64), &uniform_counts), 7378);
        assert_eq!(containers_tiered(&arm_tiers("capgeo125", 64), &uniform_counts), 7455);
        assert_eq!(containers_tiered(&arm_tiers("capgeo150", 64), &uniform_counts), 8180);
        assert_eq!(containers_tiered(&arm_tiers("capgeo200", 64), &uniform_counts), 9116);
        assert_eq!(containers_tiered(&arm_tiers("capall", 64), &uniform_counts), 6977);

        let log_uniform_counts = counts("logunif", OBJECT_COUNT);
        assert_eq!(containers_variable_length(&log_uniform_counts), 1592);
        assert_eq!(containers_tiered(&arm_tiers("wgeo", 64), &log_uniform_counts), 1771);
        assert_eq!(containers_tiered(&arm_tiers("capall", 64), &log_uniform_counts), 1788);
    }

    /// 分布守恒。
    #[test]
    fn counts_conserve_objects() {
        for distribution in ["uniform", "logunif", "discrete"] {
            let total_objects: u64 = counts(distribution, OBJECT_COUNT).iter().sum();
            assert_eq!(total_objects, OBJECT_COUNT, "dist={distribution}");
        }
    }
}
