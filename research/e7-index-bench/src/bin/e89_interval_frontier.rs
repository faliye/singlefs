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
enum Topology { Linear, MultiHead }

#[derive(Clone)]
struct Snapshot {
    snapshot_identifier: u64,
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
    extent_identifier: u64,
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
struct CandidateExtent {
    extent_identifier: u64,
    birth: u64,
    frontier: Vec<u64>,
}

struct World { snaps: BTreeMap<u64, Snapshot>, extents: BTreeMap<u64, Extent> }

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state >> 12; *state ^= *state << 25; *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// 与 E26 的一处**故意**差别：拓扑与共享各用一条独立 RNG 流（同种子派生），
/// 让树形状在规模扫描里保持不变——判据 2 要在固定的树上扫 extent 数。
fn build(topology: Topology, snapshot_count: u64, extents_per_snapshot: u64, share_percent: u64, seed: u64) -> World {
    let mut topology_random_state = (seed | 1) ^ 0x9E37_79B9_7F4A_7C15;
    let mut share_random_state = (seed | 1).wrapping_mul(0xBF58_476D_1CE4_E5B9) | 1;
    let mut snaps = BTreeMap::new();
    let mut extents: BTreeMap<u64, Extent> = BTreeMap::new();
    let mut next_extent_identifier = 0u64;
    for snapshot_index in 0..snapshot_count {
        let parent = if snapshot_index == 0 { None } else {
            match topology {
                Topology::Linear => Some(snapshot_index - 1),
                Topology::MultiHead => Some(xorshift(&mut topology_random_state) % snapshot_index),
            }
        };
        for _ in 0..extents_per_snapshot {
            let extent_identifier = next_extent_identifier; next_extent_identifier += 1;
            let mut refs = BTreeSet::new(); refs.insert(snapshot_index);
            extents.insert(extent_identifier, Extent { extent_identifier, birth: snapshot_index, frontier: Vec::new(), refs });
        }
        // 共享：本快照按比例继承父亲引用的 extent；**没继承的那一步就地记进前沿**
        if let Some(parent_identifier) = parent {
            let inherited: Vec<u64> = extents.values()
                .filter(|extent| extent.refs.contains(&parent_identifier)).map(|extent| extent.extent_identifier).collect();
            for inherited_identifier in inherited {
                let extent = extents.get_mut(&inherited_identifier).unwrap();
                if xorshift(&mut share_random_state) % 100 < share_percent {
                    extent.refs.insert(snapshot_index);
                } else {
                    extent.frontier.push(snapshot_index);
                }
            }
        }
        snaps.insert(snapshot_index, Snapshot { snapshot_identifier: snapshot_index, parent, txg: snapshot_index, live: true, pre: 0, post: 0 });
    }
    label(&mut snaps);
    World { snaps, extents }
}

/// 建完树一次性编 preorder 区间。标号维护（树增长时的 order-maintenance）不在射程内，
/// 见 kb 正文「口径与已知局限」。
fn label(snaps: &mut BTreeMap<u64, Snapshot>) {
    let mut children: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut root = None;
    for snapshot in snaps.values() {
        match snapshot.parent {
            Some(parent_identifier) => children.entry(parent_identifier).or_default().push(snapshot.snapshot_identifier),
            None => root = Some(snapshot.snapshot_identifier),
        }
    }
    let root = root.expect("树必须有根");
    // 迭代 DFS 发 pre；post(u) = pre(u) + 子树大小 − 1，子树大小按访问序自底向上累加
    let mut next_pre = 0u64;
    let mut stack = vec![root];
    let mut order: Vec<u64> = Vec::new();
    while let Some(visiting) = stack.pop() {
        let snapshot = snaps.get_mut(&visiting).unwrap();
        snapshot.pre = next_pre; next_pre += 1;
        order.push(visiting);
        if let Some(child_list) = children.get(&visiting) {
            // 倒序进栈 ⇒ 按 id 升序访问，确定性
            for &child in child_list.iter().rev() { stack.push(child); }
        }
    }
    let mut size: BTreeMap<u64, u64> = snaps.keys().map(|&snapshot_key| (snapshot_key, 1u64)).collect();
    for &visited in order.iter().rev() {
        if let Some(parent_identifier) = snaps[&visited].parent {
            let subtree_size = size[&visited];
            *size.get_mut(&parent_identifier).unwrap() += subtree_size;
        }
    }
    for (&snapshot_key, snapshot) in snaps.iter_mut() {
        snapshot.post = snapshot.pre + size[&snapshot_key] - 1;
    }
}

fn in_subtree(snaps: &BTreeMap<u64, Snapshot>, ancestor: u64, candidate: u64) -> bool {
    let ancestor_snapshot = &snaps[&ancestor];
    let candidate_preorder = snaps[&candidate].pre;
    ancestor_snapshot.pre <= candidate_preorder && candidate_preorder <= ancestor_snapshot.post
}

/// 与 `snapshot_identifier` 可比的快照集合 = 祖先 ∪ 后代（含自己）。E26 同构。
fn comparable(world: &World, snapshot_identifier: u64) -> BTreeSet<u64> {
    let mut ancestors = BTreeSet::new();
    let mut ancestor_walk_next = Some(snapshot_identifier);
    while let Some(ancestor_walk_current) = ancestor_walk_next {
        if !ancestors.insert(ancestor_walk_current) { break; }
        ancestor_walk_next = world.snaps.get(&ancestor_walk_current).and_then(|snapshot| snapshot.parent);
    }
    let mut comparable_set = ancestors.clone();
    for snapshot in world.snaps.values() {
        let mut descendant_check_next = Some(snapshot.snapshot_identifier);
        while let Some(descendant_check_current) = descendant_check_next {
            if descendant_check_current == snapshot_identifier { comparable_set.insert(snapshot.snapshot_identifier); break; }
            descendant_check_next = world.snaps.get(&descendant_check_current).and_then(|parent_snapshot| parent_snapshot.parent);
        }
    }
    comparable_set
}

/// 真值：枚举全部活快照。**不走任何臂的代码。**
fn truly_free_set(world: &World) -> BTreeSet<u64> {
    let live: BTreeSet<u64> = world.snaps.values().filter(|snapshot| snapshot.live).map(|snapshot| snapshot.snapshot_identifier).collect();
    world.extents.values()
        .filter(|extent| extent.refs.iter().all(|referrer| !live.contains(referrer)))
        .map(|extent| extent.extent_identifier).collect()
}

/// 基线臂（付查找）：E26 的 deadlist 臂同构。定义句原样贴
/// （verify-before-claiming：引用决策去推导之前先贴定义）：
///   「块 b 被快照 S 引用 ⟺ birth(b) ≤ S.txg < death(b)，左闭右开」
///   「death(b) = 最后一个活引用被摘掉且被发布的 checkpoint 号；仍在活树里则为 ∞」
fn paylookup_arm(world: &World, victim: u64) -> (BTreeSet<u64>, u64) {
    let mut lookups = 0u64;
    let victim_snapshot = &world.snaps[&victim];
    let chain = comparable(world, victim);
    let mut freed_extents = BTreeSet::new();
    for extent in world.extents.values() {
        if !extent.refs.contains(&victim) { continue; }
        let has_sibling_ref = extent.refs.iter().any(|referrer| *referrer != victim && !chain.contains(referrer));
        let death_finite = if has_sibling_ref {
            lookups += 1;                            // O(1) 性质在这里破掉
            !extent.refs.iter().any(|&referrer| referrer != victim
                && world.snaps.get(&referrer).map(|referrer_snapshot| referrer_snapshot.live).unwrap_or(false))
        } else {
            !extent.refs.iter().any(|&referrer| referrer != victim
                && world.snaps.get(&referrer).map(|referrer_snapshot| referrer_snapshot.live).unwrap_or(false))
        };
        if !death_finite { continue; }
        let death = victim_snapshot.txg + 1;
        if !world.snaps.values().any(|snap| snap.live && snap.snapshot_identifier != victim
            && extent.birth <= snap.txg && snap.txg < death) {
            freed_extents.insert(extent.extent_identifier);
        }
    }
    (freed_extents, lookups)
}

/// 候选臂的观测量。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct CandidateCost {
    /// 反向索引查找——判据 2 要求恒为 0
    lookups: u64,
    /// 快照表探查次数 = Σ（1 + 前沿大小），只对被删快照可见的 extent 计
    snapshot_table_checks: u64,
    /// 被检查的 extent 里最大的前沿
    maximum_frontier: u64,
}

