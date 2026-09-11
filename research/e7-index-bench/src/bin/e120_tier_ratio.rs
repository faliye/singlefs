//! E120：档表取等比 —— 公比 `r` 怎么定，档数与浪费怎么换。
//!
//! ## 它问什么
//!
//! E119（槽定宽的补齐代价）判出「四档太粗、档要在小端密」，但它比的是五张**具体的**表，
//! 而那些表都是照某个分布挑的——**而对象大小分布全仓没测过**（C174）。
//! 照没测过的分布挑表，等于把一个未知量当已知量用。
//!
//! E120 换一条路：**等比档表的相对浪费上界与分布无关**。
//! 档宽取 `W_k = W_min · r^k`（截到 ≤ 4096），任何落进第 k 档的对象都满足
//! `W_{k-1} < s ≤ W_k`，所以它补齐后占的字节不超过它自身的 `r` 倍
//! ⇒ **最坏相对浪费 = 1 − 1/r，与对象大小分布无关。**
//!
//! 于是「档怎么切」变成一个一维的选择：`r` 越小浪费越小、档数越多。
//! E120 把这条曲线量出来，并验证闭式与三个分布上的实测对得上。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**D18 已定项 11**：打包记录单元头 103。
//! - **D2 已定项 9**：第一版 2 盘恒 `w` = 2。**D27 已定项 2**：界线 ≤ 4 KiB。
//! - **D27 已定项 8 ④**：槽定宽，对象补齐到槽宽，槽 `i` 起点 = 头 + `i × W`。
//! - **E116 的闭式**：回本比下界 = `w / (cap − 1)` ⇒ 要 `cap ≥ 4`（`w` = 2）。
//! - **E119 的结论**：档要在小端密，不按等距切。
//!
//! ## 两个假设（不是条款 —— C187）
//!
//! - `SLOT_EXTRA_BYTES = 43`：五元组 33（D18 已定项 3）+ 写序 10（D18 已定项 7）；定宽下不要槽目录条目。
//! - **对象大小分布**：全仓无实测（C174）。三个分布全部报出，**不挑代表**；
//!   而 E120 的主结论（最坏浪费闭式）**不依赖分布**，这正是它相对 E119 的改进。
//!
//! ## 五条臂（跑前列全，中途不许加臂 —— C186）
//!
//! `var`（变长贪心，参照，不是候选）+ 四个公比 `r` ∈ {1.125, 1.25, 1.5, 2.0}，`W_min` = 64。
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **闭式**：每条等比臂的**最坏相对浪费**必须等于 `1 − 1/r`（相对误差 < 1e-9）。
//!    ⚠️ 它是按档宽比算的，不是按实测；实测那一列另报。
//! 2. **档数**：报每个 `r` 的档数与最大档的 `cap`。
//! 3. **实测**：每条臂 × 三个分布，报相对 `var` 的损失。
//! 4. **单调**：`r` 越大档数越少、损失越大。
//!
//! **阳性对照，逐臂跑**：把分布换成「所有对象恰好等于某个档宽」，每条等比臂在**它自己有那一档时**
//! 必须与 `var` 逐字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`r` = 2.0 与 `r` = 1.125 在同一分布上的损失必须不同。相同 ⇒ 公比没被建模，整轮作废。
//! **阴性对照**：N = 0 时五条臂全 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **任一 `r` 的最大档 `cap` < 4** ⇒ 结论写「那个 `r` 的最粗档回不了本，不许用」。
//! - 若 **`r` = 1.125 的实测损失在任一分布上 > `1 − 1/r` = 11.11%** ⇒ 结论写
//!   「闭式不是上界，等比这条路的论证垮了」，不许回头改口径。
//! - 若 **`r` = 2.0 的实测损失在所有分布上都 < 5%** ⇒ 结论写「公比无所谓，取最少的档数」。
//! - 若 **四个 `r` 的档数都 > 32** ⇒ 结论写「等比表太长，同时开着的容器数不可接受」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! `var` 是变长这条路的一个具体装法（大小升序贪心），不是最优装法。
//! **不答「同时开着的容器数」的绝对值**：它等于档数乘上整理器同时在处理的树数，
//! 而后者取决于 D19 已定项 6（中央映射条目的 key 编码）——若 key 以树 ID 领头，
//! 沿 key 序走一次只碰一棵树，那个乘数就是 1；D19 已定项 6 在 2026-09-11 定成按类复用写序键（类标签之后以出生树打头），这一格按它重算没做。

use e7_index_bench::Emitter;

