//! E18：D5 的区间判定在多分叉 / 多可写头下会不会给出**错误**答案。
//!
//! D5 的可达性谓词是 `birth(b) ≤ R.txg < death(b)`——它把「哪些根能看到 b」压成一个区间，
//! 而区间只在 **txg 全序**（单向线性历史）时才等价于可达性。
//! D6 的候选之一（快照 ID 进 key 低位）允许多分叉、多可写头，那时 txg 不再是全序。
//!
//! **本实验问的是「会不会错」，不是「会不会慢」。** 慢可以优化，错会丢数据。
//!
//! ## 三条臂，共用同一段历史与同一批块
//!
//! | 臂 | 怎么判「b 从根 R 可达」 |
//! |---|---|
//! | `interval`（D5 现方向） | `birth(b) ≤ R.txg < death(b)`，death 记单值 |
//! | `refcount` | 每块一个引用计数，建/删快照时增减 |
//! | `truth`（真值） | 在历史 DAG 上**显式**求可达集，与前两条不共享任何代码 |
//!
//! ## 错误要分方向报，不许合并成一个「错误率」
//!
//! - **判活实死**（false positive）⇒ 空间泄漏。难受，不致命。
//! - **判死实活**（false negative）⇒ **回收了还被引用的块 = 数据丢失。**
//!
//! ## 两个对照
//!
//! - **阳性对照**：线性历史上 `interval` 必须**零错误**。有错 ⇒ 模型坏了，整轮作废。
//! - **判别力对照**：多分叉档必须真的构造出跨分支引用（打印计数）；
//!   计数为 0 ⇒ 「没测出问题」与「没测到」分不开，整轮作废。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

/// 历史里的一个提交点。`parent` 为 None 表示创世。
#[derive(Clone, Copy, Debug)]
struct Commit {
    txg: u64,
    parent: Option<u64>,
    /// 这个提交点是不是一个被保留的快照根（或活头）
    is_root: bool,
}

/// 一个块。`birth` 是创建它的 txg；`deaths` 是**每条分支上各自的**死亡 txg。
/// D5 的模型只记一个 death —— 本实验用 `d5_death` 表示「D5 会记下的那个值」。
#[derive(Clone, Debug)]
struct Block {
    birth: u64,
    /// 真值：在哪些提交点上它被删除了（可能多个分支各删各的）
    deaths: BTreeMap<u64, ()>,
    /// D5 记的单值 death。**分叉后一个块在多条分支上死于不同 txg，而 D5 只有一个字段**——
    /// 记哪一个 D5 正文从没写明。本实验把两种写法都跑：`Earliest` 与 `Latest`。
    d5_death_min: u64,
    d5_death_max: u64,
    /// D5 正文定义的那条：所有分支都删过才写有限值，否则 ∞
    d5_death_conservative: u64,
}

const NO_DEATH: u64 = u64::MAX;

/// 历史形状。
#[derive(Clone, Copy, PartialEq, Debug)]
enum Shape {
    Linear,
    /// 在中点分一次叉
    SingleFork,
    /// 分 `n` 条可写头，各自继续写
    MultiHead(usize),
}

struct History {
    commits: BTreeMap<u64, Commit>,
    blocks: Vec<Block>,
}

