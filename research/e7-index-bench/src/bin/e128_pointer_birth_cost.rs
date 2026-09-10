//! E128：指针带出生 (树, txg) 之后，本工程**真实条目宽**的点查代价。
//!
//! D19 未定项 7 的候选甲把指针头部 31 → 47，连带把 inode 树内部条目 93 → 109、
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
//! `Tree` / `gen_keys` / `bench` 三段与 `e20_fanout.rs` 同源。没有并成一份的理由是：
//! E20 的变异表按**原文匹配**改它自己那个文件，移走那三段会让 E20 已入库的
//! 5 条变异证明当场失配。⇒ 代价是手抄，兜底是**跨装置闸**：
//! 67 与 111 两档必须落回 E20 产物里那两个数（`show-me-test.md`「立在装置之间」）。
//! ⚠️ 闸只钉 DRAM 档那两个点，抄漏在别的档上不会被它抓住。

use e7_index_bench::Emitter;
use std::time::Instant;

const NODE_HDR: usize = 64;
const NODE_BYTES: usize = 16384; // D8 已定项 2 钉死的常量，本实验不扫节点档

/// 跨装置闸的两个基准，逐字取自 `research/results/e20-sweep-2026-08-29.out`
/// 的 `node_bytes=16384 keys=8388608` 两行。
const XDEV_REF: [(usize, f64); 2] = [(67, 466.99), (111, 547.50)];
const XDEV_TOL: f64 = 0.15;

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
        for node in 0..total {
            for i in 0..slots {
                let k = i as u64;
                let mix = (node as u64) * (slots as u64) + i as u64;
                let off = node * t.node_bytes + NODE_HDR + i * t.entry_bytes;
                t.buf[off..off + 8].copy_from_slice(&k.to_le_bytes());
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

    #[inline(never)]
    fn lookup(&self, key: u64) -> usize {
        let mut node = 0usize;
        let mut slot = 0usize;
        let bits = (self.slots as f64).log2().ceil() as u32;
        for lvl in 0..self.depth {
            let sk = (key >> (lvl as u32 * bits)) % (self.slots as u64);
            let (mut lo, mut hi) = (0usize, self.slots);
            while lo < hi {
                let mid = (lo + hi) / 2;
                if self.key_at(node, mid) < sk { lo = mid + 1; } else { hi = mid; }
            }
            slot = lo.min(self.slots - 1);
            if lvl + 1 < self.depth {
                node = self.child_at(node, slot) % self.n_nodes;
            }
        }
        slot
    }

    fn footprint(&self) -> usize { self.buf.len() }
}

fn gen_keys(t: &Tree, iters: usize, seed: u64) -> Vec<u64> {
    let mut s = seed | 1;
    let span = (t.slots * t.n_nodes) as u64;
    (0..iters).map(|_| {
        s ^= s >> 12; s ^= s << 25; s ^= s >> 27;
        s.wrapping_mul(0x2545_F491_4F6C_DD1D) % span
    }).collect()
}

fn bench(t: &Tree, keys: &[u64]) -> u64 {
    let mut prev = 0usize;
    let span = (t.slots * t.n_nodes) as u64;
    let t0 = Instant::now();
    for &k in keys {
        let kk = (k ^ (prev as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) % span;
        prev = t.lookup(kk);
    }
    let ns = t0.elapsed().as_nanos() as u64;
    std::hint::black_box(prev);
    ns
}

/// 甲把子指针 59 → 75 之后，仓里几个已登记的派生数各变成什么。
/// **纯算术**，与计时无关；单测逐个钉死绝对值。
/// ⚠️ **不报「树表每层几棵」**，只报条目宽。那个数的分母仓里有两个口径：
/// `22-单元原子性怎么合成.md:495` 逐字「(16384−68−32)/67 = 16284/67 = 243」用 16284，
/// 而这里的 `NODE_BYTES - NODE_HDR` 是 16320。两者在 121 上都给 134、**只在 137 上差 1**
/// （118 与 119）——`.claude/rules/mutation-sampling.md` 说的第三类：
/// 在选定的取样点上恰好同值，基线那一格对上了就没人会去查另一格。
/// E128 不挑口径，把这一格留给 C157（树表容量在两处按不同条目宽算）。
/// 条目宽本身两个口径一致（纯字段表求和），所以照报。
fn derived(child_ptr: usize) -> (usize, usize, usize, usize) {
    let inode_entry = 8 + 26 + child_ptr;          // 分隔 key 8 + 身份引用 26 + 子指针
    let ledger_entry = 22 + child_ptr;             // 记账 key 22 + 子指针
    let root_record = 194 - 59 * 2 + child_ptr * 2; // 树表单元指针 + 实例表单元指针各一份
    let tree_entry = 121 - 59 + child_ptr;         // D8 已定项 8 的 121 字节条目
    (inode_entry, ledger_entry, root_record, tree_entry)
}

const ARMS: [(&str, usize); 6] = [
    ("inode_bing", 93), ("inode_jia", 109),
    ("ledger_bing", 81), ("ledger_jia", 97),
    ("xdev_67", 67), ("xdev_111", 111),
];
const KEY_TIERS: [usize; 4] = [1 << 11, 1 << 15, 1 << 19, 1 << 23];

fn main() {
    let iters: usize = std::env::args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(2_000_000);
    let rounds = 5usize;
    let inner = 3usize;
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| { out.push_str(&s); out.push('\n'); };

    say(em.emit_raw(&format!(
        "name=config iters={iters} rounds={rounds} inner={inner} node_bytes={NODE_BYTES} node_hdr={NODE_HDR}")));

    // ── 纯算术那一半：甲 / 丙 各自的派生数 ──
    for (arm, ptr) in [("bing", 59usize), ("jia", 75)] {
        let (ie, le, rr, te) = derived(ptr);
        say(em.emit_raw(&format!(
            "name=derived arm={arm} child_ptr={ptr} inode_entry={ie} ledger_entry={le} \
root_record={rr} tree_entry={te}")));
    }

    // ── 阳性对照：每条臂各跑一遍（判据 1）──
    let mut bad = 0;
    for (arm, w) in ARMS {
        let small = Tree::new(KEY_TIERS[0], w, NODE_BYTES);
        let big = Tree::new(KEY_TIERS[3], w, NODE_BYTES);
        let ks = gen_keys(&small, iters / 8, 100);
        let kb = gen_keys(&big, iters / 8, 100);
        let ns_s = (0..inner).map(|_| bench(&small, &ks)).min().unwrap();
        let ns_b = (0..inner).map(|_| bench(&big, &kb)).min().unwrap();
        let ratio = ns_b as f64 / ns_s as f64;
        let ok = ratio >= 2.0;
        if !ok { bad += 1; }
        say(em.emit_raw(&format!(
            "name=poscontrol arm={arm} entry_bytes={w} slots={} depth_small={} depth_big={} \
ns_small={ns_s} ns_big={ns_b} ratio={ratio:.2} ok={ok}", big.slots, small.depth, big.depth)));
    }
    if bad > 0 {
        say(em.finish()); print!("{out}");
        eprintln!("E128: {bad} 条臂的 L1 档与 DRAM 档差不到 2 倍 —— 那些臂没在量缓存，本轮作废");
        std::process::exit(4);
    }

    // ── 主扫描：5 轮 × 6 臂 × 4 档工作集 ──
    let mut dram: Vec<(String, usize, Vec<f64>)> = Vec::new();
    for (arm, w) in ARMS {
        let mut per_round_dram = Vec::new();
        for &nk in &KEY_TIERS {
            let t = Tree::new(nk, w, NODE_BYTES);
            let ks = gen_keys(&t, iters, 7);
            for r in 0..rounds {
                let ns = (0..inner).map(|_| bench(&t, &ks)).min().unwrap();
                let per = ns as f64 / iters as f64;
                if nk == KEY_TIERS[3] { per_round_dram.push(per); }
                say(em.emit_raw(&format!(
                    "name=e128 arm={arm} entry_bytes={w} keys={nk} round={r} slots={} depth={} \
nodes={} footprint_bytes={} ns_per_lookup={per:.2}",
                    t.slots, t.depth, t.n_nodes, t.footprint())));
            }
        }
        dram.push((arm.to_string(), w, per_round_dram));
    }

    // ── 跨装置闸（判据 2）：67 与 111 必须落回 E20 产物那两个数 ──
    let mut xbad = 0;
    for (w, refv) in XDEV_REF {
        let mine = dram.iter().find(|(_, ww, _)| *ww == w).expect("臂缺失");
        let med = median(&mine.2);
        let dev = (med - refv).abs() / refv;
        let ok = dev <= XDEV_TOL;
        if !ok { xbad += 1; }
        say(em.emit_raw(&format!(
            "name=xdev entry_bytes={w} e20_ref={refv:.2} e128_median={med:.2} \
deviation={dev:.4} tol={XDEV_TOL} ok={ok}")));
    }
    if xbad > 0 {
        say(em.finish()); print!("{out}");
        eprintln!("E128: 跨装置闸 {xbad} 格超差 —— 这套装置与 E20 对同一个量报不同的数，本轮作废");
        std::process::exit(5);
    }

    // ── 结论那一格：逐轮比，5 轮同向才下结论（判据 4）──
    for (a, b, label) in [("inode_bing", "inode_jia", "inode"), ("ledger_bing", "ledger_jia", "ledger")] {
        let x = &dram.iter().find(|(n, _, _)| n == a).unwrap().2;
        let y = &dram.iter().find(|(n, _, _)| n == b).unwrap().2;
        let slower = (0..rounds).filter(|&i| y[i] > x[i]).count();
        let ratios: Vec<f64> = (0..rounds).map(|i| y[i] / x[i]).collect();
        let verdict = if slower == rounds { "jia_slower_every_round" }
            else if slower == 0 { "jia_never_slower" } else { "unstable" };
        say(em.emit_raw(&format!(
            "name=verdict pair={label} bing_median={:.2} jia_median={:.2} ratio_median={:.4} \
ratio_min={:.4} ratio_max={:.4} rounds_jia_slower={slower}/{rounds} verdict={verdict}",
            median(x), median(y), median(&ratios),
            ratios.iter().cloned().fold(f64::INFINITY, f64::min),
            ratios.iter().cloned().fold(f64::NEG_INFINITY, f64::max))));
    }

    say(em.finish());
    print!("{out}");
}

fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[s.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值断言（判据 3）**：槽数逐档钉死。
    /// 只让六条臂互相比，测不出「六条一起错」——比如 NODE_HDR 抄错。
    #[test]
    fn slots_are_pinned_per_width() {
        for (w, want) in [(93usize, 175usize), (109, 149), (81, 201), (97, 168), (67, 243), (111, 147)] {
            let t = Tree::new(1 << 20, w, NODE_BYTES);
            assert_eq!(t.slots, want, "entry_bytes={w}");
        }
    }

    /// 本工程两处已登记的扇出必须由同一个 NODE_HDR 复现出来：
    /// D8 已定项 6 的 inode 树 175、D18 的记账树 201。对不上说明节点头常量抄错了。
    #[test]
    fn published_fanouts_come_out_of_the_same_header_constant() {
        assert_eq!((NODE_BYTES - NODE_HDR) / 93, 175);
        assert_eq!((NODE_BYTES - NODE_HDR) / 81, 201);
    }

    /// **绝对值断言（判据 3）**：2²³ key 上的树深逐档钉死。
    #[test]
    fn depth_at_dram_tier_is_pinned() {
        for (w, want) in [(93usize, 4usize), (109, 4), (81, 4), (97, 4), (67, 3), (111, 4)] {
            let t = Tree::new(1 << 23, w, NODE_BYTES);
            assert_eq!(t.depth, want, "entry_bytes={w}");
        }
    }

    /// 93 与 109 在 DRAM 档上**同深**——所以两者之差是纯缓存效应，不掺树深跳变。
    /// 这条一旦不成立，整个「甲多付多少」的读法要改。
    #[test]
    fn inode_pair_is_compared_at_equal_depth() {
        let a = Tree::new(1 << 23, 93, NODE_BYTES);
        let b = Tree::new(1 << 23, 109, NODE_BYTES);
        assert_eq!(a.depth, b.depth, "两条臂树深不同，差值里掺了跳深");
    }

    /// **绝对值断言**：派生数逐个钉死，不靠「甲比丙大」这种互比。
    /// 写成加法不写减法：常量变异改大时减法会编译期溢出，被记成无效变异。
    #[test]
    fn derived_widths_are_pinned() {
        let (ie, le, rr, te) = derived(59);
        assert_eq!(ie, 93);
        assert_eq!(le, 81);
        assert_eq!(rr, 194);
        assert_eq!(te, 121);
        let (ie2, le2, rr2, te2) = derived(75);
        assert_eq!(ie2, 109);
        assert_eq!(le2, 97);
        assert_eq!(rr2, 226);
        assert_eq!(te2, 137);
    }

    /// 甲多出来的那 16 字节要正好落在每一条含子指针的条目上，一处不多一处不少。
    #[test]
    fn jia_adds_exactly_sixteen_per_child_pointer() {
        let (ie, le, rr, te) = derived(59);
        let (ie2, le2, rr2, te2) = derived(75);
        assert_eq!(ie2, ie + 16);
        assert_eq!(le2, le + 16);
        assert_eq!(rr2, rr + 32); // 根记录里有两个单元指针
        assert_eq!(te2, te + 16);
    }

    /// key 必须按 entry_bytes 跨步写——读错位置会取到 0。
    #[test]
    fn keys_are_laid_out_at_stride() {
        let t = Tree::new(1 << 12, 93, NODE_BYTES);
        for i in 0..t.slots { assert_eq!(t.key_at(3, i), i as u64); }
    }

    /// 查找必须散开：全落一处说明布局或搜索键取法错了。
    #[test]
    fn lookups_spread_across_nodes() {
        let t = Tree::new(1 << 20, 93, NODE_BYTES);
        let mut seen = std::collections::BTreeSet::new();
        let mut s = 12345u64;
        for _ in 0..2000 {
            s ^= s >> 12; s ^= s << 25; s ^= s >> 27;
            seen.insert(t.lookup(s.wrapping_mul(0x2545_F491_4F6C_DD1D)));
        }
        assert!(seen.len() > 50, "2000 次查找只落到 {} 个不同 slot 上", seen.len());
    }

    /// 节点内二分必须命中恰好等于搜索键的那个槽。
    #[test]
    fn binary_search_lands_exactly_on_the_search_key() {
        let t = Tree::new(100, 93, NODE_BYTES);
        assert_eq!(t.depth, 1);
        for k in 0..t.slots as u64 {
            assert_eq!(t.lookup(k), (k as usize) % t.slots, "key={k}");
        }
    }

    /// 子指针不许与 key 重叠——重叠了指针追逐读的是 key，整条依赖链是假的。
    #[test]
    fn child_pointers_live_beside_keys_not_on_top_of_them() {
        let t = Tree::new(1 << 16, 93, NODE_BYTES);
        assert!((0..t.slots).any(|i| t.child_at(5, i) as u64 != t.key_at(5, i)),
            "每个条目的子指针都等于它的 key——child_at 读到了 key 的位置");
    }

    /// 条目变宽必须让扇出降、工作集涨。不变说明没按 entry_bytes 跨步。
    #[test]
    fn wider_entries_shrink_fanout_and_grow_footprint() {
        let a = Tree::new(1 << 16, 93, NODE_BYTES);
        let b = Tree::new(1 << 16, 109, NODE_BYTES);
        assert!(b.slots < a.slots, "扇出没降：{} -> {}", a.slots, b.slots);
        assert!(b.footprint() > a.footprint(), "工作集没涨");
    }

    /// 跨装置闸的两个基准是常量，抄错了这条会红。
    #[test]
    fn cross_device_reference_matches_the_stored_e20_artifact() {
        assert_eq!(XDEV_REF[0].0, 67);
        assert_eq!(XDEV_REF[1].0, 111);
        assert!((XDEV_REF[0].1 - 466.99).abs() < 1e-9);
        assert!((XDEV_REF[1].1 - 547.50).abs() < 1e-9);
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
        let repo = 16384 - 68 - 32;      // 22-单元原子性怎么合成.md:495 逐字
        let here = NODE_BYTES - NODE_HDR;
        assert_eq!(repo, 16284);
        assert_eq!(here, 16320);
        assert_eq!(repo / 121, here / 121);            // 丙那一臂：两个口径同值
        assert_eq!(repo / 137 + 1, here / 137);        // 甲那一臂：差 1
        assert_eq!(repo / 137, 118);
        assert_eq!(here / 137, 119);
    }

    /// median 取中位不取均值——一次异常慢的轮次不许把结论拉走。
    #[test]
    fn median_picks_the_middle_not_the_mean() {
        assert!((median(&[1.0, 2.0, 100.0]) - 2.0).abs() < 1e-9);
    }
}
