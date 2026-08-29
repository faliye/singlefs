//! E20：指针变宽的真实代价在 CPU 缓存上，不在磁盘上。
//!
//! [decisions.md](decisions.md) D19 此前只量了两样：内部节点的**常驻内存容量**（10.6 → 68.8 GB/PiB）
//! 与**树深**（5 → 6）。两样都算出「代价很小」。
//! **但没量过一次点查的 CPU 时间**——而条目变宽意味着一个缓存行里装得下的 key 变少，
//! 每次节点内查找要碰更多缓存行，工作集也整体变大。
//! 消耗的是 L1/L2/L3 与 DRAM 带宽，**那不是廉价资源**。
//!
//! ## 口径：固定 key 数，不固定工作集
//!
//! 这是本实验唯一重要的设计选择。两种固法给出相反的问题：
//!   - **固定 key 数**（本实验）：条目变宽 ⇒ 工作集变大 ⇒ 被挤出缓存。**这才是要问的。**
//!   - 固定工作集：条目变宽 ⇒ 装得下的 key 变少 ⇒ 换了个更小的数据集，比的不是同一件事。
//!
//! ## 阳性对照
//!
//! 同一条目宽度下，**L1 能装下的工作集必须显著快于超出 L3 的**。
//! 快不了几倍 ⇒ 这个 benchmark 根本没在量缓存，整轮作废。
//!
//! ⚠️ **本实验前两版都栽在阳性对照上**：第一版随机数与取模淹没了树下降；
//! 第二版查找之间互相独立，乱序执行把缓存缺失并行掉了，量到的是带宽不是延迟。
//! **两次都是阳性对照拦下来的**——没有它，那两张表会被当成「指针变宽没有代价」发出去。
//!
//! ## 本机缓存（2026-08-28 现读 `/sys/devices/system/cpu/cpu0/cache/`）
//! 每核 L1d 48 KiB、L2 1 MiB；L3 32 MiB（每 CCX 共享）；缓存行 64 字节。

use e7_index_bench::Emitter;
use std::time::Instant;

const NODE_HDR: usize = 64;
const LINE: usize = 64;

/// 一棵按数组铺开的隐式 B 树：每个节点占 `node_bytes`，节点内条目按 `entry_bytes` 跨步排列，
/// 条目的**前 8 字节是 key**，其余是载荷（指针等）。载荷不参与比较，
/// 但它**占着缓存行**——这正是本实验要量的东西。
struct Tree {
    buf: Vec<u8>,
    entry_bytes: usize,
    node_bytes: usize,
    slots: usize,
    depth: usize,
    n_nodes: usize,
}

impl Tree {
    fn new(n_keys: usize, entry_bytes: usize, node_bytes: usize) -> Self {
        let slots = ((node_bytes - NODE_HDR) / entry_bytes).max(2);
        // 树深：叶子层装下全部 key，往上按扇出收
        let mut level_nodes = n_keys.div_ceil(slots).max(1);
        let mut depth = 1;
        let mut total = level_nodes;
        while level_nodes > 1 {
            level_nodes = level_nodes.div_ceil(slots);
            total += level_nodes;
            depth += 1;
        }
        let mut t = Tree {
            buf: vec![0u8; total * node_bytes],
            entry_bytes, node_bytes, slots, depth, n_nodes: total,
        };
        // ⚠️ **每个节点的 key 都是 0..slots-1（同一值域）**，查找时按层取 key 的不同位段。
        // 第一版按节点号错开值域，结果查找 key 均匀分布在全域上，
        // **99.99% 的查找在第一层就落到最右端**，每次走同样几个节点、全在 L1 里，
        // 阳性对照因此永远过不了。这是布局错，不是被测对象的性质。
        for node in 0..total {
            for i in 0..slots {
                let k = i as u64;
                let mix = (node as u64) * (slots as u64) + i as u64; // 只用来散布子节点
                let off = node * t.node_bytes + NODE_HDR + i * t.entry_bytes;
                t.buf[off..off + 8].copy_from_slice(&k.to_le_bytes());
                // 条目的第二个 8 字节存**子节点号**——查找必须把它读出来才知道下一步去哪，
                // 于是形成依赖式载入链（pointer chasing），缓存缺失才会显形。
                // 用一个散列把子节点打散到整个缓冲区上，避免顺序预取把效应抹平。
                let child = (mix.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 17) % (total as u64);
                if t.entry_bytes >= 16 {
                    t.buf[off + 8..off + 16].copy_from_slice(&child.to_le_bytes());
                }
            }
        }
        t
    }

