//! E39：记账结构 COW 还是原地 —— D22 已定项 3 的方向定案要验的那一步。
//!
//! ## 它验的是哪一句
//!
//! D22 已定项 3 的方向定案（2026-08-29，用户）取 COW、挂父指针下，理由逐字是
//! 「单元自包含 ⇒ 父指针那条路不需要额外的定位约定」。**那是推论，没测过。**
//!
//! ## 两条臂在盘上的差别，不是「写法不同」，是 mkfs 阶段就分叉
//!
//! - **COW**：记账叶挂在父指针下，每次提交写到**新地址**，并重写路径上的全部祖先。
//!   mkfs **不预留**任何固定位置。
//! - **原地**：记账住在 mkfs 预留的固定位置，每次提交**覆盖**那几个块。
//!   为了能检出撕裂要配槽轮换（D22 已定三），所以预留量翻倍。
//!
//! ## 判据（experiments.md E39）
//!
//! 1. **写放大**：COW 臂每次提交多写的块数 ≤ 原地臂 2 倍 ⇒ 方向定案站得住。
//! 2. **恢复代价**：原地臂撕裂后必须是全盘扫描量级，COW 臂必须是 O(到根的路径)。
//!    **这一条才是方向定案的真正依据，写放大只是它的价钱。**
//!
//! ⚠️ 阳性对照：把 COW 臂的「每次新地址」换成「原地覆盖同一地址」，
//! 写放大必须显著下降——否则「COW 更贵」根本没被模型量到，整轮作废。

use e7_index_bench::Emitter;
use std::collections::BTreeSet;

/// 完美树：扇出 F、树高 H ⇒ 叶数 F^H。
#[derive(Debug, Clone, Copy, PartialEq)]
struct Tree { fanout: u64, height: u32 }

impl Tree {
    fn leaves(&self) -> u64 { (self.fanout as u64).pow(self.height) }
    /// 叶 `i` 在第 `l` 层的祖先编号（l = 1 是叶的父，l = height 是根）。
    fn ancestor(&self, leaf: u64, l: u32) -> u64 { leaf / self.fanout.pow(l) }
}

/// 一次提交碰到的叶子集合怎么摊在树上。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Spread {
    /// D25 已定的粗粒度：一次 fsync 带 8 叶、落在 1 条共享脊柱上 ⇒ 叶子连号。
    Clustered,
    /// 对照：同样多的叶子，均匀撒在整棵树上 ⇒ 祖先几乎不共享。
    Scattered,
}

fn touched(t: &Tree, spread: Spread, per_commit: u64, commit: u64) -> BTreeSet<u64> {
    let l = t.leaves();
    let mut s = BTreeSet::new();
    match spread {
        Spread::Clustered => {
            let base = (commit.wrapping_mul(per_commit)) % l;
            for k in 0..per_commit { s.insert((base + k) % l); }
        }
        Spread::Scattered => {
            let stride = (l / per_commit.max(1)).max(1);
            for k in 0..per_commit { s.insert((commit + k * stride) % l); }
        }
    }
    s
}

/// COW 一次提交写的块数 = 叶数 + 各层**去重后**的祖先数。
/// 祖先要去重：共享脊柱正是 D25 粗粒度买到的那个东西。
fn cow_writes(t: &Tree, leaves: &BTreeSet<u64>) -> u64 {
    let mut n = leaves.len() as u64;
    for l in 1..=t.height {
        let anc: BTreeSet<u64> = leaves.iter().map(|&i| t.ancestor(i, l)).collect();
        n += anc.len() as u64;
    }
    n
}

/// 原地一次提交写的块数 = 叶数。祖先不动——父指针不变，因为地址没变。
fn inplace_writes(_t: &Tree, leaves: &BTreeSet<u64>) -> u64 { leaves.len() as u64 }

#[derive(Debug, Clone, Copy, PartialEq)]
struct Arm {
    writes: u64,
    /// mkfs 要为记账预留几块。原地要预留**全部节点**（不只是叶）且因槽轮换翻倍；COW 不预留。
    reserved: u64,
    /// 一次撕裂会不会打掉**唯一**一份 ⇒ 1 = 会。
    torn_loses_only_copy: u64,
    /// 撕裂之后要读多少块才能把账重建出来。
    torn_rebuild_reads: u64,
}

/// 一棵完美树的**全部**节点数：叶 + 各层内部节点。
/// 原地那一侧要给它们全部在盘上留固定位置，不只是叶子。
fn total_nodes(t: &Tree) -> u64 {
    let mut n = t.leaves();
    let mut level = t.leaves();
    for _ in 1..=t.height { level = level.div_ceil(t.fanout); n += level; }
    n
}

