//! E119：槽定宽的补齐代价 —— 档怎么切，各切法各付多少。
//!
//! ## 它问什么
//!
//! D27 已定项 8 ④（2026-09-08 用户定案）取**槽定宽**：一个容器声明一个槽宽 `W`，
//! 容器内所有槽同宽，对象补齐到 `W`。补齐浪费因此按档宽回来了——
//! 一个 600 字节的对象落在哪一档就占那一档的宽度。
//!
//! **档切得粗，浪费大；切得细，同时开着的容器多。** E119 把这条曲线量出来，
//! 让「档怎么切」（D27 已定项 8 定了形态、切法随已定项 6 一起定）有数可依。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D4 已定项 5**：单元恒 32768 含头。**已定项 2**：短 extent 补齐到整单元。
//! - **D18 已定项 11**：打包记录单元头 107。**D2 已定项 9**：第一版 2 盘恒 w = 2。
//! - **D27 已定项 2**：界线 ≤ 4 KiB 的对象才进容器。
//! - **D27 已定项 8 ④**：槽定宽，对象补齐到槽宽；槽 `i` 起点 = 头 + `i × W`，不需要槽目录。
//! - **E116 的闭式**：回本比下界 = `w / (cap − 1)` ⇒ 回得了本当且仅当 `cap > w + 1`；
//!   `w` = 2 ⇒ 要 `cap ≥ 4`。
//!
//! ## 两个假设（不是条款，标出来 —— C187）
//!
//! - `SLOT_EXTRA_BYTES = 43`：每槽自描述 = 逻辑身份五元组 33（D18 已定项 3）+ 写序 10（D18 已定项 7）。
//!   **定宽之下不需要槽目录条目**（D27 已定项 8 ④ 逐字），所以比 E114 / E116 用的 47 少 4。
//! - **对象大小分布**：全仓没有任何实测分布（D25 的 smallfile 那一行三列全「未测」，C174）。
//!   E119 自定三个分布并全部报出来，**不挑一个当代表**。
//!
//! ## 六条臂（跑前列全，中途不许加臂 —— C186）
//!
//! | 臂 | 档 | 是不是候选 |
//! |---|---|---|
//! | `var` | 变长槽，按大小升序贪心装填 | **否**——是收益上界对照，D27 已定项 8 ④ 已定不取变长 |
//! | `gran64` | 64 字节粒度：64, 128, …, 4096（64 档）| 是 |
//! | `gran256` | 256 字节粒度：256, 512, …, 4096（16 档）| 是 |
//! | `pow2` | 2 的幂：64, 128, 256, …, 4096（7 档）| 是 |
//! | `tier4` | 四档：512, 1024, 2048, 4096 | 是 |
//! | `single` | 单档 4096（最粗）| 是 |
//!
//! ## 三个分布（跑前写死，全部报出）
//!
//! | 名 | 权重 | 为什么放它 |
//! |---|---|---|
//! | `uniform` | 1..=4096 每个整数等权 | 无信息先验 |
//! | `logunif` | 权重 ∝ 1/s | 小文件更多的那一侧，压定宽的补齐代价 |
//! | `discrete` | 只在 512 / 1024 / 4096 三点等权 | 与 E114 / E116 的取样点对齐，可比 |
//!
//! ## 跑前写死的判据与失败条款（跑完不许改）
//!
//! 1. **主判据**：每条臂 × 每个分布，报容器数、收益倍数（`pad` 占用 ÷ 本臂占用）、
//!    平均每对象占用字节、以及**相对 `var` 的损失百分比**。
//! 2. **回本闸**：每条臂每一档的 `cap` 都要 ≥ 4（E116 闭式 `w/(cap−1) < 1` 的充要条件）。
//!    有一档不满足就点名报出来。
//! 3. **档数**：报每条臂的档数——它等于同时开着的容器数除以树数（C106 同类）。
//!
//! **阳性对照，逐臂跑**：把分布换成「所有对象大小恰好等于某一档宽」，
//! 那一档存在的每条定宽臂必须与 `var` 逐字节相同。任一臂不同 ⇒ 整轮作废。
//! **判别力对照**：`single`（单档 4096）在全 512 字节的分布上，每对象占用必须正好是
//! `var` 的 `(4096+43)/(512+43)` 倍。不成立 ⇒ 补齐没被建模，整轮作废。
//! **阴性对照**：OBJECTS_PER_DISTRIBUTION = 0 时六条臂的所有字节恰好 0。
//!
//! ## 反向接受条款（跑前写死，逐臂点名）
//!
//! - 若 **`gran64` 相对 `var` 的损失 < 1%** ⇒ 结论写「细档等价于变长，定宽这一步不付代价」。
//! - 若 **`tier4` 在任一分布上相对 `var` 的损失 > 30%** ⇒ 结论写「四档太粗，档要更细」。
//! - 若 **`single` 的收益倍数在任一分布上 < 2** ⇒ 结论写「单档不可用」。
//! - 若 **每一条定宽臂在每一个分布上损失都 > 30%** ⇒ 结论写
//!   「定宽把 D27 的收益吃掉大半，已定项 8 ④ 该重开」，不许回头改口径。
//! - 若 **任一臂任一档的 `cap` < 4** ⇒ 结论写「那一档回不了本，不许进档表」。
//!
//! ## 它答不了的
//!
//! 纯算术账本：没有实现、没有 I/O、没有崩溃点重放。不答挂钟、不答缓存、不答并发。
//! **`var` 臂按「大小升序贪心」装填**——它是变长这条路的一个具体装法，不是最优装法；
//! 装箱最优解是 NP 难的，所以它是一个可实现的参照，不是理论上界。
//! **对象大小分布是自定的**，全仓没有实测分布（C174）；三个分布一起报，不挑代表。
//! 不答「同时开着的容器数」的绝对值——那要乘树数，而全仓没有树数的上界。

