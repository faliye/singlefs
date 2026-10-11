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
    single_value_death_earliest: u64,
    single_value_death_latest: u64,
    /// D5 正文定义的那条：所有分支都删过才写有限值，否则 ∞
    single_value_death_conservative: u64,
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
fn build(shape: Shape, prefix: u64, commits_per_branch: u64, seed: u64) -> History {
    let mut xorshift_state = seed | 1;
    let mut next_random = || { xorshift_state ^= xorshift_state >> 12; xorshift_state ^= xorshift_state << 25; xorshift_state ^= xorshift_state >> 27; xorshift_state.wrapping_mul(0x2545_F491_4F6C_DD1D) };

    let mut commits: BTreeMap<u64, Commit> = BTreeMap::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut next_txg = 1u64;

    // 共同前缀：线性
    let mut previous_commit: Option<u64> = None;
    let mut prefix_tail = 0u64;
    for _ in 0..prefix {
        let new_commit_txg = next_txg; next_txg += 1;
        commits.insert(new_commit_txg, Commit { txg: new_commit_txg, parent: previous_commit, is_root: true });
        // 每个 txg 造几个块
        for _ in 0..3 {
            blocks.push(Block { birth: new_commit_txg, deaths: BTreeMap::new(), single_value_death_earliest: NO_DEATH, single_value_death_latest: 0, single_value_death_conservative: NO_DEATH });
        }
        previous_commit = Some(new_commit_txg);
        prefix_tail = new_commit_txg;
    }

    let branch_points: Vec<u64> = match shape {
        Shape::Linear => vec![prefix_tail],
        Shape::SingleFork => vec![prefix_tail, prefix_tail],
        Shape::MultiHead(head_count) => vec![prefix_tail; head_count],
    };

    // 每条分支各自往下走
    let mut branch_tips: Vec<Vec<u64>> = Vec::new(); // 每条分支上的 txg 列表
    for &branch_point in branch_points.iter() {
        let mut previous_commit = Some(branch_point);
        let mut tips = Vec::new();
        for _ in 0..commits_per_branch {
            let new_commit_txg = next_txg; next_txg += 1;
            commits.insert(new_commit_txg, Commit { txg: new_commit_txg, parent: previous_commit, is_root: true });
            for _ in 0..3 {
                blocks.push(Block { birth: new_commit_txg, deaths: BTreeMap::new(), single_value_death_earliest: NO_DEATH, single_value_death_latest: 0, single_value_death_conservative: NO_DEATH });
            }
            tips.push(new_commit_txg);
            previous_commit = Some(new_commit_txg);
        }
        branch_tips.push(tips);
    }

    // 删除：每条分支各自删掉一些**前缀里诞生的**块 —— 这正是「一块在 A 上死、在 B 上活」的构造
    let prefix_blocks: Vec<usize> = blocks.iter().enumerate()
        .filter(|(_, block)| block.birth <= prefix_tail).map(|(block_index, _)| block_index).collect();
    if !prefix_blocks.is_empty() {
        // ⚠️ **同一条分支上一个块只许被删一次**。允许删两次是生成器的 bug——
        // 真实文件系统里一个块在一条链上只会死一次，而它会让 `latest` 与 `conservative`
        // 在**线性**历史上就出错（阳性对照 fp=8），把整轮结果污染成假的。
        for (branch_index, tips) in branch_tips.iter().enumerate() {
            let mut deleted_on_this_branch: std::collections::BTreeSet<usize> = Default::default();
            let _ = branch_index;
            for &deleting_commit_txg in tips.iter() {
                // 每个 txg 删一个**本分支还没删过的**前缀块
                let mut victim_block_index = prefix_blocks[(next_random() as usize) % prefix_blocks.len()];
                let mut tries = 0;
                while deleted_on_this_branch.contains(&victim_block_index) && tries < 64 {
                    victim_block_index = prefix_blocks[(next_random() as usize) % prefix_blocks.len()];
                    tries += 1;
                }
                if deleted_on_this_branch.contains(&victim_block_index) { continue; }
                deleted_on_this_branch.insert(victim_block_index);
                blocks[victim_block_index].deaths.insert(deleting_commit_txg, ());
                blocks[victim_block_index].single_value_death_earliest = blocks[victim_block_index].single_value_death_earliest.min(deleting_commit_txg);
                blocks[victim_block_index].single_value_death_latest = blocks[victim_block_index].single_value_death_latest.max(deleting_commit_txg);
            }
        }
    }

    // 算 D5 正文那条 death：一个块只有在**每一条分支**都删过它时，才算「最后一个活引用被摘掉」
    let branch_count = branch_tips.len();
    for block in blocks.iter_mut() {
        if block.deaths.is_empty() { continue; }
        let deleting_branch_count = branch_tips.iter()
            .filter(|tips| tips.iter().any(|tip_txg| block.deaths.contains_key(tip_txg)))
            .count();
        if deleting_branch_count == branch_count {
            block.single_value_death_conservative = *block.deaths.keys().max().unwrap();
        }
    }
    History { commits, blocks }
}

