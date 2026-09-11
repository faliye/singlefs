//! E71：记账 key 的条目数与宽度 —— D5 已定项 1 的 key 形态欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - D5 已定项 1：key 是 `(统计量, 树 ID, 设备, 代)`。
//! - D5 已定项 4（2026-09-01 用户定案）：**统计量取九个**。逐字的九项与它们的维度：
//!   1 已分配字节（**带**设备维）、2 空闲字节（**带**）、3 不可回收字节（**带**）、
//!   4 待删除但意图未完成占用（不带，全池承诺量）、5 defer 队列待释放按代（**带**）、
//!   6 已承诺预留量（不带）、7 扩展点配额已用量（不带）、8 每树独占字节（不带）、9 每树共享字节（不带）。
//!   并逐字写：「**key 的维度元组不是每个统计量都用满**，
//!   不按树分的那几项用一个保留树 ID（取 0）当『全池 / 无归属』行。」
//!   以及：「⚠️ **E71 的公式现在算得出绝对值了**：`统计量数 × 树数 × 设备数 × K`，统计量数 = 9。」
//! - D5 已定项 2：`K = 根环槽总数 + 1`（保守口径）；每 checkpoint 的记账写次数是
//!   「`统计量数` 变成 `统计量数 × 2`（一写一删）」；E53 真设备实测的最坏回退是 1 ⇒ K 下界 2。
//! - D22 已定项 2：根环 **R = 3** 个区域，每区槽数 **S 住超级块，1..16 之间取值不碰任何一条边**
//!   ⇒ 槽总数 = 3S，保守 `K = 3S + 1`。
//! - D16 已定项 5：**`T_dirty` = 2 GiB**（2026-08-31 用户定案，明说是初值）。
//! - E54：「代取 checkpoint 号 + 增量丢弃 ≡ 只留最近 K 代」，且当时的公式
//!   `key 数 = 统计量数 × K` **少了维度组合那两段**。
//!
//! ## 判据（E71 正文跑前写死，跑完不许改）
//!
//! 1. **绝对值断言**：条目数必须恰好等于 `s × t × d × K`，不许只做臂间互比。
//! 2. 一次 checkpoint 的记账写字节数 ≤ D16 已定项 5 的 `T_dirty` 的 1%，
//!    否则判「记账自己就吃掉了脏数据预算」。
//! 3. 四个自变量各自单调：任一个加倍，条目数必须恰好加倍。
//!
//! ## 失败条款
//!
//! - **阳性对照**：把设备维去掉（回到 E54 当时的公式），条目数必须**下降 d 倍**；
//!   没下降说明维度根本没进 key，整轮作废。
//! - 若 s 取不到真值，结果一律带「按 s = N 算」的口径记录，不许写成绝对结论。
//!   ⇒ **s 现在取到真值了（9，D5 已定项 4），这一条不再适用。**
//!
//! ## ⚠️ 判据 1 与 3 说的那个公式，只对「维度用满」的读法成立
//!
//! `s × t × d × K` 把九个统计量**一律**乘上树维与设备维，而 D5 已定项 4 逐字说
//! 「**维度元组不是每个统计量都用满**」⇒ **那个公式是上界，不是条目数。**
//! 判据是在 D5 已定项 4 定案**之前**写的（正文当时逐字写着「绝对值今天算不出来」），
//! 所以本实验**照原样跑上界臂**（判据 1 / 3 / 阳性对照全部落在它身上），
//! **另加两条按分项计维的臂**，把差额量出来。**判据一个字没改。**
//!
//! ## ⚠️ 一处 D5 已定项 4 没定的东西，本实验不许替它定
//!
//! 「哪几个统计量**带树维**」D5 已定项 4 那张表**没有列**（它只列了带不带设备维）。
//! ⇒ 本实验按两种读法各跑一臂，**不挑一个当答案**：
//! 窄读（只有第 8、9 项按树分）与宽读（九项都按树分）。差额一并报出。
//!
//! ## 它答不了的
//!
//! 纯算术模型：没有记账树实现、没有 btree、没有 write buffer，文件操作 0 处。
//! **key 各段的字节宽度仓里一处都没定过**（D5 已定项 3 只定了 key 的形态，没定宽度），
//! 所以宽度是本实验的**假设**，按三档各算一次，不许把某一档的字节数当成结论。

use e7_index_bench::Emitter;

