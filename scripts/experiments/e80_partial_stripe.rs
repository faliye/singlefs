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
//! D=8 盘、OBJECT_COUNT 个对象各 UNITS_PER_OBJECT 个逻辑单元；负载：先全量初写，再 ROUNDS 轮随机批量 COW 重写
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

const DEVICE_COUNT: usize = 8;
const DEVICES_PER_GROUP: usize = 4; // D2 已定项 8：对象绑组
const OBJECT_COUNT: usize = 24;
const UNITS_PER_OBJECT: usize = 12;
const ROUNDS: usize = 4000;
/// 惰性臂每轮修几条部分条带（后台整理的预算）。
const REPAIR_BUDGET: usize = 2;
/// 排空阶段的轮数上限——只是防呆，正常几百轮内必清空。
const DRAIN_ROUND_LIMIT: usize = 100_000;

/// C59（种子折叠成同一个状态）教训：先乘法混淆再保证非零，不许 `seed | 1`。
struct RandomGenerator(u64);
impl RandomGenerator {
    fn new(seed: u64) -> Self {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if state == 0 {
            state = 0xDEAD_BEEF;
        }
        RandomGenerator(state)
    }
    fn next_random(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, exclusive_upper_bound: u64) -> u64 {
        self.next_random() % exclusive_upper_bound
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
    /// (device, cell 下标) 数据成员
    data_members: Vec<(usize, usize)>,
    parity_cell: (usize, usize),
    live_count: usize,
    /// 已进惰性修复队列（去重用）
    is_queued_for_repair: bool,
    /// 已被修复或整条回收，格子全放掉了
    is_retired: bool,
}

struct World {
    cells: Vec<Vec<Cell>>, // 每盘一列格
    stripes: Vec<Stripe>,
    /// 对象 → 组（4 块盘）
    device_group_by_object: Vec<[usize; DEVICES_PER_GROUP]>,
    /// 对象逻辑单元 → (条带号, 成员序号)
    stripe_member_by_object_unit: Vec<Vec<(u32, usize)>>,
    free_pool: Vec<(usize, usize)>,
    /// 惰性臂的部分条带队列（先进先修）
    repair_queue: VecDeque<u32>,
    extra_writes: u64,
    total_writes: u64,
    repair_count: u64,
    peak_pinned: u64,
}

impl World {
    fn new() -> World {
        World {
            cells: vec![Vec::new(); DEVICE_COUNT],
            stripes: Vec::new(),
            device_group_by_object: Vec::new(),
            stripe_member_by_object_unit: vec![vec![(u32::MAX, 0); UNITS_PER_OBJECT]; OBJECT_COUNT],
            free_pool: Vec::new(),
            repair_queue: VecDeque::new(),
            extra_writes: 0,
            total_writes: 0,
            repair_count: 0,
            peak_pinned: 0,
        }
    }

    fn used_cell_count(&self, device: usize) -> usize {
        self.cells[device].iter().filter(|cell| !matches!(cell, Cell::Free)).count()
    }

    /// 拿一格：优先复用空闲池（对陈旧字节最坏），否则该盘尾部追加新格。
    fn take_cell(&mut self, device: usize) -> usize {
        if let Some(pool_position) = self.free_pool.iter().position(|&(pool_device, _)| pool_device == device) {
            let (_, cell_index) = self.free_pool.swap_remove(pool_position);
            self.cells[device][cell_index] = Cell::Reused; // 先标复用，写入者随后改
            return cell_index;
        }
        self.cells[device].push(Cell::Free);
        self.cells[device].len() - 1
    }

