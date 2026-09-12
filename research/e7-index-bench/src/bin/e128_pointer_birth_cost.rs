//! E128：指针带出生 (树, txg) 之后，本工程**真实条目宽**的点查代价。
//!
//! D19 已定项 7 的候选甲把指针头部 31 → 47，连带把 inode 树内部条目 93 → 109、
//! 记账树内部条目 81 → 97。用户 2026-09-10 判这一格「另走一轮」，
//! 理由逐字是「甲的一样代价（E20 实测 67 字节条目在 16 KiB 节点上 2.22×）
//! **这一笔从没进过任何一条腿的材料**」。
//!
//! ⚠️ **E20 量的不是这一笔**：它的档位是 16 / 24 / 32 / 40 / 67 / 111，
//! 而本工程真实的两组是 93 → 109 与 81 → 97，一档都不在其中。
//! 把 67 与 111 那两个比值拿来代替，是拿另一个量的数回答这一问。
//!
//! **判据、阈值、作废条款在 `research/prompts/e128-preregistration.md`，写于本文件之前。**
//! 本实验不设「慢多少算可接受」的阈值——那个数属于判决，不属于实验。
//!
//! ## 为什么另立新号，不在 E20 上加臂
//!
//! E20 的判据 2026-08-29 写死、产物已入库、`replay.sh` 已注册，与
//! E95 → E127 那次同一条理由。
//!
//! ## 这份模型是 E20 那份的手抄件，靠跨装置闸兜着
//!
//! `Tree` / `generate_lookup_keys` / `bench` 三段与 `e20_fanout.rs` 同源。没有并成一份的理由是：
//! E20 的变异表按**原文匹配**改它自己那个文件，移走那三段会让 E20 已入库的
//! 5 条变异证明当场失配。⇒ 代价是手抄，兜底是**跨装置闸**：
//! 67 与 111 两档必须落回 E20 产物里那两个数（`show-me-test.md`「立在装置之间」）。
//! ⚠️ 闸只钉 DRAM 档那两个点，抄漏在别的档上不会被它抓住。

use e7_index_bench::Emitter;
use std::time::Instant;

const NODE_HEADER_BYTES: usize = 64;
const NODE_BYTES: usize = 16384; // D8 已定项 2 钉死的常量，本实验不扫节点档