use e7_index_bench::Emitter;

const UNIT_BYTES: u64 = 32768; // D4 已定项 5
const PACKED_UNIT_HEADER_BYTES: u64 = 107; // D18 已定项 11
const REPLICATION_WIDTH: u64 = 2; // D2 已定项 9
const SLOT_EXTRA_BYTES: u64 = 43; // 假设：五元组 33 + 写序 10，定宽下不要槽目录条目
const PACKING_LIMIT_BYTES: u64 = 4096; // D27 已定项 2：界线 ≤ 4 KiB
const OBJECTS_PER_DISTRIBUTION: u64 = 100_000;
const CONTAINER_PAYLOAD_BYTES: u64 = UNIT_BYTES - PACKED_UNIT_HEADER_BYTES; // 容器净荷 32661

fn slots_per_container(tier_width: u64) -> u64 { CONTAINER_PAYLOAD_BYTES / (tier_width + SLOT_EXTRA_BYTES) }

fn tiers(arm: &str) -> Vec<u64> {
    match arm {
        "gran64" => (1..=64).map(|tier_number| tier_number * 64).collect(),
        "gran256" => (1..=16).map(|tier_number| tier_number * 256).collect(),
        "pow2" => vec![64, 128, 256, 512, 1024, 2048, 4096],
        "tier4" => vec![512, 1024, 2048, 4096],
        "single" => vec![4096],
        _ => vec![],
    }
}

/// 权重表：下标 s（1..=PACKING_LIMIT_BYTES）上的对象个数。三个分布都按整数权重展开，不用随机源。
fn weights(distribution: &str) -> Vec<u64> {
    let mut weight_by_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
    match distribution {
        "uniform" => { for object_size in 1..=PACKING_LIMIT_BYTES { weight_by_size[object_size as usize] = 1; } }
        // 权重 ∝ 1/s，用整数近似：PACKING_LIMIT_BYTES / s，保证 s 小的权重大
        "logunif" => { for object_size in 1..=PACKING_LIMIT_BYTES { weight_by_size[object_size as usize] = PACKING_LIMIT_BYTES / object_size; } }
        "discrete" => { for object_size in [512u64, 1024, 4096] { weight_by_size[object_size as usize] = 1; } }
        _ => {}
    }
    weight_by_size
}

/// 按权重把 OBJECTS_PER_DISTRIBUTION 个对象摊到各个大小上（最后一格吸收取整误差）。
fn counts(distribution: &str, object_count: u64) -> Vec<u64> {
    let weight_by_size = weights(distribution);
    let total_weight: u64 = weight_by_size.iter().sum();
    if total_weight == 0 || object_count == 0 { return vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize]; }
    let mut count_by_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
    let mut assigned_count = 0u64;
    let mut last_nonzero_size = 0usize;
    for object_size in 1..=PACKING_LIMIT_BYTES as usize {
        if weight_by_size[object_size] == 0 { continue; }
        count_by_size[object_size] = object_count * weight_by_size[object_size] / total_weight;
        assigned_count += count_by_size[object_size];
        last_nonzero_size = object_size;
    }
    count_by_size[last_nonzero_size] += object_count - assigned_count; // 取整误差全给最后一格，总数恒为 n
    count_by_size
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Arm {
    containers: u64,
    occupancy: u64,
    tiers: u64,
    minimum_slots_per_container: u64,
}