    /// 写一条条带：batch 个数据单元 + 1 parity，列取组内最空 w 盘。
    /// ⚠️ 条带号必须**先占位再填**：修复路径会在本循环中途往 `stripes` 推新条目，
    /// 进循环前抓 `len()` 当号会撞车（2026-09-02 第一版实测 panic，改成占位后消失）。
    fn write_stripe(&mut self, object: usize, units: &[usize], arm: Arm) {
        let stripe_width = (units.len() + 1).clamp(2, 4);
        assert_eq!(stripe_width, units.len() + 1, "批 1..=3 下 clamp 不应截断");
        let mut stripe_devices: Vec<usize> = self.device_group_by_object[object].to_vec();
        stripe_devices.sort_by_key(|&device| self.used_cell_count(device));
        stripe_devices.truncate(stripe_width);
        let stripe_number = self.stripes.len() as u32;
        self.stripes.push(Stripe {
            data_members: Vec::new(),
            parity_cell: (usize::MAX, usize::MAX),
            live_count: 0,
            is_queued_for_repair: false,
            is_retired: false,
        });
        for (member_index, &unit) in units.iter().enumerate() {
            let device = stripe_devices[member_index];
            let cell_index = self.take_cell(device);
            self.cells[device][cell_index] = Cell::Live(object as u32, unit as u32, stripe_number);
            self.stripes[stripe_number as usize].data_members.push((device, cell_index));
            self.stripes[stripe_number as usize].live_count += 1;
            self.total_writes += 1;
            // 释放旧单元（初写时 stripe_member_by_object_unit 是 MAX，没有旧的）。
            // stripe_member_by_object_unit 先指向新家再释放旧的——修复扫幸存者时不许再把这个单元搬一次。
            let (old_stripe_number, _) = self.stripe_member_by_object_unit[object][unit];
            self.stripe_member_by_object_unit[object][unit] = (stripe_number, member_index);
            if old_stripe_number != u32::MAX {
                self.free_member(old_stripe_number, object as u32, unit as u32, arm);
            }
        }
        let parity_device = stripe_devices[stripe_width - 1];
        let parity_cell_index = self.take_cell(parity_device);
        self.cells[parity_device][parity_cell_index] = Cell::Parity(stripe_number);
        self.total_writes += 1;
        self.stripes[stripe_number as usize].parity_cell = (parity_device, parity_cell_index);
    }

    /// 整条回收：把条带还占着的格（钉住的、parity）全放进池，条带退休。
    fn release_stripe(&mut self, stripe_number: u32) {
        let member_cells = self.stripes[stripe_number as usize].data_members.clone();
        let parity_cell = self.stripes[stripe_number as usize].parity_cell;
        for (device, cell_index) in member_cells {
            if matches!(self.cells[device][cell_index], Cell::FreedIntact(owner_stripe_number) if owner_stripe_number == stripe_number) {
                self.cells[device][cell_index] = Cell::Free;
                self.free_pool.push((device, cell_index));
            }
        }
        if matches!(self.cells[parity_cell.0][parity_cell.1], Cell::Parity(owner_stripe_number) if owner_stripe_number == stripe_number) {
            self.cells[parity_cell.0][parity_cell.1] = Cell::Free;
            self.free_pool.push(parity_cell);
        }
        self.stripes[stripe_number as usize].is_retired = true;
    }

