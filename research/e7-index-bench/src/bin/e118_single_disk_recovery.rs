//! # E118 单盘自恢复：链臂 / ditto 臂 / 无冗余臂
//!
//! **问的是** D18 已定项 4（单盘自恢复的形态）。该项当时未定；2026-09-01 三方论证之后
//! 写下了**六条开线前必须还的举证**，本实验逐条还。
//!
//! ## 形态定义（举证 1：写不出偏移与宽度即红）
//!
//! **链臂显式取「同批链」，不取版本链。** 版本链已被 D18 否决清单否掉两次
//! （「子块记录当前父节点物理地址」判「时序上不可能：COW 是子先写父后写」）。
//! 同批链 = 每个单元的头里带**同一次发布中分配顺序上的前后邻居的落点**：
//! 落点粒度 16384（D3 已定项 7），单盘不需要设备维 ⇒ 槽号 6 字节 × 2 = **12 字节**，
//! 住类身份段（位置随实现定，宽度定死 12）。谁维护：发布路径，写头时就填。
//!
//! **ditto 臂取 ZFS 的真实形态**（举证 3：对照臂不许是稻草人）：
//! 数据集元数据 2 份、全池元数据 3 份，副本放置**至少隔 1/8 盘**，
//! 用户数据 1 份。ZFS 自陈总开销约 2%，因为元数据本来只占 1.5–2%。
//! ⚠️ **这是外部实现，只能用来提出形态与指出该测哪条路径**
//! （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做，是线索不是证据」）。
//! 与本工程的一处已知差异：ZFS 的间接块是纯指针数组，本工程的索引节点带 key 区间
//! （D18 已定项 2），所以元数据占比不一定相同——本实验把占比当参数扫。
//!
//! **无冗余臂**：每个单元一份，什么都不加。
//!
//! ## 故障模型
//!
//! 盘 = `DISK_SLOTS` 个 16 KiB 落点槽。故障 = **一段连续区间整个毁掉**
//! （对齐 ZFS 作者那次破坏性验证「抹掉分区前 1 GB」）。区间起点扫两处：盘首与盘中。
//! 一个单元**恢复得出** ⟺ 它至少有一份副本落在未毁区。
//!
//! ## 跑前写死的判据与失败条款
//!
//! 1. 三条臂在同一组故障上逐格报「恢复出多少元数据单元 / 数据单元」与「额外开销字节」。
//! 2. **阳性对照逐臂跑**：故障长度 = 0 时三条臂的恢复数必须**都等于**总单元数。
//!    任一臂不满足 ⇒ 整轮作废。
//! 3. **阴性对照**：故障 = 整盘 ⇒ 三条臂恢复数都是 0。
//! 4. **举证 5，钉绝对值**：链臂毁掉恰好一个单元之后，恢复数必须**恰好** = 未毁时 − 1。
//!    若出现「断点之后全丢」，说明链的形态选错了，如实记，不许改判据。
//! 5. **举证 2**：链臂相对 D3 已定项 3 的分配记录**净增**了什么——
//!    分配记录已能答「落点记为已分配却读不出合法单元」⇒ 存在性证据已经有。
//!    本实验把「检测到但恢复不出」单独计数，让净增量可见。
//! 6. **判别力**：三条臂必须至少在一个故障点上给出不同的恢复数；全格相同即装置分不开它们。
//!
//! **失败条款**：若链臂的恢复数在**所有**故障点上都等于无冗余臂，
//! 如实记「链臂买到的是检测不是恢复」，**不许**为了让它好看而改判据或改形态。
//!
//! ## 它答不了的
//!
//! 纯计数模型：没有真实 I/O、没有崩溃点重放、不建模坏道的空间相关性
//! （真实盘的坏区不是均匀随机的）。不答「元数据占比在本工程是多少」——那当参数扫。
//! 确定性模型，**跑 N 轮说明的是没有隐藏状态，不是统计上稳定**。