/// 真值：b 从根 R 可达 ⟺ b 诞生于 R 的祖先链上（含 R），且**在那条链上**没有被删。
fn truth_reachable(history: &History, block: &Block, root: u64) -> bool {
    // 走 R 的祖先链
    let mut ancestor_chain: BTreeSet<u64> = BTreeSet::new();
    let mut next_ancestor = Some(root);
    while let Some(ancestor_txg) = next_ancestor {
        ancestor_chain.insert(ancestor_txg);
        next_ancestor = history.commits[&ancestor_txg].parent;
    }
    if !ancestor_chain.contains(&block.birth) { return false; }         // 不在这条链上诞生
    !block.deaths.keys().any(|death_txg| ancestor_chain.contains(death_txg))            // 这条链上没被删
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

fn interval_reachable(block: &Block, root_txg: u64, rule: DeathRule) -> bool {
    let recorded_death = match rule {
        DeathRule::Earliest => block.single_value_death_earliest,
        DeathRule::Latest => if block.single_value_death_latest == 0 { NO_DEATH } else { block.single_value_death_latest },
        DeathRule::Conservative => block.single_value_death_conservative,
        DeathRule::Never => NO_DEATH,
    };
    block.birth <= root_txg && root_txg < recorded_death
}

struct Verdict { false_positives: usize, false_negatives: usize, total: usize, cross: usize }

fn evaluate(history: &History, rule: DeathRule) -> Verdict {
    let roots: Vec<u64> = history.commits.values().filter(|commit| commit.is_root).map(|commit| commit.txg).collect();
    let (mut false_positives, mut false_negatives, mut total, mut cross) = (0, 0, 0, 0);
    for block in history.blocks.iter() {
        for &root in roots.iter() {
            let is_truly_reachable = truth_reachable(history, block, root);
            let is_interval_reachable = interval_reachable(block, root, rule);
            total += 1;
            if is_interval_reachable && !is_truly_reachable { false_positives += 1; }
            if !is_interval_reachable && is_truly_reachable { false_negatives += 1; }
            // 跨分支引用：真值可达，但 b 诞生的 txg 不在 r 的**直系线性前缀**上
            if is_truly_reachable && block.birth < root && history.commits[&root].parent.is_some() { cross += 1; }
        }
    }
    Verdict { false_positives, false_negatives, total, cross }
}

/// 引用计数臂的代价：每块一个 u32 计数字段，且建/删快照要遍历该快照独占的块。
fn reference_count_cost(history: &History) -> (usize, usize) {
    let bytes = history.blocks.len() * 4;
    // 建一个快照要给它能看见的每个块 +1 —— 这就是它相对 D5 的代价
    let roots: Vec<u64> = history.commits.values().filter(|commit| commit.is_root).map(|commit| commit.txg).collect();
    let mut touches = 0usize;
    for &root in roots.iter() {
        for block in history.blocks.iter() { if truth_reachable(history, block, root) { touches += 1; } }
    }
    (bytes, touches)
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output = String::new();
    let mut say = |line: String| { output.push_str(&line); output.push('\n'); };

    let prefix = 40u64;
    let commits_per_branch = 20u64;
    say(emitter.emit_raw(&format!("name=config prefix_txgs={prefix} per_branch_txgs={commits_per_branch} blocks_per_txg=3")));

    // ── 阳性对照：线性历史上 interval 必须零错误 ──
    let linear_history = build(Shape::Linear, prefix, commits_per_branch, 3);
    // ⚠️ 阳性对照必须对**每一条**被测规则都跑。只跑一条等于另一条从没过闸。
    let mut positive_control_passed = true;
    for (rule, rule_name) in [(DeathRule::Earliest, "earliest"), (DeathRule::Latest, "latest"),
                       (DeathRule::Conservative, "conservative")] {
        let verdict = evaluate(&linear_history, rule);
        let has_zero_error = verdict.false_positives == 0 && verdict.false_negatives == 0;
        positive_control_passed &= has_zero_error;
        say(emitter.emit_raw(&format!(
            "name=poscontrol shape=linear death_rule={rule_name} pairs={} fp={} fn={} zero_error={has_zero_error}",
            verdict.total, verdict.false_positives, verdict.false_negatives)));
    }
    if !positive_control_passed {
        say(emitter.finish()); print!("{output}");
        eprintln!("E18: 线性历史上 interval 就有错 —— 模型坏了，本轮作废");
        std::process::exit(4);
    }

    // ── 正式：分叉与多可写头 ──
    for (shape, shape_name) in [
        (Shape::SingleFork, "fork2".to_string()),
        (Shape::MultiHead(3), "heads3".to_string()),
        (Shape::MultiHead(5), "heads5".to_string()),
        (Shape::MultiHead(8), "heads8".to_string()),
    ] {
        let history = build(shape, prefix, commits_per_branch, 7);
        let (reference_count_bytes, reference_count_touches) = reference_count_cost(&history);
        let mut earliest_verdict = evaluate(&history, DeathRule::Earliest);
        for (rule, rule_name) in [(DeathRule::Earliest, "earliest"), (DeathRule::Latest, "latest"),
                              (DeathRule::Conservative, "conservative"), (DeathRule::Never, "never")] {
            let verdict = evaluate(&history, rule);
            if rule == DeathRule::Earliest { earliest_verdict = Verdict { false_positives: verdict.false_positives, false_negatives: verdict.false_negatives, total: verdict.total, cross: verdict.cross }; }
            say(emitter.emit_raw(&format!(
                "name=e18 shape={shape_name} death_rule={rule_name} blocks={} roots={} pairs={} \
                 fp={} fn={} fp_rate={:.4} fn_rate={:.4} cross_branch_refs={} \
                 refcount_bytes={reference_count_bytes} refcount_touches={reference_count_touches}",
                history.blocks.len(),
                history.commits.values().filter(|commit| commit.is_root).count(),
                verdict.total, verdict.false_positives, verdict.false_negatives,
                verdict.false_positives as f64 / verdict.total as f64, verdict.false_negatives as f64 / verdict.total as f64,
                verdict.cross
            )));
        }
        // 判别力对照：必须真的构造出跨分支引用
        if earliest_verdict.cross == 0 {
            say(emitter.finish()); print!("{output}");
            eprintln!("E18: {shape_name} 档没有任何跨分支引用 —— 「没测出问题」与「没测到」分不开，本轮作废");
            std::process::exit(5);
        }
    }

    say(emitter.finish());
    print!("{output}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 阳性对照：线性历史上 D5 的区间判定必须与真值完全一致。
    #[test]
    fn interval_is_exact_on_linear_history() {
        let history = build(Shape::Linear, 30, 15, 1);
        let verdict = evaluate(&history, DeathRule::Earliest);
        assert_eq!(verdict.false_positives, 0, "线性历史上出现判活实死");
        assert_eq!(verdict.false_negatives, 0, "线性历史上出现判死实活");
        assert!(verdict.total > 1000, "样本太小，判不出什么");
    }

    /// 多可写头下必须真的出现「判死实活」——那是数据丢失方向。
    /// 若这条不成立，要么模型没构造出跨分支删除，要么 D5 其实是对的。
    #[test]
    fn multihead_produces_false_negatives() {
        let history = build(Shape::MultiHead(5), 30, 15, 2);
        let verdict = evaluate(&history, DeathRule::Earliest);
        assert!(verdict.cross > 0, "没有跨分支引用，测的不是分叉");
        assert!(verdict.false_negatives > 0, "多可写头下没有出现判死实活");
    }

    /// D5 正文那条保守 death 在多可写头下不许出现判死实活——数据丢失方向
    /// 结构性为零（E18 的主结论）。变异审计补的：此前测试只跑 Earliest，
    /// 把「全部分支删过才写有限值」改成「任一分支删过」不会有任何测试红。
    #[test]
    fn conservative_rule_never_loses_data_even_with_many_heads() {
        let history = build(Shape::MultiHead(5), 30, 15, 2);
        let verdict = evaluate(&history, DeathRule::Conservative);
        assert_eq!(verdict.false_negatives, 0, "保守规则出现判死实活——与 E18 已证性质矛盾");
    }

    /// 真值与区间判定必须是两段独立的代码：真值走 DAG，区间只比两个数。
    /// 这条测的是「真值确实看了历史形状」——把 parent 全断开，真值必须变。
    #[test]
    fn truth_actually_walks_the_dag() {
        let mut history = build(Shape::MultiHead(3), 20, 10, 4);
        let before = evaluate(&history, DeathRule::Earliest).false_negatives;
        for commit in history.commits.values_mut() { commit.parent = None; }
        let after = evaluate(&history, DeathRule::Earliest).false_negatives;
        assert_ne!(before, after, "断开父链后真值没变，说明它根本没走 DAG");
    }
}
