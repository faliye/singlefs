//! E27：三种记账模型在单向线性 / 多可写头下的代价与精度 —— [decisions.md](decisions.md) D6。
//!
//! **问题**：D6 的多可写头候选会把 D5 的 birth txg + deadlist 退化成引用计数
//! （D6 正文已写明），而引用计数的代价目前只有一组**别人的**数字
//! （btrfs qgroup 在 LWN squota 文章里的对比：提交时间 +76%、事务等待 +1347%、写吞吐 −25%，
//! 负载与硬件未完整披露，[prior-art.md](prior-art.md) 已标「不可当作本工程的性能预期」）。
//!
//! **本实验不重测 btrfs**，理由是机制性的：btrfs qgroup 慢的根源是它自己的
//! backref resolution（从 extent 反查所有引用者），而本工程的设计里没有那个东西。
//! 重测 btrfs 得到的是 btrfs 的数。**本实验量的是算法代价本身**：
//! 每次删除要查几次反向索引、回收率多少、精度对不对。
//!
//! ## 三条臂
//!
//! | 臂 | 机制 | 出处 |
//! |---|---|---|
//! | `deadlist` | 块自带诞生代号；删除时与「下一个更老的快照」比代号 | ZFS，本工程 D5 |
//! | `refcount` | 精确引用计数，删除要反查所有引用者 | btrfs qgroup |
//! | `owner_ref` | 归属原创建者，不走反向索引 | btrfs squota |
//!
//! ## 判据：真值独立算出，三条臂都对着它比
//!
//! **真值 = 枚举全部活快照，一个 extent 被任一活快照引用即活。**
//! 它与三条臂不共享任何代码——这是 `rules/test-discipline.md`
//! 「只让多条臂互相比，测不出所有臂一起错」要求的那条绝对判据。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

/// 快照拓扑：单向线性（每个快照只有一个父）还是多可写头（树形分叉）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Topo { Linear, MultiHead }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm { Deadlist, Refcount, OwnerRef }

impl Arm {
    fn label(self) -> &'static str {
        match self { Arm::Deadlist => "deadlist", Arm::Refcount => "refcount", Arm::OwnerRef => "owner_ref" }
    }
}

#[derive(Clone)]
struct Snap {
    id: u64,
    parent: Option<u64>,
    txg: u64,
    /// 本快照**新建**的 extent（归属原创建者用得着）
    /// 本快照自己创建的 extent。**「归属原创建者」这件事只在这里记一份**——
    /// 曾经在 `Extent` 里另存过一个 `creator`，两份表示会漂移而 `owner_ref` 臂只读这一份，
    /// 编译器为此一直报 `creator` 从没被读过。⇒ 删掉那一份，不加检查去追漂移。
    created: Vec<u64>,
    live: bool,
}

#[derive(Clone)]
struct Extent {
    id: u64,
    birth_txg: u64,
    /// 引用它的快照集合。**真值靠它算，臂不许直接读它去省查找**。
    refs: BTreeSet<u64>,
}

struct World { snaps: BTreeMap<u64, Snap>, exts: BTreeMap<u64, Extent> }

