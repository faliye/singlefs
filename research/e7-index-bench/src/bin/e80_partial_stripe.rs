//! E80：条带的部分释放 —— w ≥ 3 的条带有成员被 COW 释放之后，parity 还护得住谁。
//!
//! ## 为什么要有这个实验
//!
//! 三条已定条款合起来，造出一类全仓零覆盖的状态（2026-09-02 grep 证实
//! 「条带 × 部分释放 / 成员死亡」零命中）：
//!
//! - D2（RAID 条带策略）已定项 6：`w = clamp(攒批后写入量 + 1, 2, 4)`——批 ≥ 2 时 w ≥ 3；
//! - D2 已定项 10：「一个单元整个落在一列上」——**一条 w≥3 的条带耦合多个不同单元**，
//!   E63（分配器每次写用几列）口径逐字「一条 w 宽条带带 w−1 格数据、1 格 parity」；
//! - COW（项目前提）：每次覆写 = 释放旧单元 ⇒ **部分释放是常态不是边角**。
//!
//! 释放的成员空间一旦被复用（写了别的数据），同条带幸存成员的重建输入就没了——
//! 重建一个丢失列要 parity + 其余全部数据列的**原始字节**。
//! bcachefs 的桶级纠删码为此付「stripe 里还有活数据就不能复用 bucket」（D2 备查一节原话），
//! 而本工程**哪条规则都没定**。⚠️ 第一版 2 盘恒 w=2（1 数据 + 1 副本），条带不耦合任何别的单元
//! ⇒ **这个洞在第一版的全部测试里都不可见**，正是「罕见分支会腐坏」的形状。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D2 已定项 8：「建对象时按当时最空的 g 块盘选一组；此后该对象的每条条带只在组内取
//!   最空的 w 块（w 由已定项 6 给）」，g 初值 4。
//! - D2 写的粒度：「不让两个生命周期不同的对象共享同一个物理映射单元」——
//!   它管的是设备内部映射单元；**条带耦合是同一形状的问题在 parity 维上的再现**。
//! - D5（快照 / 空间记账机制）已定项 4 统计量表：第 3 项「不可回收字节」——
//!   zoned 线立的量；若采「条带死透才回收」，钉住的空间正好落进这个已有统计量。
//! - D26（后台整理与放置回收）腿一：常驻、自动的后台整理——**惰性修复臂就挂在这条腿上**。
//!
//! ## 模型
//!
//! D=8 盘、OBJ 个对象各 NUNITS 个逻辑单元；负载：先全量初写，再 ROUNDS 轮随机批量 COW 重写
//! （批大小 1..=3 ⇒ w ∈ {2,3,4}）。对象绑组 g=4（D2 已定项 8），条带列取组内最空 w 盘。
//! 四条臂只在「释放的成员空间怎么处置」上不同：
//!
//! | 臂 | 处置 | 该臂要证的 |
//! |---|---|---|
//! | free_now | 释放即可复用（defer 只延迟不阻止，按最坏建模） | 单盘失效后丢多少**活**数据 |
//! | pin_stripe | 条带全部数据成员死透，整条带（含 parity）才回收 | 钉住多少空间（不可回收量） |
//! | restripe_on_free | 成员一死就把幸存者**整批**重落一条新条带 | 立刻修的写放大 |
//! | lazy_restripe | 成员死了先钉住；后台每轮修 REPAIR_BUDGET 条部分条带（D26 腿一的形态） | 钉住有界 + 修得比立刻修便宜多少 |
//!
//! **第二轮口径变更（2026-09-02）**：restripe_on_free 与 lazy_restripe 共用同一个修复实现
//! `repair_stripe`（幸存者整批落一条新条带），两臂唯一的差别是**何时修**——
//! 第一轮的 eager 臂按「每个幸存者单独 w=2 重落」计，是上界口径，本轮起废弃。
//! 负载加一维：uniform（均匀随机）与 skew（80% 的重写落在 20% 的对象上）。
//!
//! 失效评估：对每一块盘各算一次「该盘失效后不可重建的活单元数」（重建要求其余 w−1 个成员
//! 的块都未被复用改写），报全盘平均与最坏。
//!
//! ## 判据（跑前写死，跑完不许改；第二轮新增的三条标明）
//!
//! 1. **手算锚点**：单条 w=4 条带、释放一列并复用、坏另一列 ⇒ free_now 恰丢 1、
//!    pin_stripe 恰钉 1 格且丢 0。对不上整轮作废。
//! 2. **正确性格**：pin_stripe、restripe_on_free、lazy_restripe 的丢失数必须**恒为 0**
//!    （全部盘 × 全部种子 × 两种负载）。
//! 3. **阴性对照**：全部批取 1（恒 w=2）时各臂逐格相同、丢失 0、钉住 0——
//!    这一格证明「第一版看不见这个洞」。
//! 4. **守恒**：每臂每轮 live + parity + pinned + free + reused 必须等于已分配总格数，破了作废。
//! 5. 代价（钉住比例、写放大）如实报，选哪条是决策不是实验。
//! 6. （二轮）**惰性臂的排空**：负载停止后按预算继续修，队列必须在有界轮数内清空、钉住归零。
//! 7. （二轮）**惰性 ≤ 立刻**：同种子同负载下 lazy 的额外写不多于 eager——
//!    两臂共用修复实现，lazy 只可能少修（整条死透的免修、多次释放并到一次修）。
//! 8. （二轮）skew 与 uniform 必须产出不同的世界（负载参数真的在起作用）。
//!
//! ## 它答不了的
//!
//! 计数模型，文件操作 0 处；不建模 defer 窗口的时长（free_now 按「最坏：立即复用」计）、
//! 修复与前台写的争用、parity ≥ 2（D2 自陈 parity 几格从没定过）。
//! 批大小分布取均匀 1..=3；skew 取 80/20，都是选定的场景点。

