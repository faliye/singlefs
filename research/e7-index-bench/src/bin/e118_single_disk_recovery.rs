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
const GRAIN: u64 = 16384;
/// D4 已定项 1：元数据侧 16 KiB、数据侧 32 KiB。数据单元占两个槽。
const META_UNIT: u64 = 16384;
const DATA_UNIT: u64 = 32768;
/// 4 GiB 盘 = 262144 个 16 KiB 槽。
const DISK_SLOTS: u64 = 262_144;
/// 同批链：前后邻居各一个槽号 6 字节。
const CHAIN_BYTES: u64 = 6 + 6;
/// ZFS 的放置政策：副本至少隔 1/8 盘。
const DITTO_SPREAD: u64 = DISK_SLOTS / 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    None,
    Ditto,
    Chain,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::None => "none",
            Arm::Ditto => "ditto",
            Arm::Chain => "chain",
        }
    }
    /// 一个元数据单元有几份。**没有通配臂**：加一种臂编译器会报。
    fn meta_copies(self, pool_wide: bool) -> u64 {
        match self {
            Arm::None | Arm::Chain => 1,
            Arm::Ditto => {
                if pool_wide {
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
            Arm::None | Arm::Chain | Arm::Ditto => 1,
        }
    }
}

/// 第 `i` 份副本落在哪个槽：第 0 份在原位，其后每份再隔 1/8 盘。
fn copy_slot(base: u64, i: u64) -> u64 {
    (base + i * DITTO_SPREAD) % DISK_SLOTS
}

/// 故障形态。**连续区间对链臂是最不利的取样点**（前后邻居一起死），
/// 只用它比对等于给链臂设了个稻草人故障模型 ⇒ 另加散点一档，每 `k` 个槽毁一个。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fault {
    /// 一段连续区间整个毁掉（对齐 ZFS 那次「抹掉分区前 1 GB」）。
    Span { start: u64, len: u64 },
    /// 每 `k` 个槽毁一个，毁掉的总数与同规模的 Span 相同。
    Scatter { k: u64 },
}

impl Fault {
    fn name(self) -> &'static str {
        match self {
            Fault::Span { .. } => "span",
            Fault::Scatter { .. } => "scatter",
        }
    }
    fn destroyed(self, s: u64) -> bool {
        match self {
            Fault::Span { start, len } => s >= start && s < start + len,
            Fault::Scatter { k } => k != 0 && s % k == 0,
        }
    }
}

struct Layout {
    /// 元数据单元的基准槽（全池元数据在前）。
    meta_base: Vec<u64>,
    pool_wide: usize,
    data_base: Vec<u64>,
}

/// 元数据占比 `meta_permille`（千分比）、全池元数据占元数据的 `pool_permille`。
fn layout(meta_permille: u64, pool_permille: u64) -> Layout {
    // 盘上先铺元数据槽，再铺数据槽（数据单元占两个槽）。
    let meta_slots = DISK_SLOTS * meta_permille / 1000;
    let data_slots = DISK_SLOTS - meta_slots;
    let n_meta = meta_slots;
    let n_data = data_slots / 2;
    let pool_wide = (n_meta * pool_permille / 1000) as usize;
    Layout {
        meta_base: (0..n_meta).collect(),
        pool_wide,
        data_base: (0..n_data).map(|i| meta_slots + i * 2).collect(),
    }
}

struct Outcome {
    recovered_meta: u64,
    recovered_data: u64,
    /// 举证 2：毁了但**被邻居指认出来**的单元数（检测到，恢复不出）。
    detected_only: u64,
    overhead_bytes: u64,
}

fn run(arm: Arm, l: &Layout, f: Fault) -> Outcome {
    let mut recovered_meta = 0;
    for (i, &b) in l.meta_base.iter().enumerate() {
        let copies = arm.meta_copies(i < l.pool_wide);
        if (0..copies).any(|c| !f.destroyed(copy_slot(b, c))) {
            recovered_meta += 1;
        }
    }
    let mut recovered_data = 0;
    let mut detected_only = 0;
    for (i, &b) in l.data_base.iter().enumerate() {
        let alive = (0..arm.data_copies()).any(|c| !f.destroyed(copy_slot(b, c)));
        if alive {
            recovered_data += 1;
        } else if arm == Arm::Chain {
            // 同批链：前后邻居任一存活即指认得出这个单元存在过。
            let prev_alive = i > 0 && !f.destroyed(l.data_base[i - 1]);
            let next_alive = i + 1 < l.data_base.len() && !f.destroyed(l.data_base[i + 1]);
            if prev_alive || next_alive {
                detected_only += 1;
            }
        }
    }
    let n_meta = l.meta_base.len() as u64;
    let n_data = l.data_base.len() as u64;
    let overhead_bytes = match arm {
        Arm::None => 0,
        Arm::Chain => (n_meta + n_data) * CHAIN_BYTES,
        Arm::Ditto => {
            let pw = l.pool_wide as u64;
            (pw * 2 + (n_meta - pw)) * META_UNIT
        }
    };
    Outcome {
        recovered_meta,
        recovered_data,
        detected_only,
        overhead_bytes,
    }
}