/// 候选臂：只拿得到（诞生节点，前沿）与快照树标号——**refs 不在入参里**。
/// 「S 还看得见 e 吗」与「还剩几个活引用者」都是区间算术：
///   剩余活引用 = 诞生子树内活快照数 − Σ 前沿子树内活快照数 − 1（victim 自己）。
/// 前沿子树互不相交且都在诞生子树内（反链）⇒ 容斥恰好一层。
fn frontier_arm(
    snaps: &BTreeMap<u64, Snapshot>,
    live_pre: &BTreeSet<u64>,
    extents: &[CandidateExtent],
    victim: u64,
) -> (BTreeSet<u64>, CandidateCost) {
    let mut cost = CandidateCost::default();
    let mut freed_extents = BTreeSet::new();
    for extent in extents {
        let victim_sees = in_subtree(snaps, extent.birth, victim)
            && !extent.frontier.iter().any(|&frontier_node| in_subtree(snaps, frontier_node, victim));
        if !victim_sees { continue; }
        cost.snapshot_table_checks += 1 + extent.frontier.len() as u64;
        cost.maximum_frontier = cost.maximum_frontier.max(extent.frontier.len() as u64);
        let birth_snapshot = &snaps[&extent.birth];
        let total = live_pre.range(birth_snapshot.pre..=birth_snapshot.post).count() as u64;
        let shadowed: u64 = extent.frontier.iter().map(|&frontier_node| {
            let frontier_snapshot = &snaps[&frontier_node];
            live_pre.range(frontier_snapshot.pre..=frontier_snapshot.post).count() as u64
        }).sum();
        // victim 可见 ⇒ victim 在诞生子树内且不在任何前沿子树内 ⇒ 恰好被 total 数进一次
        let remaining = total - shadowed - 1;
        if remaining == 0 { freed_extents.insert(extent.extent_identifier); }
    }
    (freed_extents, cost)
}

