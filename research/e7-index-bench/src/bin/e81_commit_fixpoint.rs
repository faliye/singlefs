//! E81：提交的固定点 —— 给分配记录树 / 记账树写新节点本身要分配空间、而分配又改这两棵树，
//! 这个自指循环收不收敛、放大多少、被什么变量驱动。
//!
//! ## 为什么要有这个实验
//!
//! 三条已定条款合起来让「记录分配的结构」全部变成 COW 树：
//! D3（空间分配）已定项 3（分配记录住独立 keyspace 的 btree，key = 落点）、
//! D5（快照 / 空间记账机制）已定项 3（记账住 btree）、D22（单元原子性怎么合成）已定项 3
//! （记账 COW 挂父指针下）。⇒ 每次发布要写的树节点自己要分配块，块的分配又要写进分配记录树
//! ——**收敛性与轮数上界全仓零覆盖**（C83（提交固定点没人回答）），而 I-8.1（环几何够大）
//! 要的「任一事务的最坏 journal 占用」没有上界就算不出。ZFS 用 sync passes 处理同型问题——
//! 线索不是证据，未在本工程验证。
//!
//! ## 模型
//!
//! 分配记录树：leaf_id = 落点 / LEAF_CAP，内部层按扇出 F 折叠到根。发布 = U 个用户块 +
//! ACC 个记账树节点起步，固定点循环：本轮新脏的树节点各要一个新块，新块的分配记录下一轮插树；
//! 已脏节点本次发布只 COW 一次，再插新记录不再多付。**驱动变量是「新块的落点在 key 空间上
//! 聚不聚簇」**——三条臂：
//!
//! | 臂 | 用户块落点 | 元数据（树节点）落点 | 模拟的分配器政策 |
//! |---|---|---|---|
//! | clustered | 顺序段 | 顺序段 | 全部从一个 bump 段出 |
//! | scattered | 全 key 空间随机 | 全 key 空间随机 | 复用空闲块、不管落点散布 |
//! | meta_clustered | 全 key 空间随机 | 顺序段 | 只把**提交自身产生的块**聚簇 |
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 手算锚点：LEAF_CAP=4、4 叶 + 1 根、U=3、无记账、clustered ⇒ 恰 3 轮收敛、
//!    树脏节点恰 3（两叶一根）、元数据块恰 3。对不上整轮作废。
//! 2. 守恒：插进树的记录数 == 分配出去的块数（两个独立计数器，不共享自增点）。
//! 3. 每格必须在 ROUNDS_CAP 内收敛（收敛 = 一轮零新脏节点）；轮数与放大如实报。
//! 4. 阳性对照：U=12 时 scattered 的元数据块 ≥ 5 × clustered（机制在场）；
//!    **同种子下 scattered ≥ 3 × meta_clustered**（两臂的用户散布逐位相同，唯一差别是
//!    元数据落点 ⇒ 差值就是链），且 meta_clustered 的轮数 ≤ 4、
//!    元数据块 ≤ 2U + 层数 + ACC + 4（第一轮基数上界加零头——没有链的形状）。
//!    任一不中 ⇒ 模型没有判别力，整轮作废。
//! 5. 不判「分配器必须聚簇」为定案——那是决策；本实验只交数字与机制。
//!
//! ⚠️ **判据 4 后半在首次试跑时修订过一次（2026-09-02，任何正式轮入库之前）**：
//! 初版写「meta_clustered ≤ 2 × clustered」，比较基算错了——meta_clustered 的第一轮
//! 基数天然是 ~2U + ACC（用户记录随机散布，各脏一叶一祖先），那是**起步成本不是链**；
//! 拿全聚簇臂当分母把两样东西混在一个比值里。修订成上面的形态：
//! 同种子 scattered 对比隔离出链，绝对上界钉住「没有链」。
//!
//! ## 它答不了的
//!
//! 计数模型，文件操作 0 处。树的分裂/合并没建（记录按整格覆盖计）；LEAF_CAP=512、F=256
//! 是 16 KiB 节点下的量级参数不是实测；「已脏节点再插不多付」假设发布末尾一次性 COW
//! ——若实现边插边 COW，放大更大，本模型给的是下界。scattered 的期望值是生日碰撞量级，
//! 只报实测不推公式。