/// D5 已定项 4：统计量取九个。
const STATISTIC_COUNT: usize = 9;
/// D16 已定项 5：`T_dirty` = 2 GiB。
const DIRTY_BYTES_THRESHOLD: u64 = 2 * 1024 * 1024 * 1024;
/// 判据 2 的预算线：`T_dirty` 的 1%。
const ACCOUNTING_WRITE_BUDGET_BYTES: u64 = DIRTY_BYTES_THRESHOLD / 100;
/// D22 已定项 2：根环区域数 R = 3。
const RING_REGIONS: u64 = 3;

/// 九个统计量各自用不用得上两个维度。**次序与 D5 已定项 4 那张表逐行对应。**
/// `per_device` 那一列逐字来自该表；`per_tree` 那一列**该表没有**，见文件头的警示。
struct Statistic {
    name: &'static str,
    per_device: bool,
    /// 窄读：只有「每树独占 / 每树共享」按树分。
    per_tree_narrow: bool,
}

const TABLE: [Statistic; STATISTIC_COUNT] = [
    Statistic { name: "allocated",        per_device: true,  per_tree_narrow: false },
    Statistic { name: "free",             per_device: true,  per_tree_narrow: false },
    Statistic { name: "unreclaimable",    per_device: true,  per_tree_narrow: false },
    Statistic { name: "pending_delete",   per_device: false, per_tree_narrow: false },
    Statistic { name: "defer_pending",    per_device: true,  per_tree_narrow: false },
    Statistic { name: "reserved",         per_device: false, per_tree_narrow: false },
    Statistic { name: "ext_quota_used",   per_device: false, per_tree_narrow: false },
    Statistic { name: "per_tree_exclusive", per_device: false, per_tree_narrow: true },
    Statistic { name: "per_tree_shared",  per_device: false, per_tree_narrow: true },
];

/// 三条臂。**没有 `_ =>` 通配臂**：加第四条读法时这里编译不过。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 判据 1 / 3 / 阳性对照落在这条臂上：`s × t × d × K`，维度用满。
    UpperBound,
    /// 窄读：只有第 8、9 项按树分；设备维按 D5 已定项 4 那张表。
    PerStatisticNarrow,
    /// 宽读：九项都按树分；设备维同上。
    PerStatisticWide,
}

impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::UpperBound => "upper_bound",
            Arm::PerStatisticNarrow => "per_stat_narrow",
            Arm::PerStatisticWide => "per_stat_wide",
        }
    }
}

/// 一代里的条目数（key 组合数）。`with_device` 为 false 时去掉设备维——阳性对照用。
fn entries_per_generation(arm: Arm, tree_count: u64, device_count: u64, with_device: bool) -> u64 {
    let device_factor = |uses: bool| if with_device && uses { device_count } else { 1 };
    match arm {
        Arm::UpperBound => STATISTIC_COUNT as u64 * tree_count * device_factor(true),
        Arm::PerStatisticNarrow => TABLE
            .iter()
            .map(|statistic| if statistic.per_tree_narrow { tree_count } else { 1 } * device_factor(statistic.per_device))
            .sum(),
        Arm::PerStatisticWide => TABLE.iter().map(|statistic| tree_count * device_factor(statistic.per_device)).sum(),
    }
}

/// 记账树的总条目数：一代的条目数 × 保留的代数 K。
fn total_entries(arm: Arm, tree_count: u64, device_count: u64, kept_generations: u64, with_device: bool) -> u64 {
    entries_per_generation(arm, tree_count, device_count, with_device) * kept_generations
}

/// 一次 checkpoint 的记账写字节数。D5 已定项 2 逐字：写次数是 `统计量数 × 2`（一写一删）
/// ⇒ 触及的是**一代**的全部条目，各写一次、各删一次。K 不进这个式子。
fn checkpoint_bytes(arm: Arm, tree_count: u64, device_count: u64, key_width_bytes: u64, value_width_bytes: u64) -> u64 {
    2 * entries_per_generation(arm, tree_count, device_count, true) * (key_width_bytes + value_width_bytes)
}