use e7_index_bench::Emitter;
use std::collections::VecDeque;

const DEVS: usize = 8;
const G: usize = 4; // D2 已定项 8：对象绑组
const OBJ: usize = 24;
const NUNITS: usize = 12;
const ROUNDS: usize = 4000;
/// 惰性臂每轮修几条部分条带（后台整理的预算）。
const REPAIR_BUDGET: usize = 2;
/// 排空阶段的轮数上限——只是防呆，正常几百轮内必清空。
const DRAIN_CAP: usize = 100_000;

/// C59（种子折叠成同一个状态）教训：先乘法混淆再保证非零，不许 `seed | 1`。
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if s == 0 {
            s = 0xDEAD_BEEF;
        }
        Rng(s)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    FreeNow,
    PinStripe,
    RestripeOnFree,
    LazyRestripe,
}
impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::FreeNow => "free_now",
            Arm::PinStripe => "pin_stripe",
            Arm::RestripeOnFree => "restripe_on_free",
            Arm::LazyRestripe => "lazy_restripe",
        }
    }
}

/// 一格（一个块）的状态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Cell {
    /// 活数据：(对象, 逻辑单元, 条带号)
    Live(u32, u32, u32),
    /// parity：(条带号)
    Parity(u32),
    /// 已释放、原始字节还在（可用于重建）——free_now 下在空闲池里，pin/lazy 下被钉住
    FreedIntact(u32),
    /// 被复用改写过（原始字节没了）
    Reused,
    Free,
}

struct Stripe {
    /// (dev, cell 下标) 数据成员
    data: Vec<(usize, usize)>,
    parity: (usize, usize),
    live_count: usize,
    /// 已进惰性修复队列（去重用）
    queued: bool,
    /// 已被修复或整条回收，格子全放掉了
    retired: bool,
}

struct World {
    cells: Vec<Vec<Cell>>, // 每盘一列格
    stripes: Vec<Stripe>,
    /// 对象 → 组（4 块盘）
    group: Vec<[usize; G]>,
    /// 对象逻辑单元 → (条带号, 成员序号)
    loc: Vec<Vec<(u32, usize)>>,
    free_pool: Vec<(usize, usize)>,
    /// 惰性臂的部分条带队列（先进先修）
    repair_queue: VecDeque<u32>,
    extra_writes: u64,
    total_writes: u64,
    repairs: u64,
    peak_pinned: u64,
}