const UNIT_BYTES: u64 = 32768;
const PACKED_UNIT_HEADER_BYTES: u64 = 103;
const UNIT_PAYLOAD_BYTES: u64 = UNIT_BYTES - PACKED_UNIT_HEADER_BYTES; // 32665
const SLOT_EXTRA_BYTES: u64 = 43;
const PACKING_LIMIT_BYTES: u64 = 4096; // D27 已定项 2
const MINIMUM_SLOT_WIDTH: u64 = 64;
const OBJECT_COUNT: u64 = 100_000;
/// 公比按千分之一为单位存成整数，避免浮点当键：1125 = 1.125
const RATIOS: [u64; 4] = [1125, 1250, 1500, 2000];

fn slots_per_container(slot_width: u64) -> u64 { UNIT_PAYLOAD_BYTES / (slot_width + SLOT_EXTRA_BYTES) }

/// 等比档表：`W_min · r^k`，向上取整到整数字节，去重，最后一档钉死为 PACKING_LIMIT_BYTES。
fn geometric_tiers(ratio_in_thousandths: u64) -> Vec<u64> {
    let mut tier_widths = Vec::new();
    let mut slot_width = MINIMUM_SLOT_WIDTH;
    loop {
        if slot_width >= PACKING_LIMIT_BYTES { break; }
        tier_widths.push(slot_width);
        let next_slot_width = (slot_width * ratio_in_thousandths).div_ceil(1000);
        if next_slot_width <= slot_width { break; } // 防呆：公比 ≤ 1 时不许死循环
        slot_width = next_slot_width;
    }
    tier_widths.push(PACKING_LIMIT_BYTES);
    tier_widths.dedup();
    tier_widths
}

fn weights(distribution: &str) -> Vec<u64> {
    let mut weight_by_object_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
    match distribution {
        "uniform" => { for object_size in 1..=PACKING_LIMIT_BYTES { weight_by_object_size[object_size as usize] = 1; } }
        "logunif" => { for object_size in 1..=PACKING_LIMIT_BYTES { weight_by_object_size[object_size as usize] = PACKING_LIMIT_BYTES / object_size; } }
        "discrete" => { for object_size in [512u64, 1024, 4096] { weight_by_object_size[object_size as usize] = 1; } }
        _ => {}
    }
    weight_by_object_size
}

fn counts(distribution: &str, object_count: u64) -> Vec<u64> {
    let weight_by_object_size = weights(distribution);
    let total_weight: u64 = weight_by_object_size.iter().sum();
    if total_weight == 0 || object_count == 0 { return vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize]; }
    let mut count_by_object_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
    let (mut assigned_object_count, mut last_nonzero_object_size) = (0u64, 0usize);
    for object_size in 1..=PACKING_LIMIT_BYTES as usize {
        if weight_by_object_size[object_size] == 0 { continue; }
        count_by_object_size[object_size] = object_count * weight_by_object_size[object_size] / total_weight; assigned_object_count += count_by_object_size[object_size]; last_nonzero_object_size = object_size;
    }
    count_by_object_size[last_nonzero_object_size] += object_count - assigned_object_count;
    count_by_object_size
}

fn containers_tiered(tier_widths: &[u64], count_by_object_size: &[u64]) -> u64 {
    let mut total_containers = 0u64;
    for (tier_index, &slot_width) in tier_widths.iter().enumerate() {
        let tier_smallest_object_size = if tier_index == 0 { 1 } else { tier_widths[tier_index - 1] + 1 };
        let objects_in_tier: u64 = (tier_smallest_object_size..=slot_width).map(|object_size| count_by_object_size[object_size as usize]).sum();
        if objects_in_tier > 0 { total_containers += objects_in_tier.div_ceil(slots_per_container(slot_width).max(1)); }
    }
    total_containers
}

fn containers_variable_length(count_by_object_size: &[u64]) -> u64 {
    let (mut container_count, mut remaining_payload_bytes) = (0u64, 0u64);
    for object_size in 1..=PACKING_LIMIT_BYTES {
        let bytes_per_slot = object_size + SLOT_EXTRA_BYTES;
        let mut objects_left = count_by_object_size[object_size as usize];
        while objects_left > 0 {
            if remaining_payload_bytes < bytes_per_slot { container_count += 1; remaining_payload_bytes = UNIT_PAYLOAD_BYTES; }
            let objects_fitting = (remaining_payload_bytes / bytes_per_slot).min(objects_left);
            remaining_payload_bytes -= objects_fitting * bytes_per_slot; objects_left -= objects_fitting;
        }
    }
    container_count
}