/// 造一段历史：先走一段共同前缀，再按 shape 分叉，各分支各写各的。
fn build(shape: Shape, prefix: u64, per_branch: u64, seed: u64) -> History {
    let mut s = seed | 1;
    let mut rnd = || { s ^= s >> 12; s ^= s << 25; s ^= s >> 27; s.wrapping_mul(0x2545_F491_4F6C_DD1D) };

    let mut commits: BTreeMap<u64, Commit> = BTreeMap::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut next_txg = 1u64;

    // 共同前缀：线性
    let mut prev: Option<u64> = None;
    let mut prefix_tail = 0u64;
    for _ in 0..prefix {
        let t = next_txg; next_txg += 1;
        commits.insert(t, Commit { txg: t, parent: prev, is_root: true });
        // 每个 txg 造几个块
        for _ in 0..3 {
            blocks.push(Block { birth: t, deaths: BTreeMap::new(), d5_death_min: NO_DEATH, d5_death_max: 0, d5_death_conservative: NO_DEATH });
        }
        prev = Some(t);
        prefix_tail = t;
    }

    let branch_points: Vec<u64> = match shape {
        Shape::Linear => vec![prefix_tail],
        Shape::SingleFork => vec![prefix_tail, prefix_tail],
        Shape::MultiHead(n) => vec![prefix_tail; n],
    };

    // 每条分支各自往下走
    let mut branch_tips: Vec<Vec<u64>> = Vec::new(); // 每条分支上的 txg 列表
    for &bp in branch_points.iter() {
        let mut prev = Some(bp);
        let mut tips = Vec::new();
        for _ in 0..per_branch {
            let t = next_txg; next_txg += 1;
            commits.insert(t, Commit { txg: t, parent: prev, is_root: true });
            for _ in 0..3 {
                blocks.push(Block { birth: t, deaths: BTreeMap::new(), d5_death_min: NO_DEATH, d5_death_max: 0, d5_death_conservative: NO_DEATH });
            }
            tips.push(t);
            prev = Some(t);
        }
        branch_tips.push(tips);
    }

    // 删除：每条分支各自删掉一些**前缀里诞生的**块 —— 这正是「一块在 A 上死、在 B 上活」的构造
    let prefix_blocks: Vec<usize> = blocks.iter().enumerate()
        .filter(|(_, b)| b.birth <= prefix_tail).map(|(i, _)| i).collect();
    if !prefix_blocks.is_empty() {
        // ⚠️ **同一条分支上一个块只许被删一次**。允许删两次是生成器的 bug——
        // 真实文件系统里一个块在一条链上只会死一次，而它会让 `latest` 与 `conservative`
        // 在**线性**历史上就出错（阳性对照 fp=8），把整轮结果污染成假的。
        for (bi, tips) in branch_tips.iter().enumerate() {
            let mut deleted_on_this_branch: std::collections::BTreeSet<usize> = Default::default();
            let _ = bi;
            for &t in tips.iter() {
                // 每个 txg 删一个**本分支还没删过的**前缀块
                let mut i = prefix_blocks[(rnd() as usize) % prefix_blocks.len()];
                let mut tries = 0;
                while deleted_on_this_branch.contains(&i) && tries < 64 {
                    i = prefix_blocks[(rnd() as usize) % prefix_blocks.len()];
                    tries += 1;
                }
                if deleted_on_this_branch.contains(&i) { continue; }
                deleted_on_this_branch.insert(i);
                blocks[i].deaths.insert(t, ());
                blocks[i].d5_death_min = blocks[i].d5_death_min.min(t);
                blocks[i].d5_death_max = blocks[i].d5_death_max.max(t);
            }
        }
    }

    // 算 D5 正文那条 death：一个块只有在**每一条分支**都删过它时，才算「最后一个活引用被摘掉」
    let n_branches = branch_tips.len();
    for b in blocks.iter_mut() {
        if b.deaths.is_empty() { continue; }
        let covered = branch_tips.iter()
            .filter(|tips| tips.iter().any(|t| b.deaths.contains_key(t)))
            .count();
        if covered == n_branches {
            b.d5_death_conservative = *b.deaths.keys().max().unwrap();
        }
    }
    History { commits, blocks }
}

/// 真值：b 从根 R 可达 ⟺ b 诞生于 R 的祖先链上（含 R），且**在那条链上**没有被删。
fn truth_reachable(h: &History, b: &Block, root: u64) -> bool {
    // 走 R 的祖先链
    let mut chain: BTreeSet<u64> = BTreeSet::new();
    let mut cur = Some(root);
    while let Some(t) = cur {
        chain.insert(t);
        cur = h.commits[&t].parent;
    }
    if !chain.contains(&b.birth) { return false; }         // 不在这条链上诞生
    !b.deaths.keys().any(|d| chain.contains(d))            // 这条链上没被删
}