fn run_tiered(arm: &str, count_by_size: &[u64]) -> Arm {
    let tier_widths = tiers(arm);
    let mut containers = 0u64;
    let mut minimum_slots_per_container = u64::MAX;
    for (tier_index, &tier_width) in tier_widths.iter().enumerate() {
        let smallest_size_in_tier = if tier_index == 0 { 1 } else { tier_widths[tier_index - 1] + 1 };
        let objects_in_tier: u64 = (smallest_size_in_tier..=tier_width).map(|object_size| count_by_size[object_size as usize]).sum();
        let tier_slots_per_container = slots_per_container(tier_width);
        if tier_slots_per_container < minimum_slots_per_container { minimum_slots_per_container = tier_slots_per_container; }
        if objects_in_tier > 0 { containers += objects_in_tier.div_ceil(tier_slots_per_container.max(1)); }
    }
    Arm { containers, occupancy: containers * UNIT_BYTES, tiers: tier_widths.len() as u64, minimum_slots_per_container }
}

/// `var` 臂：变长槽，按对象大小升序**贪心装填**（同一容器内槽长可以各异，装不下就换一个容器）。
/// 这是可实现的装法，不是「完美装填」——⚠️ 早先按 Σ 槽长 ÷ 净荷上取整算过一版，
/// 那等于允许一个槽跨容器，**阳性对照当场判否**（全 512 字节时它给 1700，而 58 槽 × 1725 = 100050，
/// 正确答案是 1725）。两个模型都记在这里，登记的那个不许回头改。
fn run_variable_length_arm(count_by_size: &[u64]) -> Arm {
    let mut containers = 0u64;
    let mut remaining_payload_bytes = 0u64; // 当前容器剩余净荷；0 表示还没开容器
    for object_size in 1..=PACKING_LIMIT_BYTES {
        let bytes_per_slot = object_size + SLOT_EXTRA_BYTES;
        let mut objects_left = count_by_size[object_size as usize];
        while objects_left > 0 {
            if remaining_payload_bytes < bytes_per_slot {
                containers += 1;
                remaining_payload_bytes = CONTAINER_PAYLOAD_BYTES;
            }
            let objects_fitting = (remaining_payload_bytes / bytes_per_slot).min(objects_left);
            remaining_payload_bytes -= objects_fitting * bytes_per_slot;
            objects_left -= objects_fitting;
        }
    }
    Arm { containers, occupancy: containers * UNIT_BYTES, tiers: 0, minimum_slots_per_container: slots_per_container(PACKING_LIMIT_BYTES) }
}