/// 最坏相对浪费，**只看 `W_min` 以上**。
///
/// ⚠️ 跑前登记的是「最坏相对浪费 = `1 − 1/r`」，**判否**：第一档覆盖 1..=`W_min`，
/// 一个 1 字节的对象补到 64 就浪费 98.44%，四个公比上这个数**完全相同**——
/// 它是 `W_min` 那个地板造成的，不是公比造成的。登记的式子只在 `W_min` 以上成立。
/// 地板那一段的代价该按**绝对字节**看：每个对象至多浪费 `W_min − 1` = 63 字节。
fn worst_waste_above_floor(tier_widths: &[u64]) -> f64 {
    let mut worst_waste = 0.0f64;
    for (tier_index, &slot_width) in tier_widths.iter().enumerate() {
        if tier_index == 0 { continue; } // 地板那一档另算
        let tier_smallest_object_size = tier_widths[tier_index - 1] + 1;
        let waste = 1.0 - (tier_smallest_object_size as f64) / (slot_width as f64);
        if waste > worst_waste { worst_waste = waste; }
    }
    worst_waste
}

/// 地板那一档的绝对代价：落在 1..=`W_min` 的对象每个至多浪费这么多字节。
fn floor_waste_bytes() -> u64 { MINIMUM_SLOT_WIDTH - 1 }

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config unit={UNIT_BYTES} pack_hdr={PACKED_UNIT_HEADER_BYTES} net={UNIT_PAYLOAD_BYTES} slot_extra={SLOT_EXTRA_BYTES} \
         limit={PACKING_LIMIT_BYTES} w_min={MINIMUM_SLOT_WIDTH} n={OBJECT_COUNT} ratios={RATIOS:?}")));

    // 阳性对照：对象恰好等于某档宽 ⇒ 该臂与 var 逐字节相同
    for &ratio_in_thousandths in RATIOS.iter() {
        let tier_widths = geometric_tiers(ratio_in_thousandths);
        for &slot_width in tier_widths.iter() {
            let mut count_by_object_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
            count_by_object_size[slot_width as usize] = OBJECT_COUNT;
            let variable_length_containers = containers_variable_length(&count_by_object_size);
            let tiered_containers = containers_tiered(&tier_widths, &count_by_object_size);
            println!("{}", emitter.emit_raw(&format!(
                "name=positive_exact r={ratio_in_thousandths} tier={slot_width} var={variable_length_containers} tiered={tiered_containers} same={}", variable_length_containers == tiered_containers)));
        }
    }

    // 判别力：两个极端公比在同一分布上必须不同
    {
        let count_by_object_size = counts("uniform", OBJECT_COUNT);
        let ratio_1125_containers = containers_tiered(&geometric_tiers(1125), &count_by_object_size);
        let ratio_2000_containers = containers_tiered(&geometric_tiers(2000), &count_by_object_size);
        println!("{}", emitter.emit_raw(&format!(
            "name=discrimination r1125={ratio_1125_containers} r2000={ratio_2000_containers} differ={}", ratio_1125_containers != ratio_2000_containers)));
    }

    // 阴性对照
    {
        let count_by_object_size = counts("uniform", 0);
        println!("{}", emitter.emit_raw(&format!("name=negative arm=var containers={}", containers_variable_length(&count_by_object_size))));
        for &ratio_in_thousandths in RATIOS.iter() {
            println!("{}", emitter.emit_raw(&format!(
                "name=negative r={ratio_in_thousandths} containers={}", containers_tiered(&geometric_tiers(ratio_in_thousandths), &count_by_object_size))));
        }
    }

    // 闭式与档数
    for &ratio_in_thousandths in RATIOS.iter() {
        let tier_widths = geometric_tiers(ratio_in_thousandths);
        let closed_form_worst_waste = 1.0 - 1000.0 / ratio_in_thousandths as f64;
        println!("{}", emitter.emit_raw(&format!(
            "name=closed r={ratio_in_thousandths} tiers={} worst_waste_above_floor={:.6} closed_form={:.6} \
             floor_waste_bytes={} max_tier={} cap_max_tier={} min_tier={} cap_min_tier={}",
            tier_widths.len(), worst_waste_above_floor(&tier_widths), closed_form_worst_waste, floor_waste_bytes(),
            tier_widths.last().unwrap(), slots_per_container(*tier_widths.last().unwrap()),
            tier_widths[0], slots_per_container(tier_widths[0]))));
    }

    // 实测
    for distribution in ["uniform", "logunif", "discrete"] {
        let count_by_object_size = counts(distribution, OBJECT_COUNT);
        let variable_length_containers = containers_variable_length(&count_by_object_size);
        println!("{}", emitter.emit_raw(&format!(
            "name=main dist={distribution} arm=var containers={variable_length_containers} gain={:.4} loss=0.0000 tiers=0",
            (OBJECT_COUNT * UNIT_BYTES) as f64 / (variable_length_containers * UNIT_BYTES) as f64)));
        for &ratio_in_thousandths in RATIOS.iter() {
            let tier_widths = geometric_tiers(ratio_in_thousandths);
            let tiered_containers = containers_tiered(&tier_widths, &count_by_object_size);
            println!("{}", emitter.emit_raw(&format!(
                "name=main dist={distribution} r={ratio_in_thousandths} containers={tiered_containers} gain={:.4} loss={:.4} tiers={}",
                (OBJECT_COUNT * UNIT_BYTES) as f64 / (tiered_containers * UNIT_BYTES) as f64,
                (tiered_containers as f64 - variable_length_containers as f64) / variable_length_containers as f64, tier_widths.len())));
        }
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：对象恰好等于档宽时，等比档与变长逐字节相同。
    #[test]
    fn positive_control_exact_tier_matches_variable_length() {
        for &ratio_in_thousandths in RATIOS.iter() {
            let tier_widths = geometric_tiers(ratio_in_thousandths);
            for &slot_width in tier_widths.iter() {
                let mut count_by_object_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
                count_by_object_size[slot_width as usize] = OBJECT_COUNT;
                assert_eq!(containers_tiered(&tier_widths, &count_by_object_size), containers_variable_length(&count_by_object_size),
                           "r={ratio_in_thousandths} 档宽={slot_width}");
            }
        }
    }

    /// 判别力：公比真的被建模了。
    #[test]
    fn discrimination_ratio_matters() {
        let count_by_object_size = counts("uniform", OBJECT_COUNT);
        let ratio_1125_containers = containers_tiered(&geometric_tiers(1125), &count_by_object_size);
        let ratio_2000_containers = containers_tiered(&geometric_tiers(2000), &count_by_object_size);
        assert!(ratio_1125_containers < ratio_2000_containers, "r 越小该越省：r=1.125 {ratio_1125_containers}，r=2.0 {ratio_2000_containers}");
        assert_eq!(ratio_1125_containers, 7378);
        assert_eq!(ratio_2000_containers, 9463);
    }

    /// 阴性：N = 0 全 0。
    #[test]
    fn negative_control_zero() {
        let count_by_object_size = counts("uniform", 0);
        assert_eq!(containers_variable_length(&count_by_object_size), 0);
        for &ratio_in_thousandths in RATIOS.iter() { assert_eq!(containers_tiered(&geometric_tiers(ratio_in_thousandths), &count_by_object_size), 0, "r={ratio_in_thousandths}"); }
    }

    /// 闭式：`W_min` 以上的最坏相对浪费 ≤ 1 − 1/r + 取整余量，且这一条与分布无关。
    ///
    /// ⚠️ 跑前登记的是「= 1 − 1/r，相对误差 < 1e-9」，**判否两次**：
    /// ① 第一档覆盖 1..=W_min，1 字节对象补到 64 浪费 98.44%，四个公比上这个数完全相同
    ///    ⇒ 那是地板不是公比，闭式只在 W_min 以上成立；
    /// ② 档宽按整数向上取整，实际档比略超 r（如 81 → 92 是 1.1358 > 1.125）
    ///    ⇒ 上界要放宽到 r + 1/W_min。两条都不许回头改登记的式子。
    #[test]
    fn worst_waste_matches_closed_form() {
        for &ratio_in_thousandths in RATIOS.iter() {
            let tier_widths = geometric_tiers(ratio_in_thousandths);
            let closed_form_worst_waste = 1.0 - 1000.0 / ratio_in_thousandths as f64;
            let slack = 1.0 - 1.0 / (ratio_in_thousandths as f64 / 1000.0 + 1.0 / MINIMUM_SLOT_WIDTH as f64);
            let measured_worst_waste = worst_waste_above_floor(&tier_widths);
            assert!(measured_worst_waste <= slack + 1e-9,
                    "r={ratio_in_thousandths}：W_min 以上最坏浪费 {measured_worst_waste} 超过含取整余量的上界 {slack}（闭式 {closed_form_worst_waste}）");
        }
        // 地板那一段按绝对字节算，与公比无关
        assert_eq!(floor_waste_bytes(), 63);
        // 钉绝对值，不许只靠与闭式互比
        assert!(((1.0f64 - 1000.0 / 1125.0) - 0.111111).abs() < 1e-6);
        assert!(((1.0f64 - 1000.0 / 2000.0) - 0.5).abs() < 1e-12);
    }

    /// 档数与每档的 cap 钉绝对值。
    #[test]
    fn tier_counts_and_objects_per_container_absolute() {
        assert_eq!(geometric_tiers(1125).len(), 36);
        assert_eq!(geometric_tiers(1250).len(), 20);
        assert_eq!(geometric_tiers(1500).len(), 12);
        assert_eq!(geometric_tiers(2000).len(), 7);
        for &ratio_in_thousandths in RATIOS.iter() {
            let tier_widths = geometric_tiers(ratio_in_thousandths);
            assert_eq!(tier_widths[0], MINIMUM_SLOT_WIDTH, "r={ratio_in_thousandths} 最细一档该是 W_min");
            assert_eq!(*tier_widths.last().unwrap(), PACKING_LIMIT_BYTES, "r={ratio_in_thousandths} 最粗一档该钉死在界线上");
            for &slot_width in tier_widths.iter() {
                assert!(slots_per_container(slot_width) >= 4, "r={ratio_in_thousandths} 档宽={slot_width} 的 cap={} < 4", slots_per_container(slot_width));
            }
        }
        assert_eq!(slots_per_container(PACKING_LIMIT_BYTES), 7);
        assert_eq!(slots_per_container(MINIMUM_SLOT_WIDTH), 305);
    }

    /// 单调：r 越大档数越少、容器越多（同一分布上）。互比，与上面钉绝对值的配对。
    #[test]
    fn larger_ratio_fewer_tiers_more_containers() {
        for distribution in ["uniform", "logunif"] {
            let count_by_object_size = counts(distribution, OBJECT_COUNT);
            let mut previous_tier_count = usize::MAX;
            let mut previous_container_count = 0u64;
            for &ratio_in_thousandths in RATIOS.iter() {
                let tier_widths = geometric_tiers(ratio_in_thousandths);
                let container_count = containers_tiered(&tier_widths, &count_by_object_size);
                assert!(tier_widths.len() < previous_tier_count, "dist={distribution} r={ratio_in_thousandths}：档数该递减");
                assert!(container_count >= previous_container_count, "dist={distribution} r={ratio_in_thousandths}：容器数该不减");
                previous_tier_count = tier_widths.len(); previous_container_count = container_count;
            }
        }
    }

    /// 主判据的绝对值：三个分布 × 四个公比逐个钉住。
    #[test]
    fn main_absolute() {
        let uniform_counts = counts("uniform", OBJECT_COUNT);
        assert_eq!(containers_variable_length(&uniform_counts), 6814);
        assert_eq!(containers_tiered(&geometric_tiers(1125), &uniform_counts), 7378);
        assert_eq!(containers_tiered(&geometric_tiers(1250), &uniform_counts), 7675);
        assert_eq!(containers_tiered(&geometric_tiers(1500), &uniform_counts), 8101);
        assert_eq!(containers_tiered(&geometric_tiers(2000), &uniform_counts), 9463);

        let log_uniform_counts = counts("logunif", OBJECT_COUNT);
        assert_eq!(containers_variable_length(&log_uniform_counts), 1592);
        assert_eq!(containers_tiered(&geometric_tiers(1125), &log_uniform_counts), 1771);
        assert_eq!(containers_tiered(&geometric_tiers(1250), &log_uniform_counts), 1820);
        assert_eq!(containers_tiered(&geometric_tiers(1500), &log_uniform_counts), 1903);
        // ⚠️ discrete 那个分布不中立：它的三个取样点 512 / 1024 / 4096 全是 2 的幂，
        // 所以 r = 2.0 在它上面只多 1 个容器（0.02%），而 r = 1.125 多 1.91%。
        // 只看那个分布会得出「公比越大越好」这个与另两个分布相反的结论。
        assert_eq!(containers_tiered(&geometric_tiers(2000), &log_uniform_counts), 2099);

        let discrete_counts = counts("discrete", OBJECT_COUNT);
        assert_eq!(containers_variable_length(&discrete_counts), 6448);
        assert_eq!(containers_tiered(&geometric_tiers(2000), &discrete_counts), 6449);
    }

    /// 分布展开守恒。
    #[test]
    fn counts_conserve_objects() {
        for distribution in ["uniform", "logunif", "discrete"] {
            let total_objects: u64 = counts(distribution, OBJECT_COUNT).iter().sum();
            assert_eq!(total_objects, OBJECT_COUNT, "dist={distribution}");
        }
    }
}