/// 保守口径的 K：`根环槽总数 + 1 = R × S + 1`（D5 已定项 2 + D22 已定项 2）。
fn kept_generations_conservative(slots_per_region: u64) -> u64 {
    RING_REGIONS * slots_per_region + 1
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config stats={STATISTIC_COUNT} t_dirty={DIRTY_BYTES_THRESHOLD} budget={ACCOUNTING_WRITE_BUDGET_BYTES} \
             ring_regions={RING_REGIONS} model=arithmetic file_ops=0"
        ))
    );

    // 九项各自的维度用量，逐项报出来——差额的来源必须看得见。
    let statistics_with_device_dimension = TABLE.iter().filter(|statistic| statistic.per_device).count();
    let statistics_with_tree_dimension_narrow = TABLE.iter().filter(|statistic| statistic.per_tree_narrow).count();
    for statistic in TABLE.iter() {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=stat_dims stat={} per_device={} per_tree_narrow={}",
                statistic.name,
                u8::from(statistic.per_device),
                u8::from(statistic.per_tree_narrow)
            ))
        );
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=dim_counts stats={STATISTIC_COUNT} with_device={statistics_with_device_dimension} with_tree_narrow={statistics_with_tree_dimension_narrow}"
        ))
    );

    // ── K 的两个口径 ─────────────────────────────────────────────────
    for slots_per_region in [1u64, 4, 8, 16] {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=k_range slots_per_region={slots_per_region} ring_slots={} k_conservative={} k_measured=2",
                RING_REGIONS * slots_per_region,
                kept_generations_conservative(slots_per_region)
            ))
        );
    }

    // ── 主扫：三条臂 × (t, d, K) ────────────────────────────────────
    for tree_count in [1u64, 8, 64, 1024] {
        for device_count in [1u64, 2, 8, 64] {
            for kept_generations in [2u64, kept_generations_conservative(8)] {
                for arm in [Arm::UpperBound, Arm::PerStatisticNarrow, Arm::PerStatisticWide] {
                    let total_entry_count = total_entries(arm, tree_count, device_count, kept_generations, true);
                    let generation_entry_count = entries_per_generation(arm, tree_count, device_count, true);
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=entries arm={} t={tree_count} d={device_count} k={kept_generations} per_generation={generation_entry_count} \
                             total={total_entry_count}",
                            arm.tag()
                        ))
                    );
                }
            }
        }
    }

    // ── 判据 2：一次 checkpoint 的记账写字节 vs T_dirty 的 1% ────────────
    // key 宽度仓里没定过，按三档各算一次（假设，不是结论）。
    for (width_tag, key_width_bytes, value_width_bytes) in [("narrow", 14u64, 8u64), ("mid", 22, 8), ("wide", 34, 16)] {
        for tree_count in [1u64, 8, 64, 1024, 8192] {
            for device_count in [1u64, 8, 64] {
                for arm in [Arm::UpperBound, Arm::PerStatisticNarrow] {
                    let checkpoint_write_bytes = checkpoint_bytes(arm, tree_count, device_count, key_width_bytes, value_width_bytes);
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=ckpt_budget width={width_tag} key_w={key_width_bytes} val_w={value_width_bytes} \
                             arm={} t={tree_count} d={device_count} bytes={checkpoint_write_bytes} budget={ACCOUNTING_WRITE_BUDGET_BYTES} over={}",
                            arm.tag(),
                            u8::from(checkpoint_write_bytes > ACCOUNTING_WRITE_BUDGET_BYTES)
                        ))
                    );
                }
            }
        }
    }

    // ── 判据 3：四个自变量各自加倍，上界臂的条目数必须恰好加倍 ──────────────
    let baseline_total = total_entries(Arm::UpperBound, 64, 8, 25, true);
    for (doubled_variable, doubled_total) in [
        ("t", total_entries(Arm::UpperBound, 128, 8, 25, true)),
        ("d", total_entries(Arm::UpperBound, 64, 16, 25, true)),
        ("k", total_entries(Arm::UpperBound, 64, 8, 50, true)),
    ] {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=monotonic arm=upper_bound var={doubled_variable} base={baseline_total} doubled={doubled_total} \
                 exactly_2x={}",
                u8::from(doubled_total == 2 * baseline_total)
            ))
        );
    }
    // 同一个加倍试验在按分项计维的臂上**不成立**，这是结果不是 bug——报出来。
    let narrow_baseline_total = total_entries(Arm::PerStatisticNarrow, 64, 8, 25, true);
    let narrow_doubled_total = total_entries(Arm::PerStatisticNarrow, 128, 8, 25, true);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=monotonic arm=per_stat_narrow var=t base={narrow_baseline_total} doubled={narrow_doubled_total} \
             exactly_2x={} ratio_x1000={}",
            u8::from(narrow_doubled_total == 2 * narrow_baseline_total),
            narrow_doubled_total * 1000 / narrow_baseline_total
        ))
    );

    // ── 阳性对照：去掉设备维，上界臂必须恰好降 d 倍 ───────────────────────
    for arm in [Arm::UpperBound, Arm::PerStatisticNarrow, Arm::PerStatisticWide] {
        let with_device_total = total_entries(arm, 64, 8, 25, true);
        let without_device_total = total_entries(arm, 64, 8, 25, false);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=positive_control_drop_device arm={} with={with_device_total} without={without_device_total} \
                 ratio_x1000={} exactly_d={}",
                arm.tag(),
                with_device_total * 1000 / without_device_total,
                u8::from(with_device_total == 8 * without_device_total)
            ))
        );
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 的绝对值断言**：上界臂的条目数恰好等于 `s × t × d × K`。
    #[test]
    fn criterion1_absolute_upper_bound_formula() {
        // s=9, t=64, d=8, K=25 ⇒ 9 × 64 × 8 × 25 = 115_200
        assert_eq!(total_entries(Arm::UpperBound, 64, 8, 25, true), 115_200);
        assert_eq!(9 * 64 * 8 * 25, 115_200, "手算");
        // 另取一格，避免单点巧合：s=9, t=1, d=1, K=2 ⇒ 18
        assert_eq!(total_entries(Arm::UpperBound, 1, 1, 2, true), 18);
    }

    /// **绝对值断言**：九项里 4 项带设备维、2 项按窄读带树维。
    /// 这两个数直接决定上界与真实条目数的差额。
    #[test]
    fn absolute_dimension_counts_from_space_accounting_decision_item_4() {
        assert_eq!(STATISTIC_COUNT, 9);
        assert_eq!(TABLE.iter().filter(|statistic| statistic.per_device).count(), 4);
        assert_eq!(TABLE.iter().filter(|statistic| statistic.per_tree_narrow).count(), 2);
    }

    /// **绝对值断言**：一代的条目数，三条臂逐格手算。t=64, d=8。
    #[test]
    fn absolute_entries_per_generation_all_three_arms() {
        // 上界：9 × 64 × 8 = 4608
        assert_eq!(entries_per_generation(Arm::UpperBound, 64, 8, true), 4608);
        // 窄读：4 项带设备（各 8）+ 3 项都不带（各 1）+ 2 项按树（各 64）
        //      = 4×8 + 3×1 + 2×64 = 32 + 3 + 128 = 163
        assert_eq!(entries_per_generation(Arm::PerStatisticNarrow, 64, 8, true), 163);
        // 宽读：九项都按树 ⇒ 4 项 64×8 + 5 项 64×1 = 2048 + 320 = 2368
        assert_eq!(entries_per_generation(Arm::PerStatisticWide, 64, 8, true), 2368);
    }

    /// **判据 3**：上界臂上，t / d / K 各自加倍必须恰好加倍。
    #[test]
    fn criterion3_upper_bound_doubles_exactly() {
        let baseline_total = total_entries(Arm::UpperBound, 64, 8, 25, true);
        assert_eq!(total_entries(Arm::UpperBound, 128, 8, 25, true), 2 * baseline_total);
        assert_eq!(total_entries(Arm::UpperBound, 64, 16, 25, true), 2 * baseline_total);
        assert_eq!(total_entries(Arm::UpperBound, 64, 8, 50, true), 2 * baseline_total);
    }

    /// 同一个加倍在窄读臂上**不成立**——只有 2/9 项按树分。
    /// **这是结果，不是 bug**：钉住它，免得下次有人把上界当条目数。
    #[test]
    fn per_statistic_arm_does_not_double_with_tree_count() {
        let baseline_total = entries_per_generation(Arm::PerStatisticNarrow, 64, 8, true);
        let doubled_total = entries_per_generation(Arm::PerStatisticNarrow, 128, 8, true);
        assert_eq!(baseline_total, 163);
        assert_eq!(doubled_total, 4 * 8 + 3 + 2 * 128); // = 291
        assert_eq!(doubled_total, 291);
        assert!(doubled_total < 2 * baseline_total, "291 < 326");
    }

    /// **阳性对照**：去掉设备维，上界臂必须恰好降 d 倍。
    #[test]
    fn positive_control_dropping_device_dimension_divides_by_device_count() {
        let with_device_total = total_entries(Arm::UpperBound, 64, 8, 25, true);
        let without_device_total = total_entries(Arm::UpperBound, 64, 8, 25, false);
        assert_eq!(with_device_total, 8 * without_device_total, "去掉设备维必须恰好降 8 倍");
        assert_eq!(without_device_total, 9 * 64 * 25);
    }

    /// 阳性对照在按分项计维的臂上降得**少于** d 倍——只有 4/9 项带设备维。
    #[test]
    fn positive_control_drops_less_on_per_statistic_arm() {
        let with_device_total = entries_per_generation(Arm::PerStatisticNarrow, 64, 8, true);
        let without_device_total = entries_per_generation(Arm::PerStatisticNarrow, 64, 8, false);
        assert_eq!(with_device_total, 163);
        assert_eq!(without_device_total, 4 + 3 + 2 * 64); // 4 项各 1 + 3 项各 1 + 2 项各 64 = 135
        assert_eq!(without_device_total, 135);
        assert!(with_device_total < 8 * without_device_total, "163 远小于 1080");
    }

    /// **判据 2 的绝对值断言**：预算线是 2 GiB 的 1%。
    #[test]
    fn criterion2_absolute_budget_line() {
        assert_eq!(DIRTY_BYTES_THRESHOLD, 2_147_483_648);
        assert_eq!(ACCOUNTING_WRITE_BUDGET_BYTES, 21_474_836);
    }

    /// **判据 2 的过界点**：上界臂在 mid 宽度（key 22 + value 8 = 30 B）下，
    /// t=1024、d=64 时一次 checkpoint 要写 2 × 9 × 1024 × 64 × 30 = 35_389_440 B，
    /// **超预算 1.65 倍**。
    #[test]
    fn criterion2_upper_bound_blows_the_budget() {
        let upper_bound_checkpoint_bytes = checkpoint_bytes(Arm::UpperBound, 1024, 64, 22, 8);
        assert_eq!(upper_bound_checkpoint_bytes, 35_389_440);
        assert!(upper_bound_checkpoint_bytes > ACCOUNTING_WRITE_BUDGET_BYTES);
        // 窄读臂在同一格远在预算内：(4×64 + 3 + 2×1024) = 2307 个条目，× 2 × 30 = 138_420
        let narrow_checkpoint_bytes = checkpoint_bytes(Arm::PerStatisticNarrow, 1024, 64, 22, 8);
        assert_eq!(entries_per_generation(Arm::PerStatisticNarrow, 1024, 64, true), 2307);
        assert_eq!(narrow_checkpoint_bytes, 138_420);
        // 上界臂比窄读臂贵 255 倍——差额全部来自「维度用满」这个读法。
        assert_eq!(upper_bound_checkpoint_bytes / narrow_checkpoint_bytes, 255);
        assert!(narrow_checkpoint_bytes < ACCOUNTING_WRITE_BUDGET_BYTES);
    }

    /// **K 的两个口径**：D22 已定项 2 的 R=3、S∈1..16 ⇒ 保守 K 从 4 到 49；
    /// E53 真设备实测的最坏回退 1 ⇒ K = 2。
    #[test]
    fn absolute_kept_generations_range_from_ring_geometry() {
        assert_eq!(RING_REGIONS, 3);
        assert_eq!(kept_generations_conservative(1), 4);
        assert_eq!(kept_generations_conservative(8), 25);
        assert_eq!(kept_generations_conservative(16), 49);
    }

    /// E54 当时那个公式少了两段——按它算出来的数必须**小 t×d 倍**。
    #[test]
    fn e54_old_formula_undercounts_by_tree_count_times_device_count() {
        let e54_old_formula_total = STATISTIC_COUNT as u64 * 25; // 统计量数 × K
        let current_formula_total = total_entries(Arm::UpperBound, 64, 8, 25, true);
        assert_eq!(e54_old_formula_total, 225);
        assert_eq!(current_formula_total, e54_old_formula_total * 64 * 8);
    }

    /// 单一自变量取 1 时三条臂必须收敛到同一个数——没有树、没有设备，维度组合退化。
    #[test]
    fn all_arms_agree_when_dimensions_are_one() {
        for arm in [Arm::UpperBound, Arm::PerStatisticNarrow, Arm::PerStatisticWide] {
            assert_eq!(entries_per_generation(arm, 1, 1, true), 9, "{arm:?}");
        }
    }
}