use e7_index_bench::Emitter;

/// 16 KiB 节点 / ~32 B 分配记录条目的量级。
const LEAF_CAP: u64 = 512;
/// 内部节点扇出的量级。
const FANOUT: u64 = 256;
/// key 空间的叶子数（≈ 1 TiB 池 / 32 KiB 单元 / LEAF_CAP）。
const T_LEAVES: u64 = 65536;
/// 记账树每次发布的脏节点数（叶 + 根，D5（快照 / 空间记账机制）的九个统计量装得进一叶）。
const ACC_NODES: u64 = 2;
const ROUNDS_CAP: usize = 100_000;

/// C59（种子折叠成同一个状态）教训：乘法混淆，不许 `seed | 1`。
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
    Clustered,
    Scattered,
    MetaClustered,
}
impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::Clustered => "clustered",
            Arm::Scattered => "scattered",
            Arm::MetaClustered => "meta_clustered",
        }
    }
}

/// 树几何：叶 + 若干内部层到根。levels[0] = 叶层节点数。
struct Geometry {
    leaf_cap: u64,
    levels: Vec<u64>,
}
impl Geometry {
    fn new(leaf_cap: u64, fanout: u64, t_leaves: u64) -> Geometry {
        let mut levels = vec![t_leaves];
        let mut n = t_leaves;
        while n > 1 {
            n = n.div_ceil(fanout);
            levels.push(n);
        }
        Geometry { leaf_cap, levels }
    }
    fn total_keys(&self) -> u64 {
        self.levels[0] * self.leaf_cap
    }
}

/// 一次发布的固定点结果。
#[derive(Debug)]
struct Outcome {
    rounds: usize,
    converged: bool,
    /// 分配记录树本次发布新脏的节点数（各层合计）。
    tree_dirty: u64,
    /// 元数据块合计（树脏节点 + 记账节点）。
    meta_blocks: u64,
    /// 守恒的两个独立计数器。
    records_inserted: u64,
    blocks_allocated: u64,
}

/// 跑一次发布的固定点。`acc` 关掉时可复现手算锚点。
fn publish(geo: &Geometry, arm: Arm, u_user: u64, acc: u64, seed: u64) -> Outcome {
    let fanout = FANOUT;
    let mut rng = Rng::new(seed);
    // 逐层脏标记
    let mut dirty: Vec<Vec<bool>> = geo.levels.iter().map(|&n| vec![false; n as usize]).collect();
    let mut bump = geo.total_keys() / 2; // 聚簇段起点：key 空间中部的一个空段
    let mut records_inserted = 0u64;
    let mut blocks_allocated = 0u64;
    let mut tree_dirty = 0u64;

    // 起步 pending：用户块 + 记账节点的块
    let mut pending: Vec<u64> = Vec::new();
    for _ in 0..u_user {
        blocks_allocated += 1;
        pending.push(match arm {
            Arm::Clustered => {
                bump += 1;
                bump - 1
            }
            Arm::Scattered | Arm::MetaClustered => rng.below(geo.total_keys()),
        });
    }
    for _ in 0..acc {
        blocks_allocated += 1;
        pending.push(match arm {
            Arm::Clustered | Arm::MetaClustered => {
                bump += 1;
                bump - 1
            }
            Arm::Scattered => rng.below(geo.total_keys()),
        });
    }

    let mut rounds = 0;
    let mut converged = false;
    while rounds < ROUNDS_CAP {
        if pending.is_empty() {
            converged = true;
            break;
        }
        rounds += 1;
        // 插记录，收集本轮新脏节点
        let mut newly = 0u64;
        for &off in &pending {
            records_inserted += 1;
            let mut id = off / geo.leaf_cap;
            for level in dirty.iter_mut() {
                if !level[id as usize] {
                    level[id as usize] = true;
                    newly += 1;
                }
                id /= fanout;
            }
        }
        tree_dirty += newly;
        // 新脏节点各要一个块，其记录下一轮插
        pending = (0..newly)
            .map(|_| {
                blocks_allocated += 1;
                match arm {
                    Arm::Clustered | Arm::MetaClustered => {
                        bump += 1;
                        bump - 1
                    }
                    Arm::Scattered => rng.below(geo.total_keys()),
                }
            })
            .collect();
    }
    Outcome {
        rounds,
        converged,
        tree_dirty,
        meta_blocks: tree_dirty + acc,
        records_inserted,
        blocks_allocated,
    }
}

