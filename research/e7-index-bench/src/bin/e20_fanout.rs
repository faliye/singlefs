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

const NODE_HEADER_BYTES: usize = 64;
const CACHE_LINE_BYTES: usize = 64;

/// 一棵按数组铺开的隐式 B 树：每个节点占 `node_bytes`，节点内条目按 `entry_bytes` 跨步排列，
/// 条目的**前 8 字节是 key**，其余是载荷（指针等）。载荷不参与比较，
/// 但它**占着缓存行**——这正是本实验要量的东西。
struct Tree {
    buffer: Vec<u8>,
    entry_bytes: usize,
    node_bytes: usize,
    slots: usize,
    depth: usize,
    node_count: usize,
}

impl Tree {
    fn new(key_count: usize, entry_bytes: usize, node_bytes: usize) -> Self {
        let slots = ((node_bytes - NODE_HEADER_BYTES) / entry_bytes).max(2);
        // 树深：叶子层装下全部 key，往上按扇出收
        let mut level_nodes = key_count.div_ceil(slots).max(1);
        let mut depth = 1;
        let mut total_nodes = level_nodes;
        while level_nodes > 1 {
            level_nodes = level_nodes.div_ceil(slots);
            total_nodes += level_nodes;
            depth += 1;
        }
        let mut tree = Tree {
            buffer: vec![0u8; total_nodes * node_bytes],
            entry_bytes, node_bytes, slots, depth, node_count: total_nodes,
        };
        // ⚠️ **每个节点的 key 都是 0..slots-1（同一值域）**，查找时按层取 key 的不同位段。
        // 第一版按节点号错开值域，结果查找 key 均匀分布在全域上，
        // **99.99% 的查找在第一层就落到最右端**，每次走同样几个节点、全在 L1 里，
        // 阳性对照因此永远过不了。这是布局错，不是被测对象的性质。
        for node in 0..total_nodes {
            for slot_index in 0..slots {
                let key = slot_index as u64;
                let child_scatter_seed = (node as u64) * (slots as u64) + slot_index as u64; // 只用来散布子节点
                let offset = node * tree.node_bytes + NODE_HEADER_BYTES + slot_index * tree.entry_bytes;
                tree.buffer[offset..offset + 8].copy_from_slice(&key.to_le_bytes());
                // 条目的第二个 8 字节存**子节点号**——查找必须把它读出来才知道下一步去哪，
                // 于是形成依赖式载入链（pointer chasing），缓存缺失才会显形。
                // 用一个散列把子节点打散到整个缓冲区上，避免顺序预取把效应抹平。
                let child = (child_scatter_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 17) % (total_nodes as u64);
                if tree.entry_bytes >= 16 {
                    tree.buffer[offset + 8..offset + 16].copy_from_slice(&child.to_le_bytes());
                }
            }
        }
        tree
    }

    #[inline(always)]
    fn child_at(&self, node: usize, slot_index: usize) -> usize {
        let offset = node * self.node_bytes + NODE_HEADER_BYTES + slot_index * self.entry_bytes + 8;
        u64::from_le_bytes(self.buffer[offset..offset + 8].try_into().unwrap()) as usize
    }

    #[inline(always)]
    fn key_at(&self, node: usize, slot_index: usize) -> u64 {
        let offset = node * self.node_bytes + NODE_HEADER_BYTES + slot_index * self.entry_bytes;
        u64::from_le_bytes(self.buffer[offset..offset + 8].try_into().unwrap())
    }

    /// 一次点查：从根往下走 `depth` 层，每层在节点内二分。
    /// 返回落到的叶子里的 slot 号，防止被优化掉。
    #[inline(never)]
    fn lookup(&self, key: u64) -> usize {
        let mut node = 0usize;
        let mut slot = 0usize;
        let bits_per_level = (self.slots as f64).log2().ceil() as u32;
        for level in 0..self.depth {
            // 按层取 key 的不同位段作为本层的搜索键 —— 保证每层的二分都落在节点值域内，
            // 且落点随 key 均匀铺开（基数下降）
            let level_search_key = (key >> (level as u32 * bits_per_level)) % (self.slots as u64);
            // 节点内二分 —— 每次比较读一个 key，而 key 之间隔着 entry_bytes
            let (mut lower_bound, mut upper_bound) = (0usize, self.slots);
            while lower_bound < upper_bound {
                let midpoint = (lower_bound + upper_bound) / 2;
                if self.key_at(node, midpoint) < level_search_key { lower_bound = midpoint + 1; } else { upper_bound = midpoint; }
            }
            slot = lower_bound.min(self.slots - 1);
            if level + 1 < self.depth {
                // **从条目里读**出下一层节点号 —— 依赖式载入，缓存缺失无法被预取掩盖
                node = self.child_at(node, slot) % self.node_count;
            }
        }
        slot
    }

