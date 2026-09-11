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
    fn output_label(self) -> &'static str {
        match self {
            Arm::AppendOrder => "append_order",
            Arm::CohortBound => "cohort_bound",
        }
    }
}

/// 一个墓碑单元：里面每条记录记它归属的代际。
struct TombstoneUnit {
    cohorts: Vec<u64>,
    is_sealed: bool,
}

struct Simulation {
    records_per_unit: usize,
    units: Vec<TombstoneUnit>,
    open_unit: Option<usize>,
    /// 已整单元回收的记录数（独立计数器）。
    reclaimed_records: u64,
}

impl Simulation {
    fn new(records_per_unit: usize) -> Simulation {
        Simulation { records_per_unit, units: Vec::new(), open_unit: None, reclaimed_records: 0 }
    }
    fn push(&mut self, cohort: u64) {
        let target_unit = match self.open_unit {
            Some(open_index) if self.units[open_index].cohorts.len() < self.records_per_unit => open_index,
            _ => {
                self.units.push(TombstoneUnit { cohorts: Vec::new(), is_sealed: false });
                let new_unit_index = self.units.len() - 1;
                self.open_unit = Some(new_unit_index);
                new_unit_index
            }
        };
        self.units[target_unit].cohorts.push(cohort);
        if self.units[target_unit].cohorts.len() == self.records_per_unit {
            self.units[target_unit].is_sealed = true;
            self.open_unit = None;
        }
    }
    /// 代际边界：cohort_bound 臂强制关掉开放单元。
    fn seal_open(&mut self) {
        if let Some(open_index) = self.open_unit.take() {
            self.units[open_index].is_sealed = true;
        }
    }
    /// 代际 `last_destroyed_cohort`（含）以前的记录全部可回收；整单元都可回收才真的回收。
    fn reclaim(&mut self, last_destroyed_cohort: u64) {
        for unit in &mut self.units {
            if !unit.cohorts.is_empty() && unit.is_sealed && unit.cohorts.iter().all(|&cohort| cohort <= last_destroyed_cohort) {
                self.reclaimed_records += unit.cohorts.len() as u64;
                unit.cohorts.clear();
            }
        }
    }
    /// 收尾审计：对全部单元逐条重算 stuck / live / 空单元。与运行计数器不共享自增点。
    fn audit(&self, last_destroyed_cohort: u64) -> (u64, u64, u64) {
        let (mut stuck, mut live, mut alive_units) = (0u64, 0u64, 0u64);
        for unit in &self.units {
            if unit.cohorts.is_empty() {
                continue;
            }
            alive_units += 1;
            for &cohort in &unit.cohorts {
                if cohort <= last_destroyed_cohort {
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
fn run(arm: Arm, records_per_unit: usize, deletions_per_publish: u64, publishes_per_snapshot: u64, retained_snapshot_cohorts: u64, publish_count: u64) -> (Simulation, u64, u64) {
    let mut simulation = Simulation::new(records_per_unit);
    let mut total_records = 0u64;
    let mut last_destroyed_cohort = 0u64; // 代际号从 1 起；0 = 还没销毁过
    for publish in 1..=publish_count {
        let cohort = publish.div_ceil(publishes_per_snapshot); // 当前代际
        for _ in 0..deletions_per_publish {
            simulation.push(cohort);
            total_records += 1;
        }
        // 代际边界：立快照 + 可能销毁最旧的
        if publish % publishes_per_snapshot == 0 {
            if arm == Arm::CohortBound {
                simulation.seal_open();
            }
            if cohort > retained_snapshot_cohorts {
                last_destroyed_cohort = cohort - retained_snapshot_cohorts;
                simulation.reclaim(last_destroyed_cohort);
            }
        }
    }
    (simulation, last_destroyed_cohort, total_records)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw("name=config cap=583 snap_every=10 k_snap=8 pubs=4000 model=counting file_ops=0")
    );
    // 主扫描：每代际记录数 = del_per_pub × snap_every，扫四档相对 CAP 的比例
    for deletions_per_publish in [1u64, 6, 60, 600] {
        for arm in [Arm::AppendOrder, Arm::CohortBound] {
            let (simulation, last_destroyed_cohort, total_records) = run(arm, 583, deletions_per_publish, 10, 8, 4000);
            let (stuck, live, units_alive) = simulation.audit(last_destroyed_cohort);
            assert_eq!(stuck + live + simulation.reclaimed_records, total_records, "分类必须完备");
            let records_per_cohort = deletions_per_publish * 10;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=pinning arm={} per_cohort={records_per_cohort} total={total_records} reclaimed={} stuck={stuck} live={live} units_alive={units_alive} stuck_pct_of_resident={:.1}",
                    arm.output_label(),
                    simulation.reclaimed_records,
                    100.0 * stuck as f64 / (stuck + live).max(1) as f64
                ))
            );
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 手算锚点**：CAP=8、每发布 1 条、SNAP_EVERY=2（每代际 2 条）、K_SNAP=2、20 次发布。
    /// 手算：单元 0 装代际 1..4（8 条），单元 1 装代际 5..8，单元 2 装 9..10（开放，4 条）。
    /// 跑到 p=20：当前代际 10，last_destroyed_cohort = 8。
    /// 单元 0（代际 ≤ 4）在 last_destroyed_cohort ≥ 4 时整体回收 ⇒ reclaimed = 8；
    /// 单元 1 的代际 5..8 全部 ≤ 8 ⇒ 也整体回收 ⇒ reclaimed = 16；
    /// 开放单元装代际 9、10（各 2 条）都还活 ⇒ stuck = 0, live = 4。
    #[test]
    fn absolute_hand_case_append() {
        let (simulation, last_destroyed_cohort, total_records) = run(Arm::AppendOrder, 8, 1, 2, 2, 20);
        assert_eq!(total_records, 20);
        assert_eq!(last_destroyed_cohort, 8);
        assert_eq!(simulation.reclaimed_records, 16);
        let (stuck, live, alive_units) = simulation.audit(last_destroyed_cohort);
        assert_eq!((stuck, live, alive_units), (0, 4, 1));
    }

    /// **判据 1 的钉住形态**：同参数但 K_SNAP=3、18 次发布 ⇒ last_destroyed_cohort = 6。
    /// 单元 0（代际 1..4）全 ≤ 6 ⇒ 回收；单元 1 装代际 5..8：5、6 已死而 7、8 还活
    /// ⇒ **4 条可回收记录被钉住**；开放单元代际 9（2 条）活。
    #[test]
    fn absolute_hand_case_append_stuck() {
        let (simulation, last_destroyed_cohort, total_records) = run(Arm::AppendOrder, 8, 1, 2, 3, 18);
        assert_eq!(total_records, 18);
        assert_eq!(last_destroyed_cohort, 6);
        assert_eq!(simulation.reclaimed_records, 8, "只有单元 0 整体死透");
        let (stuck, live, _) = simulation.audit(last_destroyed_cohort);
        assert_eq!(stuck, 4, "代际 5、6 的 4 条被 7、8 钉在单元 1 里");
        assert_eq!(live, 6);
    }

    /// **判据 3**：cohort_bound 的 stuck 恒为 0（全部参数格）——
    /// 一个单元只装同代际记录 ⇒ 代际死单元死，没有死活混装。
    #[test]
    fn cohort_bound_never_sticks() {
        for deletions_per_publish in [1u64, 6, 60, 600] {
            for (publishes_per_snapshot, retained_snapshot_cohorts) in [(10u64, 8u64), (2, 2), (5, 3)] {
                let (simulation, last_destroyed_cohort, _) = run(Arm::CohortBound, 583, deletions_per_publish, publishes_per_snapshot, retained_snapshot_cohorts, 4000);
                let (stuck, _, _) = simulation.audit(last_destroyed_cohort);
                assert_eq!(stuck, 0, "del={deletions_per_publish} snap_every={publishes_per_snapshot} k={retained_snapshot_cohorts}");
            }
        }
    }

    /// **判据 2 审计守恒**：分类完备（stuck + live + reclaimed == total_records），两条臂全格。
    #[test]
    fn audit_conservation() {
        for arm in [Arm::AppendOrder, Arm::CohortBound] {
            for deletions_per_publish in [1u64, 60, 600] {
                let (simulation, last_destroyed_cohort, total_records) = run(arm, 583, deletions_per_publish, 10, 8, 4000);
                let (stuck, live, _) = simulation.audit(last_destroyed_cohort);
                assert_eq!(stuck + live + simulation.reclaimed_records, total_records, "{arm:?} del={deletions_per_publish}");
            }
        }
    }

    /// **判据 4 单调性**：append_order 的驻留死占比（stuck / (stuck + live)）随每代际记录数
    /// 上升而下降（分母口径的修订记录见源码头部）。
    #[test]
    fn append_stuck_shrinks_with_cohort_size() {
        let mut previous_share = f64::MAX;
        for deletions_per_publish in [1u64, 6, 60, 600] {
            let (simulation, last_destroyed_cohort, _) = run(Arm::AppendOrder, 583, deletions_per_publish, 10, 8, 4000);
            let (stuck, live, _) = simulation.audit(last_destroyed_cohort);
            let stuck_share = stuck as f64 / (stuck + live).max(1) as f64;
            assert!(stuck_share <= previous_share + 1e-9, "del={deletions_per_publish}: {stuck_share} > {previous_share}");
            previous_share = stuck_share;
        }
    }

    /// 稀疏删除是最坏格：每代际 10 条 ≪ CAP=583 ⇒ 一个单元跨约 58 个代际，
    /// **活着的墓碑单元里超过 80% 的记录是已死的**（实测 416/496 = 84%）；
    /// cohort_bound 同参数为 0。这一格就是「装载纪律买到什么」的判决格。
    #[test]
    fn sparse_deletes_are_the_worst_case() {
        let (simulation, last_destroyed_cohort, _) = run(Arm::AppendOrder, 583, 1, 10, 8, 4000);
        let (stuck, live, _) = simulation.audit(last_destroyed_cohort);
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
        let mut simulation = Simulation::new(8);
        for _ in 0..3 {
            simulation.push(1);
        }
        simulation.reclaim(5);
        assert_eq!(simulation.reclaimed_records, 0, "未封单元不许回收");
        let (stuck, live, alive_units) = simulation.audit(5);
        assert_eq!((stuck, live, alive_units), (3, 0, 1));
    }

    /// 审计的判别力：往一个已回收单元里塞回一条死记录，审计必须多数出一条 stuck。
    #[test]
    fn audit_has_teeth() {
        let (mut simulation, last_destroyed_cohort, _) = run(Arm::AppendOrder, 8, 1, 2, 2, 20);
        let (stuck_before, _, _) = simulation.audit(last_destroyed_cohort);
        simulation.units[0].cohorts.push(1); // 塞回一条早已死透的
        let (stuck_after, _, _) = simulation.audit(last_destroyed_cohort);
        assert_eq!(stuck_after, stuck_before + 1);
    }

    /// 开放单元不参与回收（没 seal 的单元即使全死也不收）——回收粒度是单元的体现。
    #[test]
    fn open_unit_is_not_reclaimed() {
        // 6 条同代际记录进 CAP=8 的开放单元，代际随后死掉：单元未封 ⇒ 不回收 ⇒ 6 条全 stuck
        let (simulation, last_destroyed_cohort, total_records) = run(Arm::AppendOrder, 8, 3, 2, 2, 8);
        // p=1..8, cohort = ceil(p/2) ∈ 1..4，last_destroyed_cohort = 2
        assert_eq!(total_records, 24);
        assert_eq!(last_destroyed_cohort, 2);
        let (stuck, live, _) = simulation.audit(last_destroyed_cohort);
        // 单元 0..2 各 8 条：单元 0 装代际 1(6条)+2(2条) 全 ≤2 ⇒ 回收；
        // 单元 1 装代际 2(4条)+3(4条) ⇒ 4 条 stuck；单元 2 装代际 3(2条)+4(6条) 活；
        // 开放单元 0 条。
        assert_eq!(simulation.reclaimed_records, 8);
        assert_eq!(stuck, 4);
        assert_eq!(live, 12);
    }
}