fn main() {
    let mut em = Emitter::new();
    let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config leaf_cap={LEAF_CAP} fanout={FANOUT} t_leaves={T_LEAVES} levels={} acc_nodes={ACC_NODES} model=counting file_ops=0",
            geo.levels.len()
        ))
    );
    for arm in [Arm::Clustered, Arm::Scattered, Arm::MetaClustered] {
        for u in [1u64, 12, 128, 1024] {
            // scattered 有随机性 ⇒ 5 种子；clustered 确定 ⇒ 种子不影响（有单测钉住）
            for seed in [11u64, 22, 33, 44, 55] {
                let o = publish(&geo, arm, u, ACC_NODES, seed);
                assert!(o.converged, "必须收敛");
                assert_eq!(o.records_inserted, o.blocks_allocated, "守恒破了");
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=fixpoint arm={} u={u} seed={seed} rounds={} tree_dirty={} meta_blocks={} journal_items={} amp_pct={:.1}",
                        arm.tag(),
                        o.rounds,
                        o.tree_dirty,
                        o.meta_blocks,
                        u + o.meta_blocks,
                        100.0 * o.meta_blocks as f64 / u as f64
                    ))
                );
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 手算锚点**：LEAF_CAP=4、4 叶 + 1 根、U=3、无记账、clustered。
    /// 手算：r1 三条用户记录 → 叶 0 + 根新脏（2 块）；r2 两条记录 → 叶 0 已脏、
    /// 叶 1 新脏（1 块）；r3 一条记录 → 叶 1 已脏 → 0 新 ⇒ 恰 3 轮、树脏 3、元数据块 3。
    #[test]
    fn absolute_hand_case() {
        let geo = Geometry::new(4, 4, 4);
        let o = publish(&geo, Arm::Clustered, 3, 0, 1);
        assert!(o.converged);
        assert_eq!(o.rounds, 3);
        assert_eq!(o.tree_dirty, 3);
        assert_eq!(o.meta_blocks, 3);
        assert_eq!(o.records_inserted, 6, "3 用户 + 3 元数据");
        assert_eq!(o.blocks_allocated, 6);
    }

    /// **判据 2 守恒**：全部臂 × 规模，插树记录数恒等于分配块数。
    #[test]
    fn conservation() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        for arm in [Arm::Clustered, Arm::Scattered, Arm::MetaClustered] {
            for u in [1u64, 12, 1024] {
                let o = publish(&geo, arm, u, ACC_NODES, 11);
                assert_eq!(o.records_inserted, o.blocks_allocated, "{arm:?} u={u}");
            }
        }
    }

    /// **判据 3**：全格收敛。
    #[test]
    fn everything_converges() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        for arm in [Arm::Clustered, Arm::Scattered, Arm::MetaClustered] {
            for u in [1u64, 12, 128, 1024] {
                for seed in [11u64, 22, 33, 44, 55] {
                    let o = publish(&geo, arm, u, ACC_NODES, seed);
                    assert!(o.converged, "{arm:?} u={u} seed={seed}");
                    assert!(o.rounds < ROUNDS_CAP);
                }
            }
        }
    }

    /// **判据 4 阳性对照（机制在场）**：U=12 时 scattered 的元数据块 ≥ 5 × clustered。
    /// 不中 ⇒ 模型分不出散布与聚簇，整轮作废。
    #[test]
    fn positive_control_scattered_explodes() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        let c = publish(&geo, Arm::Clustered, 12, ACC_NODES, 11);
        for seed in [11u64, 22, 33, 44, 55] {
            let s = publish(&geo, Arm::Scattered, 12, ACC_NODES, seed);
            assert!(
                s.meta_blocks >= 5 * c.meta_blocks,
                "seed={seed}: scattered {} < 5×clustered {}",
                s.meta_blocks,
                c.meta_blocks
            );
        }
    }

    /// **判据 4 后半（规则有效）**：只把元数据聚簇就足以断链——
    /// 同种子下 scattered ≥ 3 × meta_clustered（两臂用户散布逐位相同，差值就是链）；
    /// meta_clustered 的轮数 ≤ 4、元数据块 ≤ 2U + 层数 + ACC + 4（没有链的形状）。
    #[test]
    fn meta_clustering_alone_breaks_the_chain() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        let u = 12u64;
        let no_chain_cap = 2 * u + geo.levels.len() as u64 + ACC_NODES + 4;
        for seed in [11u64, 22, 33, 44, 55] {
            let s = publish(&geo, Arm::Scattered, u, ACC_NODES, seed);
            let m = publish(&geo, Arm::MetaClustered, u, ACC_NODES, seed);
            assert!(
                s.meta_blocks >= 3 * m.meta_blocks,
                "seed={seed}: scattered {} < 3×meta_clustered {}",
                s.meta_blocks,
                m.meta_blocks
            );
            assert!(m.rounds <= 4, "seed={seed}: rounds={}", m.rounds);
            assert!(
                m.meta_blocks <= no_chain_cap,
                "seed={seed}: {} > 无链上界 {no_chain_cap}",
                m.meta_blocks
            );
        }
    }

    /// clustered 对种子不敏感（它根本不用随机数）。
    #[test]
    fn clustered_is_deterministic_across_seeds() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        let a = publish(&geo, Arm::Clustered, 128, ACC_NODES, 11);
        let b = publish(&geo, Arm::Clustered, 128, ACC_NODES, 99);
        assert_eq!(a.rounds, b.rounds);
        assert_eq!(a.meta_blocks, b.meta_blocks);
    }

    /// 记账节点计入元数据与守恒：ACC 从 0 → 2，元数据块恰 +2（clustered，记录都落进已脏叶时）。
    #[test]
    fn accounting_nodes_are_counted() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        let without = publish(&geo, Arm::Clustered, 12, 0, 11);
        let with = publish(&geo, Arm::Clustered, 12, ACC_NODES, 11);
        assert_eq!(with.meta_blocks, without.meta_blocks + ACC_NODES);
        assert_eq!(with.records_inserted, without.records_inserted + ACC_NODES);
    }

    /// 树几何自检：65536 叶、扇出 256 ⇒ 叶层 + 256 节点层 + 根，共 3 层。
    #[test]
    fn geometry_levels() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        assert_eq!(geo.levels, vec![65536, 256, 1]);
    }

    /// 不同种子给 scattered 不同世界；(2,3) 这对不许折叠（C59（种子折叠成同一个状态））。
    #[test]
    fn seeds_differ() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        let a = publish(&geo, Arm::Scattered, 128, ACC_NODES, 2);
        let b = publish(&geo, Arm::Scattered, 128, ACC_NODES, 3);
        assert!(
            a.meta_blocks != b.meta_blocks || a.rounds != b.rounds,
            "种子 2 与 3 折叠成了同一个世界"
        );
    }

    /// 放大对 U 单调不增（每用户块的元数据摊薄）：clustered 下 U=1024 的摊薄好于 U=12。
    #[test]
    fn clustered_amortizes() {
        let geo = Geometry::new(LEAF_CAP, FANOUT, T_LEAVES);
        let small = publish(&geo, Arm::Clustered, 12, ACC_NODES, 11);
        let big = publish(&geo, Arm::Clustered, 1024, ACC_NODES, 11);
        let amp_small = small.meta_blocks as f64 / 12.0;
        let amp_big = big.meta_blocks as f64 / 1024.0;
        assert!(amp_big < amp_small);
    }
}