    fn footprint(&self) -> usize { self.buffer.len() }
    #[allow(dead_code)]
    /// 一次点查最少要碰几个缓存行：每层二分约 log2(slots) 次比较，
    /// 每次比较大概率落在不同的缓存行上（条目跨步 ≥ entry_bytes）。
    fn lines_per_lookup(&self) -> usize {
        let comparisons = (self.slots as f64).log2().ceil() as usize;
        // 一个缓存行里能装下几个条目的 key（跨步 entry_bytes）
        let entries_per_cache_line = (CACHE_LINE_BYTES / self.entry_bytes).max(1);
        self.depth * comparisons.div_ceil(entries_per_cache_line).max(1) * entries_per_cache_line.min(comparisons).max(1) / entries_per_cache_line.max(1)
            + self.depth * comparisons / entries_per_cache_line.max(1)
    }
}

/// **查找 key 在计时区之外预先生成** —— 否则随机数与取模会淹没树下降本身。
fn generate_lookup_keys(tree: &Tree, lookup_count: usize, seed: u64) -> Vec<u64> {
    let mut random_state = seed | 1;
    let key_span = (tree.slots * tree.node_count) as u64;
    (0..lookup_count).map(|_| {
        random_state ^= random_state >> 12; random_state ^= random_state << 25; random_state ^= random_state >> 27;
        random_state.wrapping_mul(0x2545_F491_4F6C_DD1D) % key_span
    }).collect()
}