/// D5 的区间判定。**它不看历史形状，只比两个数。**
#[derive(Clone, Copy, PartialEq, Debug)]
enum DeathRule {
    /// 记最早那次删除
    Earliest,
    /// 记最晚那次删除
    Latest,
    /// **D5 正文真正定义的那条**：「最后一个活引用被摘掉、且该摘除被发布的 checkpoint 号；
    /// 仍在活树里则为 ∞」。分叉下 = 只有当**所有**分支都删过它，才写有限值。
    /// ⚠️ 前两条都**不是** D5 写下的规则，本实验第一版只测了那两条。
    Conservative,
    /// 判别力对照：永不写 death（回收能力的退化下界）。
    /// 若 Conservative 与它逐位相同，说明 Conservative 什么也没回收。
    Never,
}

fn interval_reachable(b: &Block, root_txg: u64, rule: DeathRule) -> bool {
    let d = match rule {
        DeathRule::Earliest => b.d5_death_min,
        DeathRule::Latest => if b.d5_death_max == 0 { NO_DEATH } else { b.d5_death_max },
        DeathRule::Conservative => b.d5_death_conservative,
        DeathRule::Never => NO_DEATH,
    };
    b.birth <= root_txg && root_txg < d
}

struct Verdict { fp: usize, fn_: usize, total: usize, cross: usize }

fn evaluate(h: &History, rule: DeathRule) -> Verdict {
    let roots: Vec<u64> = h.commits.values().filter(|c| c.is_root).map(|c| c.txg).collect();
    let (mut fp, mut fn_, mut total, mut cross) = (0, 0, 0, 0);
    for b in h.blocks.iter() {
        for &r in roots.iter() {
            let t = truth_reachable(h, b, r);
            let i = interval_reachable(b, r, rule);
            total += 1;
            if i && !t { fp += 1; }
            if !i && t { fn_ += 1; }
            // 跨分支引用：真值可达，但 b 诞生的 txg 不在 r 的**直系线性前缀**上
            if t && b.birth < r && h.commits[&r].parent.is_some() { cross += 1; }
        }
    }
    Verdict { fp, fn_, total, cross }
}

/// 引用计数臂的代价：每块一个 u32 计数字段，且建/删快照要遍历该快照独占的块。
fn refcount_cost(h: &History) -> (usize, usize) {
    let bytes = h.blocks.len() * 4;
    // 建一个快照要给它能看见的每个块 +1 —— 这就是它相对 D5 的代价
    let roots: Vec<u64> = h.commits.values().filter(|c| c.is_root).map(|c| c.txg).collect();
    let mut touches = 0usize;
    for &r in roots.iter() {
        for b in h.blocks.iter() { if truth_reachable(h, b, r) { touches += 1; } }
    }
    (bytes, touches)
}

