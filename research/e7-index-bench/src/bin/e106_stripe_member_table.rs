//! E106：条带成员表的载体——单盘失效之后哪些格重建不出来，以及各条载体的代价。
//!
//! **它答的是** D2 已定项 12（parity 列的位置与发现机制）的一格：成员表放在哪，
//! 决定了「一次单盘失效之后，坏盘上的格还能不能重建」。四条臂：
//!
//! | 臂 | 成员表住哪 | 自举依赖 |
//! |---|---|---|
//! | 甲   | 码 3 打包记录（类型 5），容器与数据同池摆放 | 容器自己也落在条带里，丢了要先重建容器 |
//! | 甲镜 | 同甲，但容器恒走 w = 2 镜像 | 一次单盘失效丢不掉容器 |
//! | 乙   | parity 格自己的头（成员单元预留恒零区） | 无（parity 自证）|
//! | 丙   | 哪儿都不住（今天的状态）| 不适用，阳性对照 |
//!
//! 臂名不用拉丁字母：`A2` 与 D9（加密） 的前提编号 A2（超级块是明文）撞名，doc-lint 判红。
//!
//! ## 判据（跑前写死，2026-09-06）
//!
//! 1. **阳性对照**：臂丙的不可解格数必须**恰好等于**坏盘上的活格数（另行独立数出来）。
//!    对不上说明这套度量分不出差别，**整轮作废**。
//! 2. 臂乙与臂甲镜的不可解格数恒 0，且这是**结构性的**：乙没有依赖边，甲镜的容器丢不掉。
//! 3. 臂甲的不可解格数按种子钉死绝对值。**接受两种结果**：全部种子上都是 0 ⇒
//!    提案 P4 那个担心不成立，候选甲不需要镜像补丁；有非 0 ⇒ 那条自举链是实的。
//!    这句话在跑之前写下（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`
//!    「结果反过来我接不接受」）。
//! 4. **阴性对照**：不失效（无坏盘）⇒ 四条臂的不可解格数全 0。
//! 5. 代价按 16 TiB、90% 填充、单元 32768 算，报 ppm 绝对值；单元数必须等于 E97
//!    已入库的 483 183 820（同一口径的交叉校验，对不上说明两个模型有一个算错了）。
//! 6. 不可解的判定用不动点迭代；**必须证明迭代真的在动**：给出至少一个种子上
//!    「第一轮解不开、后面几轮解开」的格数非 0，否则那个循环等于没跑。
//!
//! **不答**：挂钟、真实分配器的落点分布、多盘同坏（w = 3 只容一格）、
//! 修复的写放大（E80 量过那一格）。落点分布取均匀随机是**建模选择**，不是实测。

use e7_index_bench::Emitter;

/// 码 3 容器 32768、头 107（D18 已定项 11）。
const UNIT_BYTES: u64 = 32768;
const PACKED_HEADER_BYTES: u64 = 107;
/// 条带成员记录定长 56：格宽标志 1 + w 1 + 自己是第几列 1 + 成员表 4 × 10 + 条带诞生代 8 + 补齐 5。
const STRIPE_RECORD_BYTES: u64 = 56;
/// 臂乙的恒零预留区：共同前缀 42 + fsid 8 + 诞生代号 8 + 写序 10 + 成员数 1 + 成员表 40，
/// 向上取到 16 的倍数 ⇒ 128。
const PARITY_HEADER_RESERVE_BYTES: u64 = 128;
/// 16 TiB。
const POOL_CAPACITY_BYTES: u64 = 16 * 1024 * 1024 * 1024 * 1024;
const FILL_RATIO: f64 = 0.9;

fn records_per_container() -> u64 {
    (UNIT_BYTES - PACKED_HEADER_BYTES) / STRIPE_RECORD_BYTES
}

/// 容器容量做成参数：真实几何是 583，而 583 让容器只占落点的千分之二，
/// 自举那条依赖链在那个几何上**碰不到**。小容量档是灵敏度探针（不是真实配置），
/// 用来证明不动点迭代真的在动、并找出这条链从哪个比例起开始咬人。
#[derive(Clone, Copy, PartialEq, Debug)]
enum Placement {
    /// 均匀随机挑 w 块盘。**这是建模选择，不是本工程的规则。**
    Random,
    /// D2 已定项 8 的「取组内最空的 w 块」：确定性、全局同步 ⇒ 容器与它描述的格**正相关**。
    Emptiest,
}

#[derive(Clone, Copy, Debug)]
struct LayoutParameters {
    device_count: usize,
    container_capacity_in_records: u64,
    placement: Placement,
    /// 加盘窗口：最后一块盘是刚加进来的空盘，其余盘已经写到 `prefilled_slots` 槽。
    /// D2（RAID 条带策略） 已定项 8 的「取最空的」是确定性且全局同步的规则 ⇒ 新盘会被每条新条带选中，
    /// 重平衡重写出来的容器也堆在它上面 ⇒ 容器与它描述的格**正相关**，
    /// 而「稳态恒 0」的全部承重点正是那个独立性（2026-09-06 第二轮攻击腿 5.3）。
    prefilled_slots: u64,
}