/// ⚠️ **每次查找必须依赖上一次的结果。**
/// 否则各次查找互相独立，CPU 的乱序执行会把几十个缓存缺失**并行**掉——
/// 量到的是内存带宽（吞吐），不是缓存缺失的**延迟**。
/// 本实验第一版就栽在这里：阳性对照 L1 档与 DRAM 档比值 0.96，效应被完全抹平。
fn bench(tree: &Tree, keys: &[u64]) -> u64 {
    let mut previous_slot = 0usize;
    let key_span = (tree.slots * tree.node_count) as u64;
    let start_time = Instant::now();
    for &key in keys {
        // 把上一次的结果混进这一次的 key —— 形成串行依赖链
        let chained_key = (key ^ (previous_slot as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) % key_span;
        previous_slot = tree.lookup(chained_key);
    }
    let elapsed_nanoseconds = start_time.elapsed().as_nanos() as u64;
    std::hint::black_box(previous_slot);
    elapsed_nanoseconds
}

fn main() {
    let lookup_count: usize = std::env::args().nth(1).and_then(|argument| argument.parse().ok()).unwrap_or(2_000_000);
    let rounds = 3;
    let mut emitter = Emitter::new();
    let mut output = String::new();
    let mut say = |line: String| { output.push_str(&line); output.push('\n'); };

    say(emitter.emit_raw(&format!("name=config iters={lookup_count} rounds={rounds} line_bytes={CACHE_LINE_BYTES} node_hdr={NODE_HEADER_BYTES}")));

    // 条目宽度：16=key+8B 指针（ext2 级）；24=E17 的缓冲条目；
    // 32=ZFS 那个 256 位槽；40≈本工程指针头部；67=头部+2 副本；111=头部+4+2 条带六列
    let entry_widths = [16usize, 24, 32, 40, 67, 111];
    // key 数：跨越 L1(48K) / L2(1M) / L3(32M) / DRAM 四档
    let key_counts = [1usize << 11, 1 << 15, 1 << 19, 1 << 23];

    // ── 阳性对照：L1 档必须显著快于 DRAM 档 ──
    //
    // ⚠️ **必须对每一个产出结论的配置各跑一遍，不许只跑一个点**（2026-08-29 对抗验证改）。
    // 此前这里硬编码 `entry=16 / node=4096` 跑一次就放行，
    // 而本实验产出结论的是 **entry=40 × 六档节点** 那组扫描——**那组从没过闸**。
    // 这正是 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记下的 E18 教训在重演：
    // 「阳性对照只跑一条臂，等于另一条从没过闸，**而它偏偏就是产出结论的那一条**」。
    // 尤其要紧的是 8 KiB 那一档：它比 4K 和 16K 都差、四轮稳定、至今无解释——
    // **一个没过闸的配置上的反常，分不清是性质还是量错了。**
    {
        let mut failed_configuration_count = 0;
        for &node_bytes in &[2048usize, 4096, 8192, 16384, 32768, 65536] {
            for &entry_bytes in &[16usize, 40, 111] {
                let small = Tree::new(key_counts[0], entry_bytes, node_bytes);
                let big = Tree::new(key_counts[3], entry_bytes, node_bytes);
                let small_keys = generate_lookup_keys(&small, lookup_count / 8, 100);
                let big_keys = generate_lookup_keys(&big, lookup_count / 8, 100);
                let small_nanoseconds = (0..rounds).map(|_| bench(&small, &small_keys)).min().unwrap();
                let big_nanoseconds = (0..rounds).map(|_| bench(&big, &big_keys)).min().unwrap();
                let ratio = big_nanoseconds as f64 / small_nanoseconds as f64;
                let passes_ratio_threshold = ratio >= 2.0;
                if !passes_ratio_threshold { failed_configuration_count += 1; }
                say(emitter.emit_raw(&format!(
                    "name=poscontrol node_bytes={node_bytes} entry_bytes={entry_bytes} small_footprint={} \
big_footprint={} ns_small={small_nanoseconds} ns_big={big_nanoseconds} ratio={ratio:.2} ok={passes_ratio_threshold}",
                    small.footprint(), big.footprint())));
            }
        }
        if failed_configuration_count > 0 {
            say(emitter.finish()); print!("{output}");
            eprintln!("E20: 有 {failed_configuration_count} 个配置的 L1 档与 DRAM 档差不到 2 倍 —— 那些配置没在量缓存，本轮作废");
            std::process::exit(4);
        }
    }

    // ⚠️ **两个点不构成最优点。** 首版只测 4 KiB 与 16 KiB，于是「16 KiB 更好」
    // 被读成了「16 KiB 是最佳」——中间与外面从没测过。扫全档才判得了单调还是有拐点。
    for &node_bytes in &[2048usize, 4096, 8192, 16384, 32768, 65536] {
        for &key_count in &key_counts {
            for &entry_bytes in &entry_widths {
                let tree = Tree::new(key_count, entry_bytes, node_bytes);
                let lookup_keys = generate_lookup_keys(&tree, lookup_count, 7);
                let elapsed_nanoseconds = (0..rounds).map(|_| bench(&tree, &lookup_keys)).min().unwrap();
                say(emitter.emit_raw(&format!(
                    "name=e20 node_bytes={node_bytes} keys={key_count} entry_bytes={entry_bytes} \
                     slots={} depth={} nodes={} footprint_bytes={} \
                     ns_per_lookup={:.2} lookups_per_s={:.0}",
                    tree.slots, tree.depth, tree.node_count, tree.footprint(),
                    elapsed_nanoseconds as f64 / lookup_count as f64, lookup_count as f64 / (elapsed_nanoseconds as f64 / 1e9)
                )));
            }
        }
    }

    say(emitter.finish());
    print!("{output}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 条目变宽必须让扇出下降、工作集变大。不变说明布局没按 entry_bytes 跨步。
    #[test]
    fn wider_entries_shrink_fanout_and_grow_footprint() {
        let narrow_tree = Tree::new(1 << 16, 16, 4096);
        let wide_tree = Tree::new(1 << 16, 111, 4096);
        assert!(wide_tree.slots < narrow_tree.slots, "扇出没下降：{} -> {}", narrow_tree.slots, wide_tree.slots);
        assert!(wide_tree.footprint() > narrow_tree.footprint(), "工作集没变大");
    }

    /// 节点内二分必须真的按 entry_bytes 跨步读 key —— 读错位置会取到 0。
    #[test]
    fn keys_are_laid_out_at_stride() {
        let tree = Tree::new(1 << 12, 40, 4096);
        for slot_index in 0..tree.slots { assert_eq!(tree.key_at(3, slot_index), slot_index as u64); }
    }

    /// 查找必须真的散开：不同的 key 要落到不同的叶子节点上。
    /// 全落到同一处说明布局或搜索键取法错了——第一版就栽在这里。
    #[test]
    fn lookups_spread_across_nodes() {
        let tree = Tree::new(1 << 20, 16, 4096);
        let mut distinct_slots = std::collections::BTreeSet::new();
        let mut random_state = 12345u64;
        for _ in 0..2000 {
            random_state ^= random_state >> 12; random_state ^= random_state << 25; random_state ^= random_state >> 27;
            distinct_slots.insert(tree.lookup(random_state.wrapping_mul(0x2545_F491_4F6C_DD1D)));
        }
        assert!(distinct_slots.len() > 50, "2000 次查找只落到 {} 个不同 slot 上", distinct_slots.len());
    }

    /// 节点内二分必须命中恰好等于搜索键的那个槽：值域是 0..slots-1，
    /// depth=1 时 slot 就是 key % slots，逐个钉死（变异审计补的：
    /// 此前比较方向写反不会有任何测试红）。
    #[test]
    fn binary_search_lands_exactly_on_the_search_key() {
        let tree = Tree::new(100, 16, 4096);
        assert_eq!(tree.depth, 1);
        for key in 0..tree.slots as u64 {
            assert_eq!(tree.lookup(key), (key as usize) % tree.slots, "key={key}");
        }
    }

    /// 子指针必须存在条目的第二个 8 字节里，不与 key 重叠——
    /// 否则指针追逐读的是 key，整条依赖链是假的。
    #[test]
    fn child_pointers_live_beside_keys_not_on_top_of_them() {
        let tree = Tree::new(1 << 16, 16, 4096);
        let any_child_differs_from_key = (0..tree.slots).any(|slot_index| tree.child_at(5, slot_index) as u64 != tree.key_at(5, slot_index));
        assert!(any_child_differs_from_key, "每个条目的子指针都等于它的 key——child_at 读到了 key 的位置");
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
