//! E89：嵌套区间标号 + 删除前沿 —— C35（多可写头的 O(N) 没有优化过）点名的建模验证。
//!
//! **问题**：D6 取多可写头后，删快照要 O(N) 次反向索引查找
//! （E26 实测 8192 个 extent 时 213 次，log-log 斜率 1.32）。
//! D6 正文挂着的待验优化：把「子树里还有没有活快照」从遍历变成一次区间包含判定，
//! 再配一个删除前沿（每 extent 记它在哪几个分支上被删了的极小反链）。
//!
//! **前置「树还是 DAG」已由 D6 已定项 1 解掉**：每头一棵自己的树、克隆自单一 origin，
//! 全部已定决策里没有合并两个头的操作 ⇒ 谱系是树，preorder 区间标号适用。
//!
//! ## 两条臂 + 真值
//!
//! | 臂 | 机制 |
//! |---|---|
//! | `paylookup` | D6 现行乙（E26 的 deadlist 臂同构）：旁支引用退回反向查找，每次 +1 |
//! | `frontier`  | 每 extent 存（诞生节点，删除前沿）；活快照集按 preorder 标号做区间计数 |
//!
//! **真值 = 枚举全部活快照，被任一活快照引用即活**（与两臂零共享代码）。
//!
//! ## 计费口径（跑前写死）
//!
//! 「查找」= 反向索引查询（规模随 extent 总数长，真盘上一次 btree 下降）。
//! 查快照表不算查找（规模 = 快照数，常驻内存，与 E26 的 deadlist 臂免费读快照
//! txg / 存活位同权），但候选臂的快照表探查次数单独报出，不藏进免费项。
//! 「被删快照引用了哪些 extent」的枚举两臂都不计费——D6 已定项 1 每头一棵树，
//! 删头枚举的是自己那棵树。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Topo { Linear, MultiHead }

#[derive(Clone)]
struct Snap {
    id: u64,
    parent: Option<u64>,
    txg: u64,
    live: bool,
    /// preorder 标号；子树(u) = { v : pre(u) ≤ pre(v) ≤ post(u) }
    pre: u64,
    /// 子树里最大的 preorder 标号
    post: u64,
}

#[derive(Clone)]
struct Extent {
    id: u64,
    /// 创建它的快照节点（E26 的 birth_txg 同义：本模型里 txg = 快照 id）
    birth: u64,
    /// 删除前沿：继承没发生的那一步就地记下的节点（记账是事务的副产品，无事后扫描）。
    /// 结构上是反链——前沿节点不引用它，引用只沿引用链向下传。
    frontier: Vec<u64>,
    /// 引用它的快照集合。**真值靠它算；候选臂的函数签名里拿不到它**（见 CandExtent）。
    refs: BTreeSet<u64>,
}

/// 候选臂看得到的那一份：**没有 refs 字段**。
/// 想抄真值答案就编译不过（machine-first：用类型让非法状态无法表示）。
struct CandExtent {
    id: u64,
    birth: u64,
    frontier: Vec<u64>,
}

struct World { snaps: BTreeMap<u64, Snap>, exts: BTreeMap<u64, Extent> }