impl World {
    fn new() -> World {
        World {
            cells: vec![Vec::new(); DEVS],
            stripes: Vec::new(),
            group: Vec::new(),
            loc: vec![vec![(u32::MAX, 0); NUNITS]; OBJ],
            free_pool: Vec::new(),
            repair_queue: VecDeque::new(),
            extra_writes: 0,
            total_writes: 0,
            repairs: 0,
            peak_pinned: 0,
        }
    }

    fn used(&self, d: usize) -> usize {
        self.cells[d].iter().filter(|c| !matches!(c, Cell::Free)).count()
    }

    /// 拿一格：优先复用空闲池（对陈旧字节最坏），否则该盘尾部追加新格。
    fn take_cell(&mut self, dev: usize) -> usize {
        if let Some(pos) = self.free_pool.iter().position(|&(d, _)| d == dev) {
            let (_, idx) = self.free_pool.swap_remove(pos);
            self.cells[dev][idx] = Cell::Reused; // 先标复用，写入者随后改
            return idx;
        }
        self.cells[dev].push(Cell::Free);
        self.cells[dev].len() - 1
    }

    /// 写一条条带：batch 个数据单元 + 1 parity，列取组内最空 w 盘。
    /// ⚠️ 条带号必须**先占位再填**：修复路径会在本循环中途往 `stripes` 推新条目，
    /// 进循环前抓 `len()` 当号会撞车（2026-09-02 第一版实测 panic，改成占位后消失）。
    fn write_stripe(&mut self, obj: usize, units: &[usize], arm: Arm) {
        let w = (units.len() + 1).clamp(2, 4);
        assert_eq!(w, units.len() + 1, "批 1..=3 下 clamp 不应截断");
        let mut devs: Vec<usize> = self.group[obj].to_vec();
        devs.sort_by_key(|&d| self.used(d));
        devs.truncate(w);
        let sid = self.stripes.len() as u32;
        self.stripes.push(Stripe {
            data: Vec::new(),
            parity: (usize::MAX, usize::MAX),
            live_count: 0,
            queued: false,
            retired: false,
        });
        for (k, &u) in units.iter().enumerate() {
            let d = devs[k];
            let idx = self.take_cell(d);
            self.cells[d][idx] = Cell::Live(obj as u32, u as u32, sid);
            self.stripes[sid as usize].data.push((d, idx));
            self.stripes[sid as usize].live_count += 1;
            self.total_writes += 1;
            // 释放旧单元（初写时 loc 是 MAX，没有旧的）。
            // loc 先指向新家再释放旧的——修复扫幸存者时不许再把这个单元搬一次。
            let (old_sid, _) = self.loc[obj][u];
            self.loc[obj][u] = (sid, k);
            if old_sid != u32::MAX {
                self.free_member(old_sid, obj as u32, u as u32, arm);
            }
        }
        let pd = devs[w - 1];
        let pidx = self.take_cell(pd);
        self.cells[pd][pidx] = Cell::Parity(sid);
        self.total_writes += 1;
        self.stripes[sid as usize].parity = (pd, pidx);
    }

    /// 整条回收：把条带还占着的格（钉住的、parity）全放进池，条带退休。
    fn release_stripe(&mut self, sid: u32) {
        let members = self.stripes[sid as usize].data.clone();
        let parity = self.stripes[sid as usize].parity;
        for (d, i) in members {
            if matches!(self.cells[d][i], Cell::FreedIntact(x) if x == sid) {
                self.cells[d][i] = Cell::Free;
                self.free_pool.push((d, i));
            }
        }
        if matches!(self.cells[parity.0][parity.1], Cell::Parity(x) if x == sid) {
            self.cells[parity.0][parity.1] = Cell::Free;
            self.free_pool.push(parity);
        }
        self.stripes[sid as usize].retired = true;
    }