/// 跨装置闸的两个基准，逐字取自 `research/results/e20-sweep-2026-08-29.out`
/// 的 `node_bytes=16384 keys=8388608` 两行。
const CROSS_APPARATUS_E20_REFERENCE_NANOSECONDS_PER_LOOKUP: [(usize, f64); 2] = [(67, 466.99), (111, 547.50)];
const CROSS_APPARATUS_RELATIVE_TOLERANCE: f64 = 0.15;

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
        for node in 0..total_nodes {
            for slot_index in 0..slots {
                let key = slot_index as u64;
                let child_scatter_seed = (node as u64) * (slots as u64) + slot_index as u64;
                let offset = node * tree.node_bytes + NODE_HEADER_BYTES + slot_index * tree.entry_bytes;
                tree.buffer[offset..offset + 8].copy_from_slice(&key.to_le_bytes());
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

    #[inline(never)]
    fn lookup(&self, key: u64) -> usize {
        let mut node = 0usize;
        let mut slot = 0usize;
        let bits_per_level = (self.slots as f64).log2().ceil() as u32;
        for level in 0..self.depth {
            let level_search_key = (key >> (level as u32 * bits_per_level)) % (self.slots as u64);
            let (mut lower_bound, mut upper_bound) = (0usize, self.slots);
            while lower_bound < upper_bound {
                let midpoint = (lower_bound + upper_bound) / 2;
                if self.key_at(node, midpoint) < level_search_key { lower_bound = midpoint + 1; } else { upper_bound = midpoint; }
            }
            slot = lower_bound.min(self.slots - 1);
            if level + 1 < self.depth {
                node = self.child_at(node, slot) % self.node_count;
            }
        }
        slot
    }

    fn footprint(&self) -> usize { self.buffer.len() }
}

fn generate_lookup_keys(tree: &Tree, lookup_count: usize, seed: u64) -> Vec<u64> {
    let mut random_state = seed | 1;
    let key_span = (tree.slots * tree.node_count) as u64;
    (0..lookup_count).map(|_| {
        random_state ^= random_state >> 12; random_state ^= random_state << 25; random_state ^= random_state >> 27;
        random_state.wrapping_mul(0x2545_F491_4F6C_DD1D) % key_span
    }).collect()
}

fn bench(tree: &Tree, keys: &[u64]) -> u64 {
    let mut previous_slot = 0usize;
    let key_span = (tree.slots * tree.node_count) as u64;
    let start_time = Instant::now();
    for &key in keys {
        let chained_key = (key ^ (previous_slot as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) % key_span;
        previous_slot = tree.lookup(chained_key);
    }
    let elapsed_nanoseconds = start_time.elapsed().as_nanos() as u64;
    std::hint::black_box(previous_slot);
    elapsed_nanoseconds
}

/// 甲把子指针 59 → 75 之后，仓里几个已登记的派生数各变成什么。
/// **纯算术**，与计时无关；单测逐个钉死绝对值。
/// ⚠️ **不报「树表每层几棵」**，只报条目宽。那个数的分母仓里有两个口径：
/// `22-单元原子性怎么合成.md:495` 逐字「(16384−68−32)/67 = 16284/67 = 243」用 16284，
/// 而这里的 `NODE_BYTES - NODE_HEADER_BYTES` 是 16320。两者在 121 上都给 134、**只在 137 上差 1**
/// （118 与 119）——`.claude/rules/mutation-sampling.md` 说的第三类：
/// 在选定的取样点上恰好同值，基线那一格对上了就没人会去查另一格。
/// E128 不挑口径，把这一格留给 C157（树表容量在两处按不同条目宽算）。
/// 条目宽本身两个口径一致（纯字段表求和），所以照报。
fn derived_widths_for_child_pointer(child_pointer_bytes: usize) -> (usize, usize, usize, usize) {
    let inode_entry = 8 + 26 + child_pointer_bytes;          // 分隔 key 8 + 身份引用 26 + 子指针
    let ledger_entry = 22 + child_pointer_bytes;             // 记账 key 22 + 子指针
    let root_record = 194 - 59 * 2 + child_pointer_bytes * 2; // 树表单元指针 + 实例表单元指针各一份
    let tree_entry = 121 - 59 + child_pointer_bytes;         // D8 已定项 8 的 121 字节条目
    (inode_entry, ledger_entry, root_record, tree_entry)
}

const ARMS: [(&str, usize); 6] = [
    ("inode_bing", 93), ("inode_jia", 109),
    ("ledger_bing", 81), ("ledger_jia", 97),
    ("xdev_67", 67), ("xdev_111", 111),
];
const KEY_TIERS: [usize; 4] = [1 << 11, 1 << 15, 1 << 19, 1 << 23];

fn main() {
    let lookup_count: usize = std::env::args().nth(1).and_then(|argument| argument.parse().ok()).unwrap_or(2_000_000);
    let rounds = 5usize;
    let timed_repetitions_per_round = 3usize;
    let mut emitter = Emitter::new();
    let mut output = String::new();
    let mut say = |line: String| { output.push_str(&line); output.push('\n'); };

    say(emitter.emit_raw(&format!(
        "name=config iters={lookup_count} rounds={rounds} inner={timed_repetitions_per_round} node_bytes={NODE_BYTES} node_hdr={NODE_HEADER_BYTES}")));

    // ── 纯算术那一半：甲 / 丙 各自的派生数 ──
    for (arm, child_pointer_bytes) in [("bing", 59usize), ("jia", 75)] {
        let (inode_entry, ledger_entry, root_record, tree_entry) = derived_widths_for_child_pointer(child_pointer_bytes);
        say(emitter.emit_raw(&format!(
            "name=derived arm={arm} child_ptr={child_pointer_bytes} inode_entry={inode_entry} ledger_entry={ledger_entry} \
root_record={root_record} tree_entry={tree_entry}")));
    }

    // ── 阳性对照：每条臂各跑一遍（判据 1）──
    let mut failed_positive_control_count = 0;
    for (arm, entry_bytes) in ARMS {
        let small = Tree::new(KEY_TIERS[0], entry_bytes, NODE_BYTES);
        let big = Tree::new(KEY_TIERS[3], entry_bytes, NODE_BYTES);
        let small_keys = generate_lookup_keys(&small, lookup_count / 8, 100);
        let big_keys = generate_lookup_keys(&big, lookup_count / 8, 100);
        let small_nanoseconds = (0..timed_repetitions_per_round).map(|_| bench(&small, &small_keys)).min().unwrap();
        let big_nanoseconds = (0..timed_repetitions_per_round).map(|_| bench(&big, &big_keys)).min().unwrap();
        let ratio = big_nanoseconds as f64 / small_nanoseconds as f64;
        let passes_ratio_threshold = ratio >= 2.0;
        if !passes_ratio_threshold { failed_positive_control_count += 1; }
        say(emitter.emit_raw(&format!(
            "name=poscontrol arm={arm} entry_bytes={entry_bytes} slots={} depth_small={} depth_big={} \
ns_small={small_nanoseconds} ns_big={big_nanoseconds} ratio={ratio:.2} ok={passes_ratio_threshold}", big.slots, small.depth, big.depth)));
    }
    if failed_positive_control_count > 0 {
        say(emitter.finish()); print!("{output}");
        eprintln!("E128: {failed_positive_control_count} 条臂的 L1 档与 DRAM 档差不到 2 倍 —— 那些臂没在量缓存，本轮作废");
        std::process::exit(4);
    }

    // ── 主扫描：5 轮 × 6 臂 × 4 档工作集 ──
    let mut dram_tier_nanoseconds_per_lookup_by_arm: Vec<(String, usize, Vec<f64>)> = Vec::new();
    for (arm, entry_bytes) in ARMS {
        let mut dram_nanoseconds_per_lookup_per_round = Vec::new();
        for &key_count in &KEY_TIERS {
            let tree = Tree::new(key_count, entry_bytes, NODE_BYTES);
            let lookup_keys = generate_lookup_keys(&tree, lookup_count, 7);
            for round in 0..rounds {
                let elapsed_nanoseconds = (0..timed_repetitions_per_round).map(|_| bench(&tree, &lookup_keys)).min().unwrap();
                let nanoseconds_per_lookup = elapsed_nanoseconds as f64 / lookup_count as f64;
                if key_count == KEY_TIERS[3] { dram_nanoseconds_per_lookup_per_round.push(nanoseconds_per_lookup); }
                say(emitter.emit_raw(&format!(
                    "name=e128 arm={arm} entry_bytes={entry_bytes} keys={key_count} round={round} slots={} depth={} \
nodes={} footprint_bytes={} ns_per_lookup={nanoseconds_per_lookup:.2}",
                    tree.slots, tree.depth, tree.node_count, tree.footprint())));
            }
        }
        dram_tier_nanoseconds_per_lookup_by_arm.push((arm.to_string(), entry_bytes, dram_nanoseconds_per_lookup_per_round));
    }

    // ── 跨装置闸（判据 2）：67 与 111 必须落回 E20 产物那两个数 ──
    let mut failed_cross_apparatus_count = 0;
    for (entry_bytes, e20_reference_nanoseconds_per_lookup) in CROSS_APPARATUS_E20_REFERENCE_NANOSECONDS_PER_LOOKUP {
        let this_apparatus_arm = dram_tier_nanoseconds_per_lookup_by_arm.iter().find(|(_, arm_entry_bytes, _)| *arm_entry_bytes == entry_bytes).expect("臂缺失");
        let this_apparatus_median_nanoseconds_per_lookup = median(&this_apparatus_arm.2);
        let relative_deviation = (this_apparatus_median_nanoseconds_per_lookup - e20_reference_nanoseconds_per_lookup).abs() / e20_reference_nanoseconds_per_lookup;
        let is_within_tolerance = relative_deviation <= CROSS_APPARATUS_RELATIVE_TOLERANCE;
        if !is_within_tolerance { failed_cross_apparatus_count += 1; }
        say(emitter.emit_raw(&format!(
            "name=xdev entry_bytes={entry_bytes} e20_ref={e20_reference_nanoseconds_per_lookup:.2} e128_median={this_apparatus_median_nanoseconds_per_lookup:.2} \
deviation={relative_deviation:.4} tol={CROSS_APPARATUS_RELATIVE_TOLERANCE} ok={is_within_tolerance}")));
    }
    if failed_cross_apparatus_count > 0 {
        say(emitter.finish()); print!("{output}");
        eprintln!("E128: 跨装置闸 {failed_cross_apparatus_count} 格超差 —— 这套装置与 E20 对同一个量报不同的数，本轮作废");
        std::process::exit(5);
    }

    // ── 结论那一格：逐轮比，5 轮同向才下结论（判据 4）──
    for (bing_arm_label, jia_arm_label, pair_label) in [("inode_bing", "inode_jia", "inode"), ("ledger_bing", "ledger_jia", "ledger")] {
        let bing_dram_per_round = &dram_tier_nanoseconds_per_lookup_by_arm.iter().find(|(arm_label, _, _)| arm_label == bing_arm_label).unwrap().2;
        let jia_dram_per_round = &dram_tier_nanoseconds_per_lookup_by_arm.iter().find(|(arm_label, _, _)| arm_label == jia_arm_label).unwrap().2;
        let rounds_jia_slower = (0..rounds).filter(|&round| jia_dram_per_round[round] > bing_dram_per_round[round]).count();
        let jia_to_bing_ratio_per_round: Vec<f64> = (0..rounds).map(|round| jia_dram_per_round[round] / bing_dram_per_round[round]).collect();
        let verdict = if rounds_jia_slower == rounds { "jia_slower_every_round" }
            else if rounds_jia_slower == 0 { "jia_never_slower" } else { "unstable" };
        say(emitter.emit_raw(&format!(
            "name=verdict pair={pair_label} bing_median={:.2} jia_median={:.2} ratio_median={:.4} \
ratio_min={:.4} ratio_max={:.4} rounds_jia_slower={rounds_jia_slower}/{rounds} verdict={verdict}",
            median(bing_dram_per_round), median(jia_dram_per_round), median(&jia_to_bing_ratio_per_round),
            jia_to_bing_ratio_per_round.iter().cloned().fold(f64::INFINITY, f64::min),
            jia_to_bing_ratio_per_round.iter().cloned().fold(f64::NEG_INFINITY, f64::max))));
    }

    say(emitter.finish());
    print!("{output}");
}

fn median(values: &[f64]) -> f64 {
    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|left, right| left.partial_cmp(right).unwrap());
    sorted_values[sorted_values.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言（判据 3）**：槽数逐档钉死。
    /// 只让六条臂互相比，测不出「六条一起错」——比如 NODE_HEADER_BYTES 抄错。
    #[test]
    fn slots_are_pinned_per_width() {
        for (entry_bytes, expected_slots) in [(93usize, 175usize), (109, 149), (81, 201), (97, 168), (67, 243), (111, 147)] {
            let tree = Tree::new(1 << 20, entry_bytes, NODE_BYTES);
            assert_eq!(tree.slots, expected_slots, "entry_bytes={entry_bytes}");
        }
    }

    /// 本工程两处已登记的扇出必须由同一个 NODE_HEADER_BYTES 复现出来：
    /// D8 已定项 6 的 inode 树 175、D18 的记账树 201。对不上说明节点头常量抄错了。
    #[test]
    fn published_fanouts_come_out_of_the_same_header_constant() {
        assert_eq!((NODE_BYTES - NODE_HEADER_BYTES) / 93, 175);
        assert_eq!((NODE_BYTES - NODE_HEADER_BYTES) / 81, 201);
    }

    /// **绝对值断言（判据 3）**：2²³ key 上的树深逐档钉死。
    #[test]
    fn depth_at_dram_tier_is_pinned() {
        for (entry_bytes, expected_depth) in [(93usize, 4usize), (109, 4), (81, 4), (97, 4), (67, 3), (111, 4)] {
            let tree = Tree::new(1 << 23, entry_bytes, NODE_BYTES);
            assert_eq!(tree.depth, expected_depth, "entry_bytes={entry_bytes}");
        }
    }

    /// 93 与 109 在 DRAM 档上**同深**——所以两者之差是纯缓存效应，不掺树深跳变。
    /// 这条一旦不成立，整个「甲多付多少」的读法要改。
    #[test]
    fn inode_pair_is_compared_at_equal_depth() {
        let bing_inode_tree = Tree::new(1 << 23, 93, NODE_BYTES);
        let jia_inode_tree = Tree::new(1 << 23, 109, NODE_BYTES);
        assert_eq!(bing_inode_tree.depth, jia_inode_tree.depth, "两条臂树深不同，差值里掺了跳深");
    }

    /// **绝对值断言**：派生数逐个钉死，不靠「甲比丙大」这种互比。
    /// 写成加法不写减法：常量变异改大时减法会编译期溢出，被记成无效变异。
    #[test]
    fn derived_widths_are_pinned() {
        let (bing_inode_entry, bing_ledger_entry, bing_root_record, bing_tree_entry) = derived_widths_for_child_pointer(59);
        assert_eq!(bing_inode_entry, 93);
        assert_eq!(bing_ledger_entry, 81);
        assert_eq!(bing_root_record, 194);
        assert_eq!(bing_tree_entry, 121);
        let (jia_inode_entry, jia_ledger_entry, jia_root_record, jia_tree_entry) = derived_widths_for_child_pointer(75);
        assert_eq!(jia_inode_entry, 109);
        assert_eq!(jia_ledger_entry, 97);
        assert_eq!(jia_root_record, 226);
        assert_eq!(jia_tree_entry, 137);
    }

    /// 甲多出来的那 16 字节要正好落在每一条含子指针的条目上，一处不多一处不少。
    #[test]
    fn jia_adds_exactly_sixteen_per_child_pointer() {
        let (bing_inode_entry, bing_ledger_entry, bing_root_record, bing_tree_entry) = derived_widths_for_child_pointer(59);
        let (jia_inode_entry, jia_ledger_entry, jia_root_record, jia_tree_entry) = derived_widths_for_child_pointer(75);
        assert_eq!(jia_inode_entry, bing_inode_entry + 16);
        assert_eq!(jia_ledger_entry, bing_ledger_entry + 16);
        assert_eq!(jia_root_record, bing_root_record + 32); // 根记录里有两个单元指针
        assert_eq!(jia_tree_entry, bing_tree_entry + 16);
    }

    /// key 必须按 entry_bytes 跨步写——读错位置会取到 0。
    #[test]
    fn keys_are_laid_out_at_stride() {
        let tree = Tree::new(1 << 12, 93, NODE_BYTES);
        for slot_index in 0..tree.slots { assert_eq!(tree.key_at(3, slot_index), slot_index as u64); }
    }

    /// 查找必须散开：全落一处说明布局或搜索键取法错了。
    #[test]
    fn lookups_spread_across_nodes() {
        let tree = Tree::new(1 << 20, 93, NODE_BYTES);
        let mut distinct_slots = std::collections::BTreeSet::new();
        let mut random_state = 12345u64;
        for _ in 0..2000 {
            random_state ^= random_state >> 12; random_state ^= random_state << 25; random_state ^= random_state >> 27;
            distinct_slots.insert(tree.lookup(random_state.wrapping_mul(0x2545_F491_4F6C_DD1D)));
        }
        assert!(distinct_slots.len() > 50, "2000 次查找只落到 {} 个不同 slot 上", distinct_slots.len());
    }

    /// 节点内二分必须命中恰好等于搜索键的那个槽。
    #[test]
    fn binary_search_lands_exactly_on_the_search_key() {
        let tree = Tree::new(100, 93, NODE_BYTES);
        assert_eq!(tree.depth, 1);
        for key in 0..tree.slots as u64 {
            assert_eq!(tree.lookup(key), (key as usize) % tree.slots, "key={key}");
        }
    }

    /// 子指针不许与 key 重叠——重叠了指针追逐读的是 key，整条依赖链是假的。
    #[test]
    fn child_pointers_live_beside_keys_not_on_top_of_them() {
        let tree = Tree::new(1 << 16, 93, NODE_BYTES);
        assert!((0..tree.slots).any(|slot_index| tree.child_at(5, slot_index) as u64 != tree.key_at(5, slot_index)),
            "每个条目的子指针都等于它的 key——child_at 读到了 key 的位置");
    }

    /// 条目变宽必须让扇出降、工作集涨。不变说明没按 entry_bytes 跨步。
    #[test]
    fn wider_entries_shrink_fanout_and_grow_footprint() {
        let bing_inode_tree = Tree::new(1 << 16, 93, NODE_BYTES);
        let jia_inode_tree = Tree::new(1 << 16, 109, NODE_BYTES);
        assert!(jia_inode_tree.slots < bing_inode_tree.slots, "扇出没降：{} -> {}", bing_inode_tree.slots, jia_inode_tree.slots);
        assert!(jia_inode_tree.footprint() > bing_inode_tree.footprint(), "工作集没涨");
    }

    /// 跨装置闸的两个基准是常量，抄错了这条会红。
    #[test]
    fn cross_apparatus_reference_matches_the_stored_e20_artifact() {
        assert_eq!(CROSS_APPARATUS_E20_REFERENCE_NANOSECONDS_PER_LOOKUP[0].0, 67);
        assert_eq!(CROSS_APPARATUS_E20_REFERENCE_NANOSECONDS_PER_LOOKUP[1].0, 111);
        assert!((CROSS_APPARATUS_E20_REFERENCE_NANOSECONDS_PER_LOOKUP[0].1 - 466.99).abs() < 1e-9);
        assert!((CROSS_APPARATUS_E20_REFERENCE_NANOSECONDS_PER_LOOKUP[1].1 - 547.50).abs() < 1e-9);
    }

    /// 六条臂就是本工程真实的条目宽，改一条就等于换了被测对象。
    /// 没有这一条，把 `inode_jia` 从 109 改成 93 一个测试都不会红。
    #[test]
    fn arms_are_the_projects_real_entry_widths() {
        assert_eq!(ARMS[0], ("inode_bing", 93));
        assert_eq!(ARMS[1], ("inode_jia", 109));
        assert_eq!(ARMS[2], ("ledger_bing", 81));
        assert_eq!(ARMS[3], ("ledger_jia", 97));
        assert_eq!(ARMS[4], ("xdev_67", 67));
        assert_eq!(ARMS[5], ("xdev_111", 111));
        assert_eq!(KEY_TIERS, [2048, 32768, 524288, 8388608]);
    }

    /// 两个口径在 121 上同值、在 137 上差 1——这一条把那件事钉住，
    /// 免得后来的人以为「树表每层几棵」在仓里只有一个数。
    #[test]
    fn the_two_tree_table_denominators_agree_at_121_and_disagree_at_137() {
        let unit_atomicity_decision_tree_table_denominator = 16384 - 68 - 32;      // 22-单元原子性怎么合成.md:495 逐字
        let this_experiment_tree_table_denominator = NODE_BYTES - NODE_HEADER_BYTES;
        assert_eq!(unit_atomicity_decision_tree_table_denominator, 16284);
        assert_eq!(this_experiment_tree_table_denominator, 16320);
        assert_eq!(unit_atomicity_decision_tree_table_denominator / 121, this_experiment_tree_table_denominator / 121);            // 丙那一臂：两个口径同值
        assert_eq!(unit_atomicity_decision_tree_table_denominator / 137 + 1, this_experiment_tree_table_denominator / 137);        // 甲那一臂：差 1
        assert_eq!(unit_atomicity_decision_tree_table_denominator / 137, 118);
        assert_eq!(this_experiment_tree_table_denominator / 137, 119);
    }

    /// median 取中位不取均值——一次异常慢的轮次不许把结论拉走。
    #[test]
    fn median_picks_the_middle_not_the_mean() {
        assert!((median(&[1.0, 2.0, 100.0]) - 2.0).abs() < 1e-9);
    }
}
