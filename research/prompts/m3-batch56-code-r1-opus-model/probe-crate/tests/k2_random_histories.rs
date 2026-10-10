//! K2-1（m3-batch56-code-r1 opus）：旋钮下的随机历史（建、从记录最少的叶删、删最大的活号、随便删、挂着回退到最近几版之一、
//! 干净地重新可写挂载换实例），每一步之后带同一组旋钮值交池级 checker 的新入口；第 56 批 G2 三判（第一片是 1、相邻加一、每片至少一条）
//! 在合法镜像上一条都不许红。种子 OPUS_SEED_FIRST..=OPUS_SEED_LAST，每个种子 OPUS_OPS 步，OPUS_THREADS 个线程分种子跑。
mod common_b56;

use common_b56::{violations, verdict_name, Knobs, Pool};
use singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs;
use singlefs_core::address::InstanceGeneration;
use singlefs_format::INODE_LEAF_RECORDS;
use singlefs_harness::history::SeededRandomSource;
use singlefs_harness::memory_pool::SparseBlockDevice;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn pick(random: &mut SeededRandomSource, length: usize) -> usize {
    usize::try_from(random.below(u64::try_from(length).expect("小"))).expect("小")
}

fn one_seed(base: &Pool<SparseBlockDevice>, seed: u64, operations: u64) -> String {
    let started = std::time::Instant::now();
    let mut random = SeededRandomSource::from_seed(seed);
    let knobs = Knobs {
        tombstone: 1 + random.below(3),
        lineage: 1 + random.below(2),
        retirement: 1 + random.below(2),
        node_entries: [0usize, 2, 3][pick(&mut random, 3)],
    };
    let mut pool = base.clone();
    pool.allocator.set_deletion_switches(knobs.switches());
    pool.create(u64::from(INODE_LEAF_RECORDS) + 1 + random.below(8));
    let mut versions: Vec<(InstanceGeneration, u64)> = vec![(pool.instance, pool.txg())];
    let mut counts = std::collections::BTreeMap::<&'static str, u64>::new();
    let (mut checks, mut violating, mut held_9_22, mut held_9_37) = (0u64, 0u64, 0u64, 0u64);
    // K4：每一步之后的 inode 叶形状——不是最右的叶至少 117 条（P）、相邻两片之和至少 234（Q）。
    let (mut leaves_below_117_not_rightmost, mut adjacent_pairs_below_234, mut smallest_non_rightmost) = (0u64, 0u64, usize::MAX);
    let mut first_violation = String::from("none");
    // 迭代上界：operations。
    for _ in 0..operations {
        let choice = random.below(100);
        let live = pool.live_inodes();
        let name = if choice < 22 || live.is_empty() {
            pool.create(1 + random.below(3));
            "create"
        } else if choice < 42 {
            let smallest = pool
                .output
                .inode_leaf_containers
                .iter()
                .filter(|leaf| leaf.contents.records.iter().any(|record| record.inode != 1))
                .min_by_key(|leaf| leaf.contents.records.len())
                .expect("有活着的号就有这样一片");
            let candidates: Vec<u64> = smallest.contents.records.iter().map(|record| record.inode).filter(|inode| *inode != 1).collect();
            pool.unlink(candidates[pick(&mut random, candidates.len())], knobs);
            "unlink-smallest-leaf"
        } else if choice < 62 {
            pool.unlink(*live.iter().max().expect("非空"), knobs);
            "unlink-largest"
        } else if choice < 72 {
            pool.unlink(live[pick(&mut random, live.len())], knobs);
            "unlink-any"
        } else if choice < 92 {
            let recent: Vec<(InstanceGeneration, u64)> = versions.iter().rev().take(12).copied().filter(|version| version.1 != pool.txg()).collect();
            if recent.is_empty() {
                "rollback-none"
            } else {
                match pool.roll_back(recent[pick(&mut random, recent.len())]) {
                    Ok(()) => "rollback-ok",
                    Err(_) => "rollback-refused",
                }
            }
        } else {
            match Pool::mounted_from(&pool.image(), knobs) {
                Ok(remounted) => {
                    pool = remounted;
                    "remount"
                }
                Err(_) => "remount-refused",
            }
        };
        *counts.entry(name).or_default() += 1;
        versions.push((pool.instance, pool.txg()));
        let verdicts = check_pool_image_given_the_container_capacity_knobs(&pool.image(), knobs.checker_knobs());
        checks += 1;
        let found = violations(&verdicts);
        if !found.is_empty() {
            violating += 1;
            if first_violation == "none" {
                first_violation = format!("{name}: {found:?}");
            }
        }
        let counts: Vec<usize> = pool.output.inode_leaf_containers.iter().map(|leaf| leaf.contents.records.len()).collect();
        if counts.len() >= 2 {
            for count in &counts[..counts.len() - 1] {
                smallest_non_rightmost = smallest_non_rightmost.min(*count);
                leaves_below_117_not_rightmost += u64::from(*count < 117);
            }
        }
        adjacent_pairs_below_234 += counts.windows(2).filter(|pair| pair[0] + pair[1] < 234).count() as u64;
        held_9_22 += u64::from(verdict_name(&verdicts, "I-9.22") == "Holds");
        held_9_37 += u64::from(verdict_name(&verdicts, "I-9.37") == "Holds");
    }
    format!(
        "OPUS_K2R seed={seed} knobs={knobs:?} ops={counts:?} checks={checks} violating={violating} held_I-9.22={held_9_22} held_I-9.37={held_9_37} leaves_below_117_not_rightmost={leaves_below_117_not_rightmost} adjacent_pairs_below_234={adjacent_pairs_below_234} smallest_non_rightmost={smallest_non_rightmost} final={} first_violation={first_violation} elapsed={:.1}s",
        pool.tree_summary(),
        started.elapsed().as_secs_f64()
    )
}

#[test]
fn opus_probe_b56_random_knob_histories() {
    let first = env_u64("OPUS_SEED_FIRST", 1);
    let last = env_u64("OPUS_SEED_LAST", 2);
    let operations = env_u64("OPUS_OPS", 20);
    let threads = env_u64("OPUS_THREADS", 2);
    let base = Pool::after_the_first_file();
    let seeds: Vec<u64> = (first..=last).collect();
    let lines: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|thread_index| {
                let (base, seeds) = (&base, &seeds);
                scope.spawn(move || {
                    seeds
                        .iter()
                        .copied()
                        .filter(|seed| (seed - first) % threads == thread_index)
                        .map(|seed| {
                            let line = one_seed(base, seed, operations);
                            eprintln!("{line}");
                            line
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
    });
    eprintln!("OPUS_K2R_DONE seeds={}", lines.len());
}