/// 确定性伪随机：LCG，只用来选设备，不参与任何判定。
struct LinearCongruentialGenerator(u64);
impl LinearCongruentialGenerator {
    fn next_random(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    /// 从 device_count 块盘里取 pick_count 块互不相同的（pick_count ≤ device_count）。
    fn pick_distinct_devices(&mut self, device_count: usize, pick_count: usize) -> Vec<usize> {
        let mut device_pool: Vec<usize> = (0..device_count).collect();
        let mut picked = Vec::with_capacity(pick_count);
        for pick_index in 0..pick_count {
            let swap_index = pick_index + (self.next_random() as usize) % (device_pool.len() - pick_index);
            device_pool.swap(pick_index, swap_index);
            picked.push(device_pool[pick_index]);
        }
        picked
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum GridKind { Ordinary, Container, Parity }

#[derive(Clone, Debug)]
struct Grid {
    device: usize,
    slot: u64,
    kind: GridKind,
    stripe_index: usize,
}

#[derive(Clone, Debug)]
struct Layout {
    grids: Vec<Grid>,
    /// 每条条带的成员（下标进 grids）
    stripe_members: Vec<Vec<usize>>,
    /// 逻辑容器号 → 它的各份副本在 grids 里的下标（臂甲镜下一个容器有两份）
    containers: Vec<Vec<usize>>,
    /// 每个 grid 的记录住在哪个容器号；臂乙 / 丙为 None
    record_container: Vec<Option<usize>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm { PooledContainer, MirroredContainer, ParityHeader, NoMemberTable }

/// 容器数的不动点：容器自己也占落点、也要一条记录。
fn container_count(arm: Arm, ordinary_unit_count: u64, stripe_widths: &[usize], container_capacity_in_records: u64) -> u64 {
    if arm == Arm::ParityHeader || arm == Arm::NoMemberTable { return 0; }
    let mut container_total = 0u64;
    for _ in 0..64 {
        let unit_count = ordinary_unit_count + container_total;
        let location_count = unit_count + stripe_count(unit_count, stripe_widths);
        let next_container_total = location_count.div_ceil(container_capacity_in_records);
        if next_container_total == container_total { break; }
        container_total = next_container_total;
    }
    container_total
}

/// 单元按宽度循环攒批，每条条带装 w−1 个单元 ⇒ 条带数。逐条带循环。
fn stripe_count_by_loop(unit_count: u64, stripe_widths: &[usize]) -> u64 {
    let mut units_left = unit_count;
    let mut stripe_total = 0u64;
    let mut stripe_index = 0usize;
    while units_left > 0 {
        let stripe_width = stripe_widths[stripe_index % stripe_widths.len()] as u64;
        let units_in_this_stripe = (stripe_width - 1).min(units_left);
        units_left -= units_in_this_stripe;
        stripe_total += 1;
        stripe_index += 1;
    }
    stripe_total
}

/// 闭式：一轮宽度循环装 Σ(w−1) 个单元、出 widths.len() 条条带；余数再逐条带补。
/// 4.8 亿个单元上逐条带循环跑不动（第一版的代价式子因此慢到 40 秒），闭式与循环版
/// 在小值上逐格相等，由单测 `closed_form_matches_loop` 钉住。
fn stripe_count(unit_count: u64, stripe_widths: &[usize]) -> u64 {
    let units_per_cycle: u64 = stripe_widths.iter().map(|&stripe_width| stripe_width as u64 - 1).sum();
    let full_cycles = unit_count / units_per_cycle;
    let mut units_left = unit_count % units_per_cycle;
    let mut stripe_total = full_cycles * stripe_widths.len() as u64;
    let mut position_in_cycle = 0usize;
    while units_left > 0 {
        let units_in_this_stripe = (stripe_widths[position_in_cycle] as u64 - 1).min(units_left);
        units_left -= units_in_this_stripe;
        stripe_total += 1;
        position_in_cycle += 1;
    }
    stripe_total
}

fn build_layout(arm: Arm, ordinary_unit_count: u64, parameters: LayoutParameters, stripe_widths: &[usize], seed: u64) -> Layout {
    let device_count = parameters.device_count;
    let containers_needed = container_count(arm, ordinary_unit_count, stripe_widths, parameters.container_capacity_in_records);
    let mut random_generator = LinearCongruentialGenerator(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut grids: Vec<Grid> = Vec::new();
    let mut stripe_members: Vec<Vec<usize>> = Vec::new();
    let mut next_slot_by_device = vec![parameters.prefilled_slots; device_count];
    if parameters.prefilled_slots > 0 { next_slot_by_device[device_count - 1] = 0; }

    // 待摆放的单元：先普通单元，再容器（容器号 = 出现顺序）
    let mut pending_unit_kinds: Vec<GridKind> = Vec::new();
    for _ in 0..ordinary_unit_count { pending_unit_kinds.push(GridKind::Ordinary); }
    for _ in 0..containers_needed { pending_unit_kinds.push(GridKind::Container); }

    let mut next_pending_index = 0usize;
    let mut width_cycle_position = 0usize;
    while next_pending_index < pending_unit_kinds.len() {
        let unit_kind = pending_unit_kinds[next_pending_index];
        // 臂甲镜：容器恒走 w = 2 镜像（两份副本，无 parity 格）
        let is_mirrored = arm == Arm::MirroredContainer && unit_kind == GridKind::Container;
        let stripe_width = if is_mirrored { 2 } else { stripe_widths[width_cycle_position % stripe_widths.len()] };
        width_cycle_position += 1;
        let picked_devices = match parameters.placement {
            Placement::Random => random_generator.pick_distinct_devices(device_count, stripe_width),
            Placement::Emptiest => {
                let mut devices_by_emptiness: Vec<usize> = (0..device_count).collect();
                devices_by_emptiness.sort_by_key(|&device| (next_slot_by_device[device], device));
                devices_by_emptiness.truncate(stripe_width);
                devices_by_emptiness
            }
        };
        let stripe_index = stripe_members.len();
        let mut member_grids = Vec::new();
        if is_mirrored {
            for &device in &picked_devices {
                let new_grid_index = grids.len();
                grids.push(Grid { device, slot: next_slot_by_device[device], kind: GridKind::Container, stripe_index });
                next_slot_by_device[device] += 1;
                member_grids.push(new_grid_index);
            }
            next_pending_index += 1;
        } else {
            // 臂甲镜里容器恒走镜像 ⇒ 一个攒批不许既装普通单元又装容器，
            // 否则边界那一批会把容器裹进 w = 4 的条带，镜像那条规则等于没生效
            let same_kind_run_length = if arm == Arm::MirroredContainer {
                pending_unit_kinds[next_pending_index..].iter().take_while(|&&queued_kind| queued_kind == unit_kind).count()
            } else {
                pending_unit_kinds.len() - next_pending_index
            };
            let units_in_this_stripe = (stripe_width - 1).min(same_kind_run_length);
            for column in 0..units_in_this_stripe {
                let device = picked_devices[column];
                let new_grid_index = grids.len();
                grids.push(Grid { device, slot: next_slot_by_device[device], kind: pending_unit_kinds[next_pending_index + column], stripe_index });
                next_slot_by_device[device] += 1;
                member_grids.push(new_grid_index);
            }
            // parity 格落在剩下那一列
            let device = picked_devices[stripe_width - 1];
            let new_grid_index = grids.len();
            grids.push(Grid { device, slot: next_slot_by_device[device], kind: GridKind::Parity, stripe_index });
            next_slot_by_device[device] += 1;
            member_grids.push(new_grid_index);
            next_pending_index += units_in_this_stripe;
        }
        stripe_members.push(member_grids);
    }

    // 逻辑容器：臂甲镜下两份镜像是**同一个**容器的两份副本，不是两个容器。
    // 按 grid 顺序聚合——镜像的两份是同一条镜像条带里连着放的
    let mut containers: Vec<Vec<usize>> = Vec::new();
    for (grid_index, grid) in grids.iter().enumerate() {
        if grid.kind != GridKind::Container { continue; }
        if arm == Arm::MirroredContainer {
            let is_second_mirror_copy = containers.last().map(|last_container_copies: &Vec<usize>| {
                grids[last_container_copies[0]].stripe_index == grid.stripe_index && last_container_copies.len() < 2
            }).unwrap_or(false);
            if is_second_mirror_copy { containers.last_mut().unwrap().push(grid_index); continue; }
        }
        containers.push(vec![grid_index]);
    }

    // 记录按 key =(设备, 槽号) 排序装容器 —— btree 叶按 key 顺序打包
    let mut record_container = vec![None; grids.len()];
    if !containers.is_empty() {
        let mut grids_in_key_order: Vec<usize> = (0..grids.len()).collect();
        grids_in_key_order.sort_by_key(|&grid_index| (grids[grid_index].device, grids[grid_index].slot));
        let container_capacity_in_records = parameters.container_capacity_in_records as usize;
        // 落点数比不动点估的多一格就会溢出最后一个容器 —— 静默夹住会让「记录都装得下」
        // 这件事永远为真而没人发现，所以这里判死
        assert!(grids_in_key_order.len().div_ceil(container_capacity_in_records) <= containers.len(),
            "容器不够装：落点 {} 容量 {container_capacity_in_records} 容器 {}", grids_in_key_order.len(), containers.len());
        for (rank_in_key_order, &grid_index) in grids_in_key_order.iter().enumerate() {
            record_container[grid_index] = Some(rank_in_key_order / container_capacity_in_records);
        }
    }
    Layout { grids, stripe_members, containers, record_container }
}

/// 单盘失效之后重建不出来的格数；`iters` 报不动点迭代真的推进了几轮、第一轮之后又解开多少。
fn count_unsolvable_grids(arm: Arm, layout: &Layout, maybe_failed_device: Option<usize>) -> (u64, u64, u64) {
    let failed_device = match maybe_failed_device { Some(device) => device, None => return (0, 0, 0) };
    let lost_grid_indices: Vec<usize> = layout.grids.iter().enumerate()
        .filter(|(_, grid)| grid.device == failed_device).map(|(grid_index, _)| grid_index).collect();
    if arm == Arm::NoMemberTable { return (lost_grid_indices.len() as u64, 0, 0); }
    if arm == Arm::ParityHeader { return (0, 0, 0); }

    let mut is_solved = vec![false; layout.grids.len()];
    let mut solved_in_first_round = 0u64;
    let mut round_count = 0u64;
    loop {
        let mut solved_any_this_round = false;
        for &lost_grid_index in &lost_grid_indices {
            if is_solved[lost_grid_index] { continue; }
            let stripe_index = layout.grids[lost_grid_index].stripe_index;
            for &member in &layout.stripe_members[stripe_index] {
                let Some(container) = layout.record_container[member] else { continue };
                // 任一副本可读或已重建，这条记录就到手
                if layout.containers[container].iter().any(|&copy_grid| layout.grids[copy_grid].device != failed_device || is_solved[copy_grid]) {
                    is_solved[lost_grid_index] = true;
                    solved_any_this_round = true;
                    break;
                }
            }
        }
        round_count += 1;
        if round_count == 1 { solved_in_first_round = lost_grid_indices.iter().filter(|&&lost_grid_index| is_solved[lost_grid_index]).count() as u64; }
        if !solved_any_this_round { break; }
    }
    let unsolvable_count = lost_grid_indices.iter().filter(|&&lost_grid_index| !is_solved[lost_grid_index]).count() as u64;
    let solved_after_first_round = lost_grid_indices.iter().filter(|&&lost_grid_index| is_solved[lost_grid_index]).count() as u64 - solved_in_first_round;
    (unsolvable_count, round_count, solved_after_first_round)
}

/// 池刚建好、条带表只有一个容器时，那个容器落在**它自己描述的那条条带**里。
/// 这一格是第一版提案 P4 说的「成对互指」的真形态：不是互指，是自指，
/// 而且它落在 w ≥ 3 的第一条条带上（2026-09-06 反推腿第五节）。
/// 加盘窗口：其余盘已写满 `prefilled_slots` 槽，最后一块是空盘；按「取最空的」摆放，
/// 新盘一定进每条新条带。返回布局与新盘号。
fn add_disk_window(arm: Arm, ordinary_unit_count: u64, device_count: usize, prefilled_slots: u64, stripe_widths: &[usize]) -> (Layout, usize) {
    let parameters = LayoutParameters { device_count, container_capacity_in_records: records_per_container(), placement: Placement::Emptiest, prefilled_slots };
    (build_layout(arm, ordinary_unit_count, parameters, stripe_widths, 1), device_count - 1)
}

fn first_stripe_self_reference(device_count: usize, seed: u64) -> (Layout, usize) {
    let layout = build_layout(Arm::PooledContainer, 2, LayoutParameters { device_count, container_capacity_in_records: records_per_container(), placement: Placement::Random, prefilled_slots: 0 }, &[4usize], seed);
    let container_device = layout.grids[layout.containers[0][0]].device;
    (layout, container_device)
}

fn lost_count(layout: &Layout, failed_device: usize) -> u64 {
    layout.grids.iter().filter(|grid| grid.device == failed_device).count() as u64
}

/// 池里被占用的落点总数 = 容量 × 填充率 / 单元大小。**含数据格、parity 格与容器**——
/// 第一版把它当成「数据格数」，于是往一块声称 90% 填充的盘里摆了 126.3% 的格
/// （2026-09-06 第二轮攻击腿 5.5 算出来的）。判据 5 只钉了数据格数，没钉这个量。
fn occupied_slots() -> u64 {
    (POOL_CAPACITY_BYTES as f64 * FILL_RATIO / UNIT_BYTES as f64) as u64
}

/// 一条臂在固定 90% 占用下装得下多少**数据格**。
/// parity 按每条条带一格（宽度循环 3/4 ⇒ 每 5 个成员格配 2 个 parity 格）算；
/// 容器自己也是成员格、也要 parity、也要一条自己的记录。
fn data_units(arm: Arm, stripe_widths: &[usize], container_capacity_in_records: u64) -> (u64, u64) {
    let occupied_slot_count = occupied_slots();
    // members = 数据格 + 容器；occ = members + parity(members)
    let member_count = {
        let mut lower_bound = 0u64;
        let mut upper_bound = occupied_slot_count;
        while lower_bound < upper_bound {
            let midpoint = (lower_bound + upper_bound + 1) / 2;
            if midpoint + stripe_count(midpoint, stripe_widths) <= occupied_slot_count { lower_bound = midpoint; } else { upper_bound = midpoint - 1; }
        }
        lower_bound
    };
    let copies_per_container = match arm { Arm::MirroredContainer => 2, Arm::PooledContainer => 1, _ => 0 };
    if copies_per_container == 0 { return (member_count, 0); }
    // 记录数 = 全部落点（含 parity 与容器自己）；容器数 = 记录数 / container_capacity_in_records，副本数 × copies_per_container
    let mut container_total = 0u64;
    for _ in 0..64 {
        let data_unit_count = member_count.saturating_sub(copies_per_container * container_total);
        let location_count = data_unit_count + copies_per_container * container_total + stripe_count(data_unit_count + copies_per_container * container_total, stripe_widths);
        let next_container_total = location_count.div_ceil(container_capacity_in_records);
        if next_container_total == container_total { break; }
        container_total = next_container_total;
    }
    (member_count.saturating_sub(copies_per_container * container_total), container_total)
}

/// 这条臂**吃掉多少用户数据**，相对臂丙（不带成员表）的差，按容量的 ppm 报。
/// 口径换过一次：第一版报的是「元数据自己占多少字节」，分母对不上填充率。
fn cost_bytes(arm: Arm, stripe_widths: &[usize]) -> (u64, f64) {
    let (baseline_data_units, _) = data_units(Arm::NoMemberTable, stripe_widths, records_per_container());
    let cost_in_bytes = match arm {
        Arm::NoMemberTable => 0,
        // 臂乙不多占格，它从每个成员格的净荷里扣 H 字节（含容器所在的那些格：臂乙没有容器）
        Arm::ParityHeader => PARITY_HEADER_RESERVE_BYTES * baseline_data_units,
        Arm::PooledContainer | Arm::MirroredContainer => {
            let (data_unit_count, _) = data_units(arm, stripe_widths, records_per_container());
            (baseline_data_units - data_unit_count) * UNIT_BYTES
        }
    };
    (cost_in_bytes, cost_in_bytes as f64 * 1e6 / POOL_CAPACITY_BYTES as f64)
}

fn arm_name(arm: Arm) -> &'static str {
    match arm { Arm::PooledContainer => "甲_容器同池", Arm::MirroredContainer => "甲镜_容器镜像", Arm::ParityHeader => "乙_parity自证", Arm::NoMemberTable => "丙_无成员表" }
}

fn main() {
    let mut emitter = Emitter::new();
    let stripe_widths = [3usize, 4];
    let device_count = 8usize;
    let ordinary_unit_count = 4096u64;
    let real_parameters = LayoutParameters { device_count, container_capacity_in_records: records_per_container(), placement: Placement::Random, prefilled_slots: 0 };
    println!("{}", emitter.emit_raw(&format!(
        "name=config note=条带成员表的载体 devs={device_count} n_ord={ordinary_unit_count} widths=3,4 unit_bytes={UNIT_BYTES} \
packed_hdr={PACKED_HEADER_BYTES} rec={STRIPE_RECORD_BYTES} per_container={} reserve_h={PARITY_HEADER_RESERVE_BYTES} cap_bytes={POOL_CAPACITY_BYTES} fill={FILL_RATIO}",
        records_per_container())));

    for arm in [Arm::PooledContainer, Arm::MirroredContainer, Arm::ParityHeader, Arm::NoMemberTable] {
        for seed in 1..=5u64 {
            let layout = build_layout(arm, ordinary_unit_count, real_parameters, &stripe_widths, seed);
            let (unsolvable_count, round_count, solved_after_first_round) = count_unsolvable_grids(arm, &layout, Some(0));
            let lost_grid_count = lost_count(&layout, 0);
            println!("{}", emitter.emit_raw(&format!(
                "name=fail1 arm={} seed={seed} grids={} stripes={} containers={} lost={lost_grid_count} unsolvable={unsolvable_count} rounds={round_count} solved_after_first={solved_after_first_round}",
                arm_name(arm), layout.grids.len(), layout.stripe_members.len(), layout.containers.len())));
        }
        let layout = build_layout(arm, ordinary_unit_count, real_parameters, &stripe_widths, 1);
        let (unsolvable_without_failure, _, _) = count_unsolvable_grids(arm, &layout, None);
        let (cost_in_bytes, cost_in_parts_per_million) = cost_bytes(arm, &stripe_widths);
        println!("{}", emitter.emit_raw(&format!(
            "name=summary arm={} no_failure_unsolvable={unsolvable_without_failure} cost_bytes={cost_in_bytes} cost_ppm={cost_in_parts_per_million:.1}",
            arm_name(arm))));
    }
    // 加盘窗口：确定性「取最空的」放置 ⇒ 容器与它描述的格正相关
    for arm in [Arm::PooledContainer, Arm::MirroredContainer, Arm::ParityHeader, Arm::NoMemberTable] {
        let (layout, new_device) = add_disk_window(arm, 4096, 8, 4096, &stripe_widths);
        let (unsolvable_count, round_count, solved_after_first_round) = count_unsolvable_grids(arm, &layout, Some(new_device));
        println!("{}", emitter.emit_raw(&format!(
            "name=add_disk arm={} devs=8 prefill=4096 new_dev={new_device} containers={} lost={} unsolvable={unsolvable_count} rounds={round_count} solved_after_first={solved_after_first_round}",
            arm_name(arm), layout.containers.len(), lost_count(&layout, new_device))));
    }
    // 自指那一格：池刚建好、树只有一个叶，容器落在它自己描述的那条条带里
    for seed in 1..=5u64 {
        let (layout, container_device) = first_stripe_self_reference(4, seed);
        let (unsolvable_count, round_count, solved_after_first_round) = count_unsolvable_grids(Arm::PooledContainer, &layout, Some(container_device));
        let is_self_referential = layout.stripe_members[layout.grids[layout.containers[0][0]].stripe_index]
            .iter().all(|&member| layout.record_container[member] == Some(0));
        println!("{}", emitter.emit_raw(&format!(
            "name=first_stripe arm={} seed={seed} devs=4 w=4 containers={} container_dev={container_device} self_referential={is_self_referential} lost={} unsolvable={unsolvable_count} rounds={round_count} solved_after_first={solved_after_first_round}",
            arm_name(Arm::PooledContainer), layout.containers.len(), lost_count(&layout, container_device))));
    }
    // 灵敏度：容器容量往下扫，看自举那条链从哪个比例起开始咬人。
    for container_capacity_in_records in [583u64, 64, 16, 8, 4, 2] {
        for seed in 1..=5u64 {
            let parameters = LayoutParameters { device_count, container_capacity_in_records, placement: Placement::Random, prefilled_slots: 0 };
            let layout = build_layout(Arm::PooledContainer, ordinary_unit_count, parameters, &stripe_widths, seed);
            let (unsolvable_count, round_count, solved_after_first_round) = count_unsolvable_grids(Arm::PooledContainer, &layout, Some(0));
            let container_fraction = layout.containers.len() as f64 / layout.grids.len() as f64;
            println!("{}", emitter.emit_raw(&format!(
                "name=cap_sweep arm={} cap={container_capacity_in_records} seed={seed} containers={} container_frac={container_fraction:.4} lost={} unsolvable={unsolvable_count} rounds={round_count} solved_after_first={solved_after_first_round}",
                arm_name(Arm::PooledContainer), layout.containers.len(), lost_count(&layout, 0))));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const STRIPE_WIDTH_CYCLE: [usize; 2] = [3, 4];
    fn real_parameters() -> LayoutParameters { LayoutParameters { device_count: 8, container_capacity_in_records: records_per_container(), placement: Placement::Random, prefilled_slots: 0 } }

    /// 判据 5：被占用的落点数与 E97（记账与分配记录的条目编码） 已入库的口径对得上（16 TiB、90%、32768）。
    #[test]
    fn unit_count_matches_e97() {
        assert_eq!(occupied_slots(), 483_183_820);
        assert_eq!(POOL_CAPACITY_BYTES, 17_592_186_044_416);
    }

    /// **摆进去的格不许超过盘上有的槽**。第一版把 `occupied_slots()` 当成数据格数，
    /// 再往上加 parity 与容器 ⇒ 往一块 90% 填充的盘里摆了 126.3% 的格，
    /// 而当时那条钉绝对值的断言只钉了数据格数，对真正喂进成本式的那个量不设防
    /// （2026-09-06 第二轮攻击腿 5.5）。
    #[test]
    fn occupancy_is_self_consistent() {
        let total_slot_count = POOL_CAPACITY_BYTES / UNIT_BYTES;
        assert_eq!(total_slot_count, 536_870_912);
        for arm in [Arm::NoMemberTable, Arm::ParityHeader, Arm::PooledContainer, Arm::MirroredContainer] {
            let (data_unit_count, container_total) = data_units(arm, &STRIPE_WIDTH_CYCLE, records_per_container());
            let copies_per_container = match arm { Arm::MirroredContainer => 2, Arm::PooledContainer => 1, _ => 0 };
            let member_count = data_unit_count + copies_per_container * container_total;
            let occupied_location_count = member_count + stripe_count(member_count, &STRIPE_WIDTH_CYCLE);
            assert!(occupied_location_count <= occupied_slots(), "{arm:?}: locs={occupied_location_count} 超过占用槽数");
            assert!(occupied_location_count + 2 >= occupied_slots(), "{arm:?}: locs={occupied_location_count} 没把占用填满，口径松了");
            assert!(occupied_location_count <= total_slot_count);
        }
    }

    /// 记录容量钉绝对值：(32768 − 107) / 56。
    #[test]
    fn records_per_container_is_pinned() {
        assert_eq!(records_per_container(), 583);
        assert_eq!(PACKED_HEADER_BYTES + STRIPE_RECORD_BYTES * 583 + 13, UNIT_BYTES);
    }

    /// 判据 1 阳性对照：臂丙的不可解格数恰等于坏盘上的活格数，另行独立数出来。
    #[test]
    fn positive_control_unsolvable_count_equals_lost_count() {
        for seed in 1..=5 {
            let layout = build_layout(Arm::NoMemberTable, 4096, real_parameters(), &STRIPE_WIDTH_CYCLE, seed);
            let (unsolvable_count, _, _) = count_unsolvable_grids(Arm::NoMemberTable, &layout, Some(0));
            assert_eq!(unsolvable_count, lost_count(&layout, 0));
            assert!(unsolvable_count > 0, "阳性对照必须真的丢了东西");
        }
    }

    /// 判据 2：臂乙与臂甲镜恒 0，且甲镜的容器真的一个都没落在坏盘上。
    #[test]
    fn parity_header_and_mirrored_container_arms_are_zero() {
        for seed in 1..=5 {
            for arm in [Arm::ParityHeader, Arm::MirroredContainer] {
                let layout = build_layout(arm, 4096, real_parameters(), &STRIPE_WIDTH_CYCLE, seed);
                let (unsolvable_count, _, _) = count_unsolvable_grids(arm, &layout, Some(0));
                assert_eq!(unsolvable_count, 0, "arm={:?} seed={seed}", arm);
            }
            let mirrored_layout = build_layout(Arm::MirroredContainer, 4096, real_parameters(), &STRIPE_WIDTH_CYCLE, seed);
            let containers_touching_failed_device = mirrored_layout.containers.iter().filter(|copies| copies.iter().any(|&copy_grid| mirrored_layout.grids[copy_grid].device == 0)).count();
            let mirrored_stripe_first_members: Vec<usize> = mirrored_layout.stripe_members.iter().filter(|members| members.len() == 2).map(|members| members[0]).collect();
            assert!(!mirrored_stripe_first_members.is_empty(), "甲镜必须真的有镜像条带");
            assert!(containers_touching_failed_device > 0, "镜像的另一份仍可能落在坏盘上，只是丢不掉整条");
        }
    }

    /// 判据 4 阴性对照：不失效 ⇒ 四条臂全 0。
    #[test]
    fn negative_control_no_failure() {
        for arm in [Arm::PooledContainer, Arm::MirroredContainer, Arm::ParityHeader, Arm::NoMemberTable] {
            let layout = build_layout(arm, 4096, real_parameters(), &STRIPE_WIDTH_CYCLE, 1);
            assert_eq!(count_unsolvable_grids(arm, &layout, None).0, 0);
        }
    }

    /// 判据 3 + 6：臂甲的绝对值按种子钉死，且不动点迭代真的推进过。
    #[test]
    fn pooled_container_arm_absolute_values() {
        let mut observed = Vec::new();
        let mut solved_after_first_round_total = 0u64;
        for seed in 1..=5 {
            let layout = build_layout(Arm::PooledContainer, 4096, real_parameters(), &STRIPE_WIDTH_CYCLE, seed);
            let (unsolvable_count, round_count, solved_after_first_round) = count_unsolvable_grids(Arm::PooledContainer, &layout, Some(0));
            observed.push((unsolvable_count, round_count, solved_after_first_round));
            solved_after_first_round_total += solved_after_first_round;
        }
        assert_eq!(observed, vec![(0, 2, 0), (0, 2, 0), (0, 2, 0), (0, 2, 0), (0, 2, 0)], "{observed:?}");
        assert_eq!(solved_after_first_round_total, 0);
    }

    /// 容器数的不动点：容器自己也要一条记录，所以 c 略大于 units / 583。
    #[test]
    fn container_fixpoint_accounts_for_itself() {
        let container_total = container_count(Arm::PooledContainer, 483_183_820, &STRIPE_WIDTH_CYCLE, records_per_container());
        assert!(container_total > 483_183_820 / 583, "{container_total}");
        let unit_count = 483_183_820 + container_total;
        let location_count = unit_count + stripe_count(unit_count, &STRIPE_WIDTH_CYCLE);
        assert_eq!(container_total, location_count.div_ceil(records_per_container()));
    }

    /// 代价的绝对值：三条臂各自的 ppm，口径是「相对臂丙少装多少用户数据」。
    /// 2026-09-06 换过一次口径（第二轮攻击腿 5.5）：第一版报的是「元数据自己占多少字节」，
    /// 而它把 parity 与容器加在 90% 之上 ⇒ 126.3% 的格。换口径之后三条臂的排序也变了：
    /// 旧口径「甲 2166 < 乙 3516 < 甲镜 4333」，自洽口径「甲 1544 < 乙 2511 < 甲镜 3088」。
    #[test]
    fn cost_absolute() {
        assert_eq!(occupied_slots(), 483_183_820);
        let (baseline_data_units, _) = data_units(Arm::NoMemberTable, &STRIPE_WIDTH_CYCLE, records_per_container());
        assert_eq!(baseline_data_units, 345_131_300);
        let (_, no_member_table_parts_per_million) = cost_bytes(Arm::NoMemberTable, &STRIPE_WIDTH_CYCLE);
        assert_eq!(no_member_table_parts_per_million, 0.0);
        let (parity_header_bytes, parity_header_parts_per_million) = cost_bytes(Arm::ParityHeader, &STRIPE_WIDTH_CYCLE);
        assert_eq!(parity_header_bytes, PARITY_HEADER_RESERVE_BYTES * baseline_data_units);
        assert!((parity_header_parts_per_million - 2511.2).abs() < 0.2, "乙={parity_header_parts_per_million}");
        let (pooled_container_bytes, pooled_container_parts_per_million) = cost_bytes(Arm::PooledContainer, &STRIPE_WIDTH_CYCLE);
        let (mirrored_container_bytes, mirrored_container_parts_per_million) = cost_bytes(Arm::MirroredContainer, &STRIPE_WIDTH_CYCLE);
        assert_eq!(pooled_container_bytes, 27_157_757_952);
        assert_eq!(mirrored_container_bytes, 2 * pooled_container_bytes);
        assert!((pooled_container_parts_per_million - 1543.7).abs() < 0.2, "甲={pooled_container_parts_per_million}");
        assert!((mirrored_container_parts_per_million - 3087.5).abs() < 0.2, "甲镜={mirrored_container_parts_per_million}");
        // 容器数：两条臂的逻辑容器数相同，差别在副本数
        let (_, pooled_container_total) = data_units(Arm::PooledContainer, &STRIPE_WIDTH_CYCLE, records_per_container());
        let (_, mirrored_container_total) = data_units(Arm::MirroredContainer, &STRIPE_WIDTH_CYCLE, records_per_container());
        assert_eq!((pooled_container_total, mirrored_container_total), (828_789, 828_789));
    }

    /// **臂甲那条路自己的阳性对照**：手工摆一个互指的死锁，度量必须报非 0。
    /// 少了它，「随机摆放下恒 0」分不清是结论如此，还是这条路根本报不出非 0
    /// （`.claude/singlefs-ai-sop/rules/test-discipline.md`「检查本身也可能是错的」）。
    #[test]
    fn deadlock_is_reachable_by_construction() {
        // 盘 0 坏。条带 0 的成员记录全住容器 1（在盘 0 上），条带 1 的成员记录全住容器 0（也在盘 0 上）。
        let grids = vec![
            Grid { device: 0, slot: 0, kind: GridKind::Container, stripe_index: 0 }, // 0：容器 0
            Grid { device: 1, slot: 0, kind: GridKind::Ordinary,  stripe_index: 0 }, // 1
            Grid { device: 2, slot: 0, kind: GridKind::Parity,    stripe_index: 0 }, // 2
            Grid { device: 0, slot: 1, kind: GridKind::Container, stripe_index: 1 }, // 3：容器 1
            Grid { device: 1, slot: 1, kind: GridKind::Ordinary,  stripe_index: 1 }, // 4
            Grid { device: 2, slot: 1, kind: GridKind::Parity,    stripe_index: 1 }, // 5
        ];
        let layout = Layout {
            grids,
            stripe_members: vec![vec![0, 1, 2], vec![3, 4, 5]],
            containers: vec![vec![0], vec![3]],
            record_container: vec![Some(1), Some(1), Some(1), Some(0), Some(0), Some(0)],
        };
        let (unsolvable_count, round_count, solved_after_first_round) = count_unsolvable_grids(Arm::PooledContainer, &layout, Some(0));
        assert_eq!((unsolvable_count, round_count, solved_after_first_round), (2, 1, 0), "互指的两个容器必须都解不开");
        // 只要把条带 0 的一条记录挪进一个活着的容器，死锁就解开 —— 证明报的是死锁不是别的
        let mut layout_with_moved_record = layout.clone();
        layout_with_moved_record.record_container[1] = Some(0);
        assert_eq!(count_unsolvable_grids(Arm::PooledContainer, &layout_with_moved_record, Some(0)).0, 2, "记录仍在坏盘上的容器里，照旧解不开");
        let mut layout_with_container_on_live_device = layout.clone();
        layout_with_container_on_live_device.containers = vec![vec![0], vec![3]];
        layout_with_container_on_live_device.grids[3].device = 1; // 容器 1 搬到活盘上
        assert_eq!(count_unsolvable_grids(Arm::PooledContainer, &layout_with_container_on_live_device, Some(0)).0, 0);
    }

    /// **加盘窗口那一格：确定性放置让臂甲退化**（2026-09-06 第二轮攻击腿 5.3 给的真实场景）。
    /// 「稳态恒 0」的承重点是「容器落哪块盘」与「它描述的格落哪块盘」独立，
    /// 而 D2（RAID 条带策略） 已定项 8 的「取最空的」把两者绑在一起。
    #[test]
    fn add_disk_window_leaves_pooled_container_arm_solvable() {
        let (pooled_layout, new_device) = add_disk_window(Arm::PooledContainer, 4096, 8, 4096, &STRIPE_WIDTH_CYCLE);
        let (pooled_unsolvable_count, _, _) = count_unsolvable_grids(Arm::PooledContainer, &pooled_layout, Some(new_device));
        let lost_grid_count = lost_count(&pooled_layout, new_device);
        assert!(lost_grid_count > 0, "新盘上必须真的落了东西");
        // 2026-09-06 第二轮攻击腿 5.3 预言臂甲在这一格会退化成阳性对照臂丙。**建模之后没复现**：
        // 一次攒批里的 w−1 个单元被摊到 w−1 个不同的列上，容器与它描述的格因此仍落在不同盘上
        // （实测容器落点 [0,7,2,3,7,5,7,0,1,7]，10 个里 4 个在新盘）。
        // 如实记成 0，并把「为什么没复现」钉成断言：新盘上每条条带至多一个格。
        assert_eq!(pooled_unsolvable_count, 0, "臂甲在加盘窗口上实得 {pooled_unsolvable_count} / 丢 {lost_grid_count}");
        let mut grids_on_new_device_per_stripe = std::collections::HashMap::new();
        for grid in &pooled_layout.grids {
            if grid.device == new_device { *grids_on_new_device_per_stripe.entry(grid.stripe_index).or_insert(0u32) += 1; }
        }
        assert!(grids_on_new_device_per_stripe.values().all(|&grid_count| grid_count == 1), "一条条带在同一块盘上不许有两个格");
        // 臂丙（阳性对照）在同一格上恒等于丢失格数；臂甲有多接近它，就是这条失真有多大
        let (no_member_table_layout, _) = add_disk_window(Arm::NoMemberTable, 4096, 8, 4096, &STRIPE_WIDTH_CYCLE);
        let (no_member_table_unsolvable_count, _, _) = count_unsolvable_grids(Arm::NoMemberTable, &no_member_table_layout, Some(new_device));
        assert_eq!(no_member_table_unsolvable_count, lost_count(&no_member_table_layout, new_device));
        // 臂甲镜与臂乙在同一格上仍是 0
        for arm in [Arm::MirroredContainer, Arm::ParityHeader] {
            let (arm_layout, arm_new_device) = add_disk_window(arm, 4096, 8, 4096, &STRIPE_WIDTH_CYCLE);
            assert_eq!(count_unsolvable_grids(arm, &arm_layout, Some(arm_new_device)).0, 0, "{arm:?}");
        }
    }

    /// **容器单独一次发布写出时，`w` 按 D2（RAID 条带策略） 已定项 6 的公式自动等于 2**
    /// ⇒ 那次写自动是镜像（D9（加密）：w = 2 的 parity 就是逐字节相同的第二副本）
    /// ⇒ 臂甲在这个写入形态下退化成臂甲镜，自举那条边消失。
    /// 这是纯算术，不是模型的性质：`clamp(1 + 1, 2, 4) = 2`。
    #[test]
    fn lone_container_publish_is_mirrored_by_the_width_formula() {
        let stripe_width = (1u64 + 1).clamp(2, 4);
        assert_eq!(stripe_width, 2);
        // 攒批里只要还有别的单元，公式就给 3 或 4，容器与数据同条带
        assert_eq!((2u64 + 1).clamp(2, 4), 3);
        assert_eq!((3u64 + 1).clamp(2, 4), 4);
        assert_eq!((9u64 + 1).clamp(2, 4), 4);
    }

    /// **自指那一格必须报非 0**：池刚建好、条带表只有一个容器、它落在自己描述的那条条带里，
    /// 容器所在盘一坏，这条条带的四条记录全没了 ⇒ 无论按什么顺序都解不开。
    /// 这一格是 2026-09-06 反推腿第五节打中的，先做成必须报非 0 的世界再改规则
    /// （`.claude/singlefs-ai-sop/rules/show-me-test.md`「对抗轮打中的每一条，先在模型里做成一个必须报非 0 的世界」）。
    #[test]
    fn first_stripe_self_reference_is_unsolvable() {
        for seed in 1..=5 {
            let (layout, container_device) = first_stripe_self_reference(4, seed);
            assert_eq!(layout.containers.len(), 1, "seed={seed}：这一格要的是只有一个容器");
            let stripe_index = layout.grids[layout.containers[0][0]].stripe_index;
            assert!(layout.stripe_members[stripe_index].iter().all(|&member| layout.record_container[member] == Some(0)),
                "seed={seed}：四条记录必须全在那个容器里，否则这一格没造出来");
            let (unsolvable_count, _, _) = count_unsolvable_grids(Arm::PooledContainer, &layout, Some(container_device));
            assert!(unsolvable_count > 0, "seed={seed}：自指的容器丢了必须解不开，实得 {unsolvable_count}");
            assert_eq!(unsolvable_count, lost_count(&layout, container_device), "seed={seed}：坏盘上的格一个都救不回来");
            // 臂乙（parity 自证）与臂甲镜（容器镜像）在同一格上仍是 0 —— 差别是结构性的
            let (mirrored_layout, mirrored_container_device) = {
                let mirrored_layout = build_layout(Arm::MirroredContainer, 2, LayoutParameters { device_count: 4, container_capacity_in_records: records_per_container(), placement: Placement::Random, prefilled_slots: 0 }, &[4usize], seed);
                let first_copy_device = mirrored_layout.grids[mirrored_layout.containers[0][0]].device;
                (mirrored_layout, first_copy_device)
            };
            assert_eq!(count_unsolvable_grids(Arm::MirroredContainer, &mirrored_layout, Some(mirrored_container_device)).0, 0, "seed={seed}：镜像容器丢不掉");
        }
    }

    /// 闭式条带数与逐条带循环在小值上逐格相等。
    #[test]
    fn closed_form_matches_loop() {
        for unit_count in 0..2000u64 {
            assert_eq!(stripe_count(unit_count, &STRIPE_WIDTH_CYCLE), stripe_count_by_loop(unit_count, &STRIPE_WIDTH_CYCLE), "u={unit_count}");
        }
        for unit_count in [10_000u64, 123_457, 1_000_000] {
            assert_eq!(stripe_count(unit_count, &STRIPE_WIDTH_CYCLE), stripe_count_by_loop(unit_count, &STRIPE_WIDTH_CYCLE), "u={unit_count}");
        }
    }

    /// 条带数的算术：宽度循环 3,4 ⇒ 每两条条带装 2 + 3 = 5 个单元。
    #[test]
    fn stripe_count_arithmetic() {
        assert_eq!(stripe_count(5, &STRIPE_WIDTH_CYCLE), 2);
        assert_eq!(stripe_count(10, &STRIPE_WIDTH_CYCLE), 4);
        assert_eq!(stripe_count(1, &STRIPE_WIDTH_CYCLE), 1);
        assert_eq!(stripe_count(0, &STRIPE_WIDTH_CYCLE), 0);
    }
}

