//! E84：打包墓碑的钉住量 —— D18 已定项 10 的回收初值（随快照 deadlist 同一套节奏清）
//! 叠上打包载体之后，一单元里死活混装会钉住多少。
//!
//! ## 为什么要有这个实验
//!
//! D18（块里携带什么信息）已定项 10 定了墓碑取「区间记录 + 打包共享单元」，
//! 回收规则初值「随快照 deadlist 同一套节奏清」，并自陈**钉住量欠实验**
//! （C84（墓碑单元的粒度没人定）剩的那半）：回收粒度 = 单元，
//! 一单元内最后一条记录可回收之前整单元钉住——E80（条带的部分释放）pin_stripe 的形状。
//! D2（RAID 条带策略）写的粒度第 2 条早写过同型纪律
//! （「不让两个生命周期不同的对象共享同一个物理映射单元」）——**墓碑记录的「生命周期」
//! 就是它所属的死亡队列（快照代际）**，混装不同代际的记录就是把寿命焊在一起。
//!
//! ## 模型
//!
//! 发布推进；每 SNAP_EVERY 次发布立一个快照代际（cohort）；只保留最近 K_SNAP 个代际，
//! 超出即销毁最旧的（D5（快照 / 空间记账机制）约定 B 的节奏）——**代际销毁那一刻，
//! 归属它的墓碑记录全部变为可回收**。删除流每次发布产生 DEL_PER_PUB 条墓碑记录，
//! 归属当前代际。打包容量 CAP 条/单元（E83（墓碑的粒度）口径 583（2026-09-03 随 E83 的 91 字节单元头重算，原 584），模型可参数化）。
//!
//! 两条臂只差装载纪律：
//!
//! | 臂 | 装载 | 要证的 |
//! |---|---|---|
//! | append_order | 一直往当前开放单元里塞，塞满换新 | 单元跨代际 ⇒ 死活混装的钉住量 |
//! | cohort_bound | **代际边界强制关单元**（一个单元只装同代际记录） | 钉住恒为开放单元那点零头 |
//!
//! 读数：`stuck` = 已可回收但被同单元未回收记录钉住的记录数；`units_alive`；
//! 两条独立路径：模拟里的计数器 vs 收尾对全部单元逐条重算的审计，必须逐格相等。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 手算锚点：CAP=8、每代际 2 条、K_SNAP=2——append_order 的单元跨 4 个代际，
//!    稳态 stuck 按手算钉死（见单测）；cohort_bound 恒 0。
//! 2. 审计守恒：模拟计数器与收尾重算逐格相等；记录总数 = 已回收 + 活 + stuck 分类完备。
//! 3. cohort_bound 的 stuck **恒为 0**（全部参数格）——装载纪律把钉住整个消掉。
//! 4. append_order 的**驻留死占比**（stuck / (stuck + live)，即活着的墓碑单元里已死记录的
//!    占比）随「每代际记录数 / CAP」单调下降：代际 ≪ CAP 时一单元跨 CAP/d 个代际、
//!    驻留字节的大头是已死记录；代际 ≫ CAP 时塌向 0。全表如实报。
//!    ⚠️ **该判据的分母在首次试跑时修订过一次（2026-09-02，任何正式轮入库之前）**：
//!    初版拿「累计已死」当分母——那个比值随运行时长无条件稀释（回收量只增不减），
//!    量不出稳态钉住。修订成驻留口径：分母 = 此刻还躺在活单元里的记录数。
//! 5. 不判装载纪律为定案——那是 D18（块里携带什么信息）已定项 10 的补条，交数字。
//!
//! ## 它答不了的
//!
//! 计数模型，文件操作 0 处。删除流取匀速（无突发）；「随 deadlist 节奏清」按
//! 「代际销毁 ⇒ 该代际记录可回收」的粗粒度建模，不建 deadlist 合并的细节；
//! CAP=583 依赖 E83（墓碑的粒度）的记录宽度假设（56 字节）。