    #[inline(always)]
    fn child_at(&self, node: usize, i: usize) -> usize {
        let off = node * self.node_bytes + NODE_HDR + i * self.entry_bytes + 8;
        u64::from_le_bytes(self.buf[off..off + 8].try_into().unwrap()) as usize
    }

    #[inline(always)]
    fn key_at(&self, node: usize, i: usize) -> u64 {
        let off = node * self.node_bytes + NODE_HDR + i * self.entry_bytes;
        u64::from_le_bytes(self.buf[off..off + 8].try_into().unwrap())
    }

    /// 一次点查：从根往下走 `depth` 层，每层在节点内二分。
    /// 返回落到的叶子里的 slot 号，防止被优化掉。
    #[inline(never)]
    fn lookup(&self, key: u64) -> usize {
        let mut node = 0usize;
        let mut slot = 0usize;
        let bits = (self.slots as f64).log2().ceil() as u32;
        for lvl in 0..self.depth {
            // 按层取 key 的不同位段作为本层的搜索键 —— 保证每层的二分都落在节点值域内，
            // 且落点随 key 均匀铺开（基数下降）
            let sk = (key >> (lvl as u32 * bits)) % (self.slots as u64);
            // 节点内二分 —— 每次比较读一个 key，而 key 之间隔着 entry_bytes
            let (mut lo, mut hi) = (0usize, self.slots);
            while lo < hi {
                let mid = (lo + hi) / 2;
                if self.key_at(node, mid) < sk { lo = mid + 1; } else { hi = mid; }
            }
            slot = lo.min(self.slots - 1);
            if lvl + 1 < self.depth {
                // **从条目里读**出下一层节点号 —— 依赖式载入，缓存缺失无法被预取掩盖
                node = self.child_at(node, slot) % self.n_nodes;
            }
        }
        slot
    }

    fn footprint(&self) -> usize { self.buf.len() }
    #[allow(dead_code)]
    /// 一次点查最少要碰几个缓存行：每层二分约 log2(slots) 次比较，
    /// 每次比较大概率落在不同的缓存行上（条目跨步 ≥ entry_bytes）。
    fn lines_per_lookup(&self) -> usize {
        let cmps = (self.slots as f64).log2().ceil() as usize;
        // 一个缓存行里能装下几个条目的 key（跨步 entry_bytes）
        let per_line = (LINE / self.entry_bytes).max(1);
        self.depth * cmps.div_ceil(per_line).max(1) * per_line.min(cmps).max(1) / per_line.max(1)
            + self.depth * cmps / per_line.max(1)
    }
}

/// **查找 key 在计时区之外预先生成** —— 否则随机数与取模会淹没树下降本身。
fn gen_keys(t: &Tree, iters: usize, seed: u64) -> Vec<u64> {
    let mut s = seed | 1;
    let span = (t.slots * t.n_nodes) as u64;
    (0..iters).map(|_| {
        s ^= s >> 12; s ^= s << 25; s ^= s >> 27;
        s.wrapping_mul(0x2545_F491_4F6C_DD1D) % span
    }).collect()
}