fn candidate_view(world: &World) -> (Vec<CandidateExtent>, BTreeSet<u64>) {
    let extents = world.extents.values()
        .map(|extent| CandidateExtent { extent_identifier: extent.extent_identifier, birth: extent.birth, frontier: extent.frontier.clone() })
        .collect();
    let live_pre = world.snaps.values().filter(|snapshot| snapshot.live).map(|snapshot| snapshot.pre).collect();
    (extents, live_pre)
}

/// 结构自证 ①：由（诞生节点，前沿）重算的可见集必须与生成时的 refs 逐 extent 相等。
fn frontier_visibility_matches_refs(world: &World) -> bool {
    world.extents.values().all(|extent| {
        world.snaps.keys().all(|&snapshot_key| {
            let derived = in_subtree(&world.snaps, extent.birth, snapshot_key)
                && !extent.frontier.iter().any(|&frontier_node| in_subtree(&world.snaps, frontier_node, snapshot_key));
            derived == extent.refs.contains(&snapshot_key)
        })
    })
}

/// 结构自证 ②：前沿必须是反链（任两个前沿节点无祖先关系）。
fn frontier_is_antichain(world: &World) -> bool {
    world.extents.values().all(|extent| {
        extent.frontier.iter().all(|&first_node| extent.frontier.iter().all(|&second_node|
            first_node == second_node || !in_subtree(&world.snaps, first_node, second_node)))
    })
}