    /// 修一条部分条带：幸存者**整批**重落一条新条带，旧条带整条回收。
    /// eager 与 lazy 共用这一个实现——两臂的差别只在「何时修」，不在「怎么修」。
    fn repair_stripe(&mut self, sid: u32) {
        if self.stripes[sid as usize].retired {
            return;
        }
        let members = self.stripes[sid as usize].data.clone();
        let survivors: Vec<(u32, u32)> = members
            .iter()
            .filter_map(|&(d, i)| match self.cells[d][i] {
                Cell::Live(o, u, x) if x == sid => Some((o, u)),
                _ => None,
            })
            .collect();
        // 旧格整条放掉（活格、钉住格、parity 都进池）
        for &(d, i) in &members {
            match self.cells[d][i] {
                Cell::Live(_, _, x) if x == sid => {
                    self.cells[d][i] = Cell::Free;
                    self.free_pool.push((d, i));
                }
                Cell::FreedIntact(x) if x == sid => {
                    self.cells[d][i] = Cell::Free;
                    self.free_pool.push((d, i));
                }
                _ => {}
            }
        }
        let parity = self.stripes[sid as usize].parity;
        if matches!(self.cells[parity.0][parity.1], Cell::Parity(x) if x == sid) {
            self.cells[parity.0][parity.1] = Cell::Free;
            self.free_pool.push(parity);
        }
        self.stripes[sid as usize].retired = true;
        if survivors.is_empty() {
            return;
        }
        // 幸存者整批落一条新条带
        let before = self.total_writes;
        let w = (survivors.len() + 1).clamp(2, 4);
        let obj = survivors[0].0 as usize;
        let mut devs: Vec<usize> = self.group[obj].to_vec();
        devs.sort_by_key(|&d| self.used(d));
        devs.truncate(w);
        let nsid = self.stripes.len() as u32;
        self.stripes.push(Stripe {
            data: Vec::new(),
            parity: (usize::MAX, usize::MAX),
            live_count: 0,
            queued: false,
            retired: false,
        });
        for (k, &(o, u)) in survivors.iter().enumerate() {
            let d = devs[k];
            let idx = self.take_cell(d);
            self.cells[d][idx] = Cell::Live(o, u, nsid);
            self.stripes[nsid as usize].data.push((d, idx));
            self.stripes[nsid as usize].live_count += 1;
            self.loc[o as usize][u as usize] = (nsid, k);
            self.total_writes += 1;
        }
        let pd = devs[w - 1];
        let pidx = self.take_cell(pd);
        self.cells[pd][pidx] = Cell::Parity(nsid);
        self.total_writes += 1;
        self.stripes[nsid as usize].parity = (pd, pidx);
        self.extra_writes += self.total_writes - before;
        self.repairs += 1;
    }

    /// 惰性臂的后台一拍：按预算修队列头上的部分条带。
    fn repair_tick(&mut self) {
        for _ in 0..REPAIR_BUDGET {
            loop {
                match self.repair_queue.pop_front() {
                    Some(sid) if self.stripes[sid as usize].retired => continue, // 已死透整条回收过
                    Some(sid) => {
                        self.repair_stripe(sid);
                        break;
                    }
                    None => return,
                }
            }
        }
    }

    /// 一个条带成员死了。四条臂在这里分岔。
    fn free_member(&mut self, sid: u32, obj: u32, unit: u32, arm: Arm) {
        let s = &mut self.stripes[sid as usize];
        let mut freed_at = None;
        for &(d, i) in &s.data {
            if self.cells[d][i] == Cell::Live(obj, unit, sid) {
                freed_at = Some((d, i));
                break;
            }
        }
        let (d, i) = freed_at.expect("成员必须在条带里");
        s.live_count -= 1;
        let dead = s.live_count == 0;
        match arm {
            Arm::FreeNow => {
                // 原始字节还在，但空间立刻可复用。
                self.cells[d][i] = Cell::FreedIntact(sid);
                self.free_pool.push((d, i));
                if dead {
                    let parity = self.stripes[sid as usize].parity;
                    self.cells[parity.0][parity.1] = Cell::FreedIntact(sid);
                    self.free_pool.push(parity);
                    self.stripes[sid as usize].retired = true;
                }
            }
            Arm::PinStripe => {
                self.cells[d][i] = Cell::FreedIntact(sid);
                if dead {
                    self.release_stripe(sid);
                }
            }
            Arm::RestripeOnFree => {
                self.cells[d][i] = Cell::Free;
                self.free_pool.push((d, i));
                if dead {
                    self.release_stripe(sid);
                } else {
                    self.repair_stripe(sid);
                }
            }
            Arm::LazyRestripe => {
                // 钉住，等后台修——不进池。
                self.cells[d][i] = Cell::FreedIntact(sid);
                if dead {
                    self.release_stripe(sid);
                } else if !self.stripes[sid as usize].queued {
                    self.stripes[sid as usize].queued = true;
                    self.repair_queue.push_back(sid);
                }
            }
        }
    }