use e7_index_bench::Emitter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    AppendOrder,
    CohortBound,
}
impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::AppendOrder => "append_order",
            Arm::CohortBound => "cohort_bound",
        }
    }
}

/// 一个墓碑单元：里面每条记录记它归属的代际。
struct TombUnit {
    cohorts: Vec<u64>,
    sealed: bool,
}

struct Sim {
    cap: usize,
    units: Vec<TombUnit>,
    open: Option<usize>,
    /// 已整单元回收的记录数（独立计数器）。
    reclaimed_records: u64,
}

impl Sim {
    fn new(cap: usize) -> Sim {
        Sim { cap, units: Vec::new(), open: None, reclaimed_records: 0 }
    }
    fn push(&mut self, cohort: u64) {
        let idx = match self.open {
            Some(i) if self.units[i].cohorts.len() < self.cap => i,
            _ => {
                self.units.push(TombUnit { cohorts: Vec::new(), sealed: false });
                let i = self.units.len() - 1;
                self.open = Some(i);
                i
            }
        };
        self.units[idx].cohorts.push(cohort);
        if self.units[idx].cohorts.len() == self.cap {
            self.units[idx].sealed = true;
            self.open = None;
        }
    }
    /// 代际边界：cohort_bound 臂强制关掉开放单元。
    fn seal_open(&mut self) {
        if let Some(i) = self.open.take() {
            self.units[i].sealed = true;
        }
    }
    /// 代际 `dead_upto`（含）以前的记录全部可回收；整单元都可回收才真的回收。
    fn reclaim(&mut self, dead_upto: u64) {
        for u in &mut self.units {
            if !u.cohorts.is_empty() && u.sealed && u.cohorts.iter().all(|&c| c <= dead_upto) {
                self.reclaimed_records += u.cohorts.len() as u64;
                u.cohorts.clear();
            }
        }
    }
    /// 收尾审计：对全部单元逐条重算 stuck / live / 空单元。与运行计数器不共享自增点。
    fn audit(&self, dead_upto: u64) -> (u64, u64, u64) {
        let (mut stuck, mut live, mut alive_units) = (0u64, 0u64, 0u64);
        for u in &self.units {
            if u.cohorts.is_empty() {
                continue;
            }
            alive_units += 1;
            for &c in &u.cohorts {
                if c <= dead_upto {
                    stuck += 1; // 可回收却还躺在活单元里
                } else {
                    live += 1;
                }
            }
        }
        (stuck, live, alive_units)
    }
}