    /// 修一条部分条带：幸存者**整批**重落一条新条带，旧条带整条回收。
    /// eager 与 lazy 共用这一个实现——两臂的差别只在「何时修」，不在「怎么修」。
    fn repair_stripe(&mut self, stripe_number: u32) {
        if self.stripes[stripe_number as usize].is_retired {
            return;
        }
        let member_cells = self.stripes[stripe_number as usize].data_members.clone();
        let survivors: Vec<(u32, u32)> = member_cells
            .iter()
            .filter_map(|&(device, cell_index)| match self.cells[device][cell_index] {
                Cell::Live(object, unit, owner_stripe_number) if owner_stripe_number == stripe_number => Some((object, unit)),
                _ => None,
            })
            .collect();
        // 旧格整条放掉（活格、钉住格、parity 都进池）
        for &(device, cell_index) in &member_cells {
            match self.cells[device][cell_index] {
                Cell::Live(_, _, owner_stripe_number) if owner_stripe_number == stripe_number => {
                    self.cells[device][cell_index] = Cell::Free;
                    self.free_pool.push((device, cell_index));
                }
                Cell::FreedIntact(owner_stripe_number) if owner_stripe_number == stripe_number => {
                    self.cells[device][cell_index] = Cell::Free;
                    self.free_pool.push((device, cell_index));
                }
                _ => {}
            }
        }
        let parity_cell = self.stripes[stripe_number as usize].parity_cell;
        if matches!(self.cells[parity_cell.0][parity_cell.1], Cell::Parity(owner_stripe_number) if owner_stripe_number == stripe_number) {
            self.cells[parity_cell.0][parity_cell.1] = Cell::Free;
            self.free_pool.push(parity_cell);
        }
        self.stripes[stripe_number as usize].is_retired = true;
        if survivors.is_empty() {
            return;
        }
        // 幸存者整批落一条新条带
        let writes_before_repair = self.total_writes;
        let stripe_width = (survivors.len() + 1).clamp(2, 4);
        let object = survivors[0].0 as usize;
        let mut stripe_devices: Vec<usize> = self.device_group_by_object[object].to_vec();
        stripe_devices.sort_by_key(|&device| self.used_cell_count(device));
        stripe_devices.truncate(stripe_width);
        let new_stripe_number = self.stripes.len() as u32;
        self.stripes.push(Stripe {
            data_members: Vec::new(),
            parity_cell: (usize::MAX, usize::MAX),
            live_count: 0,
            is_queued_for_repair: false,
            is_retired: false,
        });
        for (member_index, &(object, unit)) in survivors.iter().enumerate() {
            let device = stripe_devices[member_index];
            let cell_index = self.take_cell(device);
            self.cells[device][cell_index] = Cell::Live(object, unit, new_stripe_number);
            self.stripes[new_stripe_number as usize].data_members.push((device, cell_index));
            self.stripes[new_stripe_number as usize].live_count += 1;
            self.stripe_member_by_object_unit[object as usize][unit as usize] = (new_stripe_number, member_index);
            self.total_writes += 1;
        }
        let parity_device = stripe_devices[stripe_width - 1];
        let parity_cell_index = self.take_cell(parity_device);
        self.cells[parity_device][parity_cell_index] = Cell::Parity(new_stripe_number);
        self.total_writes += 1;
        self.stripes[new_stripe_number as usize].parity_cell = (parity_device, parity_cell_index);
        self.extra_writes += self.total_writes - writes_before_repair;
        self.repair_count += 1;
    }

    /// 惰性臂的后台一拍：按预算修队列头上的部分条带。
    fn repair_tick(&mut self) {
        for _ in 0..REPAIR_BUDGET {
            loop {
                match self.repair_queue.pop_front() {
                    Some(stripe_number) if self.stripes[stripe_number as usize].is_retired => continue, // 已死透整条回收过
                    Some(stripe_number) => {
                        self.repair_stripe(stripe_number);
                        break;
                    }
                    None => return,
                }
            }
        }
    }