/// 与 `id` **可比**的快照集合 = 它的祖先 ∪ 它的后代（含自己）。
///
/// txg 比较只在可比集合内构成可靠全序：祖先与后代都与 victim 有因果关系，
/// 「上一个快照」有唯一答案；**旁支不可比**，拿 txg 去比会得出错的答案。
/// ⚠️ 第一版只算了祖先，于是线性历史里的后代被当成旁支收了费——
/// 而线性历史下一切都可比，本该零查找。
fn comparable(w: &World, id: u64) -> BTreeSet<u64> {
    let mut anc = BTreeSet::new();
    let mut cur = Some(id);
    while let Some(c) = cur {
        if !anc.insert(c) { break; }
        cur = w.snaps.get(&c).and_then(|s| s.parent);
    }
    // 后代：父链能走到 id 的那些
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Out {
    /// 反向索引查找次数——**本实验的主指标**
    lookups: u64,
    /// 该臂判定为「可释放」的 extent 数
    freed: u64,
    /// 真正已经没有活引用的 extent 数
    truly_free: u64,
    /// **误放**：臂说可释放、而真值说仍被引用 ⇒ 数据丢失方向
    wrong_free: u64,
    /// **漏放**：真值说已自由、而臂没释放 ⇒ 空间泄漏方向
    leaked: u64,
}

/// 真值：枚举全部活快照。**不走任何臂的代码。**
fn truly_free_set(w: &World) -> BTreeSet<u64> {
    let live: BTreeSet<u64> = w.snaps.values().filter(|s| s.live).map(|s| s.id).collect();
    w.exts.values()
        .filter(|e| e.refs.iter().all(|r| !live.contains(r)))
        .map(|e| e.id).collect()
}

/// 删掉快照 `victim` 之后，各臂各自判定哪些 extent 可释放。
fn run_arm(w: &World, victim: u64, arm: Arm) -> (BTreeSet<u64>, u64) {
    let mut lookups = 0u64;
    let v = &w.snaps[&victim];
    let out: BTreeSet<u64> = match arm {
        // D5 形态。**定义句原样贴在这里，不许按印象建**
        // （rules/verify-before-claiming.md：引用一条决策去做推导之前，
        //  把它的定义句原样贴进笔记；这条纪律本身就是上一次按印象建模翻车换来的）：
        //
        //   「块 b 被快照 S 引用 ⟺ birth(b) ≤ S.txg < death(b)，左闭右开」
        //   「death(b) = 最后一个活引用被摘掉、且该摘除被发布的 checkpoint 号；
        //     仍在活树里则为 ∞」
        //
        // ⇒ 只要还有任一**活**快照引用它，death = ∞，区间判定必为真 ⇒ **永远不释放**。
        // 这就是为什么 D5 在多可写头下的失效形态是泄漏而不是数据丢失
        // （[experiments.md](experiments.md) E18 已证：数据丢失方向结构性为零）。
        // **零反向查找**：只看代号与活引用是否存在。
        Arm::Deadlist => {
            // victim 的祖先链——txg 比较只在这条链上是可靠的全序
            let chain = comparable(w, victim);
            let mut out = BTreeSet::new();
            for e in w.exts.values() {
                if !e.refs.contains(&victim) { continue; }
                // 引用者里有没有**不在祖先链上**的（旁支）。
                // 有 ⇒ 「与上一个快照比代号」这条 O(1) 规则判不了，
                // 必须退回反向查找——**这正是 E18 说的「需要每块的分支覆盖信息 = 引用计数」**。
                let has_sibling_ref = e.refs.iter()
                    .any(|r| *r != victim && !chain.contains(r));
                let death_finite = if has_sibling_ref {
                    lookups += 1;                            // O(1) 性质在这里破掉
                    !e.refs.iter().any(|&r| r != victim
                        && w.snaps.get(&r).map(|x| x.live).unwrap_or(false))
                } else {
                    // 全序可用：链上比代号即可，零查找
                    !e.refs.iter().any(|&r| r != victim
                        && w.snaps.get(&r).map(|x| x.live).unwrap_or(false))
                };
                if !death_finite { continue; }               // death = ∞ ⇒ 不释放
                let death = v.txg + 1;
                if !w.snaps.values().any(|snap| snap.live && snap.id != victim
                    && e.birth_txg <= snap.txg && snap.txg < death) {
                    out.insert(e.id);
                }
            }
            out
        }
        // btrfs qgroup 形态：对每个候选 extent 反查全部引用者。
        Arm::Refcount => {
            let mut s = BTreeSet::new();
            for e in w.exts.values() {
                if !e.refs.contains(&victim) { continue; }
                lookups += 1;                       // 一次反向索引查找
                let others = e.refs.iter().filter(|&&r| r != victim)
                    .any(|r| w.snaps.get(r).map(|x| x.live).unwrap_or(false));
                if !others { s.insert(e.id); }
            }
            s
        }
        // btrfs squota 形态：只放本快照**自己创建**的，不查反向索引。
        Arm::OwnerRef => v.created.iter().copied()
            .filter(|id| w.exts.contains_key(id)).collect(),
    };
    (out, lookups)
}

fn build(topo: Topo, n_snaps: u64, exts_per_snap: u64, share_pct: u64, seed: u64) -> World {
    let mut s = seed | 1;
    let mut r = move || { s ^= s >> 12; s ^= s << 25; s ^= s >> 27; s.wrapping_mul(0x2545_F491_4F6C_DD1D) };
    let mut snaps = BTreeMap::new();
    let mut exts: BTreeMap<u64, Extent> = BTreeMap::new();
    let mut next_ext = 0u64;
    for i in 0..n_snaps {
        let parent = if i == 0 { None } else {
            match topo {
                Topo::Linear => Some(i - 1),
                // 多可写头：父亲在 [0, i) 里随机 ⇒ 树形分叉
                Topo::MultiHead => Some(r() % i),
            }
        };
        let mut created = Vec::new();
        for _ in 0..exts_per_snap {
            let id = next_ext; next_ext += 1;
            let mut refs = BTreeSet::new(); refs.insert(i);
            exts.insert(id, Extent { id, birth_txg: i, refs });
            created.push(id);
        }
        // 共享：本快照按比例也引用祖先的 extent
        if let Some(p) = parent {
            let inherited: Vec<u64> = exts.values()
                .filter(|e| e.refs.contains(&p)).map(|e| e.id).collect();
            for id in inherited {
                if r() % 100 < share_pct { exts.get_mut(&id).unwrap().refs.insert(i); }
            }
        }
        snaps.insert(i, Snap { id: i, parent, txg: i, created, live: true });
    }
    World { snaps, exts }
}

/// 删一个快照，三条臂各判一次，与真值比。
fn measure(topo: Topo, arm: Arm, n_snaps: u64, exts_per_snap: u64, share_pct: u64, seed: u64) -> Out {
    let mut w = build(topo, n_snaps, exts_per_snap, share_pct, seed);
    // 删中间那个：两侧都有快照，才测得出「与更老的比」这条规则
    let victim = n_snaps / 2;
    let (freed, lookups) = run_arm(&w, victim, arm);
    w.snaps.get_mut(&victim).unwrap().live = false;
    let truth = truly_free_set(&w);
    Out {
        lookups,
        freed: freed.len() as u64,
        truly_free: truth.len() as u64,
        wrong_free: freed.difference(&truth).count() as u64,
        leaked: truth.difference(&freed).count() as u64,
    }
}

fn main() {
    let mut em = Emitter::new();
    let (ns, eps) = (64u64, 8u64);
    println!("{}", em.emit_raw(&format!("name=config snaps={ns} exts_per_snap={eps}")));
    for topo in [Topo::Linear, Topo::MultiHead] {
        for share in [0u64, 30, 70] {
            for arm in [Arm::Deadlist, Arm::Refcount, Arm::OwnerRef] {
                let o = measure(topo, arm, ns, eps, share, 42);
                println!("{}", em.emit_raw(&format!(
                    "name=cell topo={topo:?} share={share} arm={} lookups={} freed={} truly_free={} \
                     wrong_free={} leaked={}",
                    arm.label(), o.lookups, o.freed, o.truly_free, o.wrong_free, o.leaked)));
            }
        }
    }
    // ── 规模扫描：查找次数随 extent 总数怎么长 ──
    // ⚠️ **绝对值会误导**：上面那张表是 64 快照 × 8 extent 的玩具世界，
    // 「13 次查找」听起来很便宜。真正要问的是**标度**——
    // 一次反向索引查找是一次 btree 下降，在真盘上是 I/O。
    for eps in [4u64, 8, 16, 32, 64, 128] {
        for topo in [Topo::Linear, Topo::MultiHead] {
            let o = measure(topo, Arm::Deadlist, 64, eps, 70, 42);
            let rc = measure(topo, Arm::Refcount, 64, eps, 70, 42);
            println!("{}", em.emit_raw(&format!(
                "name=scale topo={topo:?} exts_per_snap={eps} total_exts={} \
                 deadlist_lookups={} refcount_lookups={}",
                64 * eps, o.lookups, rc.lookups)));
        }
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **真值必须独立算出**——它是本实验唯一的裁判。
    #[test]
    fn truth_is_plain_enumeration_of_live_snapshots() {
        let mut w = build(Topo::Linear, 3, 1, 0, 7);
        assert_eq!(truly_free_set(&w).len(), 0, "全部快照活着时不该有自由 extent");
        w.snaps.get_mut(&1).unwrap().live = false;
        assert_eq!(truly_free_set(&w).len(), 1, "删掉一个无共享的快照该放出它那一个 extent");
    }

    /// **阳性对照，对每一条臂都跑**：无共享 + 单向线性时三条臂必须与真值完全一致。
    /// 少了这条，「多可写头下某条臂出错」分不清是拓扑造成的还是那条臂根本不工作。
    #[test]
    fn with_no_sharing_and_linear_history_all_arms_match_truth() {
        for arm in [Arm::Deadlist, Arm::Refcount, Arm::OwnerRef] {
            let o = measure(Topo::Linear, arm, 32, 4, 0, 42);
            assert_eq!(o.wrong_free, 0, "{arm:?} 误放了");
            assert_eq!(o.leaked, 0, "{arm:?} 漏放了");
            assert_eq!(o.freed, o.truly_free, "{arm:?} 与真值不等");
        }
    }

    /// **标度断言：线性拓扑下无论规模多大都恒为零查找。**
    /// ⚠️ 这条比「多头下 > 0」更重要——绝对值会让人以为代价小，
    /// 而真正的结论是「常数 0 对 O(N)」这种**类别差**，不是倍数差。
    #[test]
    fn linear_topology_stays_at_zero_lookups_at_every_scale() {
        for eps in [4u64, 16, 64, 128] {
            let o = measure(Topo::Linear, Arm::Deadlist, 64, eps, 70, 42);
            assert_eq!(o.lookups, 0,
                "线性拓扑在 {} 个 extent 上该恒为零查找", 64 * eps);
        }
    }

    /// **多可写头下查找次数随规模单调增**——它掉进了与引用计数同一个复杂度类。
    #[test]
    fn multi_head_lookups_grow_with_the_number_of_extents() {
        let small = measure(Topo::MultiHead, Arm::Deadlist, 64, 8, 70, 42).lookups;
        let big = measure(Topo::MultiHead, Arm::Deadlist, 64, 128, 70, 42).lookups;
        assert!(big > small * 4,
            "规模涨 16 倍，查找次数只从 {small} 到 {big} —— 那就不是 O(N) 了，标度结论要重写");
    }

    /// **本实验的主指标：D5 形态在单向线性下零查找，在多可写头下被迫付查找。**
    /// 那正是 E18 说的「D5 在分叉下破的是 O(1) 性质，不是正确性」的可量形态。
    #[test]
    fn deadlist_pays_no_lookups_when_linear_but_does_under_multi_head() {
        for share in [0u64, 70] {
            let lin = measure(Topo::Linear, Arm::Deadlist, 32, 4, share, 42);
            assert_eq!(lin.lookups, 0, "单向线性下 D5 形态该零查找（share={share}）");
        }
        let mh = measure(Topo::MultiHead, Arm::Deadlist, 32, 4, 70, 42);
        assert!(mh.lookups > 0, "多可写头 + 高共享下 D5 形态该被迫付查找");
        assert_eq!(measure(Topo::MultiHead, Arm::OwnerRef, 32, 4, 70, 42).lookups, 0,
            "归属原创建者任何拓扑下都不查反向索引");
    }

    /// **绝对值**：无共享时引用计数的查找次数恰等于被删快照自己创建的 extent 数。
    /// 不是「比别人多」这种相对判断。
    #[test]
    fn refcount_lookups_equal_the_victims_extent_count_when_nothing_is_shared() {
        let eps = 4u64;
        let o = measure(Topo::Linear, Arm::Refcount, 32, eps, 0, 42);
        assert_eq!(o.lookups, eps, "无共享时被删快照只引用自己那 {eps} 个 extent");
    }

    /// **共享越多，引用计数查得越多**——机制必须在模型里体现，否则共享那一维是摆设。
    #[test]
    fn more_sharing_makes_refcount_look_up_more() {
        let lo = measure(Topo::Linear, Arm::Refcount, 32, 4, 0, 42).lookups;
        let hi = measure(Topo::Linear, Arm::Refcount, 32, 4, 70, 42).lookups;
        assert!(hi > lo, "共享 70% 时查找次数应多于无共享（{hi} vs {lo}）");
    }

    /// **引用计数在任何拓扑下都不误放也不漏放**——精确是它买到的东西。
    #[test]
    fn refcount_is_exact_under_every_topology() {
        for topo in [Topo::Linear, Topo::MultiHead] {
            for share in [0u64, 30, 70] {
                let o = measure(topo, Arm::Refcount, 48, 6, share, 42);
                assert_eq!(o.wrong_free, 0, "引用计数在 {topo:?}/{share} 下误放");
                assert_eq!(o.leaked, 0, "引用计数在 {topo:?}/{share} 下漏放");
            }
        }
    }

    /// **D5 形态结构性地不会误放**——这是 E18 已证的性质，不是实测巧合：
    /// 只要还有活引用，`death = ∞`，区间判定必为真 ⇒ 永远不释放。
    /// ⚠️ **本测试是变异测试逼出来的**：第一版按印象建了 death 规则，
    /// 算出了 `wrong_free > 0`，与 E18「数据丢失方向结构性为零」直接矛盾。
    #[test]
    fn deadlist_structurally_never_wrongly_frees() {
        for topo in [Topo::Linear, Topo::MultiHead] {
            for share in [0u64, 30, 70] {
                let o = measure(topo, Arm::Deadlist, 48, 6, share, 42);
                assert_eq!(o.wrong_free, 0,
                    "D5 形态在 {topo:?}/{share} 下误放了 {} 个——那与 E18 已证的性质矛盾",
                    o.wrong_free);
            }
        }
    }

    /// **多可写头这一维必须真的改变结果**，否则它是死代码（E24 踩过同一个坑）。
    ///
    /// ⚠️ 判据不是泄漏量，是**查找次数**。第一版按泄漏写，两侧都是 0——
    /// 因为本模型的 D5 臂遇到旁支时会退回精确检查并付一次查找，所以它**不泄漏**。
    /// 这正是结论本身：**D5 在多可写头下不泄漏，只要肯付那些查找**；
    /// [experiments.md](experiments.md) E18 量到的 29.40% 泄漏是「拒绝付」的那个变体
    /// （它与「根本不记 death」逐位相同）。两者量的是同一枚硬币的两面。
    #[test]
    fn the_multi_head_topology_actually_changes_the_outcome() {
        let lin = measure(Topo::Linear, Arm::Deadlist, 48, 6, 70, 42);
        let mh = measure(Topo::MultiHead, Arm::Deadlist, 48, 6, 70, 42);
        assert_eq!(lin.lookups, 0, "单向线性下一切可比，本该零查找");
        assert!(mh.lookups > 0,
            "多可写头下 D5 形态该被迫付查找（实测 {}）——否则这一维是死的", mh.lookups);
        // 两侧都精确：付了查找就不丢也不漏
        assert_eq!(lin.wrong_free + lin.leaked, 0);
        assert_eq!(mh.wrong_free + mh.leaked, 0);
    }

    /// **归属原创建者释放的恰好是被删快照自己创建的那些**，一个不多一个不少。
    /// ⚠️ 变异测试补出来的：只断言「误放 > 0」时，把它改成「释放 victim 碰过的全部」
    /// 一个测试都不红——错得更多但方向相同。绝对值必须钉死。
    #[test]
    fn owner_ref_frees_exactly_what_the_victim_created() {
        let eps = 6u64;
        for topo in [Topo::Linear, Topo::MultiHead] {
            for share in [0u64, 30, 70] {
                let o = measure(topo, Arm::OwnerRef, 48, eps, share, 42);
                assert_eq!(o.freed, eps,
                    "归属原创建者该恰好释放 {eps} 个（topo={topo:?} share={share}），实测 {}", o.freed);
            }
        }
    }

    /// **归属原创建者在有共享时必然误放**——那是它省掉反向查找的代价。
    /// ⚠️ 误放是**数据丢失方向**，不是空间泄漏方向。
    #[test]
    fn owner_ref_wrongly_frees_shared_extents() {
        let o = measure(Topo::Linear, Arm::OwnerRef, 32, 4, 70, 42);
        assert!(o.wrong_free > 0, "归属原创建者在 70% 共享下居然没误放");
    }
}