fn run(t: &Tree, spread: Spread, per_commit: u64, commits: u64, pool_blocks: u64, cow: bool) -> Arm {
    let mut writes = 0u64;
    for c in 0..commits {
        let s = touched(t, spread, per_commit, c);
        writes += if cow { cow_writes(t, &s) } else { inplace_writes(t, &s) };
    }
    if cow {
        Arm { writes, reserved: 0, torn_loses_only_copy: 0,
              // 撕裂的是刚写出的新地址，父指针还没换过去 ⇒ 老版本原封不动，
              // 恢复只要沿旧根走一遍那条路径确认。
              torn_rebuild_reads: t.height as u64 }
    } else {
        Arm { writes, reserved: total_nodes(t) * 2, torn_loses_only_copy: 1,
              // 记账是权威态（D21）且没有副本 ⇒ 唯一重建路径是全盘扫描。
              torn_rebuild_reads: pool_blocks }
    }
}

/// 阳性对照：COW 的形状但「原地覆盖同一地址」⇒ 不重写祖先。
/// 它必须比真 COW 明显便宜，否则模型根本没在量 COW 的代价。
fn poscontrol(t: &Tree, spread: Spread, per_commit: u64, commits: u64) -> u64 {
    let mut w = 0u64;
    for c in 0..commits { w += touched(t, spread, per_commit, c).len() as u64; }
    w
}

/// 块大小。**是配置不是实测**——所有「多少字节」的数都随它线性变。
const BLOCK_BYTES: u64 = 4096;

/// 预留占池的比例，单位是**万分之一（bp）**。整数算，避免浮点在产物里抖。
fn reserved_bp(reserved_blocks: u64, pool_blocks: u64) -> u64 {
    reserved_blocks.saturating_mul(10_000) / pool_blocks
}

/// 要让原地那一侧的预留占比不超过 `pct`%，池至少要多大（字节）。
///
/// ⚠️ **可接受线取 2%（2026-08-29，用户定案）**，不是 1%：
/// 别家文件系统的元数据开销本来就接近 1%（D21 的逐字节表：本工程 4 KiB 单元 3.1%、128 KiB 单元 0.098%），
/// 而本工程的取向是 `.claude/rules/fs-design.md`「不为省空间牺牲自包含」——
/// **1% 那条线太紧，不够浪费。**
fn min_pool_bytes_for(reserved_blocks: u64, pct: u64) -> u64 {
    reserved_blocks * BLOCK_BYTES * 100 / pct
}