    /// 一个条带成员死了。四条臂在这里分岔。
    fn free_member(&mut self, stripe_number: u32, object: u32, unit: u32, arm: Arm) {
        let stripe = &mut self.stripes[stripe_number as usize];
        let mut freed_position = None;
        for &(device, cell_index) in &stripe.data_members {
            if self.cells[device][cell_index] == Cell::Live(object, unit, stripe_number) {
                freed_position = Some((device, cell_index));
                break;
            }
        }
        let (device, cell_index) = freed_position.expect("成员必须在条带里");
        stripe.live_count -= 1;
        let is_stripe_dead = stripe.live_count == 0;
        match arm {
            Arm::FreeNow => {
                // 原始字节还在，但空间立刻可复用。
                self.cells[device][cell_index] = Cell::FreedIntact(stripe_number);
                self.free_pool.push((device, cell_index));
                if is_stripe_dead {
                    let parity_cell = self.stripes[stripe_number as usize].parity_cell;
                    self.cells[parity_cell.0][parity_cell.1] = Cell::FreedIntact(stripe_number);
                    self.free_pool.push(parity_cell);
                    self.stripes[stripe_number as usize].is_retired = true;
                }
            }
            Arm::PinStripe => {
                self.cells[device][cell_index] = Cell::FreedIntact(stripe_number);
                if is_stripe_dead {
                    self.release_stripe(stripe_number);
                }
            }
            Arm::RestripeOnFree => {
                self.cells[device][cell_index] = Cell::Free;
                self.free_pool.push((device, cell_index));
                if is_stripe_dead {
                    self.release_stripe(stripe_number);
                } else {
                    self.repair_stripe(stripe_number);
                }
            }
            Arm::LazyRestripe => {
                // 钉住，等后台修——不进池。
                self.cells[device][cell_index] = Cell::FreedIntact(stripe_number);
                if is_stripe_dead {
                    self.release_stripe(stripe_number);
                } else if !self.stripes[stripe_number as usize].is_queued_for_repair {
                    self.stripes[stripe_number as usize].is_queued_for_repair = true;
                    self.repair_queue.push_back(stripe_number);
                }
            }
        }
    }

    /// 单盘失效后不可重建的活单元数。重建一个丢失列要 parity + 其余全部数据列的**原始字节**：
    /// 其余成员必须不在失效盘上（列互不同盘按构造成立，有单测钉住），
    /// 且格内字节仍属本条带（活的、或释放后未被复用的都算原始字节在）。
    fn loss_if_device_fails(&self, failed_device: usize) -> u64 {
        let mut lost_unit_count = 0;
        for (cell_index, cell) in self.cells[failed_device].iter().enumerate() {
            if let Cell::Live(_, _, stripe_number) = cell {
                let stripe = &self.stripes[*stripe_number as usize];
                let other_members_intact = stripe
                    .data_members
                    .iter()
                    .filter(|&&(member_device, member_cell_index)| !(member_device == failed_device && member_cell_index == cell_index))
                    .all(|&(member_device, member_cell_index)| {
                        member_device != failed_device
                            && match self.cells[member_device][member_cell_index] {
                                Cell::Live(_, _, owner_stripe_number) => owner_stripe_number == *stripe_number,
                                Cell::FreedIntact(owner_stripe_number) => owner_stripe_number == *stripe_number, // 字节还在，能参与重建
                                Cell::Parity(_) | Cell::Reused | Cell::Free => false,
                            }
                    });
                let (parity_device, parity_cell_index) = stripe.parity_cell;
                let parity_intact =
                    parity_device != failed_device && matches!(self.cells[parity_device][parity_cell_index], Cell::Parity(owner_stripe_number) if owner_stripe_number == *stripe_number);
                if !(other_members_intact && parity_intact) {
                    lost_unit_count += 1;
                }
            }
        }
        lost_unit_count
    }

    /// 守恒检查的五个读数。
    fn census(&self) -> (u64, u64, u64, u64, u64) {
        let (mut live_count, mut parity_count, mut freed_intact_count, mut reused_count, mut free_count) = (0, 0, 0, 0, 0);
        for device_cells in &self.cells {
            for cell in device_cells {
                match cell {
                    Cell::Live(..) => live_count += 1,
                    Cell::Parity(_) => parity_count += 1,
                    Cell::FreedIntact(_) => freed_intact_count += 1,
                    Cell::Reused => reused_count += 1,
                    Cell::Free => free_count += 1,
                }
            }
        }
        (live_count, parity_count, freed_intact_count, reused_count, free_count)
    }