fn xorshift(s: &mut u64) -> u64 {
    *s ^= *s >> 12; *s ^= *s << 25; *s ^= *s >> 27;
    s.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// 与 E26 的一处**故意**差别：拓扑与共享各用一条独立 RNG 流（同种子派生），
/// 让树形状在规模扫描里保持不变——判据 2 要在固定的树上扫 extent 数。
fn build(topo: Topo, n_snaps: u64, exts_per_snap: u64, share_pct: u64, seed: u64) -> World {
    let mut topo_s = (seed | 1) ^ 0x9E37_79B9_7F4A_7C15;
    let mut share_s = (seed | 1).wrapping_mul(0xBF58_476D_1CE4_E5B9) | 1;
    let mut snaps = BTreeMap::new();
    let mut exts: BTreeMap<u64, Extent> = BTreeMap::new();
    let mut next_ext = 0u64;
    for i in 0..n_snaps {
        let parent = if i == 0 { None } else {
            match topo {
                Topo::Linear => Some(i - 1),
                Topo::MultiHead => Some(xorshift(&mut topo_s) % i),
            }
        };
        for _ in 0..exts_per_snap {
            let id = next_ext; next_ext += 1;
            let mut refs = BTreeSet::new(); refs.insert(i);
            exts.insert(id, Extent { id, birth: i, frontier: Vec::new(), refs });
        }
        // 共享：本快照按比例继承父亲引用的 extent；**没继承的那一步就地记进前沿**
        if let Some(p) = parent {
            let inherited: Vec<u64> = exts.values()
                .filter(|e| e.refs.contains(&p)).map(|e| e.id).collect();
            for id in inherited {
                let e = exts.get_mut(&id).unwrap();
                if xorshift(&mut share_s) % 100 < share_pct {
                    e.refs.insert(i);
                } else {
                    e.frontier.push(i);
                }
            }
        }
        snaps.insert(i, Snap { id: i, parent, txg: i, live: true, pre: 0, post: 0 });
    }
    label(&mut snaps);
    World { snaps, exts }
}

/// 建完树一次性编 preorder 区间。标号维护（树增长时的 order-maintenance）不在射程内，
/// 见 kb 正文「口径与已知局限」。
fn label(snaps: &mut BTreeMap<u64, Snap>) {
    let mut children: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut root = None;
    for s in snaps.values() {
        match s.parent {
            Some(p) => children.entry(p).or_default().push(s.id),
            None => root = Some(s.id),
        }
    }
    let root = root.expect("树必须有根");
    // 迭代 DFS 发 pre；post(u) = pre(u) + 子树大小 − 1，子树大小按访问序自底向上累加
    let mut next_pre = 0u64;
    let mut stack = vec![root];
    let mut order: Vec<u64> = Vec::new();
    while let Some(u) = stack.pop() {
        let s = snaps.get_mut(&u).unwrap();
        s.pre = next_pre; next_pre += 1;
        order.push(u);
        if let Some(cs) = children.get(&u) {
            // 倒序进栈 ⇒ 按 id 升序访问，确定性
            for &c in cs.iter().rev() { stack.push(c); }
        }
    }
    let mut size: BTreeMap<u64, u64> = snaps.keys().map(|&k| (k, 1u64)).collect();
    for &u in order.iter().rev() {
        if let Some(p) = snaps[&u].parent {
            let su = size[&u];
            *size.get_mut(&p).unwrap() += su;
        }
    }
    for (&u, s) in snaps.iter_mut() {
        s.post = s.pre + size[&u] - 1;
    }
}

fn in_subtree(snaps: &BTreeMap<u64, Snap>, anc: u64, v: u64) -> bool {
    let a = &snaps[&anc];
    let pv = snaps[&v].pre;
    a.pre <= pv && pv <= a.post
}

/// 与 `id` 可比的快照集合 = 祖先 ∪ 后代（含自己）。E26 同构。
fn comparable(w: &World, id: u64) -> BTreeSet<u64> {
    let mut anc = BTreeSet::new();
    let mut cur = Some(id);
    while let Some(c) = cur {
        if !anc.insert(c) { break; }
        cur = w.snaps.get(&c).and_then(|s| s.parent);
    }
    let mut out = anc.clone();
    for s in w.snaps.values() {
        let mut c = Some(s.id);
        while let Some(x) = c {
            if x == id { out.insert(s.id); break; }
            c = w.snaps.get(&x).and_then(|y| y.parent);
        }
    }
    out
}

/// 真值：枚举全部活快照。**不走任何臂的代码。**
fn truly_free_set(w: &World) -> BTreeSet<u64> {
    let live: BTreeSet<u64> = w.snaps.values().filter(|s| s.live).map(|s| s.id).collect();
    w.exts.values()
        .filter(|e| e.refs.iter().all(|r| !live.contains(r)))
        .map(|e| e.id).collect()
}

/// 基线臂（付查找）：E26 的 deadlist 臂同构。定义句原样贴
/// （verify-before-claiming：引用决策去推导之前先贴定义）：
///   「块 b 被快照 S 引用 ⟺ birth(b) ≤ S.txg < death(b)，左闭右开」
///   「death(b) = 最后一个活引用被摘掉且被发布的 checkpoint 号；仍在活树里则为 ∞」
fn paylookup_arm(w: &World, victim: u64) -> (BTreeSet<u64>, u64) {
    let mut lookups = 0u64;
    let v = &w.snaps[&victim];
    let chain = comparable(w, victim);
    let mut out = BTreeSet::new();
    for e in w.exts.values() {
        if !e.refs.contains(&victim) { continue; }
        let has_sibling_ref = e.refs.iter().any(|r| *r != victim && !chain.contains(r));
        let death_finite = if has_sibling_ref {
            lookups += 1;                            // O(1) 性质在这里破掉
            !e.refs.iter().any(|&r| r != victim
                && w.snaps.get(&r).map(|x| x.live).unwrap_or(false))
        } else {
            !e.refs.iter().any(|&r| r != victim
                && w.snaps.get(&r).map(|x| x.live).unwrap_or(false))
        };
        if !death_finite { continue; }
        let death = v.txg + 1;
        if !w.snaps.values().any(|snap| snap.live && snap.id != victim
            && e.birth <= snap.txg && snap.txg < death) {
            out.insert(e.id);
        }
    }
    (out, lookups)
}

/// 候选臂的观测量。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct CandCost {
    /// 反向索引查找——判据 2 要求恒为 0
    lookups: u64,
    /// 快照表探查次数 = Σ（1 + 前沿大小），只对被删快照可见的 extent 计
    snap_checks: u64,
    /// 被检查的 extent 里最大的前沿
    max_frontier: u64,
}