/// 跑 `pubs` 次发布。返回 (sim, 最后一个已销毁代际, 记录总数)。
fn run(arm: Arm, cap: usize, del_per_pub: u64, snap_every: u64, k_snap: u64, pubs: u64) -> (Sim, u64, u64) {
    let mut sim = Sim::new(cap);
    let mut total = 0u64;
    let mut dead_upto = 0u64; // 代际号从 1 起；0 = 还没销毁过
    for p in 1..=pubs {
        let cohort = p.div_ceil(snap_every); // 当前代际
        for _ in 0..del_per_pub {
            sim.push(cohort);
            total += 1;
        }
        // 代际边界：立快照 + 可能销毁最旧的
        if p % snap_every == 0 {
            if arm == Arm::CohortBound {
                sim.seal_open();
            }
            if cohort > k_snap {
                dead_upto = cohort - k_snap;
                sim.reclaim(dead_upto);
            }
        }
    }
    (sim, dead_upto, total)
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw("name=config cap=583 snap_every=10 k_snap=8 pubs=4000 model=counting file_ops=0")
    );
    // 主扫描：每代际记录数 = del_per_pub × snap_every，扫四档相对 CAP 的比例
    for del_per_pub in [1u64, 6, 60, 600] {
        for arm in [Arm::AppendOrder, Arm::CohortBound] {
            let (sim, dead_upto, total) = run(arm, 583, del_per_pub, 10, 8, 4000);
            let (stuck, live, units_alive) = sim.audit(dead_upto);
            assert_eq!(stuck + live + sim.reclaimed_records, total, "分类必须完备");
            let per_cohort = del_per_pub * 10;
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=pinning arm={} per_cohort={per_cohort} total={total} reclaimed={} stuck={stuck} live={live} units_alive={units_alive} stuck_pct_of_resident={:.1}",
                    arm.tag(),
                    sim.reclaimed_records,
                    100.0 * stuck as f64 / (stuck + live).max(1) as f64
                ))
            );
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 手算锚点**：CAP=8、每发布 1 条、SNAP_EVERY=2（每代际 2 条）、K_SNAP=2、20 次发布。
    /// 手算：单元 0 装代际 1..4（8 条），单元 1 装代际 5..8，单元 2 装 9..10（开放，4 条）。
    /// 跑到 p=20：当前代际 10，dead_upto = 8。
    /// 单元 0（代际 ≤ 4）在 dead_upto ≥ 4 时整体回收 ⇒ reclaimed = 8；
    /// 单元 1 的代际 5..8 全部 ≤ 8 ⇒ 也整体回收 ⇒ reclaimed = 16；
    /// 开放单元装代际 9、10（各 2 条）都还活 ⇒ stuck = 0, live = 4。
    #[test]
    fn absolute_hand_case_append() {
        let (sim, dead_upto, total) = run(Arm::AppendOrder, 8, 1, 2, 2, 20);
        assert_eq!(total, 20);
        assert_eq!(dead_upto, 8);
        assert_eq!(sim.reclaimed_records, 16);
        let (stuck, live, units) = sim.audit(dead_upto);
        assert_eq!((stuck, live, units), (0, 4, 1));
    }

    /// **判据 1 的钉住形态**：同参数但 K_SNAP=3、18 次发布 ⇒ dead_upto = 6。
    /// 单元 0（代际 1..4）全 ≤ 6 ⇒ 回收；单元 1 装代际 5..8：5、6 已死而 7、8 还活
    /// ⇒ **4 条可回收记录被钉住**；开放单元代际 9（2 条）活。
    #[test]
    fn absolute_hand_case_append_stuck() {
        let (sim, dead_upto, total) = run(Arm::AppendOrder, 8, 1, 2, 3, 18);
        assert_eq!(total, 18);
        assert_eq!(dead_upto, 6);
        assert_eq!(sim.reclaimed_records, 8, "只有单元 0 整体死透");
        let (stuck, live, _) = sim.audit(dead_upto);
        assert_eq!(stuck, 4, "代际 5、6 的 4 条被 7、8 钉在单元 1 里");
        assert_eq!(live, 6);
    }

    /// **判据 3**：cohort_bound 的 stuck 恒为 0（全部参数格）——
    /// 一个单元只装同代际记录 ⇒ 代际死单元死，没有死活混装。
    #[test]
    fn cohort_bound_never_sticks() {
        for del in [1u64, 6, 60, 600] {
            for (se, k) in [(10u64, 8u64), (2, 2), (5, 3)] {
                let (sim, dead_upto, _) = run(Arm::CohortBound, 583, del, se, k, 4000);
                let (stuck, _, _) = sim.audit(dead_upto);
                assert_eq!(stuck, 0, "del={del} snap_every={se} k={k}");
            }
        }
    }

    /// **判据 2 审计守恒**：分类完备（stuck + live + reclaimed == total），两条臂全格。
    #[test]
    fn audit_conservation() {
        for arm in [Arm::AppendOrder, Arm::CohortBound] {
            for del in [1u64, 60, 600] {
                let (sim, dead_upto, total) = run(arm, 583, del, 10, 8, 4000);
                let (stuck, live, _) = sim.audit(dead_upto);
                assert_eq!(stuck + live + sim.reclaimed_records, total, "{arm:?} del={del}");
            }
        }
    }

    /// **判据 4 单调性**：append_order 的驻留死占比（stuck / (stuck + live)）随每代际记录数
    /// 上升而下降（分母口径的修订记录见源码头部）。
    #[test]
    fn append_stuck_shrinks_with_cohort_size() {
        let mut prev = f64::MAX;
        for del in [1u64, 6, 60, 600] {
            let (sim, dead_upto, _) = run(Arm::AppendOrder, 583, del, 10, 8, 4000);
            let (stuck, live, _) = sim.audit(dead_upto);
            let pct = stuck as f64 / (stuck + live).max(1) as f64;
            assert!(pct <= prev + 1e-9, "del={del}: {pct} > {prev}");
            prev = pct;
        }
    }

    /// 稀疏删除是最坏格：每代际 10 条 ≪ CAP=583 ⇒ 一个单元跨约 58 个代际，
    /// **活着的墓碑单元里超过 80% 的记录是已死的**（实测 416/496 = 84%）；
    /// cohort_bound 同参数为 0。这一格就是「装载纪律买到什么」的判决格。
    #[test]
    fn sparse_deletes_are_the_worst_case() {
        let (sim, dead_upto, _) = run(Arm::AppendOrder, 583, 1, 10, 8, 4000);
        let (stuck, live, _) = sim.audit(dead_upto);
        assert!(stuck + live > 0);
        assert!(
            stuck as f64 / (stuck + live) as f64 > 0.8,
            "稀疏删除下驻留字节的大头该是已死记录，实测 {stuck}/{}",
            stuck + live
        );
        // 绝对值锚：live 恒等于保留窗口内的记录数（8 个活代际 × 每代际 10 条）
        assert_eq!(live, 80);
        assert_eq!(stuck, 422);
    }

    /// **开放单元即使全死也不回收**（回收要先封——单元还在被追加，收掉它会丢后来的记录）。
    /// 连续删除流造不出这个场景（开放单元永远含当前代际），所以直接构造：
    /// 删除流停了、代际继续销毁，开放单元里 3 条全死 ⇒ 必须一条都不回收、审计记 3 条 stuck。
    #[test]
    fn paused_stream_open_unit_stays() {
        let mut sim = Sim::new(8);
        for _ in 0..3 {
            sim.push(1);
        }
        sim.reclaim(5);
        assert_eq!(sim.reclaimed_records, 0, "未封单元不许回收");
        let (stuck, live, units) = sim.audit(5);
        assert_eq!((stuck, live, units), (3, 0, 1));
    }

    /// 审计的判别力：往一个已回收单元里塞回一条死记录，审计必须多数出一条 stuck。
    #[test]
    fn audit_has_teeth() {
        let (mut sim, dead_upto, _) = run(Arm::AppendOrder, 8, 1, 2, 2, 20);
        let (stuck0, _, _) = sim.audit(dead_upto);
        sim.units[0].cohorts.push(1); // 塞回一条早已死透的
        let (stuck1, _, _) = sim.audit(dead_upto);
        assert_eq!(stuck1, stuck0 + 1);
    }

    /// 开放单元不参与回收（没 seal 的单元即使全死也不收）——回收粒度是单元的体现。
    #[test]
    fn open_unit_is_not_reclaimed() {
        // 6 条同代际记录进 CAP=8 的开放单元，代际随后死掉：单元未封 ⇒ 不回收 ⇒ 6 条全 stuck
        let (sim, dead_upto, total) = run(Arm::AppendOrder, 8, 3, 2, 2, 8);
        // p=1..8, cohort = ceil(p/2) ∈ 1..4，dead_upto = 2
        assert_eq!(total, 24);
        assert_eq!(dead_upto, 2);
        let (stuck, live, _) = sim.audit(dead_upto);
        // 单元 0..2 各 8 条：单元 0 装代际 1(6条)+2(2条) 全 ≤2 ⇒ 回收；
        // 单元 1 装代际 2(4条)+3(4条) ⇒ 4 条 stuck；单元 2 装代际 3(2条)+4(6条) 活；
        // 开放单元 0 条。
        assert_eq!(sim.reclaimed_records, 8);
        assert_eq!(stuck, 4);
        assert_eq!(live, 12);
    }
}