    /// 单盘失效后不可重建的活单元数。重建一个丢失列要 parity + 其余全部数据列的**原始字节**：
    /// 其余成员必须不在失效盘上（列互不同盘按构造成立，有单测钉住），
    /// 且格内字节仍属本条带（活的、或释放后未被复用的都算原始字节在）。
    fn loss_if_dev_fails(&self, dead_dev: usize) -> u64 {
        let mut loss = 0;
        for (i, c) in self.cells[dead_dev].iter().enumerate() {
            if let Cell::Live(_, _, sid) = c {
                let s = &self.stripes[*sid as usize];
                let others_ok = s
                    .data
                    .iter()
                    .filter(|&&(dd, ii)| !(dd == dead_dev && ii == i))
                    .all(|&(dd, ii)| {
                        dd != dead_dev
                            && match self.cells[dd][ii] {
                                Cell::Live(_, _, x) => x == *sid,
                                Cell::FreedIntact(x) => x == *sid, // 字节还在，能参与重建
                                Cell::Parity(_) | Cell::Reused | Cell::Free => false,
                            }
                    });
                let (pd, pi) = s.parity;
                let parity_ok =
                    pd != dead_dev && matches!(self.cells[pd][pi], Cell::Parity(x) if x == *sid);
                if !(others_ok && parity_ok) {
                    loss += 1;
                }
            }
        }
        loss
    }

    /// 守恒检查的五个读数。
    fn census(&self) -> (u64, u64, u64, u64, u64) {
        let (mut live, mut parity, mut freed_intact, mut reused, mut free) = (0, 0, 0, 0, 0);
        for col in &self.cells {
            for c in col {
                match c {
                    Cell::Live(..) => live += 1,
                    Cell::Parity(_) => parity += 1,
                    Cell::FreedIntact(_) => freed_intact += 1,
                    Cell::Reused => reused += 1,
                    Cell::Free => free += 1,
                }
            }
        }
        (live, parity, freed_intact, reused, free)
    }

    fn pinned_now(&self) -> u64 {
        self.census().2
    }
}

