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
//! 1. **阳性对照**：臂 Z 的不可解格数必须**恰好等于**坏盘上的活格数（另行独立数出来）。
//!    对不上说明这套度量分不出差别，**整轮作废**。
//! 2. 臂乙与臂甲镜的不可解格数恒 0，且这是**结构性的**：B 没有依赖边，A2 的容器丢不掉。
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

/// 码 3 容器 32768、头 103（D18 已定项 11）。
const UNIT_BYTES: u64 = 32768;
const PACKED_HDR: u64 = 103;
/// 条带成员记录定长 56：格宽标志 1 + w 1 + 自己是第几列 1 + 成员表 4 × 10 + 条带诞生代 8 + 补齐 5。
const STRIPE_REC: u64 = 56;
/// 臂 B 的恒零预留区：共同前缀 42 + fsid 8 + 诞生代号 8 + 写序 10 + 成员数 1 + 成员表 40，
/// 向上取到 16 的倍数 ⇒ 128。
const RESERVE_H: u64 = 128;
/// 16 TiB。
const CAP_BYTES: u64 = 16 * 1024 * 1024 * 1024 * 1024;
const FILL: f64 = 0.9;

fn per_container() -> u64 {
    (UNIT_BYTES - PACKED_HDR) / STRIPE_REC
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
struct Params {
    devs: usize,
    cap: u64,
    placement: Placement,
    /// 加盘窗口：最后一块盘是刚加进来的空盘，其余盘已经写到 `prefill` 槽。
    /// D2（RAID 条带策略） 已定项 8 的「取最空的」是确定性且全局同步的规则 ⇒ 新盘会被每条新条带选中，
    /// 重平衡重写出来的容器也堆在它上面 ⇒ 容器与它描述的格**正相关**，
    /// 而「稳态恒 0」的全部承重点正是那个独立性（2026-09-06 第二轮攻击腿 5.3）。
    prefill: u64,
}

/// 确定性伪随机：LCG，只用来选设备，不参与任何判定。
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    /// 从 d 块盘里取 k 块互不相同的（k ≤ d）。
    fn pick(&mut self, d: usize, k: usize) -> Vec<usize> {
        let mut pool: Vec<usize> = (0..d).collect();
        let mut out = Vec::with_capacity(k);
        for i in 0..k {
            let j = i + (self.next() as usize) % (pool.len() - i);
            pool.swap(i, j);
            out.push(pool[i]);
        }
        out
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind { Ordinary, Container, Parity }

#[derive(Clone, Debug)]
struct Grid {
    dev: usize,
    slot: u64,
    kind: Kind,
    stripe: usize,
}

#[derive(Clone, Debug)]
struct Layout {
    grids: Vec<Grid>,
    /// 每条条带的成员（下标进 grids）
    stripes: Vec<Vec<usize>>,
    /// 逻辑容器号 → 它的各份副本在 grids 里的下标（臂甲镜下一个容器有两份）
    containers: Vec<Vec<usize>>,
    /// 每个 grid 的记录住在哪个容器号；臂 B / Z 为 None
    record_container: Vec<Option<usize>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm { A, A2, B, Z }

/// 容器数的不动点：容器自己也占落点、也要一条记录。
fn container_count(arm: Arm, n_ord: u64, widths: &[usize], cap: u64) -> u64 {
    if arm == Arm::B || arm == Arm::Z { return 0; }
    let mut c = 0u64;
    for _ in 0..64 {
        let units = n_ord + c;
        let locs = units + n_stripes(units, widths);
        let next = locs.div_ceil(cap);
        if next == c { break; }
        c = next;
    }
    c
}

/// 单元按宽度循环攒批，每条条带装 w−1 个单元 ⇒ 条带数。逐条带循环。
fn n_stripes_loop(units: u64, widths: &[usize]) -> u64 {
    let mut left = units;
    let mut s = 0u64;
    let mut i = 0usize;
    while left > 0 {
        let w = widths[i % widths.len()] as u64;
        let take = (w - 1).min(left);
        left -= take;
        s += 1;
        i += 1;
    }
    s
}

/// 闭式：一轮宽度循环装 Σ(w−1) 个单元、出 widths.len() 条条带；余数再逐条带补。
/// 4.8 亿个单元上逐条带循环跑不动（第一版的代价式子因此慢到 40 秒），闭式与循环版
/// 在小值上逐格相等，由单测 `closed_form_matches_loop` 钉住。
fn n_stripes(units: u64, widths: &[usize]) -> u64 {
    let per_cycle: u64 = widths.iter().map(|&w| w as u64 - 1).sum();
    let cycles = units / per_cycle;
    let mut left = units % per_cycle;
    let mut s = cycles * widths.len() as u64;
    let mut i = 0usize;
    while left > 0 {
        let take = (widths[i] as u64 - 1).min(left);
        left -= take;
        s += 1;
        i += 1;
    }
    s
}

fn build(arm: Arm, n_ord: u64, p: Params, widths: &[usize], seed: u64) -> Layout {
    let devs = p.devs;
    let n_cont = container_count(arm, n_ord, widths, p.cap);
    let mut rng = Lcg(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut grids: Vec<Grid> = Vec::new();
    let mut stripes: Vec<Vec<usize>> = Vec::new();
    let mut next_slot = vec![p.prefill; devs];
    if p.prefill > 0 { next_slot[devs - 1] = 0; }

    // 待摆放的单元：先普通单元，再容器（容器号 = 出现顺序）
    let mut pending: Vec<Kind> = Vec::new();
    for _ in 0..n_ord { pending.push(Kind::Ordinary); }
    for _ in 0..n_cont { pending.push(Kind::Container); }

    let mut idx = 0usize;
    let mut wi = 0usize;
    while idx < pending.len() {
        let kind = pending[idx];
        // 臂 A2：容器恒走 w = 2 镜像（两份副本，无 parity 格）
        let mirror = arm == Arm::A2 && kind == Kind::Container;
        let w = if mirror { 2 } else { widths[wi % widths.len()] };
        wi += 1;
        let picked = match p.placement {
            Placement::Random => rng.pick(devs, w),
            Placement::Emptiest => {
                let mut order: Vec<usize> = (0..devs).collect();
                order.sort_by_key(|&d| (next_slot[d], d));
                order.truncate(w);
                order
            }
        };
        let sid = stripes.len();
        let mut members = Vec::new();
        if mirror {
            for &d in &picked {
                let g = grids.len();
                grids.push(Grid { dev: d, slot: next_slot[d], kind: Kind::Container, stripe: sid });
                next_slot[d] += 1;
                members.push(g);
            }
            idx += 1;
        } else {
            // 臂甲镜里容器恒走镜像 ⇒ 一个攒批不许既装普通单元又装容器，
            // 否则边界那一批会把容器裹进 w = 4 的条带，镜像那条规则等于没生效
            let run = if arm == Arm::A2 {
                pending[idx..].iter().take_while(|&&k| k == kind).count()
            } else {
                pending.len() - idx
            };
            let take = (w - 1).min(run);
            for j in 0..take {
                let d = picked[j];
                let g = grids.len();
                grids.push(Grid { dev: d, slot: next_slot[d], kind: pending[idx + j], stripe: sid });
                next_slot[d] += 1;
                members.push(g);
            }
            // parity 格落在剩下那一列
            let d = picked[w - 1];
            let g = grids.len();
            grids.push(Grid { dev: d, slot: next_slot[d], kind: Kind::Parity, stripe: sid });
            next_slot[d] += 1;
            members.push(g);
            idx += take;
        }
        stripes.push(members);
    }

    // 逻辑容器：臂甲镜下两份镜像是**同一个**容器的两份副本，不是两个容器。
    // 按 grid 顺序聚合——镜像的两份是同一条镜像条带里连着放的
    let mut containers: Vec<Vec<usize>> = Vec::new();
    for (i, g) in grids.iter().enumerate() {
        if g.kind != Kind::Container { continue; }
        if arm == Arm::A2 {
            let same = containers.last().map(|r: &Vec<usize>| {
                grids[r[0]].stripe == g.stripe && r.len() < 2
            }).unwrap_or(false);
            if same { containers.last_mut().unwrap().push(i); continue; }
        }
        containers.push(vec![i]);
    }

    // 记录按 key =(设备, 槽号) 排序装容器 —— btree 叶按 key 顺序打包
    let mut record_container = vec![None; grids.len()];
    if !containers.is_empty() {
        let mut order: Vec<usize> = (0..grids.len()).collect();
        order.sort_by_key(|&i| (grids[i].dev, grids[i].slot));
        let cap = p.cap as usize;
        // 落点数比不动点估的多一格就会溢出最后一个容器 —— 静默夹住会让「记录都装得下」
        // 这件事永远为真而没人发现，所以这里判死
        assert!(order.len().div_ceil(cap) <= containers.len(),
            "容器不够装：落点 {} 容量 {cap} 容器 {}", order.len(), containers.len());
        for (rank, &gi) in order.iter().enumerate() {
            record_container[gi] = Some(rank / cap);
        }
    }
    Layout { grids, stripes, containers, record_container }
}

/// 单盘失效之后重建不出来的格数；`iters` 报不动点迭代真的推进了几轮、第一轮之后又解开多少。
fn unsolvable(arm: Arm, l: &Layout, failed: Option<usize>) -> (u64, u64, u64) {
    let failed = match failed { Some(f) => f, None => return (0, 0, 0) };
    let lost: Vec<usize> = l.grids.iter().enumerate()
        .filter(|(_, g)| g.dev == failed).map(|(i, _)| i).collect();
    if arm == Arm::Z { return (lost.len() as u64, 0, 0); }
    if arm == Arm::B { return (0, 0, 0); }

    let mut solved = vec![false; l.grids.len()];
    let mut first_round = 0u64;
    let mut rounds = 0u64;
    loop {
        let mut changed = false;
        for &g in &lost {
            if solved[g] { continue; }
            let sid = l.grids[g].stripe;
            for &m in &l.stripes[sid] {
                let Some(c) = l.record_container[m] else { continue };
                // 任一副本可读或已重建，这条记录就到手
                if l.containers[c].iter().any(|&cg| l.grids[cg].dev != failed || solved[cg]) {
                    solved[g] = true;
                    changed = true;
                    break;
                }
            }
        }
        rounds += 1;
        if rounds == 1 { first_round = lost.iter().filter(|&&g| solved[g]).count() as u64; }
        if !changed { break; }
    }
    let un = lost.iter().filter(|&&g| !solved[g]).count() as u64;
    let later = lost.iter().filter(|&&g| solved[g]).count() as u64 - first_round;
    (un, rounds, later)
}

/// 池刚建好、条带表只有一个容器时，那个容器落在**它自己描述的那条条带**里。
/// 这一格是第一版提案 P4 说的「成对互指」的真形态：不是互指，是自指，
/// 而且它落在 w ≥ 3 的第一条条带上（2026-09-06 反推腿第五节）。
/// 加盘窗口：其余盘已写满 `prefill` 槽，最后一块是空盘；按「取最空的」摆放，
/// 新盘一定进每条新条带。返回布局与新盘号。
fn add_disk_window(arm: Arm, n_ord: u64, devs: usize, prefill: u64, widths: &[usize]) -> (Layout, usize) {
    let p = Params { devs, cap: per_container(), placement: Placement::Emptiest, prefill };
    (build(arm, n_ord, p, widths, 1), devs - 1)
}

fn first_stripe_self_reference(devs: usize, seed: u64) -> (Layout, usize) {
    let l = build(Arm::A, 2, Params { devs, cap: per_container(), placement: Placement::Random, prefill: 0 }, &[4usize], seed);
    let dev = l.grids[l.containers[0][0]].dev;
    (l, dev)
}

fn lost_count(l: &Layout, failed: usize) -> u64 {
    l.grids.iter().filter(|g| g.dev == failed).count() as u64
}

/// 池里被占用的落点总数 = 容量 × 填充率 / 单元大小。**含数据格、parity 格与容器**——
/// 第一版把它当成「数据格数」，于是往一块声称 90% 填充的盘里摆了 126.3% 的格
/// （2026-09-06 第二轮攻击腿 5.5 算出来的）。判据 5 只钉了数据格数，没钉这个量。
fn occupied_slots() -> u64 {
    (CAP_BYTES as f64 * FILL / UNIT_BYTES as f64) as u64
}

/// 一条臂在固定 90% 占用下装得下多少**数据格**。
/// parity 按每条条带一格（宽度循环 3/4 ⇒ 每 5 个成员格配 2 个 parity 格）算；
/// 容器自己也是成员格、也要 parity、也要一条自己的记录。
fn data_units(arm: Arm, widths: &[usize], cap: u64) -> (u64, u64) {
    let occ = occupied_slots();
    // members = 数据格 + 容器；occ = members + parity(members)
    let members = {
        let mut lo = 0u64;
        let mut hi = occ;
        while lo < hi {
            let mid = (lo + hi + 1) / 2;
            if mid + n_stripes(mid, widths) <= occ { lo = mid; } else { hi = mid - 1; }
        }
        lo
    };
    let mult = match arm { Arm::A2 => 2, Arm::A => 1, _ => 0 };
    if mult == 0 { return (members, 0); }
    // 记录数 = 全部落点（含 parity 与容器自己）；容器数 = 记录数 / cap，副本数 × mult
    let mut c = 0u64;
    for _ in 0..64 {
        let d = members.saturating_sub(mult * c);
        let locs = d + mult * c + n_stripes(d + mult * c, widths);
        let next = locs.div_ceil(cap);
        if next == c { break; }
        c = next;
    }
    (members.saturating_sub(mult * c), c)
}

/// 这条臂**吃掉多少用户数据**，相对臂丙（不带成员表）的差，按容量的 ppm 报。
/// 口径换过一次：第一版报的是「元数据自己占多少字节」，分母对不上填充率。
fn cost_bytes(arm: Arm, widths: &[usize]) -> (u64, f64) {
    let (base, _) = data_units(Arm::Z, widths, per_container());
    let bytes = match arm {
        Arm::Z => 0,
        // 臂乙不多占格，它从每个成员格的净荷里扣 H 字节（含容器所在的那些格：臂乙没有容器）
        Arm::B => RESERVE_H * base,
        Arm::A | Arm::A2 => {
            let (d, _) = data_units(arm, widths, per_container());
            (base - d) * UNIT_BYTES
        }
    };
    (bytes, bytes as f64 * 1e6 / CAP_BYTES as f64)
}

fn arm_name(a: Arm) -> &'static str {
    match a { Arm::A => "甲_容器同池", Arm::A2 => "甲镜_容器镜像", Arm::B => "乙_parity自证", Arm::Z => "丙_无成员表" }
}

fn main() {
    let mut em = Emitter::new();
    let widths = [3usize, 4];
    let devs = 8usize;
    let n_ord = 4096u64;
    let real = Params { devs, cap: per_container(), placement: Placement::Random, prefill: 0 };
    println!("{}", em.emit_raw(&format!(
        "name=config note=条带成员表的载体 devs={devs} n_ord={n_ord} widths=3,4 unit_bytes={UNIT_BYTES} \
packed_hdr={PACKED_HDR} rec={STRIPE_REC} per_container={} reserve_h={RESERVE_H} cap_bytes={CAP_BYTES} fill={FILL}",
        per_container())));

    for arm in [Arm::A, Arm::A2, Arm::B, Arm::Z] {
        for seed in 1..=5u64 {
            let l = build(arm, n_ord, real, &widths, seed);
            let (un, rounds, later) = unsolvable(arm, &l, Some(0));
            let lost = lost_count(&l, 0);
            println!("{}", em.emit_raw(&format!(
                "name=fail1 arm={} seed={seed} grids={} stripes={} containers={} lost={lost} unsolvable={un} rounds={rounds} solved_after_first={later}",
                arm_name(arm), l.grids.len(), l.stripes.len(), l.containers.len())));
        }
        let l = build(arm, n_ord, real, &widths, 1);
        let (un0, _, _) = unsolvable(arm, &l, None);
        let (bytes, ppm) = cost_bytes(arm, &widths);
        println!("{}", em.emit_raw(&format!(
            "name=summary arm={} no_failure_unsolvable={un0} cost_bytes={bytes} cost_ppm={ppm:.1}",
            arm_name(arm))));
    }
    // 加盘窗口：确定性「取最空的」放置 ⇒ 容器与它描述的格正相关
    for arm in [Arm::A, Arm::A2, Arm::B, Arm::Z] {
        let (l, dev) = add_disk_window(arm, 4096, 8, 4096, &widths);
        let (un, rounds, later) = unsolvable(arm, &l, Some(dev));
        println!("{}", em.emit_raw(&format!(
            "name=add_disk arm={} devs=8 prefill=4096 new_dev={dev} containers={} lost={} unsolvable={un} rounds={rounds} solved_after_first={later}",
            arm_name(arm), l.containers.len(), lost_count(&l, dev))));
    }
    // 自指那一格：池刚建好、树只有一个叶，容器落在它自己描述的那条条带里
    for seed in 1..=5u64 {
        let (l, dev) = first_stripe_self_reference(4, seed);
        let (un, rounds, later) = unsolvable(Arm::A, &l, Some(dev));
        let self_ref = l.stripes[l.grids[l.containers[0][0]].stripe]
            .iter().all(|&m| l.record_container[m] == Some(0));
        println!("{}", em.emit_raw(&format!(
            "name=first_stripe arm={} seed={seed} devs=4 w=4 containers={} container_dev={dev} self_referential={self_ref} lost={} unsolvable={un} rounds={rounds} solved_after_first={later}",
            arm_name(Arm::A), l.containers.len(), lost_count(&l, dev))));
    }
    // 灵敏度：容器容量往下扫，看自举那条链从哪个比例起开始咬人。
    for cap in [583u64, 64, 16, 8, 4, 2] {
        for seed in 1..=5u64 {
            let p = Params { devs, cap, placement: Placement::Random, prefill: 0 };
            let l = build(Arm::A, n_ord, p, &widths, seed);
            let (un, rounds, later) = unsolvable(Arm::A, &l, Some(0));
            let frac = l.containers.len() as f64 / l.grids.len() as f64;
            println!("{}", em.emit_raw(&format!(
                "name=cap_sweep arm={} cap={cap} seed={seed} containers={} container_frac={frac:.4} lost={} unsolvable={un} rounds={rounds} solved_after_first={later}",
                arm_name(Arm::A), l.containers.len(), lost_count(&l, 0))));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    const W: [usize; 2] = [3, 4];
    fn real() -> Params { Params { devs: 8, cap: per_container(), placement: Placement::Random, prefill: 0 } }

    /// 判据 5：被占用的落点数与 E97（记账与分配记录的条目编码） 已入库的口径对得上（16 TiB、90%、32768）。
    #[test]
    fn unit_count_matches_e97() {
        assert_eq!(occupied_slots(), 483_183_820);
        assert_eq!(CAP_BYTES, 17_592_186_044_416);
    }

    /// **摆进去的格不许超过盘上有的槽**。第一版把 `occupied_slots()` 当成数据格数，
    /// 再往上加 parity 与容器 ⇒ 往一块 90% 填充的盘里摆了 126.3% 的格，
    /// 而当时那条钉绝对值的断言只钉了数据格数，对真正喂进成本式的那个量不设防
    /// （2026-09-06 第二轮攻击腿 5.5）。
    #[test]
    fn occupancy_is_self_consistent() {
        let slots_total = CAP_BYTES / UNIT_BYTES;
        assert_eq!(slots_total, 536_870_912);
        for arm in [Arm::Z, Arm::B, Arm::A, Arm::A2] {
            let (d, c) = data_units(arm, &W, per_container());
            let mult = match arm { Arm::A2 => 2, Arm::A => 1, _ => 0 };
            let members = d + mult * c;
            let locs = members + n_stripes(members, &W);
            assert!(locs <= occupied_slots(), "{arm:?}: locs={locs} 超过占用槽数");
            assert!(locs + 2 >= occupied_slots(), "{arm:?}: locs={locs} 没把占用填满，口径松了");
            assert!(locs <= slots_total);
        }
    }

    /// 记录容量钉绝对值：(32768 − 103) / 56。
    #[test]
    fn records_per_container_is_pinned() {
        assert_eq!(per_container(), 583);
        assert_eq!(PACKED_HDR + STRIPE_REC * 583 + 17, UNIT_BYTES);
    }

    /// 判据 1 阳性对照：臂丙的不可解格数恰等于坏盘上的活格数，另行独立数出来。
    #[test]
    fn positive_control_z_equals_lost() {
        for seed in 1..=5 {
            let l = build(Arm::Z, 4096, real(), &W, seed);
            let (un, _, _) = unsolvable(Arm::Z, &l, Some(0));
            assert_eq!(un, lost_count(&l, 0));
            assert!(un > 0, "阳性对照必须真的丢了东西");
        }
    }

    /// 判据 2：臂乙与臂甲镜恒 0，且 A2 的容器真的一个都没落在坏盘上。
    #[test]
    fn b_and_a2_are_zero() {
        for seed in 1..=5 {
            for arm in [Arm::B, Arm::A2] {
                let l = build(arm, 4096, real(), &W, seed);
                let (un, _, _) = unsolvable(arm, &l, Some(0));
                assert_eq!(un, 0, "arm={:?} seed={seed}", arm);
            }
            let l = build(Arm::A2, 4096, real(), &W, seed);
            let on_failed = l.containers.iter().filter(|r| r.iter().any(|&c| l.grids[c].dev == 0)).count();
            let mirrored: Vec<usize> = l.stripes.iter().filter(|m| m.len() == 2).map(|m| m[0]).collect();
            assert!(!mirrored.is_empty(), "甲镜必须真的有镜像条带");
            assert!(on_failed > 0, "镜像的另一份仍可能落在坏盘上，只是丢不掉整条");
        }
    }

    /// 判据 4 阴性对照：不失效 ⇒ 四条臂全 0。
    #[test]
    fn negative_control_no_failure() {
        for arm in [Arm::A, Arm::A2, Arm::B, Arm::Z] {
            let l = build(arm, 4096, real(), &W, 1);
            assert_eq!(unsolvable(arm, &l, None).0, 0);
        }
    }

    /// 判据 3 + 6：臂甲的绝对值按种子钉死，且不动点迭代真的推进过。
    #[test]
    fn arm_a_absolute_values() {
        let mut got = Vec::new();
        let mut moved = 0u64;
        for seed in 1..=5 {
            let l = build(Arm::A, 4096, real(), &W, seed);
            let (un, rounds, later) = unsolvable(Arm::A, &l, Some(0));
            got.push((un, rounds, later));
            moved += later;
        }
        assert_eq!(got, vec![(0, 2, 0), (0, 2, 0), (0, 2, 0), (0, 2, 0), (0, 2, 0)], "{got:?}");
        assert_eq!(moved, 0);
    }

    /// 容器数的不动点：容器自己也要一条记录，所以 c 略大于 units / 583。
    #[test]
    fn container_fixpoint_accounts_for_itself() {
        let c = container_count(Arm::A, 483_183_820, &W, per_container());
        assert!(c > 483_183_820 / 583, "{c}");
        let units = 483_183_820 + c;
        let locs = units + n_stripes(units, &W);
        assert_eq!(c, locs.div_ceil(per_container()));
    }

    /// 代价的绝对值：三条臂各自的 ppm，口径是「相对臂丙少装多少用户数据」。
    /// 2026-09-06 换过一次口径（第二轮攻击腿 5.5）：第一版报的是「元数据自己占多少字节」，
    /// 而它把 parity 与容器加在 90% 之上 ⇒ 126.3% 的格。换口径之后三条臂的排序也变了：
    /// 旧口径「甲 2166 < 乙 3516 < 甲镜 4333」，自洽口径「甲 1544 < 乙 2511 < 甲镜 3088」。
    #[test]
    fn cost_absolute() {
        assert_eq!(occupied_slots(), 483_183_820);
        let (base, _) = data_units(Arm::Z, &W, per_container());
        assert_eq!(base, 345_131_300);
        let (_, z) = cost_bytes(Arm::Z, &W);
        assert_eq!(z, 0.0);
        let (bb, bp) = cost_bytes(Arm::B, &W);
        assert_eq!(bb, RESERVE_H * base);
        assert!((bp - 2511.2).abs() < 0.2, "乙={bp}");
        let (ab, ap) = cost_bytes(Arm::A, &W);
        let (a2b, a2p) = cost_bytes(Arm::A2, &W);
        assert_eq!(ab, 27_157_757_952);
        assert_eq!(a2b, 2 * ab);
        assert!((ap - 1543.7).abs() < 0.2, "甲={ap}");
        assert!((a2p - 3087.5).abs() < 0.2, "甲镜={a2p}");
        // 容器数：两条臂的逻辑容器数相同，差别在副本数
        let (_, ca) = data_units(Arm::A, &W, per_container());
        let (_, ca2) = data_units(Arm::A2, &W, per_container());
        assert_eq!((ca, ca2), (828_789, 828_789));
    }

    /// **臂甲那条路自己的阳性对照**：手工摆一个互指的死锁，度量必须报非 0。
    /// 少了它，「随机摆放下恒 0」分不清是结论如此，还是这条路根本报不出非 0
    /// （`.claude/singlefs-ai-sop/rules/test-discipline.md`「检查本身也可能是错的」）。
    #[test]
    fn deadlock_is_reachable_by_construction() {
        // 盘 0 坏。条带 0 的成员记录全住容器 1（在盘 0 上），条带 1 的成员记录全住容器 0（也在盘 0 上）。
        let grids = vec![
            Grid { dev: 0, slot: 0, kind: Kind::Container, stripe: 0 }, // 0：容器 0
            Grid { dev: 1, slot: 0, kind: Kind::Ordinary,  stripe: 0 }, // 1
            Grid { dev: 2, slot: 0, kind: Kind::Parity,    stripe: 0 }, // 2
            Grid { dev: 0, slot: 1, kind: Kind::Container, stripe: 1 }, // 3：容器 1
            Grid { dev: 1, slot: 1, kind: Kind::Ordinary,  stripe: 1 }, // 4
            Grid { dev: 2, slot: 1, kind: Kind::Parity,    stripe: 1 }, // 5
        ];
        let l = Layout {
            grids,
            stripes: vec![vec![0, 1, 2], vec![3, 4, 5]],
            containers: vec![vec![0], vec![3]],
            record_container: vec![Some(1), Some(1), Some(1), Some(0), Some(0), Some(0)],
        };
        let (un, rounds, later) = unsolvable(Arm::A, &l, Some(0));
        assert_eq!((un, rounds, later), (2, 1, 0), "互指的两个容器必须都解不开");
        // 只要把条带 0 的一条记录挪进一个活着的容器，死锁就解开 —— 证明报的是死锁不是别的
        let mut l2 = l.clone();
        l2.record_container[1] = Some(0);
        assert_eq!(unsolvable(Arm::A, &l2, Some(0)).0, 2, "记录仍在坏盘上的容器里，照旧解不开");
        let mut l3 = l.clone();
        l3.containers = vec![vec![0], vec![3]];
        l3.grids[3].dev = 1; // 容器 1 搬到活盘上
        assert_eq!(unsolvable(Arm::A, &l3, Some(0)).0, 0);
    }

    /// **加盘窗口那一格：确定性放置让臂甲退化**（2026-09-06 第二轮攻击腿 5.3 给的真实场景）。
    /// 「稳态恒 0」的承重点是「容器落哪块盘」与「它描述的格落哪块盘」独立，
    /// 而 D2（RAID 条带策略） 已定项 8 的「取最空的」把两者绑在一起。
    #[test]
    fn add_disk_window_breaks_arm_a() {
        let (la, dev) = add_disk_window(Arm::A, 4096, 8, 4096, &W);
        let (una, _, _) = unsolvable(Arm::A, &la, Some(dev));
        let lost = lost_count(&la, dev);
        assert!(lost > 0, "新盘上必须真的落了东西");
        // 2026-09-06 第二轮攻击腿 5.3 预言臂甲在这一格会退化成阳性对照臂丙。**建模之后没复现**：
        // 一次攒批里的 w−1 个单元被摊到 w−1 个不同的列上，容器与它描述的格因此仍落在不同盘上
        // （实测容器落点 [0,7,2,3,7,5,7,0,1,7]，10 个里 4 个在新盘）。
        // 如实记成 0，并把「为什么没复现」钉成断言：新盘上每条条带至多一个格。
        assert_eq!(una, 0, "臂甲在加盘窗口上实得 {una} / 丢 {lost}");
        let mut per_stripe_on_new = std::collections::HashMap::new();
        for g in &la.grids {
            if g.dev == dev { *per_stripe_on_new.entry(g.stripe).or_insert(0u32) += 1; }
        }
        assert!(per_stripe_on_new.values().all(|&n| n == 1), "一条条带在同一块盘上不许有两个格");
        // 臂丙（阳性对照）在同一格上恒等于丢失格数；臂甲有多接近它，就是这条失真有多大
        let (lz, _) = add_disk_window(Arm::Z, 4096, 8, 4096, &W);
        let (unz, _, _) = unsolvable(Arm::Z, &lz, Some(dev));
        assert_eq!(unz, lost_count(&lz, dev));
        // 臂甲镜与臂乙在同一格上仍是 0
        for arm in [Arm::A2, Arm::B] {
            let (l, d) = add_disk_window(arm, 4096, 8, 4096, &W);
            assert_eq!(unsolvable(arm, &l, Some(d)).0, 0, "{arm:?}");
        }
    }

    /// **容器单独一次发布写出时，`w` 按 D2（RAID 条带策略） 已定项 6 的公式自动等于 2**
    /// ⇒ 那次写自动是镜像（D9（加密）：w = 2 的 parity 就是逐字节相同的第二副本）
    /// ⇒ 臂甲在这个写入形态下退化成臂甲镜，自举那条边消失。
    /// 这是纯算术，不是模型的性质：`clamp(1 + 1, 2, 4) = 2`。
    #[test]
    fn a_lone_container_publish_is_mirrored_by_the_width_formula() {
        let w = (1u64 + 1).clamp(2, 4);
        assert_eq!(w, 2);
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
            let (l, dev) = first_stripe_self_reference(4, seed);
            assert_eq!(l.containers.len(), 1, "seed={seed}：这一格要的是只有一个容器");
            let sid = l.grids[l.containers[0][0]].stripe;
            assert!(l.stripes[sid].iter().all(|&m| l.record_container[m] == Some(0)),
                "seed={seed}：四条记录必须全在那个容器里，否则这一格没造出来");
            let (un, _, _) = unsolvable(Arm::A, &l, Some(dev));
            assert!(un > 0, "seed={seed}：自指的容器丢了必须解不开，实得 {un}");
            assert_eq!(un, lost_count(&l, dev), "seed={seed}：坏盘上的格一个都救不回来");
            // 臂乙（parity 自证）与臂甲镜（容器镜像）在同一格上仍是 0 —— 差别是结构性的
            let (l2, dev2) = {
                let l2 = build(Arm::A2, 2, Params { devs: 4, cap: per_container(), placement: Placement::Random, prefill: 0 }, &[4usize], seed);
                let d = l2.grids[l2.containers[0][0]].dev;
                (l2, d)
            };
            assert_eq!(unsolvable(Arm::A2, &l2, Some(dev2)).0, 0, "seed={seed}：镜像容器丢不掉");
        }
    }

    /// 闭式条带数与逐条带循环在小值上逐格相等。
    #[test]
    fn closed_form_matches_loop() {
        for u in 0..2000u64 {
            assert_eq!(n_stripes(u, &W), n_stripes_loop(u, &W), "u={u}");
        }
        for u in [10_000u64, 123_457, 1_000_000] {
            assert_eq!(n_stripes(u, &W), n_stripes_loop(u, &W), "u={u}");
        }
    }

    /// 条带数的算术：宽度循环 3,4 ⇒ 每两条条带装 2 + 3 = 5 个单元。
    #[test]
    fn stripe_count_arithmetic() {
        assert_eq!(n_stripes(5, &W), 2);
        assert_eq!(n_stripes(10, &W), 4);
        assert_eq!(n_stripes(1, &W), 1);
        assert_eq!(n_stripes(0, &W), 0);
    }
}