/// D3 已定项 7：落点粒度 16384 字节。
const GRAIN_BYTES: u64 = 16384;
/// D4 已定项 1：元数据侧 16 KiB、数据侧 32 KiB。数据单元占两个槽。
const META_UNIT_BYTES: u64 = 16384;
const DATA_UNIT_BYTES: u64 = 32768;
/// 4 GiB 盘 = 262144 个 16 KiB 槽。
const DISK_SLOTS: u64 = 262_144;
/// 同批链：前后邻居各一个槽号 6 字节。
const CHAIN_BYTES: u64 = 6 + 6;
/// ZFS 的放置政策：副本至少隔 1/8 盘。
const DITTO_SPREAD: u64 = DISK_SLOTS / 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    NoRedundancy,
    Ditto,
    Chain,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::NoRedundancy => "none",
            Arm::Ditto => "ditto",
            Arm::Chain => "chain",
        }
    }
    /// 一个元数据单元有几份。**没有通配臂**：加一种臂编译器会报。
    fn meta_copies(self, is_pool_wide: bool) -> u64 {
        match self {
            Arm::NoRedundancy | Arm::Chain => 1,
            Arm::Ditto => {
                if is_pool_wide {
                    3
                } else {
                    2
                }
            }
        }
    }
    /// 数据单元恒 1 份（ZFS 的 ditto 也只罩元数据）。
    fn data_copies(self) -> u64 {
        match self {
            Arm::NoRedundancy | Arm::Chain | Arm::Ditto => 1,
        }
    }
}

/// 第 `copy_index` 份副本落在哪个槽：第 0 份在原位，其后每份再隔 1/8 盘。
fn copy_slot(base_slot: u64, copy_index: u64) -> u64 {
    (base_slot + copy_index * DITTO_SPREAD) % DISK_SLOTS
}

/// 故障形态。**连续区间对链臂是最不利的取样点**（前后邻居一起死），
/// 只用它比对等于给链臂设了个稻草人故障模型 ⇒ 另加散点一档，每 `period_in_slots` 个槽毁一个。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fault {
    /// 一段连续区间整个毁掉（对齐 ZFS 那次「抹掉分区前 1 GB」）。
    Span { start_slot: u64, length_in_slots: u64 },
    /// 每 `period_in_slots` 个槽毁一个，毁掉的总数与同规模的 Span 相同。
    Scatter { period_in_slots: u64 },
}

impl Fault {
    fn name(self) -> &'static str {
        match self {
            Fault::Span { .. } => "span",
            Fault::Scatter { .. } => "scatter",
        }
    }
    fn destroyed(self, slot: u64) -> bool {
        match self {
            Fault::Span { start_slot, length_in_slots } => slot >= start_slot && slot < start_slot + length_in_slots,
            Fault::Scatter { period_in_slots } => period_in_slots != 0 && slot % period_in_slots == 0,
        }
    }
}

struct Layout {
    /// 元数据单元的基准槽（全池元数据在前）。
    meta_base: Vec<u64>,
    pool_wide_count: usize,
    data_base: Vec<u64>,
}

/// 元数据占比 `meta_permille`（千分比）、全池元数据占元数据的 `pool_permille`。
fn build_layout(meta_permille: u64, pool_permille: u64) -> Layout {
    // 盘上先铺元数据槽，再铺数据槽（数据单元占两个槽）。
    let meta_slots = DISK_SLOTS * meta_permille / 1000;
    let data_slots = DISK_SLOTS - meta_slots;
    let meta_unit_count = meta_slots;
    let data_unit_count = data_slots / 2;
    let pool_wide_count = (meta_unit_count * pool_permille / 1000) as usize;
    Layout {
        meta_base: (0..meta_unit_count).collect(),
        pool_wide_count,
        data_base: (0..data_unit_count).map(|data_index| meta_slots + data_index * 2).collect(),
    }
}

struct Outcome {
    recovered_meta: u64,
    recovered_data: u64,
    /// 举证 2：毁了但**被邻居指认出来**的单元数（检测到，恢复不出）。
    detected_only: u64,
    overhead_bytes: u64,
}