    fn pinned_now(&self) -> u64 {
        self.census().2
    }
}

/// 跑一臂：初写全部对象，再 ROUNDS 轮随机批量重写。
/// `width_two_only` 是阴性对照（批恒 1）；`is_skewed_workload` 让 80% 的重写落在前 20% 的对象上。
/// 惰性臂在负载后进入排空阶段，返回排空用的轮数。
fn run_arm(seed: u64, arm: Arm, width_two_only: bool, is_skewed_workload: bool) -> (World, u64) {
    let mut random_generator = RandomGenerator::new(seed);
    let mut world = World::new();
    for _ in 0..OBJECT_COUNT {
        let mut devices_by_usage: Vec<usize> = (0..DEVICE_COUNT).collect();
        devices_by_usage.sort_by_key(|&device| world.used_cell_count(device));
        let mut group = [0usize; DEVICES_PER_GROUP];
        group.copy_from_slice(&devices_by_usage[..DEVICES_PER_GROUP]);
        world.device_group_by_object.push(group);
    }
    for object in 0..OBJECT_COUNT {
        let mut next_unit = 0;
        while next_unit < UNITS_PER_OBJECT {
            let batch_size = if width_two_only { 1 } else { (random_generator.below(3) + 1) as usize };
            let batch_size = batch_size.min(UNITS_PER_OBJECT - next_unit);
            let units: Vec<usize> = (next_unit..next_unit + batch_size).collect();
            world.write_stripe(object, &units, arm);
            next_unit += batch_size;
        }
    }
    let hot_object_count = (OBJECT_COUNT / 5).max(1);
    for _ in 0..ROUNDS {
        let object = if is_skewed_workload && random_generator.below(10) < 8 {
            random_generator.below(hot_object_count as u64) as usize
        } else {
            random_generator.below(OBJECT_COUNT as u64) as usize
        };
        let batch_size = if width_two_only { 1 } else { (random_generator.below(3) + 1) as usize };
        let start_unit = random_generator.below((UNITS_PER_OBJECT - batch_size + 1) as u64) as usize;
        let units: Vec<usize> = (start_unit..start_unit + batch_size).collect();
        world.write_stripe(object, &units, arm);
        if arm == Arm::LazyRestripe {
            world.repair_tick();
        }
        let pinned_cells = world.pinned_now();
        if pinned_cells > world.peak_pinned {
            world.peak_pinned = pinned_cells;
        }
    }
    // 排空阶段（只有惰性臂有活干）：负载停了，后台按预算继续修。
    let mut drain_rounds = 0;
    if arm == Arm::LazyRestripe {
        for _ in 0..DRAIN_ROUND_LIMIT {
            if world.repair_queue.iter().all(|&stripe_number| world.stripes[stripe_number as usize].is_retired) {
                break;
            }
            world.repair_tick();
            drain_rounds += 1;
        }
    }
    (world, drain_rounds)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config devs={DEVICE_COUNT} g={DEVICES_PER_GROUP} obj={OBJECT_COUNT} nunits={UNITS_PER_OBJECT} rounds={ROUNDS} repair_budget={REPAIR_BUDGET} model=counting file_ops=0"
        ))
    );
    for is_skewed_workload in [false, true] {
        let workload_name = if is_skewed_workload { "skew" } else { "uniform" };
        for seed in [11u64, 22, 33, 44, 55] {
            for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
                let (world, drain_round_count) = run_arm(seed, arm, false, is_skewed_workload);
                let (live_count, parity_count, freed_intact_count, reused_count, free_count) = world.census();
                let total_cells: u64 = world.cells.iter().map(|device_cells| device_cells.len() as u64).sum();
                assert_eq!(live_count + parity_count + freed_intact_count + reused_count + free_count, total_cells, "守恒破了");
                let loss_by_failed_device: Vec<u64> = (0..DEVICE_COUNT).map(|failed_device| world.loss_if_device_fails(failed_device)).collect();
                let worst_loss = *loss_by_failed_device.iter().max().unwrap();
                let mean_loss = loss_by_failed_device.iter().sum::<u64>() as f64 / DEVICE_COUNT as f64;
                let pinned_cells = match arm {
                    Arm::PinStripe | Arm::LazyRestripe => freed_intact_count,
                    Arm::FreeNow | Arm::RestripeOnFree => 0,
                };
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=arm workload={workload_name} seed={seed} arm={} live={live_count} parity={parity_count} freed_intact={freed_intact_count} reused={reused_count} total={total_cells} loss_mean={mean_loss:.1} loss_worst={worst_loss} pinned={pinned_cells} pinned_pct_of_live={:.1} peak_pinned={} extra_writes={} write_amp_pct={:.1} repairs={} drain_rounds={drain_round_count}",
                        arm.tag(),
                        100.0 * pinned_cells as f64 / live_count as f64,
                        world.peak_pinned,
                        world.extra_writes,
                        100.0 * world.extra_writes as f64 / (world.total_writes - world.extra_writes) as f64,
                        world.repair_count
                    ))
                );
            }
        }
    }
    // 阴性对照：恒 w=2。各臂丢失与钉住都必须为 0。
    for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
        let (world, _) = run_arm(11, arm, true, false);
        let (live_count, _, freed_intact_count, _, _) = world.census();
        let worst_loss = (0..DEVICE_COUNT).map(|failed_device| world.loss_if_device_fails(failed_device)).max().unwrap();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=w2_control arm={} live={live_count} freed_intact={freed_intact_count} loss_worst={worst_loss} extra_writes={}",
                arm.tag(),
                world.extra_writes
            ))
        );
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 手算锚点**：单条 w=4 条带（3 数据 + parity），释放 u1 并复用其格，
    /// 坏掉 u2 所在盘 ⇒ free_now 恰丢 1；pin_stripe 恰钉 1 格且丢 0。
    #[test]
    fn absolute_hand_case() {
        for arm in [Arm::FreeNow, Arm::PinStripe] {
            let mut world = World::new();
            world.device_group_by_object.push([0, 1, 2, 3]);
            world.write_stripe(0, &[0, 1, 2], arm);
            let (old_stripe_number, _) = world.stripe_member_by_object_unit[0][0];
            world.write_stripe(0, &[0], arm);
            if arm == Arm::FreeNow {
                let (device, cell_index) = world.free_pool[0];
                world.cells[device][cell_index] = Cell::Reused;
            }
            let old_stripe = &world.stripes[old_stripe_number as usize];
            let (second_member_device, _) = old_stripe.data_members[1];
            let loss = world.loss_if_device_fails(second_member_device);
            match arm {
                Arm::FreeNow => assert_eq!(loss, 1, "复用挤掉了重建输入，u1 必须丢"),
                Arm::PinStripe => {
                    assert_eq!(loss, 0, "钉住的格字节还在，u1 重建得回");
                    let (_, _, freed_intact_count, _, _) = world.census();
                    assert_eq!(freed_intact_count, 1, "恰钉 1 格");
                }
                Arm::RestripeOnFree | Arm::LazyRestripe => unreachable!(),
            }
        }
    }

    /// （二轮）**惰性臂手算锚点**：同一场景在 lazy 下——修之前钉 1 格丢 0；
    /// 一拍后台修复之后钉 0、丢 0、旧条带退休、幸存者整批搬进一条新条带。
    #[test]
    fn lazy_hand_case() {
        let mut world = World::new();
        world.device_group_by_object.push([0, 1, 2, 3]);
        world.write_stripe(0, &[0, 1, 2], Arm::LazyRestripe);
        let (old_stripe_number, _) = world.stripe_member_by_object_unit[0][0];
        world.write_stripe(0, &[0], Arm::LazyRestripe);
        // 修之前：钉 1、丢 0（任一盘）
        assert_eq!(world.pinned_now(), 1);
        for failed_device in 0..4 {
            assert_eq!(world.loss_if_device_fails(failed_device), 0, "钉住期间丢失必须为 0");
        }
        world.repair_tick();
        assert!(world.stripes[old_stripe_number as usize].is_retired, "修复后旧条带退休");
        assert_eq!(world.pinned_now(), 0, "修复释放钉住格");
        for failed_device in 0..4 {
            assert_eq!(world.loss_if_device_fails(failed_device), 0);
        }
        assert_eq!(world.repair_count, 1);
    }

    /// **判据 2**：pin / eager / lazy 三臂丢失恒为 0（全部盘 × 全部种子 × 两种负载）。
    #[test]
    fn correctness_arms_lose_nothing() {
        for is_skewed_workload in [false, true] {
            for seed in [11u64, 22, 33, 44, 55] {
                for arm in [Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
                    let (world, _) = run_arm(seed, arm, false, is_skewed_workload);
                    for failed_device in 0..DEVICE_COUNT {
                        assert_eq!(
                            world.loss_if_device_fails(failed_device),
                            0,
                            "seed={seed} arm={arm:?} skew={is_skewed_workload} dev={failed_device}"
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
        let (world, _) = run_arm(11, Arm::FreeNow, false, false);
        let total_loss: u64 = (0..DEVICE_COUNT).map(|failed_device| world.loss_if_device_fails(failed_device)).sum();
        assert!(total_loss > 0, "重写负载下 free_now 必须出丢失，否则模型没有判别力");
    }

    /// **判据 3 阴性对照**：恒 w=2 时四臂丢失 0、钉住 0、额外写 0。
    /// 这一格就是「第一版 2 盘看不见这个洞」的证明。
    #[test]
    fn width_two_control_is_blind() {
        for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (world, _) = run_arm(11, arm, true, false);
            for failed_device in 0..DEVICE_COUNT {
                assert_eq!(world.loss_if_device_fails(failed_device), 0, "{arm:?}");
            }
            let (_, _, freed_intact_count, _, _) = world.census();
            if matches!(arm, Arm::PinStripe | Arm::LazyRestripe) {
                assert_eq!(freed_intact_count, 0, "w=2 条带成员一死条带就死透，钉不住任何东西（{arm:?}）");
            }
            assert_eq!(world.extra_writes, 0, "{arm:?}");
        }
    }

    /// **判据 4 守恒**：五类格数之和恒等于已分配总格数。
    #[test]
    fn conservation_holds() {
        for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (world, _) = run_arm(22, arm, false, false);
            let (live_count, parity_count, freed_intact_count, reused_count, free_count) = world.census();
            let total_cells: u64 = world.cells.iter().map(|device_cells| device_cells.len() as u64).sum();
            assert_eq!(live_count + parity_count + freed_intact_count + reused_count + free_count, total_cells, "{arm:?}");
        }
    }

    /// 活数据守恒：任何臂跑完，live 恰等于 OBJECT_COUNT × UNITS_PER_OBJECT（每逻辑单元恰一份活的）。
    #[test]
    fn live_units_exact() {
        for arm in [Arm::FreeNow, Arm::PinStripe, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (world, _) = run_arm(33, arm, false, false);
            let (live_count, _, _, _, _) = world.census();
            assert_eq!(live_count, (OBJECT_COUNT * UNITS_PER_OBJECT) as u64, "{arm:?}");
        }
    }

    /// 条带列互不同盘——重建论证的前提，破了全部丢失计数都不可信。
    #[test]
    fn stripe_columns_on_distinct_devices() {
        for arm in [Arm::FreeNow, Arm::RestripeOnFree, Arm::LazyRestripe] {
            let (world, _) = run_arm(44, arm, false, false);
            for stripe in &world.stripes {
                let mut column_devices: Vec<usize> = stripe.data_members.iter().map(|&(device, _)| device).collect();
                column_devices.push(stripe.parity_cell.0);
                let column_count = column_devices.len();
                column_devices.sort_unstable();
                column_devices.dedup();
                assert_eq!(column_devices.len(), column_count, "同一条带两列落在同一块盘（{arm:?}）");
            }
        }
    }

    /// 不同种子给出不同世界（C59（种子折叠成同一个状态）：多轮必须真的是多轮）。
    #[test]
    fn seeds_differ() {
        let census_seed_11 = run_arm(11, Arm::FreeNow, false, false).0.census();
        let census_seed_22 = run_arm(22, Arm::FreeNow, false, false).0.census();
        let census_seed_33 = run_arm(33, Arm::FreeNow, false, false).0.census();
        assert!(census_seed_11 != census_seed_22 || census_seed_22 != census_seed_33, "三个种子的世界不许折叠成同一个");
        // C59 的病灶形态：`seed | 1` 把相邻偶奇对折成同一个状态。直接钉 (2,3) 这一对。
        let world_seed_2 = run_arm(2, Arm::FreeNow, false, false).0;
        let world_seed_3 = run_arm(3, Arm::FreeNow, false, false).0;
        let losses_seed_2: Vec<u64> = (0..DEVICE_COUNT).map(|failed_device| world_seed_2.loss_if_device_fails(failed_device)).collect();
        let losses_seed_3: Vec<u64> = (0..DEVICE_COUNT).map(|failed_device| world_seed_3.loss_if_device_fails(failed_device)).collect();
        assert!(
            world_seed_2.census() != world_seed_3.census() || losses_seed_2 != losses_seed_3 || world_seed_2.stripes.len() != world_seed_3.stripes.len(),
            "种子 2 与 3 折叠成了同一个世界（C59 的形状）"
        );
    }

    /// （二轮，判据 8）skew 与 uniform 必须产出不同的世界——负载参数真的在起作用。
    #[test]
    fn skew_differs_from_uniform() {
        let uniform_world = run_arm(11, Arm::LazyRestripe, false, false).0;
        let skewed_world = run_arm(11, Arm::LazyRestripe, false, true).0;
        assert!(
            uniform_world.census() != skewed_world.census() || uniform_world.extra_writes != skewed_world.extra_writes || uniform_world.repair_count != skewed_world.repair_count,
            "skew 没起作用"
        );
    }

    /// pin 臂的钉住量在重写负载下必须非零——否则「代价」那一列量了个寂寞。
    #[test]
    fn pin_arm_pins_something() {
        let (world, _) = run_arm(11, Arm::PinStripe, false, false);
        let (_, _, freed_intact_count, _, _) = world.census();
        assert!(freed_intact_count > 0);
    }

    /// eager 臂的额外写必须非零，且不许超过总写数（算术自检）。
    #[test]
    fn restripe_amplification_is_measured() {
        let (world, _) = run_arm(11, Arm::RestripeOnFree, false, false);
        assert!(world.extra_writes > 0);
        assert!(world.extra_writes < world.total_writes);
    }

    /// （二轮，判据 6）惰性臂排空：负载停止后队列必须清空、钉住归零、丢失仍为 0。
    #[test]
    fn lazy_drains_to_zero() {
        for is_skewed_workload in [false, true] {
            let (world, drain_round_count) = run_arm(11, Arm::LazyRestripe, false, is_skewed_workload);
            assert!(drain_round_count < DRAIN_ROUND_LIMIT as u64, "排空没在上限内完成");
            assert_eq!(world.pinned_now(), 0, "排空后钉住必须归零（skew={is_skewed_workload}）");
            for failed_device in 0..DEVICE_COUNT {
                assert_eq!(world.loss_if_device_fails(failed_device), 0);
            }
        }
    }

    /// （二轮，判据 7）同种子同负载下惰性的额外写不多于立刻修——
    /// 两臂共用修复实现，lazy 只可能少修（死透免修、多次释放并到一次修）。
    #[test]
    fn lazy_not_more_expensive_than_eager() {
        for is_skewed_workload in [false, true] {
            for seed in [11u64, 22, 33, 44, 55] {
                let (eager_world, _) = run_arm(seed, Arm::RestripeOnFree, false, is_skewed_workload);
                let (lazy_world, _) = run_arm(seed, Arm::LazyRestripe, false, is_skewed_workload);
                assert!(
                    lazy_world.extra_writes <= eager_world.extra_writes,
                    "seed={seed} skew={is_skewed_workload}: lazy {} > eager {}",
                    lazy_world.extra_writes,
                    eager_world.extra_writes
                );
            }
        }
    }
}