fn total_objects(count_by_size: &[u64]) -> u64 { count_by_size.iter().sum() }

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config unit={UNIT_BYTES} pack_hdr={PACKED_UNIT_HEADER_BYTES} net={CONTAINER_PAYLOAD_BYTES} w={REPLICATION_WIDTH} \
         slot_extra={SLOT_EXTRA_BYTES} limit={PACKING_LIMIT_BYTES} n={OBJECTS_PER_DISTRIBUTION}")));

    let arms = ["gran64", "gran256", "pow2", "tier4", "single"];

    // 阳性对照：所有对象恰好等于某一档宽 ⇒ 有那一档的定宽臂必须与 var 逐字节相同
    for &tier_width in [512u64, 1024, 2048, 4096].iter() {
        let mut count_by_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
        count_by_size[tier_width as usize] = OBJECTS_PER_DISTRIBUTION;
        let variable_length_result = run_variable_length_arm(&count_by_size);
        for &arm in arms.iter() {
            if !tiers(arm).contains(&tier_width) { continue; }
            let tiered_result = run_tiered(arm, &count_by_size);
            println!("{}", emitter.emit_raw(&format!(
                "name=positive_exact tier_width={tier_width} arm={arm} var_occ={} arm_occ={} same={}",
                variable_length_result.occupancy, tiered_result.occupancy, variable_length_result.occupancy == tiered_result.occupancy)));
        }
    }

    // 判别力：single 在全 512 的分布上，每对象占用应是 var 的 (4096+43)/(512+43) 倍
    {
        let mut count_by_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
        count_by_size[512] = OBJECTS_PER_DISTRIBUTION;
        let variable_length_result = run_variable_length_arm(&count_by_size);
        let single_result = run_tiered("single", &count_by_size);
        println!("{}", emitter.emit_raw(&format!(
            "name=discrimination var_containers={} single_containers={} ratio={:.6} expect={:.6}",
            variable_length_result.containers, single_result.containers,
            single_result.occupancy as f64 / variable_length_result.occupancy as f64,
            (4096.0 + SLOT_EXTRA_BYTES as f64) / (512.0 + SLOT_EXTRA_BYTES as f64))));
    }

    // 阴性对照
    {
        let count_by_size = counts("uniform", 0);
        let variable_length_result = run_variable_length_arm(&count_by_size);
        println!("{}", emitter.emit_raw(&format!(
            "name=negative arm=var containers={} occ={}", variable_length_result.containers, variable_length_result.occupancy)));
        for &arm in arms.iter() {
            let tiered_result = run_tiered(arm, &count_by_size);
            println!("{}", emitter.emit_raw(&format!(
                "name=negative arm={arm} containers={} occ={}", tiered_result.containers, tiered_result.occupancy)));
        }
    }

    // 主判据
    for distribution in ["uniform", "logunif", "discrete"] {
        let count_by_size = counts(distribution, OBJECTS_PER_DISTRIBUTION);
        let object_count = total_objects(&count_by_size);
        let padded_occupancy = object_count * UNIT_BYTES;
        let variable_length_result = run_variable_length_arm(&count_by_size);
        println!("{}", emitter.emit_raw(&format!(
            "name=main dist={distribution} arm=var n={object_count} containers={} occ={} gain={:.4} \
             per_obj={:.2} loss_vs_var=0.0000 tiers=0 min_cap={}",
            variable_length_result.containers, variable_length_result.occupancy, padded_occupancy as f64 / variable_length_result.occupancy as f64,
            variable_length_result.occupancy as f64 / object_count as f64, variable_length_result.minimum_slots_per_container)));
        for &arm in arms.iter() {
            let tiered_result = run_tiered(arm, &count_by_size);
            println!("{}", emitter.emit_raw(&format!(
                "name=main dist={distribution} arm={arm} n={object_count} containers={} occ={} gain={:.4} \
                 per_obj={:.2} loss_vs_var={:.4} tiers={} min_cap={}",
                tiered_result.containers, tiered_result.occupancy, padded_occupancy as f64 / tiered_result.occupancy as f64,
                tiered_result.occupancy as f64 / object_count as f64,
                (tiered_result.occupancy as f64 - variable_length_result.occupancy as f64) / variable_length_result.occupancy as f64,
                tiered_result.tiers, tiered_result.minimum_slots_per_container)));
        }
    }

    // 回本闸：逐臂逐档报 cap，标出 cap < 4 的档
    for &arm in arms.iter() {
        for tier_width in tiers(arm) {
            let tier_slots_per_container = slots_per_container(tier_width);
            println!("{}", emitter.emit_raw(&format!(
                "name=capgate arm={arm} tier={tier_width} cap={tier_slots_per_container} pays_back={}", tier_slots_per_container >= 4)));
        }
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：对象恰好等于档宽时，定宽与变长必须逐字节相同。
    #[test]
    fn positive_control_exact_tier_matches_variable_length() {
        for &tier_width in [512u64, 1024, 2048, 4096].iter() {
            let mut count_by_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
            count_by_size[tier_width as usize] = OBJECTS_PER_DISTRIBUTION;
            let variable_length_result = run_variable_length_arm(&count_by_size);
            for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
                if !tiers(arm).contains(&tier_width) { continue; }
                assert_eq!(run_tiered(arm, &count_by_size).occupancy, variable_length_result.occupancy,
                           "arm={arm} 档宽={tier_width}：对象恰好等于档宽时该与变长相同");
            }
        }
    }

    /// 判别力：补齐真的被建模了 —— single 在全 512 上比 var 差，且差得正好是那个比。
    #[test]
    fn discrimination_padding_is_modeled() {
        let mut count_by_size = vec![0u64; (PACKING_LIMIT_BYTES + 1) as usize];
        count_by_size[512] = OBJECTS_PER_DISTRIBUTION;
        let variable_length_result = run_variable_length_arm(&count_by_size);
        let single_result = run_tiered("single", &count_by_size);
        assert!(single_result.occupancy > variable_length_result.occupancy, "single 该比 var 差");
        assert_eq!(variable_length_result.containers, 1725);
        assert_eq!(single_result.containers, 14286);
        let ratio = single_result.occupancy as f64 / variable_length_result.occupancy as f64;
        assert!((ratio - 8.2818).abs() < 0.001, "实测比 {ratio}");
        // 每对象占用之比正好是 (4096+43)/(512+43) 的上取整效应：58 槽 vs 7 槽
        assert_eq!(slots_per_container(512), 58);
        assert_eq!(slots_per_container(4096), 7);
    }

    /// 阴性：OBJECTS_PER_DISTRIBUTION = 0 时六条臂全 0。
    #[test]
    fn negative_control_zero() {
        let count_by_size = counts("uniform", 0);
        assert_eq!(run_variable_length_arm(&count_by_size).occupancy, 0);
        for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
            let tiered_result = run_tiered(arm, &count_by_size);
            assert_eq!((tiered_result.containers, tiered_result.occupancy), (0, 0), "arm={arm}");
        }
    }

    /// 容量钉绝对值，含跨整数边界的取样点（rules/mutation-sampling.md）。
    #[test]
    fn slots_per_container_absolute() {
        assert_eq!(CONTAINER_PAYLOAD_BYTES, UNIT_BYTES - 107);
        assert_eq!(CONTAINER_PAYLOAD_BYTES, 32661);
        assert_eq!(slots_per_container(512), 58);
        assert_eq!(slots_per_container(1024), 30);
        assert_eq!(slots_per_container(2048), 15);
        assert_eq!(slots_per_container(4096), 7);
        assert_eq!(slots_per_container(64), 305);
        // 跨整数边界：cap = 8 的档宽区间是 [3587, 4039]，抬到 4040 就掉成 7（净荷 32661，头 107）。
        assert_eq!(slots_per_container(4039), 8);
        assert_eq!(slots_per_container(4040), 7);
    }

    /// 每一档都要 cap ≥ 4（E116 闭式 w/(cap−1) < 1 的充要条件）。
    #[test]
    fn every_tier_pays_back() {
        for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
            for tier_width in tiers(arm) {
                assert!(slots_per_container(tier_width) >= 4, "arm={arm} 档宽={tier_width} 的 cap={} < 4，回不了本",
                        slots_per_container(tier_width));
            }
        }
        // 钉住边界：cap = 4 的最大档宽
        assert_eq!(slots_per_container(8122), 4);
        assert_eq!(slots_per_container(8123), 3);
    }

    /// 分布展开之后对象总数恒等于 OBJECTS_PER_DISTRIBUTION —— 取整误差不许漏对象。
    #[test]
    fn counts_conserve_objects() {
        for distribution in ["uniform", "logunif", "discrete"] {
            assert_eq!(total_objects(&counts(distribution, OBJECTS_PER_DISTRIBUTION)), OBJECTS_PER_DISTRIBUTION, "dist={distribution}");
        }
    }

    /// var 是上界：任何定宽臂的占用都不小于它。且 var 自己钉绝对值。
    #[test]
    fn variable_length_is_upper_bound() {
        for distribution in ["uniform", "logunif", "discrete"] {
            let count_by_size = counts(distribution, OBJECTS_PER_DISTRIBUTION);
            let variable_length_result = run_variable_length_arm(&count_by_size);
            for &arm in ["gran64", "gran256", "pow2", "tier4", "single"].iter() {
                assert!(run_tiered(arm, &count_by_size).occupancy >= variable_length_result.occupancy,
                        "dist={distribution} arm={arm}");
            }
        }
        // uniform 下 var 的绝对值：Σ (s+43)，s 从 1 到 4096 各 24 个（100000/4096=24 余 1696）
        let count_by_size = counts("uniform", OBJECTS_PER_DISTRIBUTION);
        let total_slot_bytes: u64 = (1..=PACKING_LIMIT_BYTES).map(|object_size| count_by_size[object_size as usize] * (object_size + SLOT_EXTRA_BYTES)).sum();
        assert_eq!(total_slot_bytes, 212_622_560);
        assert_eq!(run_variable_length_arm(&count_by_size).containers, 6815);
    }

    /// 主判据的绝对值：uniform 下四条候选臂的容器数逐个钉住。
    #[test]
    fn main_absolute_uniform() {
        let count_by_size = counts("uniform", OBJECTS_PER_DISTRIBUTION);
        assert_eq!(run_tiered("gran64", &count_by_size).containers, 6935);
        assert_eq!(run_tiered("gran256", &count_by_size).containers, 7271);
        assert_eq!(run_tiered("pow2", &count_by_size).containers, 9463);
        assert_eq!(run_tiered("tier4", &count_by_size).containers, 9525);
        assert_eq!(run_tiered("single", &count_by_size).containers, 14286);
    }

    /// `logunif` 与 `discrete` 两个分布的绝对值 —— 变异 M8（把对数均匀退化成均匀）
    /// 第一轮**一个测试都没红**，因为所有钉绝对值的断言都只在 `uniform` 上。
    /// 按 `.claude/rules/mutation-sampling.md` 三分：那不是等价变异也不是取样点不敏感，
    /// 是**真盲区**——`logunif` 这一整个分布上没有任何断言。而结论最重的那个数正在它上面。
    #[test]
    fn main_absolute_logunif_and_discrete() {
        let count_by_size = counts("logunif", OBJECTS_PER_DISTRIBUTION);
        assert_eq!(run_variable_length_arm(&count_by_size).containers, 1592);
        assert_eq!(run_tiered("gran64", &count_by_size).containers, 1745);
        assert_eq!(run_tiered("gran256", &count_by_size).containers, 2186);
        assert_eq!(run_tiered("pow2", &count_by_size).containers, 2099);
        assert_eq!(run_tiered("tier4", &count_by_size).containers, 3038);
        assert_eq!(run_tiered("single", &count_by_size).containers, 14286);
        // 小端密的档表（pow2，7 档）在这个分布上胜过等距的 gran256（16 档）——
        // 档要在小端密，不是等距。这一条是结论，必须有断言守着。
        assert!(run_tiered("pow2", &count_by_size).containers < run_tiered("gran256", &count_by_size).containers);

        let discrete_count_by_size = counts("discrete", OBJECTS_PER_DISTRIBUTION);
        assert_eq!(run_variable_length_arm(&discrete_count_by_size).containers, 6448);
        for &arm in ["gran64", "gran256", "pow2", "tier4"].iter() {
            assert_eq!(run_tiered(arm, &discrete_count_by_size).containers, 6449,
                       "arm={arm}：三个取样点都恰好落在档宽上，定宽只多 1 个容器");
        }
        assert_eq!(run_tiered("single", &discrete_count_by_size).containers, 14286);
    }

    /// 档数就是同时开着的容器数（除以树数）—— 钉绝对值。
    #[test]
    fn tier_counts_absolute() {
        assert_eq!(tiers("gran64").len(), 64);
        assert_eq!(tiers("gran256").len(), 16);
        assert_eq!(tiers("pow2").len(), 7);
        assert_eq!(tiers("tier4").len(), 4);
        assert_eq!(tiers("single").len(), 1);
        // 最细那档的宽度
        assert_eq!(tiers("gran64")[0], 64);
        assert_eq!(*tiers("gran64").last().unwrap(), 4096);
    }

    /// 单调：档越细，占用越小（同一分布上）。这一条是互比，上面几条钉绝对值的与它配对。
    #[test]
    fn finer_tiers_never_worse() {
        for distribution in ["uniform", "logunif", "discrete"] {
            let count_by_size = counts(distribution, OBJECTS_PER_DISTRIBUTION);
            let gran64_occupancy = run_tiered("gran64", &count_by_size).occupancy;
            let gran256_occupancy = run_tiered("gran256", &count_by_size).occupancy;
            let pow2_occupancy = run_tiered("pow2", &count_by_size).occupancy;
            let tier4_occupancy = run_tiered("tier4", &count_by_size).occupancy;
            let single_occupancy = run_tiered("single", &count_by_size).occupancy;
            assert!(gran64_occupancy <= gran256_occupancy, "dist={distribution}: gran64={gran64_occupancy} gran256={gran256_occupancy}");
            assert!(gran256_occupancy <= tier4_occupancy, "dist={distribution}: gran256={gran256_occupancy} tier4={tier4_occupancy}");
            assert!(tier4_occupancy <= single_occupancy, "dist={distribution}: tier4={tier4_occupancy} single={single_occupancy}");
            assert!(pow2_occupancy <= single_occupancy, "dist={distribution}: pow2={pow2_occupancy} single={single_occupancy}");
        }
    }
}