/// 全世界的前沿画像（存储侧口径：对全部 extent，不只对被检查的）。均值用千分位整数，
/// 避免浮点格式抖动破坏逐字节比对。
fn frontier_profile(world: &World) -> (u64, u64) {
    let maximum_frontier_length = world.extents.values().map(|extent| extent.frontier.len() as u64).max().unwrap_or(0);
    let total: u64 = world.extents.values().map(|extent| extent.frontier.len() as u64).sum();
    let mean_milli = if world.extents.is_empty() { 0 } else { total * 1000 / world.extents.len() as u64 };
    (maximum_frontier_length, mean_milli)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct MeasuredOutcome {
    lookups: u64,
    snapshot_table_checks: u64,
    maximum_frontier: u64,
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

fn measure(topology: Topology, arm: Arm, snapshot_count: u64, extents_per_snapshot: u64, share: u64, seed: u64) -> MeasuredOutcome {
    let mut world = build(topology, snapshot_count, extents_per_snapshot, share, seed);
    // 结构自证不过 ⇒ 记账错了，整轮作废（失败条款）
    assert!(frontier_visibility_matches_refs(&world), "前沿可见性与 refs 不等——记账错，整轮作废");
    assert!(frontier_is_antichain(&world), "前沿不是反链——记账错，整轮作废");
    let victim = snapshot_count / 2;
    let (freed, cost) = match arm {
        Arm::PayLookup => {
            let (freed_set, lookup_count) = paylookup_arm(&world, victim);
            (freed_set, CandidateCost { lookups: lookup_count, snapshot_table_checks: 0, maximum_frontier: 0 })
        }
        Arm::Frontier => {
            let (extents, live_pre) = candidate_view(&world);
            frontier_arm(&world.snaps, &live_pre, &extents, victim)
        }
    };
    world.snaps.get_mut(&victim).unwrap().live = false;
    let truth = truly_free_set(&world);
    MeasuredOutcome {
        lookups: cost.lookups,
        snapshot_table_checks: cost.snapshot_table_checks,
        maximum_frontier: cost.maximum_frontier,
        freed: freed.len() as u64,
        truly_free: truth.len() as u64,
        wrong_free: freed.difference(&truth).count() as u64,
        leaked: truth.difference(&freed).count() as u64,
    }
}

fn main() {
    let mut emitter = Emitter::new();
    let (snapshot_count, extents_per_snapshot) = (64u64, 8u64);
    println!("{}", emitter.emit_raw(&format!("name=config snaps={snapshot_count} exts_per_snap={extents_per_snapshot} victim=middle seed=42")));
    // ── 网格：拓扑 × 共享率 × 两臂，精度与代价 ──
    for topology in [Topology::Linear, Topology::MultiHead] {
        for share in [0u64, 30, 70] {
            for arm in [Arm::PayLookup, Arm::Frontier] {
                let outcome = measure(topology, arm, snapshot_count, extents_per_snapshot, share, 42);
                println!("{}", emitter.emit_raw(&format!(
                    "name=cell topo={topology:?} share={share} arm={} lookups={} snap_checks={} \
                     max_frontier={} freed={} truly_free={} wrong_free={} leaked={}",
                    arm.label(), outcome.lookups, outcome.snapshot_table_checks, outcome.maximum_frontier,
                    outcome.freed, outcome.truly_free, outcome.wrong_free, outcome.leaked)));
            }
        }
    }
    // ── 规模扫描（判据 2）：固定树（拓扑流不动），extent 数 ×32，谁在长 ──
    for extents_per_snapshot in [4u64, 8, 16, 32, 64, 128] {
        let paylookup_outcome = measure(Topology::MultiHead, Arm::PayLookup, snapshot_count, extents_per_snapshot, 70, 42);
        let frontier_outcome = measure(Topology::MultiHead, Arm::Frontier, snapshot_count, extents_per_snapshot, 70, 42);
        println!("{}", emitter.emit_raw(&format!(
            "name=scale total_exts={} paylookup_lookups={} frontier_lookups={} \
             frontier_snap_checks={} frontier_max={}",
            snapshot_count * extents_per_snapshot, paylookup_outcome.lookups, frontier_outcome.lookups, frontier_outcome.snapshot_table_checks, frontier_outcome.maximum_frontier)));
    }
    // ── 分支扫描：候选臂的结构规模跟着快照数走，不跟 extent 总数走 ──
    for swept_snapshot_count in [16u64, 32, 64, 128] {
        let frontier_outcome = measure(Topology::MultiHead, Arm::Frontier, swept_snapshot_count, 8, 70, 42);
        let world = build(Topology::MultiHead, swept_snapshot_count, 8, 70, 42);
        let (maximum_frontier_in_world, mean_frontier_milli_in_world) = frontier_profile(&world);
        println!("{}", emitter.emit_raw(&format!(
            "name=branch snaps={swept_snapshot_count} table_size={swept_snapshot_count} lookups={} max_frontier_world={maximum_frontier_in_world} \
             mean_frontier_milli={mean_frontier_milli_in_world} wrong_free={} leaked={}",
            frontier_outcome.lookups, frontier_outcome.wrong_free, frontier_outcome.leaked)));
    }
    // ── 碎删除档（D6 待验优化的限度 ①）：低共享 = 删得碎，前沿画像如实报 ──
    for share in [10u64, 30, 70, 90] {
        let world = build(Topology::MultiHead, snapshot_count, 32, share, 42);
        let (maximum_frontier_in_world, mean_frontier_milli_in_world) = frontier_profile(&world);
        let frontier_outcome = measure(Topology::MultiHead, Arm::Frontier, snapshot_count, 32, share, 42);
        println!("{}", emitter.emit_raw(&format!(
            "name=frag share={share} max_frontier_world={maximum_frontier_in_world} mean_frontier_milli={mean_frontier_milli_in_world} \
             wrong_free={} leaked={}",
            frontier_outcome.wrong_free, frontier_outcome.leaked)));
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真值必须是活快照的朴素枚举——它是唯一的裁判。
    #[test]
    fn truth_is_plain_enumeration_of_live_snapshots() {
        let mut world = build(Topology::Linear, 3, 1, 0, 7);
        assert_eq!(truly_free_set(&world).len(), 0, "全部快照活着时不该有自由 extent");
        world.snaps.get_mut(&1).unwrap().live = false;
        assert_eq!(truly_free_set(&world).len(), 1, "删掉一个无共享快照该放出它那一个 extent");
    }

    /// **阳性对照，对每条臂都跑**：Linear + 零共享下两臂都必须与真值完全一致。
    #[test]
    fn positive_control_both_arms_match_truth_when_linear_and_unshared() {
        for arm in [Arm::PayLookup, Arm::Frontier] {
            let outcome = measure(Topology::Linear, arm, 32, 4, 0, 42);
            assert_eq!(outcome.wrong_free, 0, "{arm:?} 误放了");
            assert_eq!(outcome.leaked, 0, "{arm:?} 漏放了");
            assert_eq!(outcome.freed, outcome.truly_free, "{arm:?} 与真值不等");
            // 钉绝对值：零共享时该放的恰是 victim 自己那 4 个 extent。
            // 只断言「与真值相等」的话，共享语义整个反转两边会一起错。
            assert_eq!(outcome.freed, 4, "{arm:?} 零共享下该恰放 victim 的 4 个 extent");
        }
    }

    /// **阳性对照（判据 2 的那一半）**：基线臂必须复现 E26 的增长——
    /// 规模 ×16 时查找 > 4×。不中 ⇒ 这套测量分不出「随 N 长」，整轮作废。
    #[test]
    fn baseline_reproduces_e26_growth_under_multi_head() {
        let small = measure(Topology::MultiHead, Arm::PayLookup, 64, 8, 70, 42).lookups;
        let big = measure(Topology::MultiHead, Arm::PayLookup, 64, 128, 70, 42).lookups;
        assert!(small > 0, "小规模就该有旁支查找（实测 {small}）");
        assert!(big > small * 4,
            "规模 ×16 查找只从 {small} 到 {big}——基线没复现 O(N)，测量无判别力");
    }

    /// **判据 2 钉绝对值**：候选臂反向索引查找在每个规模、每种拓扑下恒为 0。
    #[test]
    fn candidate_pays_zero_reverse_lookups_at_every_scale_and_topology() {
        for topology in [Topology::Linear, Topology::MultiHead] {
            for extents_per_snapshot in [4u64, 16, 64, 128] {
                let outcome = measure(topology, Arm::Frontier, 64, extents_per_snapshot, 70, 42);
                assert_eq!(outcome.lookups, 0,
                    "候选臂在 {topology:?}/{} extent 上付了 {} 次反向查找", 64 * extents_per_snapshot, outcome.lookups);
            }
        }
    }

    /// **判据 1**：候选臂在全部格子上误放与漏放都为 0（对着同一个真值）。
    #[test]
    fn candidate_is_exact_on_every_grid_cell() {
        for topology in [Topology::Linear, Topology::MultiHead] {
            for share in [0u64, 10, 30, 70, 90] {
                for (snapshot_count, extents_per_snapshot) in [(16u64, 4u64), (48, 6), (64, 32)] {
                    let outcome = measure(topology, Arm::Frontier, snapshot_count, extents_per_snapshot, share, 42);
                    assert_eq!(outcome.wrong_free, 0,
                        "候选臂在 {topology:?}/share={share}/n={snapshot_count}/eps={extents_per_snapshot} 误放 {}", outcome.wrong_free);
                    assert_eq!(outcome.leaked, 0,
                        "候选臂在 {topology:?}/share={share}/n={snapshot_count}/eps={extents_per_snapshot} 漏放 {}", outcome.leaked);
                }
            }
        }
    }

    /// 基线臂（付查找）也必须精确——E26 已证「付了查找就不丢也不漏」，
    /// 本模型的基线不精确就是移植错了。
    #[test]
    fn baseline_is_exact_too() {
        for topology in [Topology::Linear, Topology::MultiHead] {
            for share in [0u64, 30, 70] {
                let outcome = measure(topology, Arm::PayLookup, 48, 6, share, 42);
                assert_eq!(outcome.wrong_free + outcome.leaked, 0,
                    "基线臂在 {topology:?}/{share} 不精确——与 E26 已证性质矛盾");
            }
        }
    }

    /// **判据 3**：Linear 下候选臂退化成零成本形态——查找恒 0 且每 extent 前沿 ≤ 1，
    /// 恰好是 D5「与上一个快照比 txg」那个常数形态。
    #[test]
    fn linear_topology_degenerates_to_the_constant_form() {
        for share in [0u64, 30, 70] {
            let world = build(Topology::Linear, 64, 8, share, 42);
            let (maximum_frontier_in_world, _) = frontier_profile(&world);
            assert!(maximum_frontier_in_world <= 1, "Linear 下前沿该 ≤ 1（share={share}，实测 {maximum_frontier_in_world}）");
            let outcome = measure(Topology::Linear, Arm::Frontier, 64, 8, share, 42);
            assert_eq!(outcome.lookups, 0, "Linear 下候选臂该零查找");
        }
    }

    /// **结构自证 ①**：由（诞生节点，前沿）重算的可见集与生成时的 refs 逐 extent 相等。
    /// 记账与真值两条路子对同一世界给同一个答案，这一步不过整轮作废。
    #[test]
    fn frontier_visibility_equals_generated_refs() {
        for topology in [Topology::Linear, Topology::MultiHead] {
            for share in [0u64, 30, 70] {
                let world = build(topology, 48, 6, share, 42);
                assert!(frontier_visibility_matches_refs(&world),
                    "{topology:?}/{share} 下前沿可见性与 refs 不等");
            }
        }
    }

    /// **结构自证 ②**：前沿是反链。
    #[test]
    fn frontier_is_an_antichain_in_every_world() {
        for topology in [Topology::Linear, Topology::MultiHead] {
            for share in [0u64, 10, 70] {
                let world = build(topology, 64, 8, share, 42);
                assert!(frontier_is_antichain(&world), "{topology:?}/{share} 下前沿不是反链");
            }
        }
    }

    /// **判据 2 的上界**：前沿是快照树节点的反链 ⇒ 大小 ≤ 快照数，与 extent 总数无关。
    #[test]
    fn frontier_bound_is_snapshot_count_not_extent_count() {
        for extents_per_snapshot in [8u64, 128] {
            let world = build(Topology::MultiHead, 64, extents_per_snapshot, 70, 42);
            let (maximum_frontier_in_world, _) = frontier_profile(&world);
            assert!(maximum_frontier_in_world <= 64, "前沿 {maximum_frontier_in_world} 超过快照数 64——反链上界破了");
        }
    }

    /// **碎删除那一维必须真的动结果**（低共享 ⇒ 前沿 > 1），否则它是死代码。
    #[test]
    fn fragmented_deletion_makes_frontier_grow_beyond_one() {
        let world = build(Topology::MultiHead, 64, 32, 10, 42);
        let (maximum_frontier_in_world, _) = frontier_profile(&world);
        assert!(maximum_frontier_in_world > 1, "碎删除档前沿最大值 {maximum_frontier_in_world} 没超过 1——退化那一维没被测到");
    }

    /// **手搭小世界，逐格钉绝对值**（防「所有臂一起错」）：
    /// 树 0→{1,2}；A 生于 0、前沿 [2]（refs {0,1}）；B 生于 1（refs {1}）；
    /// C 生于 0、无前沿（refs {0,1,2}）。删 victim=1：
    /// 只有 B 该被放（A 还有 0 看着，C 还有 0 和 2 看着）。
    #[test]
    fn hand_built_world_pins_every_absolute_value() {
        let mut snaps = BTreeMap::new();
        snaps.insert(0, Snapshot { snapshot_identifier: 0, parent: None, txg: 0, live: true, pre: 0, post: 0 });
        snaps.insert(1, Snapshot { snapshot_identifier: 1, parent: Some(0), txg: 1, live: true, pre: 0, post: 0 });
        snaps.insert(2, Snapshot { snapshot_identifier: 2, parent: Some(0), txg: 2, live: true, pre: 0, post: 0 });
        label(&mut snaps);
        let mut extents = BTreeMap::new();
        extents.insert(0, Extent { extent_identifier: 0, birth: 0, frontier: vec![2],
            refs: [0u64, 1].into_iter().collect() });
        extents.insert(1, Extent { extent_identifier: 1, birth: 1, frontier: vec![],
            refs: [1u64].into_iter().collect() });
        extents.insert(2, Extent { extent_identifier: 2, birth: 0, frontier: vec![],
            refs: [0u64, 1, 2].into_iter().collect() });
        let world = World { snaps, extents };
        assert!(frontier_visibility_matches_refs(&world), "手搭世界的前沿与 refs 不等");

        let (paylookup_freed, paylookup_lookups) = paylookup_arm(&world, 1);
        assert_eq!(paylookup_freed.iter().copied().collect::<Vec<_>>(), vec![1], "基线该恰好放 B");
        // C 的 refs 里有旁支 2（不在 victim=1 的可比链 {0,1} 里）⇒ 基线恰付 1 次查找。
        // A（refs {0,1}）与 B（refs {1}）全在链上，零查找。
        assert_eq!(paylookup_lookups, 1, "基线该恰为 C 付 1 次旁支查找");

        let (candidate_extents, live_pre) = candidate_view(&world);
        let (frontier_freed, cost) = frontier_arm(&world.snaps, &live_pre, &candidate_extents, 1);
        assert_eq!(frontier_freed.iter().copied().collect::<Vec<_>>(), vec![1], "候选该恰好放 B");
        assert_eq!(cost.lookups, 0);
        // victim=1 可见的是 A(前沿 1 项)、B(0 项)、C(0 项) ⇒ 探查 = 2 + 1 + 1
        assert_eq!(cost.snapshot_table_checks, 4, "快照表探查该恰为 4");
        assert_eq!(cost.maximum_frontier, 1);
    }

    /// preorder 区间标号本身要对：子树判定与父链爬升逐对相等。
    #[test]
    fn interval_labels_agree_with_parent_chain_walk() {
        let world = build(Topology::MultiHead, 32, 2, 50, 42);
        for &ancestor in world.snaps.keys() {
            for &candidate in world.snaps.keys() {
                let mut next_to_visit = Some(candidate);
                let mut found_by_parent_walk = false;
                while let Some(current) = next_to_visit {
                    if current == ancestor { found_by_parent_walk = true; break; }
                    next_to_visit = world.snaps[&current].parent;
                }
                assert_eq!(in_subtree(&world.snaps, ancestor, candidate), found_by_parent_walk,
                    "区间判定与父链爬升在 ({ancestor},{candidate}) 上不等");
            }
        }
    }
}