fn main() {
    let mut em = Emitter::new();
    let commits = 10_000u64;
    // ⚠️ 池大小是**配置**，用来把「全盘扫描」表达成一个可比的量。
    // 4 GiB / 64 GiB / 1 TiB 三档，看恢复代价与预留占比怎么随池变。
    let per_commit = 8u64;                 // D25 已定：一次 fsync 带 8 叶
    println!("{}", em.emit_raw(&format!(
        "name=config commits={commits} per_commit={per_commit} block_bytes={BLOCK_BYTES}")));

    for pool_blocks in [1u64 << 20, 1 << 24, 1 << 28] {
        for (f, h) in [(64u64, 2u32), (64, 3), (128, 3), (256, 3)] {
            let t = Tree { fanout: f, height: h };
            for spread in [Spread::Clustered, Spread::Scattered] {
                let cow = run(&t, spread, per_commit, commits, pool_blocks, true);
                let ip = run(&t, spread, per_commit, commits, pool_blocks, false);
                let pc = poscontrol(&t, spread, per_commit, commits);
                let ratio = cow.writes as f64 / ip.writes as f64;
                // 预留占池的比例，按万分之一为单位报（避免浮点在产物里抖）
                let frac_bp = reserved_bp(ip.reserved, pool_blocks);
                println!("{}", em.emit_raw(&format!(
                    "name=cell pool_blocks={pool_blocks} pool_bytes={} fanout={f} height={h} \
                     leaves={} nodes={} spread={spread:?} \
                     cow_writes={} inplace_writes={} write_ratio={ratio:.4} \
                     cow_reserved_blocks={} inplace_reserved_blocks={} inplace_reserved_bytes={} \
                     inplace_reserved_bp_of_pool={frac_bp} inplace_min_pool_bytes_for_2pct={} \
                     cow_torn_loses_only_copy={} inplace_torn_loses_only_copy={} \
                     cow_rebuild_reads={} inplace_rebuild_reads={} inplace_rebuild_bytes={} \
                     poscontrol_writes={}",
                    pool_blocks * BLOCK_BYTES, t.leaves(), total_nodes(&t),
                    cow.writes, ip.writes, cow.reserved, ip.reserved, ip.reserved * BLOCK_BYTES,
                    min_pool_bytes_for(ip.reserved, 2),
                    cow.torn_loses_only_copy, ip.torn_loses_only_copy,
                    cow.torn_rebuild_reads, ip.torn_rebuild_reads,
                    ip.torn_rebuild_reads * BLOCK_BYTES, pc)));
            }
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言 1**：碰一个叶子时，COW 写的块数**恰等于** 1 + 树高。
    /// 由构造直接算出，不从被测代码读回来。
    #[test]
    fn one_leaf_costs_exactly_one_plus_height() {
        for h in 1..=5u32 {
            let t = Tree { fanout: 16, height: h };
            let mut s = BTreeSet::new(); s.insert(0u64);
            assert_eq!(cow_writes(&t, &s), 1 + h as u64,
                "碰一个叶子该写 1 叶 + {h} 层祖先");
        }
    }

    /// **绝对值断言 2**：碰满一个父节点下的全部 F 个叶子时，
    /// 各层祖先各只有一个 ⇒ 恰等于 F + 树高。
    #[test]
    fn a_full_sibling_group_costs_exactly_fanout_plus_height() {
        let t = Tree { fanout: 8, height: 3 };
        let s: BTreeSet<u64> = (0..8u64).collect();
        assert_eq!(cow_writes(&t, &s), 8 + 3, "同一个父下的 8 个叶子共享全部祖先");
    }

    /// **绝对值断言 3**：叶子撒开到各自独立的第 1 层父之下时，
    /// 第 1 层祖先数恰等于叶子数——共享脊柱这时不存在。
    #[test]
    fn scattered_leaves_do_not_share_their_level_one_parents() {
        let t = Tree { fanout: 8, height: 3 };          // 512 叶
        let s: BTreeSet<u64> = (0..4u64).map(|k| k * 8).collect();   // 0,8,16,24
        let anc1: BTreeSet<u64> = s.iter().map(|&i| t.ancestor(i, 1)).collect();
        assert_eq!(anc1.len(), 4, "四个叶子该落在四个不同的父下");
        assert_eq!(cow_writes(&t, &s), 4 + 4 + 1 + 1, "4 叶 + 4 个 L1 + 1 个 L2 + 1 个根");
    }

    /// **树的算术自洽**：叶数恰等于扇出^树高。
    #[test]
    fn leaves_equal_fanout_to_the_height() {
        assert_eq!(Tree { fanout: 64, height: 2 }.leaves(), 4096);
        assert_eq!(Tree { fanout: 64, height: 3 }.leaves(), 262_144);
        assert_eq!(Tree { fanout: 256, height: 3 }.leaves(), 16_777_216);
    }

    /// **原地不写祖先**——这正是它比 COW 便宜的全部来源。
    #[test]
    fn inplace_writes_exactly_the_touched_leaves() {
        let t = Tree { fanout: 64, height: 3 };
        for n in [1u64, 8, 64] {
            let s: BTreeSet<u64> = (0..n).collect();
            assert_eq!(inplace_writes(&t, &s), n, "原地只写被碰的那 {n} 个叶子");
        }
    }

    /// **判据 1**：在 D25 已定的粗粒度下，COW 的写放大不超过原地 2 倍。
    #[test]
    fn clustered_cow_write_amplification_stays_under_two() {
        for (f, h) in [(64u64, 2u32), (64, 3), (128, 3), (256, 3)] {
            let t = Tree { fanout: f, height: h };
            let cow = run(&t, Spread::Clustered, 8, 1000, 1 << 20, true);
            let ip = run(&t, Spread::Clustered, 8, 1000, 1 << 20, false);
            let r = cow.writes as f64 / ip.writes as f64;
            assert!(r <= 2.0, "粗粒度下 COW 写放大该 ≤ 2×，实测 {r:.3}（F={f} H={h}）");
        }
    }

    /// **散开时写放大会破 2 倍**——这条把「1.x 倍」钉在 D25 的定案上，
    /// 而不是钉在「COW 天生便宜」这个错觉上。
    #[test]
    fn scattering_breaks_the_two_times_bound() {
        let t = Tree { fanout: 64, height: 3 };
        let cow = run(&t, Spread::Scattered, 8, 1000, 1 << 20, true);
        let ip = run(&t, Spread::Scattered, 8, 1000, 1 << 20, false);
        let r = cow.writes as f64 / ip.writes as f64;
        assert!(r > 2.0, "叶子撒开之后祖先不共享，写放大该破 2 倍，实测 {r:.3}");
    }

    /// **判据 2**：恢复代价必须差一个数量级以上，且方向固定。
    #[test]
    fn recovery_costs_differ_by_orders_of_magnitude() {
        let t = Tree { fanout: 64, height: 3 };
        let cow = run(&t, Spread::Clustered, 8, 10, 1 << 20, true);
        let ip = run(&t, Spread::Clustered, 8, 10, 1 << 20, false);
        assert_eq!(cow.torn_loses_only_copy, 0, "COW 撕裂的是新地址，老版本还在");
        assert_eq!(ip.torn_loses_only_copy, 1, "原地撕裂打掉的是唯一一份");
        assert_eq!(cow.torn_rebuild_reads, 3, "COW 恢复是 O(到根的路径) = 树高");
        assert_eq!(ip.torn_rebuild_reads, 1 << 20, "原地恢复是全盘扫描");
        assert!(ip.torn_rebuild_reads > cow.torn_rebuild_reads * 1000);
    }

    /// **mkfs 分叉是真的**：原地要预留，COW 不要。这一条决定「先跑起来再改」行不行。
    #[test]
    fn only_inplace_forces_mkfs_to_reserve_space() {
        let t = Tree { fanout: 64, height: 2 };
        assert_eq!(run(&t, Spread::Clustered, 8, 10, 1 << 20, true).reserved, 0);
        // 4096 叶 + 64 个 L1 + 1 个根 = 4161 个节点，槽轮换翻倍 ⇒ 8322 块
        assert_eq!(total_nodes(&t), 4096 + 64 + 1);
        assert_eq!(run(&t, Spread::Clustered, 8, 10, 1 << 20, false).reserved, (4096 + 64 + 1) * 2,
            "原地要预留**全部节点**（不只是叶），且槽轮换让它翻倍");
    }

    /// **绝对值断言**：节点总数由几何直接算出，不从被测代码读回来。
    #[test]
    fn total_nodes_matches_independently_computed_arithmetic() {
        assert_eq!(total_nodes(&Tree { fanout: 64, height: 2 }), 4096 + 64 + 1);
        assert_eq!(total_nodes(&Tree { fanout: 64, height: 3 }), 262_144 + 4096 + 64 + 1);
        assert_eq!(total_nodes(&Tree { fanout: 256, height: 3 }), 16_777_216 + 65_536 + 256 + 1);
    }

    /// **绝对值断言**：占比的单位是万分之一，由算术直接钉死。
    /// ⚠️ 变异测试补出来的：把 10_000 改成 1_000 时**一个测试都没红**——
    /// 那段算术当时只活在 `main()` 里，没有任何检查看得见它。
    #[test]
    fn reserved_fraction_is_in_basis_points() {
        assert_eq!(reserved_bp(1, 10_000), 1, "万分之一就是 1 bp");
        assert_eq!(reserved_bp(100, 10_000), 100, "百分之一 = 100 bp");
        assert_eq!(reserved_bp(10_000, 10_000), 10_000, "占满就是 10000 bp");
        // F=64/H=2 在 4 GiB 池（1<<20 块）上：8322 / 1048576 = 79 bp = 0.79%
        assert_eq!(reserved_bp(8322, 1 << 20), 79);
    }

    /// **「预留多少字节」这个数必须随块大小线性走**——它是配置的函数，不是结论。
    #[test]
    fn reserved_bytes_scale_linearly_with_block_size() {
        let t = Tree { fanout: 64, height: 2 };
        let blocks = run(&t, Spread::Clustered, 8, 10, 1 << 20, false).reserved;
        assert_eq!(blocks * BLOCK_BYTES, blocks * 4096);
        // 可接受线是 2% ⇒ 池至少要 50 倍于预留
        assert_eq!(min_pool_bytes_for(blocks, 2), blocks * 4096 * 50);
        assert_eq!(min_pool_bytes_for(blocks, 1), blocks * 4096 * 100, "1% 那条线要 100 倍");
    }

    /// **阳性对照**：把「每次新地址」换成「原地覆盖」之后写量必须显著下降。
    /// 它证明模型真的在量 COW 那部分代价，而不是在量别的东西。
    #[test]
    fn poscontrol_is_cheaper_because_it_stops_rewriting_ancestors() {
        let t = Tree { fanout: 64, height: 3 };
        let cow = run(&t, Spread::Clustered, 8, 1000, 1 << 20, true).writes;
        let pc = poscontrol(&t, Spread::Clustered, 8, 1000);
        assert!(cow > pc, "真 COW 该比「不重写祖先」贵（{cow} vs {pc}）");
        assert_eq!(pc, 8 * 1000, "阳性对照每次恰好写 8 个叶子，绝对值钉死");
    }
}