/// 候选臂：只拿得到（诞生节点，前沿）与快照树标号——**refs 不在入参里**。
/// 「S 还看得见 e 吗」与「还剩几个活引用者」都是区间算术：
///   剩余活引用 = 诞生子树内活快照数 − Σ 前沿子树内活快照数 − 1（victim 自己）。
/// 前沿子树互不相交且都在诞生子树内（反链）⇒ 容斥恰好一层。
fn frontier_arm(
    snaps: &BTreeMap<u64, Snap>,
    live_pre: &BTreeSet<u64>,
    exts: &[CandExtent],
    victim: u64,
) -> (BTreeSet<u64>, CandCost) {
    let mut cost = CandCost::default();
    let mut out = BTreeSet::new();
    for e in exts {
        let victim_sees = in_subtree(snaps, e.birth, victim)
            && !e.frontier.iter().any(|&d| in_subtree(snaps, d, victim));
        if !victim_sees { continue; }
        cost.snap_checks += 1 + e.frontier.len() as u64;
        cost.max_frontier = cost.max_frontier.max(e.frontier.len() as u64);
        let b = &snaps[&e.birth];
        let total = live_pre.range(b.pre..=b.post).count() as u64;
        let shadowed: u64 = e.frontier.iter().map(|&d| {
            let ds = &snaps[&d];
            live_pre.range(ds.pre..=ds.post).count() as u64
        }).sum();
        // victim 可见 ⇒ victim 在诞生子树内且不在任何前沿子树内 ⇒ 恰好被 total 数进一次
        let remaining = total - shadowed - 1;
        if remaining == 0 { out.insert(e.id); }
    }
    (out, cost)
}

fn cand_view(w: &World) -> (Vec<CandExtent>, BTreeSet<u64>) {
    let exts = w.exts.values()
        .map(|e| CandExtent { id: e.id, birth: e.birth, frontier: e.frontier.clone() })
        .collect();
    let live_pre = w.snaps.values().filter(|s| s.live).map(|s| s.pre).collect();
    (exts, live_pre)
}

