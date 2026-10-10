//! K4（m3-batch56-code-r1 opus）：发布路径上的 inode 叶形状。五片多的叶上一次次 unlink（多半挑不是最右、记录最少的那片里的号，
//! 也删最大的活号、随便删一个，偶尔建一个），每次发布之后核两句：不是最右的叶至少 117 条（P）、相邻两片之和至少 234（Q）。
//! P 成立时「删空的那一片两边都有邻居」走不到（删空的只能是最右那片），第 56 批等价留档那一族里中间一片只有 1 条的形状就不可达。
mod common_b56;

use common_b56::{Knobs, Pool};
use singlefs_harness::history::SeededRandomSource;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

#[test]
fn opus_probe_b56_leaf_shape_stays_above_the_floor() {
    let first = env_u64("OPUS_SEED_FIRST", 1);
    let last = env_u64("OPUS_SEED_LAST", 4);
    let unlinks = env_u64("OPUS_UNLINKS", 400);
    let knobs = Knobs { tombstone: 0, lineage: 0, retirement: 0, node_entries: 0 };
    let base = Pool::after_the_first_file();
    let lines: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = (first..=last)
            .map(|seed| {
                let base = &base;
                scope.spawn(move || {
                    let mut random = SeededRandomSource::from_seed(seed);
                    let mut pool = base.clone();
                    pool.create(233 * 4 + 1 + random.below(60));
                    let (mut p_broken, mut q_broken, mut smallest_non_rightmost, mut most_leaves) = (0u64, 0u64, usize::MAX, 0usize);
                    // 迭代上界：unlinks。
                    for _ in 0..unlinks {
                        let live = pool.live_inodes();
                        let choice = random.below(100);
                        if choice < 5 || live.is_empty() {
                            pool.create(1);
                        } else if choice < 60 {
                            let leaves = &pool.output.inode_leaf_containers;
                            let candidate = leaves[..leaves.len().saturating_sub(1).max(1)]
                                .iter()
                                .filter(|leaf| leaf.contents.records.iter().any(|record| record.inode != 1))
                                .min_by_key(|leaf| leaf.contents.records.len());
                            let inodes: Vec<u64> = candidate.map_or_else(Vec::new, |leaf| leaf.contents.records.iter().map(|record| record.inode).filter(|inode| *inode != 1).collect());
                            let inode = if inodes.is_empty() { *live.iter().max().expect("非空") } else { inodes[usize::try_from(random.below(u64::try_from(inodes.len()).expect("小"))).expect("小")] };
                            pool.unlink(inode, knobs);
                        } else if choice < 80 {
                            pool.unlink(*live.iter().max().expect("非空"), knobs);
                        } else {
                            pool.unlink(live[usize::try_from(random.below(u64::try_from(live.len()).expect("小"))).expect("小")], knobs);
                        }
                        let counts: Vec<usize> = pool.output.inode_leaf_containers.iter().map(|leaf| leaf.contents.records.len()).collect();
                        most_leaves = most_leaves.max(counts.len());
                        if counts.len() >= 2 {
                            for count in &counts[..counts.len() - 1] {
                                smallest_non_rightmost = smallest_non_rightmost.min(*count);
                                p_broken += u64::from(*count < 117);
                            }
                        }
                        q_broken += counts.windows(2).filter(|pair| pair[0] + pair[1] < 234).count() as u64;
                    }
                    format!(
                        "OPUS_K4_SHAPE seed={seed} unlinks={unlinks} most_leaves={most_leaves} smallest_non_rightmost={smallest_non_rightmost} non_rightmost_below_117={p_broken} adjacent_below_234={q_broken} final={}",
                        pool.tree_summary()
                    )
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程")).collect()
    });
    for line in lines {
        eprintln!("{line}");
    }
}