fn run_arm_under_fault(arm: Arm, layout: &Layout, fault: Fault) -> Outcome {
    let mut recovered_meta = 0;
    for (meta_index, &base_slot) in layout.meta_base.iter().enumerate() {
        let copies = arm.meta_copies(meta_index < layout.pool_wide_count);
        if (0..copies).any(|copy_index| !fault.destroyed(copy_slot(base_slot, copy_index))) {
            recovered_meta += 1;
        }
    }
    let mut recovered_data = 0;
    let mut detected_only = 0;
    for (data_index, &base_slot) in layout.data_base.iter().enumerate() {
        let has_live_copy = (0..arm.data_copies()).any(|copy_index| !fault.destroyed(copy_slot(base_slot, copy_index)));
        if has_live_copy {
            recovered_data += 1;
        } else if arm == Arm::Chain {
            // 同批链：前后邻居任一存活即指认得出这个单元存在过。
            let previous_neighbour_alive = data_index > 0 && !fault.destroyed(layout.data_base[data_index - 1]);
            let next_neighbour_alive = data_index + 1 < layout.data_base.len() && !fault.destroyed(layout.data_base[data_index + 1]);
            if previous_neighbour_alive || next_neighbour_alive {
                detected_only += 1;
            }
        }
    }
    let meta_unit_count = layout.meta_base.len() as u64;
    let data_unit_count = layout.data_base.len() as u64;
    let overhead_bytes = match arm {
        Arm::NoRedundancy => 0,
        Arm::Chain => (meta_unit_count + data_unit_count) * CHAIN_BYTES,
        Arm::Ditto => {
            let pool_wide_count = layout.pool_wide_count as u64;
            (pool_wide_count * 2 + (meta_unit_count - pool_wide_count)) * META_UNIT_BYTES
        }
    };
    Outcome {
        recovered_meta,
        recovered_data,
        detected_only,
        overhead_bytes,
    }
}

fn total_bytes(layout: &Layout) -> u64 {
    layout.meta_base.len() as u64 * META_UNIT_BYTES + layout.data_base.len() as u64 * DATA_UNIT_BYTES
}