/// 跑一臂：初写全部对象，再 ROUNDS 轮随机批量重写。
/// `w2_only` 是阴性对照（批恒 1）；`skew` 让 80% 的重写落在前 20% 的对象上。
/// 惰性臂在负载后进入排空阶段，返回排空用的轮数。
fn run(seed: u64, arm: Arm, w2_only: bool, skew: bool) -> (World, u64) {
    let mut rng = Rng::new(seed);
    let mut w = World::new();
    for _ in 0..OBJ {
        let mut devs: Vec<usize> = (0..DEVS).collect();
        devs.sort_by_key(|&d| w.used(d));
        let mut grp = [0usize; G];
        grp.copy_from_slice(&devs[..G]);
        w.group.push(grp);
    }
    for o in 0..OBJ {
        let mut u = 0;
        while u < NUNITS {
            let b = if w2_only { 1 } else { (rng.below(3) + 1) as usize };
            let b = b.min(NUNITS - u);
            let units: Vec<usize> = (u..u + b).collect();
            w.write_stripe(o, &units, arm);
            u += b;
        }
    }
    let hot = (OBJ / 5).max(1);
    for _ in 0..ROUNDS {
        let o = if skew && rng.below(10) < 8 {
            rng.below(hot as u64) as usize
        } else {
            rng.below(OBJ as u64) as usize
        };
        let b = if w2_only { 1 } else { (rng.below(3) + 1) as usize };
        let start = rng.below((NUNITS - b + 1) as u64) as usize;
        let units: Vec<usize> = (start..start + b).collect();
        w.write_stripe(o, &units, arm);
        if arm == Arm::LazyRestripe {
            w.repair_tick();
        }
        let p = w.pinned_now();
        if p > w.peak_pinned {
            w.peak_pinned = p;
        }
    }
    // 排空阶段（只有惰性臂有活干）：负载停了，后台按预算继续修。
    let mut drain_rounds = 0;
    if arm == Arm::LazyRestripe {
        for _ in 0..DRAIN_CAP {
            if w.repair_queue.iter().all(|&sid| w.stripes[sid as usize].retired) {
                break;
            }
            w.repair_tick();
            drain_rounds += 1;
        }
    }
    (w, drain_rounds)
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config devs={DEVS} g={G} obj={OBJ} nunits={NUNITS} rounds={ROUNDS} repair_budget={REPAIR_BUDGET} model=counting file_ops=0"
        ))
    );
    for skew in [false, true] {
        let wl = if skew { "skew" } else { "uniform" };
        for seed in [11u64, 22, 33, 44, 55] {
            for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
                let (w, drain) = run(seed, arm, false, skew);
                let (live, parity, freed, reused, free) = w.census();
                let total: u64 = w.cells.iter().map(|c| c.len() as u64).sum();
                assert_eq!(live + parity + freed + reused + free, total, "守恒破了");
                let losses: Vec<u64> = (0..DEVS).map(|d| w.loss_if_dev_fails(d)).collect();
                let worst = *losses.iter().max().unwrap();
                let mean = losses.iter().sum::<u64>() as f64 / DEVS as f64;
                let pinned = match arm {
                    Arm::PinStripe | Arm::LazyRestripe => freed,
                    Arm::FreeNow | Arm::RestripeOnFree => 0,
                };
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=arm workload={wl} seed={seed} arm={} live={live} parity={parity} freed_intact={freed} reused={reused} total={total} loss_mean={mean:.1} loss_worst={worst} pinned={pinned} pinned_pct_of_live={:.1} peak_pinned={} extra_writes={} write_amp_pct={:.1} repairs={} drain_rounds={drain}",
                        arm.tag(),
                        100.0 * pinned as f64 / live as f64,
                        w.peak_pinned,
                        w.extra_writes,
                        100.0 * w.extra_writes as f64 / (w.total_writes - w.extra_writes) as f64,
                        w.repairs
                    ))
                );
            }
        }
    }
    // 阴性对照：恒 w=2。各臂丢失与钉住都必须为 0。
    for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
        let (w, _) = run(11, arm, true, false);
        let (live, _, freed, _, _) = w.census();
        let worst = (0..DEVS).map(|d| w.loss_if_dev_fails(d)).max().unwrap();
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=w2_control arm={} live={live} freed_intact={freed} loss_worst={worst} extra_writes={}",
                arm.tag(),
                w.extra_writes
            ))
        );
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 手算锚点**：单条 w=4 条带（3 数据 + parity），释放 u1 并复用其格，
    /// 坏掉 u2 所在盘 ⇒ free_now 恰丢 1；pin_stripe 恰钉 1 格且丢 0。
    #[test]
    fn absolute_hand_case() {
        for arm in [Arm::FreeNow, Arm::PinStripe] {
            let mut w = World::new();
            w.group.push([0, 1, 2, 3]);
            w.write_stripe(0, &[0, 1, 2], arm);
            let (old_sid, _) = w.loc[0][0];
            w.write_stripe(0, &[0], arm);
            if arm == Arm::FreeNow {
                let (d, i) = w.free_pool[0];
                w.cells[d][i] = Cell::Reused;
            }
            let s = &w.stripes[old_sid as usize];
            let (d1, _) = s.data[1];
            let loss = w.loss_if_dev_fails(d1);
            match arm {
                Arm::FreeNow => assert_eq!(loss, 1, "复用挤掉了重建输入，u1 必须丢"),
                Arm::PinStripe => {
                    assert_eq!(loss, 0, "钉住的格字节还在，u1 重建得回");
                    let (_, _, freed, _, _) = w.census();
                    assert_eq!(freed, 1, "恰钉 1 格");
                }
                Arm::RestripeOnFree | Arm::LazyRestripe => unreachable!(),
            }
        }
    }

    /// （二轮）**惰性臂手算锚点**：同一场景在 lazy 下——修之前钉 1 格丢 0；
    /// 一拍后台修复之后钉 0、丢 0、旧条带退休、幸存者整批搬进一条新条带。
    #[test]
    fn lazy_hand_case() {
        let mut w = World::new();
        w.group.push([0, 1, 2, 3]);
        w.write_stripe(0, &[0, 1, 2], Arm::LazyRestripe);
        let (old_sid, _) = w.loc[0][0];
        w.write_stripe(0, &[0], Arm::LazyRestripe);
        // 修之前：钉 1、丢 0（任一盘）
        assert_eq!(w.pinned_now(), 1);
        for d in 0..4 {
            assert_eq!(w.loss_if_dev_fails(d), 0, "钉住期间丢失必须为 0");
        }
        w.repair_tick();
        assert!(w.stripes[old_sid as usize].retired, "修复后旧条带退休");
        assert_eq!(w.pinned_now(), 0, "修复释放钉住格");
        for d in 0..4 {
            assert_eq!(w.loss_if_dev_fails(d), 0);
        }
        assert_eq!(w.repairs, 1);
    }

    /// **判据 2**：pin / eager / lazy 三臂丢失恒为 0（全部盘 × 全部种子 × 两种负载）。
    #[test]
    fn correctness_arms_lose_nothing() {
        for skew in [false, true] {
            for seed in [11u64, 22, 33, 44, 55] {
                for arm in [Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
                    let (w, _) = run(seed, arm, false, skew);
                    for d in 0..DEVS {
                        assert_eq!(
                            w.loss_if_dev_fails(d),
                            0,
                            "seed={seed} arm={arm:?} skew={skew} dev={d}"
                        );
                    }
                }
            }
        }
    }

    /// **判据 1 的负载版（阳性对照）**：free_now 在 w∈{2,3,4} 负载下必须真的丢——
    /// 丢失计数器读得出非零（C60（恒定读数没有故障注入自证）：恒零的读数与没在看是同一个样子）。
    #[test]
    fn free_now_actually_loses() {
        let (w, _) = run(11, Arm::FreeNow, false, false);
        let total: u64 = (0..DEVS).map(|d| w.loss_if_dev_fails(d)).sum();
        assert!(total > 0, "重写负载下 free_now 必须出丢失，否则模型没有判别力");
    }

    /// **判据 3 阴性对照**：恒 w=2 时四臂丢失 0、钉住 0、额外写 0。
    /// 这一格就是「第一版 2 盘看不见这个洞」的证明。
    #[test]
    fn w2_control_is_blind() {
        for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (w, _) = run(11, arm, true, false);
            for d in 0..DEVS {
                assert_eq!(w.loss_if_dev_fails(d), 0, "{arm:?}");
            }
            let (_, _, freed, _, _) = w.census();
            if matches!(arm, Arm::PinStripe | Arm::LazyRestripe) {
                assert_eq!(freed, 0, "w=2 条带成员一死条带就死透，钉不住任何东西（{arm:?}）");
            }
            assert_eq!(w.extra_writes, 0, "{arm:?}");
        }
    }

    /// **判据 4 守恒**：五类格数之和恒等于已分配总格数。
    #[test]
    fn conservation_holds() {
        for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (w, _) = run(22, arm, false, false);
            let (a, b, c, d, e) = w.census();
            let total: u64 = w.cells.iter().map(|c| c.len() as u64).sum();
            assert_eq!(a + b + c + d + e, total, "{arm:?}");
        }
    }

    /// 活数据守恒：任何臂跑完，live 恰等于 OBJ × NUNITS（每逻辑单元恰一份活的）。
    #[test]
    fn live_units_exact() {
        for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (w, _) = run(33, arm, false, false);
            let (live, _, _, _, _) = w.census();
            assert_eq!(live, (OBJ * NUNITS) as u64, "{arm:?}");
        }
    }

    /// 条带列互不同盘——重建论证的前提，破了全部丢失计数都不可信。
    #[test]
    fn stripe_columns_on_distinct_devs() {
        for arm in [Arm::FreeNow, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (w, _) = run(44, arm, false, false);
            for s in &w.stripes {
                let mut devs: Vec<usize> = s.data.iter().map(|&(d, _)| d).collect();
                devs.push(s.parity.0);
                let n = devs.len();
                devs.sort_unstable();
                devs.dedup();
                assert_eq!(devs.len(), n, "同一条带两列落在同一块盘（{arm:?}）");
            }
        }
    }

    /// 不同种子给出不同世界（C59（种子折叠成同一个状态）：多轮必须真的是多轮）。
    #[test]
    fn seeds_differ() {
        let a = run(11, Arm::FreeNow, false, false).0.census();
        let b = run(22, Arm::FreeNow, false, false).0.census();
        let c = run(33, Arm::FreeNow, false, false).0.census();
        assert!(a != b || b != c, "三个种子的世界不许折叠成同一个");
        // C59 的病灶形态：`seed | 1` 把相邻偶奇对折成同一个状态。直接钉 (2,3) 这一对。
        let w2 = run(2, Arm::FreeNow, false, false).0;
        let w3 = run(3, Arm::FreeNow, false, false).0;
        let l2: Vec<u64> = (0..DEVS).map(|d| w2.loss_if_dev_fails(d)).collect();
        let l3: Vec<u64> = (0..DEVS).map(|d| w3.loss_if_dev_fails(d)).collect();
        assert!(
            w2.census() != w3.census() || l2 != l3 || w2.stripes.len() != w3.stripes.len(),
            "种子 2 与 3 折叠成了同一个世界（C59 的形状）"
        );
    }

    /// （二轮，判据 8）skew 与 uniform 必须产出不同的世界——负载参数真的在起作用。
    #[test]
    fn skew_differs_from_uniform() {
        let u = run(11, Arm::LazyRestripe, false, false).0;
        let s = run(11, Arm::LazyRestripe, false, true).0;
        assert!(
            u.census() != s.census() || u.extra_writes != s.extra_writes || u.repairs != s.repairs,
            "skew 没起作用"
        );
    }

    /// pin 臂的钉住量在重写负载下必须非零——否则「代价」那一列量了个寂寞。
    #[test]
    fn pin_arm_pins_something() {
        let (w, _) = run(11, Arm::PinStripe, false, false);
        let (_, _, freed, _, _) = w.census();
        assert!(freed > 0);
    }

    /// eager 臂的额外写必须非零，且不许超过总写数（算术自检）。
    #[test]
    fn restripe_amplification_is_measured() {
        let (w, _) = run(11, Arm::RestripeOnFree, false, false);
        assert!(w.extra_writes > 0);
        assert!(w.extra_writes < w.total_writes);
    }

    /// （二轮，判据 6）惰性臂排空：负载停止后队列必须清空、钉住归零、丢失仍为 0。
    #[test]
    fn lazy_drains_to_zero() {
        for skew in [false, true] {
            let (w, drain) = run(11, Arm::LazyRestripe, false, skew);
            assert!(drain < DRAIN_CAP as u64, "排空没在上限内完成");
            assert_eq!(w.pinned_now(), 0, "排空后钉住必须归零（skew={skew}）");
            for d in 0..DEVS {
                assert_eq!(w.loss_if_dev_fails(d), 0);
            }
        }
    }

    /// （二轮，判据 7）同种子同负载下惰性的额外写不多于立刻修——
    /// 两臂共用修复实现，lazy 只可能少修（死透免修、多次释放并到一次修）。
    #[test]
    fn lazy_not_more_expensive_than_eager() {
        for skew in [false, true] {
            for seed in [11u64, 22, 33, 44, 55] {
                let (e, _) = run(seed, Arm::RestripeOnFree, false, skew);
                let (l, _) = run(seed, Arm::LazyRestripe, false, skew);
                assert!(
                    l.extra_writes <= e.extra_writes,
                    "seed={seed} skew={skew}: lazy {} > eager {}",
                    l.extra_writes,
                    e.extra_writes
                );
            }
        }
    }
}