/// 结构自证 ①：由（诞生节点，前沿）重算的可见集必须与生成时的 refs 逐 extent 相等。
fn frontier_visibility_matches_refs(w: &World) -> bool {
    w.exts.values().all(|e| {
        w.snaps.keys().all(|&v| {
            let derived = in_subtree(&w.snaps, e.birth, v)
                && !e.frontier.iter().any(|&d| in_subtree(&w.snaps, d, v));
            derived == e.refs.contains(&v)
        })
    })
}

/// 结构自证 ②：前沿必须是反链（任两个前沿节点无祖先关系）。
fn frontier_is_antichain(w: &World) -> bool {
    w.exts.values().all(|e| {
        e.frontier.iter().all(|&a| e.frontier.iter().all(|&b|
            a == b || !in_subtree(&w.snaps, a, b)))
    })
}

/// 全世界的前沿画像（存储侧口径：对全部 extent，不只对被检查的）。均值用千分位整数，
/// 避免浮点格式抖动破坏逐字节比对。
fn frontier_profile(w: &World) -> (u64, u64) {
    let max = w.exts.values().map(|e| e.frontier.len() as u64).max().unwrap_or(0);
    let total: u64 = w.exts.values().map(|e| e.frontier.len() as u64).sum();
    let mean_milli = if w.exts.is_empty() { 0 } else { total * 1000 / w.exts.len() as u64 };
    (max, mean_milli)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    lookups: u64,
    snap_checks: u64,
    max_frontier: u64,
    freed: u64,
    truly_free: u64,
    wrong_free: u64,
    leaked: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm { PayLookup, Frontier }

impl Arm {
    fn label(self) -> &'static str {
        match self { Arm::PayLookup => "paylookup", Arm::Frontier => "frontier" }
    }
}

fn measure(topo: Topo, arm: Arm, n_snaps: u64, eps: u64, share: u64, seed: u64) -> Out {
    let mut w = build(topo, n_snaps, eps, share, seed);
    // 结构自证不过 ⇒ 记账错了，整轮作废（失败条款）
    assert!(frontier_visibility_matches_refs(&w), "前沿可见性与 refs 不等——记账错，整轮作废");
    assert!(frontier_is_antichain(&w), "前沿不是反链——记账错，整轮作废");
    let victim = n_snaps / 2;
    let (freed, cost) = match arm {
        Arm::PayLookup => {
            let (f, l) = paylookup_arm(&w, victim);
            (f, CandCost { lookups: l, snap_checks: 0, max_frontier: 0 })
        }
        Arm::Frontier => {
            let (exts, live_pre) = cand_view(&w);
            frontier_arm(&w.snaps, &live_pre, &exts, victim)
        }
    };
    w.snaps.get_mut(&victim).unwrap().live = false;
    let truth = truly_free_set(&w);
    Out {
        lookups: cost.lookups,
        snap_checks: cost.snap_checks,
        max_frontier: cost.max_frontier,
        freed: freed.len() as u64,
        truly_free: truth.len() as u64,
        wrong_free: freed.difference(&truth).count() as u64,
        leaked: truth.difference(&freed).count() as u64,
    }
}