fn main() {
    let mut emitter = e7_index_bench::Emitter::new();
    let layout = build_layout(20, 100); // 元数据 2%，其中全池元数据占 10%
    let meta_unit_count = layout.meta_base.len() as u64;
    let data_unit_count = layout.data_base.len() as u64;
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config disk_slots={DISK_SLOTS} grain={GRAIN_BYTES} meta_units={meta_unit_count} data_units={data_unit_count} \
             ditto_spread={DITTO_SPREAD} chain_bytes={CHAIN_BYTES} total_bytes={}",
            total_bytes(&layout)
        ))
    );
    let arms = [Arm::NoRedundancy, Arm::Ditto, Arm::Chain];
    // 故障长度扫描：0（阳性对照）、1/64、1/16、1/8、1/4 盘、整盘（阴性对照）
    let span_lengths = [0u64, DISK_SLOTS / 64, DISK_SLOTS / 16, DISK_SLOTS / 8, DISK_SLOTS / 4, DISK_SLOTS];
    for &start_slot in [0u64, DISK_SLOTS / 2].iter() {
        for &span_length in span_lengths.iter() {
            for arm in arms {
                let outcome = run_arm_under_fault(arm, &layout, Fault::Span { start_slot, length_in_slots: span_length });
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=recover arm={} start={start_slot} len={span_length} rec_meta={} rec_data={} \
                         detected_only={} overhead_bytes={} overhead_ppm={}",
                        arm.name(),
                        outcome.recovered_meta,
                        outcome.recovered_data,
                        outcome.detected_only,
                        outcome.overhead_bytes,
                        outcome.overhead_bytes * 1_000_000 / total_bytes(&layout)
                    ))
                );
            }
        }
    }
    // 散点故障：每 period_in_slots 个槽毁一个，毁掉的比例与同规模的 Span 相同。
    // **这一档是给链臂的公平取样点**——连续区间会把前后邻居一起毁掉。
    // 前四个是 2 的幂、整除 DITTO_SPREAD（2^15）⇒ 与副本偏移**共振**；后四个是奇数，不共振。
    for &period_in_slots in [64u64, 16, 8, 4, 63, 17, 9, 5].iter() {
        for arm in arms {
            let outcome = run_arm_under_fault(arm, &layout, Fault::Scatter { period_in_slots });
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=recover arm={} fault={} k={period_in_slots} rec_meta={} rec_data={} \
                     detected_only={} overhead_ppm={}",
                    arm.name(),
                    Fault::Scatter { period_in_slots }.name(),
                    outcome.recovered_meta,
                    outcome.recovered_data,
                    outcome.detected_only,
                    outcome.overhead_bytes * 1_000_000 / total_bytes(&layout)
                ))
            );
        }
    }
    // 举证 5：链臂毁掉恰好一个单元
    let one_unit_gone = run_arm_under_fault(Arm::Chain, &layout, Fault::Span { start_slot: layout.data_base[10], length_in_slots: 2 });
    let intact = run_arm_under_fault(Arm::Chain, &layout, Fault::Span { start_slot: 0, length_in_slots: 0 });
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=blastradius rec_data_intact={} rec_data_one_gone={} delta={}",
            intact.recovered_data,
            one_unit_gone.recovered_data,
            intact.recovered_data - one_unit_gone.recovered_data
        ))
    );
    // 判别力：三条臂在 1/8 盘那格给的恢复数
    let mut distinct_outcomes = std::collections::BTreeSet::new();
    for arm in arms {
        let outcome = run_arm_under_fault(arm, &layout, Fault::Span { start_slot: 0, length_in_slots: DISK_SLOTS / 8 });
        distinct_outcomes.insert((outcome.recovered_meta, outcome.recovered_data));
    }
    println!(
        "{}",
        emitter.emit_raw(&format!("name=discriminate distinct_outcomes={}", distinct_outcomes.len()))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard_layout() -> Layout {
        build_layout(20, 100)
    }

    /// 判据 2：**阳性对照逐臂跑**——故障长度 0 时三条臂都必须全恢复。
    #[test]
    fn positive_control_recovers_everything_on_every_arm() {
        let layout = standard_layout();
        for arm in [Arm::NoRedundancy, Arm::Ditto, Arm::Chain] {
            let outcome = run_arm_under_fault(arm, &layout, Fault::Span { start_slot: 0, length_in_slots: 0 });
            assert_eq!(outcome.recovered_meta, layout.meta_base.len() as u64, "{}", arm.name());
            assert_eq!(outcome.recovered_data, layout.data_base.len() as u64, "{}", arm.name());
            assert_eq!(outcome.detected_only, 0, "{}", arm.name());
        }
    }

    /// 判据 3：阴性对照——整盘毁掉，三条臂都恢复 0。
    #[test]
    fn negative_control_recovers_nothing_on_every_arm() {
        let layout = standard_layout();
        for arm in [Arm::NoRedundancy, Arm::Ditto, Arm::Chain] {
            let outcome = run_arm_under_fault(arm, &layout, Fault::Span { start_slot: 0, length_in_slots: DISK_SLOTS });
            assert_eq!(outcome.recovered_meta, 0, "{}", arm.name());
            assert_eq!(outcome.recovered_data, 0, "{}", arm.name());
        }
    }

    /// 判据 4 / 举证 5：链臂毁掉恰好一个单元 ⇒ 恢复数恰好少 1。
    /// **写成加法不写减法**：变异把常量改大时减法会编译期溢出、被记成无效变异。
    #[test]
    fn chain_blast_radius_is_exactly_one_unit() {
        let layout = standard_layout();
        let intact = run_arm_under_fault(Arm::Chain, &layout, Fault::Span { start_slot: 0, length_in_slots: 0 }).recovered_data;
        let one_gone = run_arm_under_fault(Arm::Chain, &layout, Fault::Span { start_slot: layout.data_base[10], length_in_slots: 2 }).recovered_data;
        assert_eq!(intact, one_gone + 1, "断一节只该少一个单元，不该「断点之后全丢」");
    }

    /// 几何的绝对值：布局逐格钉死。
    #[test]
    fn layout_geometry_is_pinned() {
        let layout = standard_layout();
        assert_eq!(DISK_SLOTS, 262_144);
        assert_eq!(layout.meta_base.len(), 5242);
        assert_eq!(layout.pool_wide_count, 524);
        assert_eq!(layout.data_base.len(), 128_451);
        assert_eq!(DITTO_SPREAD, 32_768);
    }

    /// 判据 6：判别力——三条臂在 1/8 盘那格必须给出不止一种结果。
    #[test]
    fn arms_give_distinct_outcomes_at_one_eighth_disk() {
        let layout = standard_layout();
        let mut distinct_outcomes = std::collections::BTreeSet::new();
        for arm in [Arm::NoRedundancy, Arm::Ditto, Arm::Chain] {
            let outcome = run_arm_under_fault(arm, &layout, Fault::Span { start_slot: 0, length_in_slots: DISK_SLOTS / 8 });
            distinct_outcomes.insert((outcome.recovered_meta, outcome.recovered_data));
        }
        assert!(distinct_outcomes.len() >= 2, "三条臂全同 ⇒ 装置分不开它们");
    }

    /// 举证 2 的可见形态：链臂在数据侧买到的是**检测**不是恢复。
    #[test]
    fn chain_buys_detection_not_recovery() {
        let layout = standard_layout();
        let none = run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Span { start_slot: 0, length_in_slots: DISK_SLOTS / 8 });
        let chain = run_arm_under_fault(Arm::Chain, &layout, Fault::Span { start_slot: 0, length_in_slots: DISK_SLOTS / 8 });
        assert_eq!(chain.recovered_data, none.recovered_data, "恢复数一模一样");
        assert_eq!(chain.detected_only, 1, "只有被毁区边界那一个单元的邻居还活着");
        assert_eq!(none.detected_only, 0);
    }

    /// **给链臂的公平取样点**：散点故障下前后邻居多半还活着，
    /// 链臂能指认出的单元数应该远多于连续区间那一档。
    /// ⚠️ 但**恢复数仍然等于无冗余臂**——这正是举证 2 要看的那个净增量。
    #[test]
    fn chain_under_scatter_still_recovers_nothing_extra() {
        let layout = standard_layout();
        let none = run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Scatter { period_in_slots: 8 });
        let chain = run_arm_under_fault(Arm::Chain, &layout, Fault::Scatter { period_in_slots: 8 });
        assert_eq!(chain.recovered_data, none.recovered_data, "恢复数一模一样");
        assert_eq!(chain.recovered_meta, none.recovered_meta);
        assert!(
            chain.detected_only > 1000,
            "散点故障下链臂该指认出成千上万个，实测 {}",
            chain.detected_only
        );
        assert_eq!(none.detected_only, 0);
    }

    /// **散点故障下的恢复数钉绝对值**。
    /// ⚠️ 这条是变异测试逼出来的：`M4_散点周期判反`（把 `slot % period_in_slots == 0` 改成 `!= 0`，
    /// 于是毁掉的从 1/k 变成 (k−1)/k）**一个测试都没红**——
    /// 因为此前散点那几条断言**全是臂间互比**，没有一条钉绝对值，
    /// 三条臂一起错时互比仍然相等（`.claude/singlefs-ai-sop/rules/test-discipline.md`
    /// 「只让多条臂互相比，测不出『所有臂一起错』」）。按
    /// `.claude/rules/mutation-sampling.md` 三分，这是**真盲区**不是取样点不敏感：
    /// 反转之后任何取样点上的绝对值都变。
    #[test]
    fn scatter_recovery_counts_are_pinned_to_absolute_values() {
        let layout = standard_layout();
        let no_redundancy_period_9 = run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Scatter { period_in_slots: 9 });
        assert_eq!(no_redundancy_period_9.recovered_meta, 4659);
        assert_eq!(no_redundancy_period_9.recovered_data, 114_179);
        let ditto_period_9 = run_arm_under_fault(Arm::Ditto, &layout, Fault::Scatter { period_in_slots: 9 });
        assert_eq!(ditto_period_9.recovered_meta, 5242, "不共振时 ditto 全恢复");
        assert_eq!(ditto_period_9.recovered_data, 114_179, "数据侧 ditto 不加份，与无冗余臂同");
        let chain_period_9 = run_arm_under_fault(Arm::Chain, &layout, Fault::Scatter { period_in_slots: 9 });
        assert_eq!(chain_period_9.detected_only, 14_272);
        let no_redundancy_period_8 = run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Scatter { period_in_slots: 8 });
        assert_eq!(no_redundancy_period_8.recovered_meta, 4586);
        assert_eq!(no_redundancy_period_8.recovered_data, 96_339);
    }

    /// **一条实测出来的结构性结论**：`至少隔 1/8 盘` 这个放置政策，
    /// 在故障周期整除 1/8 盘时会让三份副本**同余、一起死**——
    /// DITTO_SPREAD = 32768 = 2^15，任何 2 的幂周期都整除它。
    /// 换成奇数周期就不共振，ditto 立刻拉开差距。
    /// ⚠️ **周期性故障是本实验自己造的形态**，真实坏道不是周期的；
    /// 这一格说明的是「放置偏移取 2 的幂有共振风险」，不是「ditto 在散点下没用」。
    #[test]
    fn ditto_resonates_with_power_of_two_periods() {
        let layout = standard_layout();
        // 共振：周期 8 整除 32768 ⇒ ditto 与无冗余臂逐格相同
        let no_redundancy_period_8 = run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Scatter { period_in_slots: 8 });
        let ditto_period_8 = run_arm_under_fault(Arm::Ditto, &layout, Fault::Scatter { period_in_slots: 8 });
        assert_eq!(ditto_period_8.recovered_meta, no_redundancy_period_8.recovered_meta, "2 的幂周期下三份副本同余");
        // 不共振：周期 9 不整除 32768 ⇒ ditto 必须严格更好
        let no_redundancy_period_9 = run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Scatter { period_in_slots: 9 });
        let ditto_period_9 = run_arm_under_fault(Arm::Ditto, &layout, Fault::Scatter { period_in_slots: 9 });
        assert!(ditto_period_9.recovered_meta > no_redundancy_period_9.recovered_meta, "d9={} n9={}", ditto_period_9.recovered_meta, no_redundancy_period_9.recovered_meta);
        assert_eq!(DITTO_SPREAD % 8, 0);
        assert_ne!(DITTO_SPREAD % 9, 0);
    }

    /// 散点与连续两种故障在同一毁坏比例下给的恢复数不同 ⇒ 故障形态本身有判别力。
    #[test]
    fn fault_shape_matters() {
        let layout = standard_layout();
        let span_outcome = run_arm_under_fault(Arm::Ditto, &layout, Fault::Span { start_slot: 0, length_in_slots: DISK_SLOTS / 8 });
        let scatter_outcome = run_arm_under_fault(Arm::Ditto, &layout, Fault::Scatter { period_in_slots: 8 });
        assert_ne!(
            (span_outcome.recovered_meta, span_outcome.recovered_data),
            (scatter_outcome.recovered_meta, scatter_outcome.recovered_data),
            "两种故障形态给同一条臂的结果若逐格相同，说明装置分不开它们"
        );
    }

    /// ditto 臂的开销闭式：元数据 2% 且全池占 10% ⇒ 额外 = (524×2 + 4718) 个 16 KiB 单元。
    #[test]
    fn ditto_overhead_is_absolute() {
        let layout = standard_layout();
        let outcome = run_arm_under_fault(Arm::Ditto, &layout, Fault::Span { start_slot: 0, length_in_slots: 0 });
        assert_eq!(outcome.overhead_bytes, (524 * 2 + 4718) * META_UNIT_BYTES);
        // 与 ZFS 自陈的「总开销约 2%」同量级：这里 ppm 值落在 [15000, 30000]
        let overhead_parts_per_million = outcome.overhead_bytes * 1_000_000 / total_bytes(&layout);
        assert!((15_000..=30_000).contains(&overhead_parts_per_million), "ppm={overhead_parts_per_million}");
    }

    /// 链臂的开销闭式：每个单元 12 字节。
    #[test]
    fn chain_overhead_is_absolute() {
        let layout = standard_layout();
        let outcome = run_arm_under_fault(Arm::Chain, &layout, Fault::Span { start_slot: 0, length_in_slots: 0 });
        assert_eq!(outcome.overhead_bytes, (5242 + 128_451) * 12);
        assert_eq!(CHAIN_BYTES, 12);
    }

    /// 无冗余臂开销恒 0，且它在任何故障下的恢复数都不高于 ditto 臂。
    #[test]
    fn no_redundancy_is_the_floor() {
        let layout = standard_layout();
        assert_eq!(run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Span { start_slot: 0, length_in_slots: 0 }).overhead_bytes, 0);
        for &span_length in [DISK_SLOTS / 64, DISK_SLOTS / 16, DISK_SLOTS / 8].iter() {
            let no_redundancy_outcome = run_arm_under_fault(Arm::NoRedundancy, &layout, Fault::Span { start_slot: 0, length_in_slots: span_length });
            let ditto_outcome = run_arm_under_fault(Arm::Ditto, &layout, Fault::Span { start_slot: 0, length_in_slots: span_length });
            assert!(ditto_outcome.recovered_meta >= no_redundancy_outcome.recovered_meta, "len={span_length}");
        }
    }

    /// 副本落点的算术：第 1 份隔 1/8 盘，第 2 份隔 2/8。
    #[test]
    fn copy_placement_arithmetic() {
        assert_eq!(copy_slot(0, 0), 0);
        assert_eq!(copy_slot(0, 1), 32_768);
        assert_eq!(copy_slot(0, 2), 65_536);
        assert_eq!(copy_slot(DISK_SLOTS - 1, 1), 32_767);
    }
}