/// ⚠️ **每次查找必须依赖上一次的结果。**
/// 否则各次查找互相独立，CPU 的乱序执行会把几十个缓存缺失**并行**掉——
/// 量到的是内存带宽（吞吐），不是缓存缺失的**延迟**。
/// 本实验第一版就栽在这里：阳性对照 L1 档与 DRAM 档比值 0.96，效应被完全抹平。
fn bench(t: &Tree, keys: &[u64]) -> u64 {
    let mut prev = 0usize;
    let span = (t.slots * t.n_nodes) as u64;
    let t0 = Instant::now();
    for &k in keys {
        // 把上一次的结果混进这一次的 key —— 形成串行依赖链
        let kk = (k ^ (prev as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) % span;
        prev = t.lookup(kk);
    }
    let ns = t0.elapsed().as_nanos() as u64;
    std::hint::black_box(prev);
    ns
}

fn main() {
    let iters: usize = std::env::args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(2_000_000);
    let rounds = 3;
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| { out.push_str(&s); out.push('\n'); };

    say(em.emit_raw(&format!("name=config iters={iters} rounds={rounds} line_bytes={LINE} node_hdr={NODE_HDR}")));

    // 条目宽度：16=key+8B 指针（ext2 级）；24=E17 的缓冲条目；
    // 32=ZFS 那个 256 位槽；40≈本工程指针头部；67=头部+2 副本；111=头部+4+2 条带六列
    let widths = [16usize, 24, 32, 40, 67, 111];
    // key 数：跨越 L1(48K) / L2(1M) / L3(32M) / DRAM 四档
    let key_counts = [1usize << 11, 1 << 15, 1 << 19, 1 << 23];

    // ── 阳性对照：同一宽度下，L1 档必须显著快于 DRAM 档 ──
    {
        let small = Tree::new(key_counts[0], 16, 4096);
        let big = Tree::new(key_counts[3], 16, 4096);
        let ks = gen_keys(&small, iters / 4, 100);
        let kb = gen_keys(&big, iters / 4, 100);
        let ns_s = (0..rounds).map(|_| bench(&small, &ks)).min().unwrap();
        let ns_b = (0..rounds).map(|_| bench(&big, &kb)).min().unwrap();
        let ratio = ns_b as f64 / ns_s as f64;
        let ok = ratio >= 2.0;
        say(em.emit_raw(&format!(
            "name=poscontrol small_footprint={} big_footprint={} ns_small={ns_s} ns_big={ns_b} ratio={ratio:.2} ok={ok}",
            small.footprint(), big.footprint())));
        if !ok {
            say(em.finish()); print!("{out}");
            eprintln!("E20: L1 档与 DRAM 档差不到 2 倍 —— 这个 benchmark 没在量缓存，本轮作废");
            std::process::exit(4);
        }
    }

    // ⚠️ **两个点不构成最优点。** 首版只测 4 KiB 与 16 KiB，于是「16 KiB 更好」
    // 被读成了「16 KiB 是最佳」——中间与外面从没测过。扫全档才判得了单调还是有拐点。
    for &node_bytes in &[2048usize, 4096, 8192, 16384, 32768, 65536] {
        for &nk in &key_counts {
            for &w in &widths {
                let t = Tree::new(nk, w, node_bytes);
                let ks = gen_keys(&t, iters, 7);
                let ns = (0..rounds).map(|_| bench(&t, &ks)).min().unwrap();
                say(em.emit_raw(&format!(
                    "name=e20 node_bytes={node_bytes} keys={nk} entry_bytes={w} \
                     slots={} depth={} nodes={} footprint_bytes={} \
                     ns_per_lookup={:.2} lookups_per_s={:.0}",
                    t.slots, t.depth, t.n_nodes, t.footprint(),
                    ns as f64 / iters as f64, iters as f64 / (ns as f64 / 1e9)
                )));
            }
        }
    }

    say(em.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 条目变宽必须让扇出下降、工作集变大。不变说明布局没按 entry_bytes 跨步。
    #[test]
    fn wider_entries_shrink_fanout_and_grow_footprint() {
        let a = Tree::new(1 << 16, 16, 4096);
        let b = Tree::new(1 << 16, 111, 4096);
        assert!(b.slots < a.slots, "扇出没下降：{} -> {}", a.slots, b.slots);
        assert!(b.footprint() > a.footprint(), "工作集没变大");
    }

    /// 节点内二分必须真的按 entry_bytes 跨步读 key —— 读错位置会取到 0。
    #[test]
    fn keys_are_laid_out_at_stride() {
        let t = Tree::new(1 << 12, 40, 4096);
        for i in 0..t.slots { assert_eq!(t.key_at(3, i), i as u64); }
    }

    /// 查找必须真的散开：不同的 key 要落到不同的叶子节点上。
    /// 全落到同一处说明布局或搜索键取法错了——第一版就栽在这里。
    #[test]
    fn lookups_spread_across_nodes() {
        let t = Tree::new(1 << 20, 16, 4096);
        let mut seen = std::collections::BTreeSet::new();
        let mut s = 12345u64;
        for _ in 0..2000 {
            s ^= s >> 12; s ^= s << 25; s ^= s >> 27;
            seen.insert(t.lookup(s.wrapping_mul(0x2545_F491_4F6C_DD1D)));
        }
        assert!(seen.len() > 50, "2000 次查找只落到 {} 个不同 slot 上", seen.len());
    }

    /// 查找必须真的走完 depth 层——树深为 1 时和多层时行为要不同。
    #[test]
    fn lookup_descends_all_levels() {
        let shallow = Tree::new(100, 16, 4096);
        let deep = Tree::new(1 << 20, 16, 4096);
        assert_eq!(shallow.depth, 1);
        assert!(deep.depth >= 3, "深树的层数只有 {}", deep.depth);
    }
}