fn total_bytes(l: &Layout) -> u64 {
    l.meta_base.len() as u64 * META_UNIT + l.data_base.len() as u64 * DATA_UNIT
}

fn main() {
    let mut em = e7_index_bench::Emitter::new();
    let l = layout(20, 100); // 元数据 2%，其中全池元数据占 10%
    let n_meta = l.meta_base.len() as u64;
    let n_data = l.data_base.len() as u64;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config disk_slots={DISK_SLOTS} grain={GRAIN} meta_units={n_meta} data_units={n_data} \
             ditto_spread={DITTO_SPREAD} chain_bytes={CHAIN_BYTES} total_bytes={}",
            total_bytes(&l)
        ))
    );
    let arms = [Arm::None, Arm::Ditto, Arm::Chain];
    // 故障长度扫描：0（阳性对照）、1/64、1/16、1/8、1/4 盘、整盘（阴性对照）
    let lens = [0u64, DISK_SLOTS / 64, DISK_SLOTS / 16, DISK_SLOTS / 8, DISK_SLOTS / 4, DISK_SLOTS];
    for &start in [0u64, DISK_SLOTS / 2].iter() {
        for &len in lens.iter() {
            for a in arms {
                let o = run(a, &l, Fault::Span { start, len });
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=recover arm={} start={start} len={len} rec_meta={} rec_data={} \
                         detected_only={} overhead_bytes={} overhead_ppm={}",
                        a.name(),
                        o.recovered_meta,
                        o.recovered_data,
                        o.detected_only,
                        o.overhead_bytes,
                        o.overhead_bytes * 1_000_000 / total_bytes(&l)
                    ))
                );
            }
        }
    }
    // 散点故障：每 k 个槽毁一个，毁掉的比例与同规模的 Span 相同。
    // **这一档是给链臂的公平取样点**——连续区间会把前后邻居一起毁掉。
    // 前四个是 2 的幂、整除 DITTO_SPREAD（2^15）⇒ 与副本偏移**共振**；后四个是奇数，不共振。
    for &k in [64u64, 16, 8, 4, 63, 17, 9, 5].iter() {
        for a in arms {
            let o = run(a, &l, Fault::Scatter { k });
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=recover arm={} fault={} k={k} rec_meta={} rec_data={} \
                     detected_only={} overhead_ppm={}",
                    a.name(),
                    Fault::Scatter { k }.name(),
                    o.recovered_meta,
                    o.recovered_data,
                    o.detected_only,
                    o.overhead_bytes * 1_000_000 / total_bytes(&l)
                ))
            );
        }
    }
    // 举证 5：链臂毁掉恰好一个单元
    let one = run(Arm::Chain, &l, Fault::Span { start: l.data_base[10], len: 2 });
    let zero = run(Arm::Chain, &l, Fault::Span { start: 0, len: 0 });
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=blastradius rec_data_intact={} rec_data_one_gone={} delta={}",
            zero.recovered_data,
            one.recovered_data,
            zero.recovered_data - one.recovered_data
        ))
    );
    // 判别力：三条臂在 1/8 盘那格给的恢复数
    let mut set = std::collections::BTreeSet::new();
    for a in arms {
        let o = run(a, &l, Fault::Span { start: 0, len: DISK_SLOTS / 8 });
        set.insert((o.recovered_meta, o.recovered_data));
    }
    println!(
        "{}",
        em.emit_raw(&format!("name=discriminate distinct_outcomes={}", set.len()))
    );
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l() -> Layout {
        layout(20, 100)
    }

    /// 判据 2：**阳性对照逐臂跑**——故障长度 0 时三条臂都必须全恢复。
    #[test]
    fn t01_positive_control_每条臂() {
        let l = l();
        for a in [Arm::None, Arm::Ditto, Arm::Chain] {
            let o = run(a, &l, Fault::Span { start: 0, len: 0 });
            assert_eq!(o.recovered_meta, l.meta_base.len() as u64, "{}", a.name());
            assert_eq!(o.recovered_data, l.data_base.len() as u64, "{}", a.name());
            assert_eq!(o.detected_only, 0, "{}", a.name());
        }
    }

    /// 判据 3：阴性对照——整盘毁掉，三条臂都恢复 0。
    #[test]
    fn t02_negative_control_每条臂() {
        let l = l();
        for a in [Arm::None, Arm::Ditto, Arm::Chain] {
            let o = run(a, &l, Fault::Span { start: 0, len: DISK_SLOTS });
            assert_eq!(o.recovered_meta, 0, "{}", a.name());
            assert_eq!(o.recovered_data, 0, "{}", a.name());
        }
    }

    /// 判据 4 / 举证 5：链臂毁掉恰好一个单元 ⇒ 恢复数恰好少 1。
    /// **写成加法不写减法**：变异把常量改大时减法会编译期溢出、被记成无效变异。
    #[test]
    fn t03_blast_radius_is_exactly_one() {
        let l = l();
        let intact = run(Arm::Chain, &l, Fault::Span { start: 0, len: 0 }).recovered_data;
        let one_gone = run(Arm::Chain, &l, Fault::Span { start: l.data_base[10], len: 2 }).recovered_data;
        assert_eq!(intact, one_gone + 1, "断一节只该少一个单元，不该「断点之后全丢」");
    }

    /// 几何的绝对值：布局逐格钉死。
    #[test]
    fn t04_layout_is_absolute() {
        let l = l();
        assert_eq!(DISK_SLOTS, 262_144);
        assert_eq!(l.meta_base.len(), 5242);
        assert_eq!(l.pool_wide, 524);
        assert_eq!(l.data_base.len(), 128_451);
        assert_eq!(DITTO_SPREAD, 32_768);
    }

    /// 判据 6：判别力——三条臂在 1/8 盘那格必须给出不止一种结果。
    #[test]
    fn t05_discriminating() {
        let l = l();
        let mut set = std::collections::BTreeSet::new();
        for a in [Arm::None, Arm::Ditto, Arm::Chain] {
            let o = run(a, &l, Fault::Span { start: 0, len: DISK_SLOTS / 8 });
            set.insert((o.recovered_meta, o.recovered_data));
        }
        assert!(set.len() >= 2, "三条臂全同 ⇒ 装置分不开它们");
    }

    /// 举证 2 的可见形态：链臂在数据侧买到的是**检测**不是恢复。
    #[test]
    fn t06_chain_buys_detection_not_recovery() {
        let l = l();
        let none = run(Arm::None, &l, Fault::Span { start: 0, len: DISK_SLOTS / 8 });
        let chain = run(Arm::Chain, &l, Fault::Span { start: 0, len: DISK_SLOTS / 8 });
        assert_eq!(chain.recovered_data, none.recovered_data, "恢复数一模一样");
        assert_eq!(chain.detected_only, 1, "只有被毁区边界那一个单元的邻居还活着");
        assert_eq!(none.detected_only, 0);
    }

    /// **给链臂的公平取样点**：散点故障下前后邻居多半还活着，
    /// 链臂能指认出的单元数应该远多于连续区间那一档。
    /// ⚠️ 但**恢复数仍然等于无冗余臂**——这正是举证 2 要看的那个净增量。
    #[test]
    fn t11_chain_under_scatter_still_recovers_nothing_extra() {
        let l = l();
        let none = run(Arm::None, &l, Fault::Scatter { k: 8 });
        let chain = run(Arm::Chain, &l, Fault::Scatter { k: 8 });
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
    /// ⚠️ 这条是变异测试逼出来的：`M4_散点周期判反`（把 `s % k == 0` 改成 `!= 0`，
    /// 于是毁掉的从 1/k 变成 (k−1)/k）**一个测试都没红**——
    /// 因为此前散点那几条断言**全是臂间互比**，没有一条钉绝对值，
    /// 三条臂一起错时互比仍然相等（`.claude/singlefs-ai-sop/rules/test-discipline.md`
    /// 「只让多条臂互相比，测不出『所有臂一起错』」）。按
    /// `.claude/rules/mutation-sampling.md` 三分，这是**真盲区**不是取样点不敏感：
    /// 反转之后任何取样点上的绝对值都变。
    #[test]
    fn t14_scatter_counts_are_absolute() {
        let l = l();
        let n9 = run(Arm::None, &l, Fault::Scatter { k: 9 });
        assert_eq!(n9.recovered_meta, 4659);
        assert_eq!(n9.recovered_data, 114_179);
        let d9 = run(Arm::Ditto, &l, Fault::Scatter { k: 9 });
        assert_eq!(d9.recovered_meta, 5242, "不共振时 ditto 全恢复");
        assert_eq!(d9.recovered_data, 114_179, "数据侧 ditto 不加份，与无冗余臂同");
        let c9 = run(Arm::Chain, &l, Fault::Scatter { k: 9 });
        assert_eq!(c9.detected_only, 14_272);
        let n8 = run(Arm::None, &l, Fault::Scatter { k: 8 });
        assert_eq!(n8.recovered_meta, 4586);
        assert_eq!(n8.recovered_data, 96_339);
    }

    /// **一条实测出来的结构性结论**：`至少隔 1/8 盘` 这个放置政策，
    /// 在故障周期整除 1/8 盘时会让三份副本**同余、一起死**——
    /// DITTO_SPREAD = 32768 = 2^15，任何 2 的幂周期都整除它。
    /// 换成奇数周期就不共振，ditto 立刻拉开差距。
    /// ⚠️ **周期性故障是本实验自己造的形态**，真实坏道不是周期的；
    /// 这一格说明的是「放置偏移取 2 的幂有共振风险」，不是「ditto 在散点下没用」。
    #[test]
    fn t13_ditto_resonates_with_power_of_two_periods() {
        let l = l();
        // 共振：周期 8 整除 32768 ⇒ ditto 与无冗余臂逐格相同
        let n8 = run(Arm::None, &l, Fault::Scatter { k: 8 });
        let d8 = run(Arm::Ditto, &l, Fault::Scatter { k: 8 });
        assert_eq!(d8.recovered_meta, n8.recovered_meta, "2 的幂周期下三份副本同余");
        // 不共振：周期 9 不整除 32768 ⇒ ditto 必须严格更好
        let n9 = run(Arm::None, &l, Fault::Scatter { k: 9 });
        let d9 = run(Arm::Ditto, &l, Fault::Scatter { k: 9 });
        assert!(d9.recovered_meta > n9.recovered_meta, "d9={} n9={}", d9.recovered_meta, n9.recovered_meta);
        assert_eq!(DITTO_SPREAD % 8, 0);
        assert_ne!(DITTO_SPREAD % 9, 0);
    }

    /// 散点与连续两种故障在同一毁坏比例下给的恢复数不同 ⇒ 故障形态本身有判别力。
    #[test]
    fn t12_fault_shape_matters() {
        let l = l();
        let span = run(Arm::Ditto, &l, Fault::Span { start: 0, len: DISK_SLOTS / 8 });
        let scat = run(Arm::Ditto, &l, Fault::Scatter { k: 8 });
        assert_ne!(
            (span.recovered_meta, span.recovered_data),
            (scat.recovered_meta, scat.recovered_data),
            "两种故障形态给同一条臂的结果若逐格相同，说明装置分不开它们"
        );
    }

    /// ditto 臂的开销闭式：元数据 2% 且全池占 10% ⇒ 额外 = (524×2 + 4718) 个 16 KiB 单元。
    #[test]
    fn t07_ditto_overhead_is_absolute() {
        let l = l();
        let o = run(Arm::Ditto, &l, Fault::Span { start: 0, len: 0 });
        assert_eq!(o.overhead_bytes, (524 * 2 + 4718) * META_UNIT);
        // 与 ZFS 自陈的「总开销约 2%」同量级：这里 ppm 值落在 [15000, 30000]
        let ppm = o.overhead_bytes * 1_000_000 / total_bytes(&l);
        assert!((15_000..=30_000).contains(&ppm), "ppm={ppm}");
    }

    /// 链臂的开销闭式：每个单元 12 字节。
    #[test]
    fn t08_chain_overhead_is_absolute() {
        let l = l();
        let o = run(Arm::Chain, &l, Fault::Span { start: 0, len: 0 });
        assert_eq!(o.overhead_bytes, (5242 + 128_451) * 12);
        assert_eq!(CHAIN_BYTES, 12);
    }

    /// 无冗余臂开销恒 0，且它在任何故障下的恢复数都不高于 ditto 臂。
    #[test]
    fn t09_none_is_the_floor() {
        let l = l();
        assert_eq!(run(Arm::None, &l, Fault::Span { start: 0, len: 0 }).overhead_bytes, 0);
        for &len in [DISK_SLOTS / 64, DISK_SLOTS / 16, DISK_SLOTS / 8].iter() {
            let n = run(Arm::None, &l, Fault::Span { start: 0, len: len });
            let d = run(Arm::Ditto, &l, Fault::Span { start: 0, len: len });
            assert!(d.recovered_meta >= n.recovered_meta, "len={len}");
        }
    }

    /// 副本落点的算术：第 1 份隔 1/8 盘，第 2 份隔 2/8。
    #[test]
    fn t10_copy_placement() {
        assert_eq!(copy_slot(0, 0), 0);
        assert_eq!(copy_slot(0, 1), 32_768);
        assert_eq!(copy_slot(0, 2), 65_536);
        assert_eq!(copy_slot(DISK_SLOTS - 1, 1), 32_767);
    }
}