fn main() {
    let mut em = Emitter::new();
    let (ns, eps) = (64u64, 8u64);
    println!("{}", em.emit_raw(&format!("name=config snaps={ns} exts_per_snap={eps} victim=middle seed=42")));
    // ── 网格：拓扑 × 共享率 × 两臂，精度与代价 ──
    for topo in [Topo::Linear, Topo::MultiHead] {
        for share in [0u64, 30, 70] {
            for arm in [Arm::PayLookup, Arm::Frontier] {
                let o = measure(topo, arm, ns, eps, share, 42);
                println!("{}", em.emit_raw(&format!(
                    "name=cell topo={topo:?} share={share} arm={} lookups={} snap_checks={} \
                     max_frontier={} freed={} truly_free={} wrong_free={} leaked={}",
                    arm.label(), o.lookups, o.snap_checks, o.max_frontier,
                    o.freed, o.truly_free, o.wrong_free, o.leaked)));
            }
        }
    }
    // ── 规模扫描（判据 2）：固定树（拓扑流不动），extent 数 ×32，谁在长 ──
    for eps in [4u64, 8, 16, 32, 64, 128] {
        let p = measure(Topo::MultiHead, Arm::PayLookup, ns, eps, 70, 42);
        let f = measure(Topo::MultiHead, Arm::Frontier, ns, eps, 70, 42);
        println!("{}", em.emit_raw(&format!(
            "name=scale total_exts={} paylookup_lookups={} frontier_lookups={} \
             frontier_snap_checks={} frontier_max={}",
            ns * eps, p.lookups, f.lookups, f.snap_checks, f.max_frontier)));
    }
    // ── 分支扫描：候选臂的结构规模跟着快照数走，不跟 extent 总数走 ──
    for n in [16u64, 32, 64, 128] {
        let f = measure(Topo::MultiHead, Arm::Frontier, n, 8, 70, 42);
        let w = build(Topo::MultiHead, n, 8, 70, 42);
        let (fmax, fmean) = frontier_profile(&w);
        println!("{}", em.emit_raw(&format!(
            "name=branch snaps={n} table_size={n} lookups={} max_frontier_world={fmax} \
             mean_frontier_milli={fmean} wrong_free={} leaked={}",
            f.lookups, f.wrong_free, f.leaked)));
    }
    // ── 碎删除档（D6 待验优化的限度 ①）：低共享 = 删得碎，前沿画像如实报 ──
    for share in [10u64, 30, 70, 90] {
        let w = build(Topo::MultiHead, ns, 32, share, 42);
        let (fmax, fmean) = frontier_profile(&w);
        let f = measure(Topo::MultiHead, Arm::Frontier, ns, 32, share, 42);
        println!("{}", em.emit_raw(&format!(
            "name=frag share={share} max_frontier_world={fmax} mean_frontier_milli={fmean} \
             wrong_free={} leaked={}",
            f.wrong_free, f.leaked)));
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真值必须是活快照的朴素枚举——它是唯一的裁判。
    #[test]
    fn truth_is_plain_enumeration_of_live_snapshots() {
        let mut w = build(Topo::Linear, 3, 1, 0, 7);
        assert_eq!(truly_free_set(&w).len(), 0, "全部快照活着时不该有自由 extent");
        w.snaps.get_mut(&1).unwrap().live = false;
        assert_eq!(truly_free_set(&w).len(), 1, "删掉一个无共享快照该放出它那一个 extent");
    }

    /// **阳性对照，对每条臂都跑**：Linear + 零共享下两臂都必须与真值完全一致。
    #[test]
    fn positive_control_both_arms_match_truth_when_linear_and_unshared() {
        for arm in [Arm::PayLookup, Arm::Frontier] {
            let o = measure(Topo::Linear, arm, 32, 4, 0, 42);
            assert_eq!(o.wrong_free, 0, "{arm:?} 误放了");
            assert_eq!(o.leaked, 0, "{arm:?} 漏放了");
            assert_eq!(o.freed, o.truly_free, "{arm:?} 与真值不等");
            // 钉绝对值：零共享时该放的恰是 victim 自己那 4 个 extent。
            // 只断言「与真值相等」的话，共享语义整个反转两边会一起错。
            assert_eq!(o.freed, 4, "{arm:?} 零共享下该恰放 victim 的 4 个 extent");
        }
    }

    /// **阳性对照（判据 2 的那一半）**：基线臂必须复现 E26 的增长——
    /// 规模 ×16 时查找 > 4×。不中 ⇒ 这套测量分不出「随 N 长」，整轮作废。
    #[test]
    fn baseline_reproduces_e26_growth_under_multi_head() {
        let small = measure(Topo::MultiHead, Arm::PayLookup, 64, 8, 70, 42).lookups;
        let big = measure(Topo::MultiHead, Arm::PayLookup, 64, 128, 70, 42).lookups;
        assert!(small > 0, "小规模就该有旁支查找（实测 {small}）");
        assert!(big > small * 4,
            "规模 ×16 查找只从 {small} 到 {big}——基线没复现 O(N)，测量无判别力");
    }

    /// **判据 2 钉绝对值**：候选臂反向索引查找在每个规模、每种拓扑下恒为 0。
    #[test]
    fn candidate_pays_zero_reverse_lookups_at_every_scale_and_topology() {
        for topo in [Topo::Linear, Topo::MultiHead] {
            for eps in [4u64, 16, 64, 128] {
                let o = measure(topo, Arm::Frontier, 64, eps, 70, 42);
                assert_eq!(o.lookups, 0,
                    "候选臂在 {topo:?}/{} extent 上付了 {} 次反向查找", 64 * eps, o.lookups);
            }
        }
    }

    /// **判据 1**：候选臂在全部格子上误放与漏放都为 0（对着同一个真值）。
    #[test]
    fn candidate_is_exact_on_every_grid_cell() {
        for topo in [Topo::Linear, Topo::MultiHead] {
            for share in [0u64, 10, 30, 70, 90] {
                for (n, eps) in [(16u64, 4u64), (48, 6), (64, 32)] {
                    let o = measure(topo, Arm::Frontier, n, eps, share, 42);
                    assert_eq!(o.wrong_free, 0,
                        "候选臂在 {topo:?}/share={share}/n={n}/eps={eps} 误放 {}", o.wrong_free);
                    assert_eq!(o.leaked, 0,
                        "候选臂在 {topo:?}/share={share}/n={n}/eps={eps} 漏放 {}", o.leaked);
                }
            }
        }
    }

    /// 基线臂（付查找）也必须精确——E26 已证「付了查找就不丢也不漏」，
    /// 本模型的基线不精确就是移植错了。
    #[test]
    fn baseline_is_exact_too() {
        for topo in [Topo::Linear, Topo::MultiHead] {
            for share in [0u64, 30, 70] {
                let o = measure(topo, Arm::PayLookup, 48, 6, share, 42);
                assert_eq!(o.wrong_free + o.leaked, 0,
                    "基线臂在 {topo:?}/{share} 不精确——与 E26 已证性质矛盾");
            }
        }
    }

    /// **判据 3**：Linear 下候选臂退化成零成本形态——查找恒 0 且每 extent 前沿 ≤ 1，
    /// 恰好是 D5「与上一个快照比 txg」那个常数形态。
    #[test]
    fn linear_topology_degenerates_to_the_constant_form() {
        for share in [0u64, 30, 70] {
            let w = build(Topo::Linear, 64, 8, share, 42);
            let (fmax, _) = frontier_profile(&w);
            assert!(fmax <= 1, "Linear 下前沿该 ≤ 1（share={share}，实测 {fmax}）");
            let o = measure(Topo::Linear, Arm::Frontier, 64, 8, share, 42);
            assert_eq!(o.lookups, 0, "Linear 下候选臂该零查找");
        }
    }

    /// **结构自证 ①**：由（诞生节点，前沿）重算的可见集与生成时的 refs 逐 extent 相等。
    /// 记账与真值两条路子对同一世界给同一个答案，这一步不过整轮作废。
    #[test]
    fn frontier_visibility_equals_generated_refs() {
        for topo in [Topo::Linear, Topo::MultiHead] {
            for share in [0u64, 30, 70] {
                let w = build(topo, 48, 6, share, 42);
                assert!(frontier_visibility_matches_refs(&w),
                    "{topo:?}/{share} 下前沿可见性与 refs 不等");
            }
        }
    }

    /// **结构自证 ②**：前沿是反链。
    #[test]
    fn frontier_is_an_antichain_in_every_world() {
        for topo in [Topo::Linear, Topo::MultiHead] {
            for share in [0u64, 10, 70] {
                let w = build(topo, 64, 8, share, 42);
                assert!(frontier_is_antichain(&w), "{topo:?}/{share} 下前沿不是反链");
            }
        }
    }

    /// **判据 2 的上界**：前沿是快照树节点的反链 ⇒ 大小 ≤ 快照数，与 extent 总数无关。
    #[test]
    fn frontier_bound_is_snapshot_count_not_extent_count() {
        for eps in [8u64, 128] {
            let w = build(Topo::MultiHead, 64, eps, 70, 42);
            let (fmax, _) = frontier_profile(&w);
            assert!(fmax <= 64, "前沿 {fmax} 超过快照数 64——反链上界破了");
        }
    }

    /// **碎删除那一维必须真的动结果**（低共享 ⇒ 前沿 > 1），否则它是死代码。
    #[test]
    fn fragmented_deletion_makes_frontier_grow_beyond_one() {
        let w = build(Topo::MultiHead, 64, 32, 10, 42);
        let (fmax, _) = frontier_profile(&w);
        assert!(fmax > 1, "碎删除档前沿最大值 {fmax} 没超过 1——退化那一维没被测到");
    }

    /// **手搭小世界，逐格钉绝对值**（防「所有臂一起错」）：
    /// 树 0→{1,2}；A 生于 0、前沿 [2]（refs {0,1}）；B 生于 1（refs {1}）；
    /// C 生于 0、无前沿（refs {0,1,2}）。删 victim=1：
    /// 只有 B 该被放（A 还有 0 看着，C 还有 0 和 2 看着）。
    #[test]
    fn hand_built_world_pins_every_absolute_value() {
        let mut snaps = BTreeMap::new();
        snaps.insert(0, Snap { id: 0, parent: None, txg: 0, live: true, pre: 0, post: 0 });
        snaps.insert(1, Snap { id: 1, parent: Some(0), txg: 1, live: true, pre: 0, post: 0 });
        snaps.insert(2, Snap { id: 2, parent: Some(0), txg: 2, live: true, pre: 0, post: 0 });
        label(&mut snaps);
        let mut exts = BTreeMap::new();
        exts.insert(0, Extent { id: 0, birth: 0, frontier: vec![2],
            refs: [0u64, 1].into_iter().collect() });
        exts.insert(1, Extent { id: 1, birth: 1, frontier: vec![],
            refs: [1u64].into_iter().collect() });
        exts.insert(2, Extent { id: 2, birth: 0, frontier: vec![],
            refs: [0u64, 1, 2].into_iter().collect() });
        let w = World { snaps, exts };
        assert!(frontier_visibility_matches_refs(&w), "手搭世界的前沿与 refs 不等");

        let (pl_freed, pl_lookups) = paylookup_arm(&w, 1);
        assert_eq!(pl_freed.iter().copied().collect::<Vec<_>>(), vec![1], "基线该恰好放 B");
        // C 的 refs 里有旁支 2（不在 victim=1 的可比链 {0,1} 里）⇒ 基线恰付 1 次查找。
        // A（refs {0,1}）与 B（refs {1}）全在链上，零查找。
        assert_eq!(pl_lookups, 1, "基线该恰为 C 付 1 次旁支查找");

        let (exts_v, live_pre) = cand_view(&w);
        let (fr_freed, cost) = frontier_arm(&w.snaps, &live_pre, &exts_v, 1);
        assert_eq!(fr_freed.iter().copied().collect::<Vec<_>>(), vec![1], "候选该恰好放 B");
        assert_eq!(cost.lookups, 0);
        // victim=1 可见的是 A(前沿 1 项)、B(0 项)、C(0 项) ⇒ 探查 = 2 + 1 + 1
        assert_eq!(cost.snap_checks, 4, "快照表探查该恰为 4");
        assert_eq!(cost.max_frontier, 1);
    }

    /// preorder 区间标号本身要对：子树判定与父链爬升逐对相等。
    #[test]
    fn interval_labels_agree_with_parent_chain_walk() {
        let w = build(Topo::MultiHead, 32, 2, 50, 42);
        for &a in w.snaps.keys() {
            for &v in w.snaps.keys() {
                let mut cur = Some(v);
                let mut walk = false;
                while let Some(x) = cur {
                    if x == a { walk = true; break; }
                    cur = w.snaps[&x].parent;
                }
                assert_eq!(in_subtree(&w.snaps, a, v), walk,
                    "区间判定与父链爬升在 ({a},{v}) 上不等");
            }
        }
    }
}