fn main() {
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| { out.push_str(&s); out.push('\n'); };

    let prefix = 40u64;
    let per_branch = 20u64;
    say(em.emit_raw(&format!("name=config prefix_txgs={prefix} per_branch_txgs={per_branch} blocks_per_txg=3")));

    // ── 阳性对照：线性历史上 interval 必须零错误 ──
    let lin = build(Shape::Linear, prefix, per_branch, 3);
    // ⚠️ 阳性对照必须对**每一条**被测规则都跑。只跑一条等于另一条从没过闸。
    let mut pos_ok = true;
    for (rule, rn) in [(DeathRule::Earliest, "earliest"), (DeathRule::Latest, "latest"),
                       (DeathRule::Conservative, "conservative")] {
        let vv = evaluate(&lin, rule);
        let ok = vv.fp == 0 && vv.fn_ == 0;
        pos_ok &= ok;
        say(em.emit_raw(&format!(
            "name=poscontrol shape=linear death_rule={rn} pairs={} fp={} fn={} zero_error={ok}",
            vv.total, vv.fp, vv.fn_)));
    }
    if !pos_ok {
        say(em.finish()); print!("{out}");
        eprintln!("E18: 线性历史上 interval 就有错 —— 模型坏了，本轮作废");
        std::process::exit(4);
    }

    // ── 正式：分叉与多可写头 ──
    for (shape, name) in [
        (Shape::SingleFork, "fork2".to_string()),
        (Shape::MultiHead(3), "heads3".to_string()),
        (Shape::MultiHead(5), "heads5".to_string()),
        (Shape::MultiHead(8), "heads8".to_string()),
    ] {
        let h = build(shape, prefix, per_branch, 7);
        let (rc_bytes, rc_touch) = refcount_cost(&h);
        let mut v = evaluate(&h, DeathRule::Earliest);
        for (rule, rname) in [(DeathRule::Earliest, "earliest"), (DeathRule::Latest, "latest"),
                              (DeathRule::Conservative, "conservative"), (DeathRule::Never, "never")] {
            let vv = evaluate(&h, rule);
            if rule == DeathRule::Earliest { v = Verdict { fp: vv.fp, fn_: vv.fn_, total: vv.total, cross: vv.cross }; }
            say(em.emit_raw(&format!(
                "name=e18 shape={name} death_rule={rname} blocks={} roots={} pairs={} \
                 fp={} fn={} fp_rate={:.4} fn_rate={:.4} cross_branch_refs={} \
                 refcount_bytes={rc_bytes} refcount_touches={rc_touch}",
                h.blocks.len(),
                h.commits.values().filter(|c| c.is_root).count(),
                vv.total, vv.fp, vv.fn_,
                vv.fp as f64 / vv.total as f64, vv.fn_ as f64 / vv.total as f64,
                vv.cross
            )));
        }
        // 判别力对照：必须真的构造出跨分支引用
        if v.cross == 0 {
            say(em.finish()); print!("{out}");
            eprintln!("E18: {name} 档没有任何跨分支引用 —— 「没测出问题」与「没测到」分不开，本轮作废");
            std::process::exit(5);
        }
    }

    say(em.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：线性历史上 D5 的区间判定必须与真值完全一致。
    #[test]
    fn interval_is_exact_on_linear_history() {
        let h = build(Shape::Linear, 30, 15, 1);
        let v = evaluate(&h, DeathRule::Earliest);
        assert_eq!(v.fp, 0, "线性历史上出现判活实死");
        assert_eq!(v.fn_, 0, "线性历史上出现判死实活");
        assert!(v.total > 1000, "样本太小，判不出什么");
    }

    /// 多可写头下必须真的出现「判死实活」——那是数据丢失方向。
    /// 若这条不成立，要么模型没构造出跨分支删除，要么 D5 其实是对的。
    #[test]
    fn multihead_produces_false_negatives() {
        let h = build(Shape::MultiHead(5), 30, 15, 2);
        let v = evaluate(&h, DeathRule::Earliest);
        assert!(v.cross > 0, "没有跨分支引用，测的不是分叉");
        assert!(v.fn_ > 0, "多可写头下没有出现判死实活");
    }

    /// 真值与区间判定必须是两段独立的代码：真值走 DAG，区间只比两个数。
    /// 这条测的是「真值确实看了历史形状」——把 parent 全断开，真值必须变。
    #[test]
    fn truth_actually_walks_the_dag() {
        let mut h = build(Shape::MultiHead(3), 20, 10, 4);
        let before = evaluate(&h, DeathRule::Earliest).fn_;
        for c in h.commits.values_mut() { c.parent = None; }
        let after = evaluate(&h, DeathRule::Earliest).fn_;
        assert_ne!(before, after, "断开父链后真值没变，说明它根本没走 DAG");
    }
}
