//! K1-1（m3-batch57-code-r1 opus）：旋钮 ≥ 2 下的随机历史（建、从记录最少的叶删、删最大的活号、随便删、挂着回退到最近几版之一、
//! 干净地重新可写挂载换实例），每一步之后带同一组旋钮值交池级 checker 的新入口；第 57 批 H1 新判「不是最右的那片记录数 == 旋钮值」
//! 在合法镜像上一条都不许红。旋钮 1 下 == 与 ≤ 同义（每片至少一条），所以这里旋钮只取 ≥ 2。
//! 两种取法：OPUS_MODE=all 三棵树都装旋钮（墓碑 2–4、谱系 2–3、退役 2–3）；OPUS_MODE=one 只一棵树装旋钮（2–3），另两棵照产品容量。
//! 种子 OPUS_SEED_FIRST..=OPUS_SEED_LAST，每个种子 OPUS_OPS 步，OPUS_THREADS 个线程分种子跑。
mod common_b57;

use common_b57::{violations, verdict_name, Knobs, Pool};
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

fn knobs_of_the_seed(random: &mut SeededRandomSource, mode: &str) -> Knobs {
    let node_entries = [0usize, 2, 3][pick(random, 3)];
    match mode {
        "all" => Knobs {
            tombstone: 2 + random.below(3),
            lineage: 2 + random.below(2),
            retirement: 2 + random.below(2),
            node_entries,
        },
        _ => {
            let value = 2 + random.below(2);
            match random.below(3) {
                0 => Knobs { tombstone: value, lineage: 0, retirement: 0, node_entries },
                1 => Knobs { tombstone: 0, lineage: value, retirement: 0, node_entries },
                _ => Knobs { tombstone: 0, lineage: 0, retirement: value, node_entries },
            }
        }
    }
}

/// 装了旋钮的那几棵这一版里不是最右的容器片数（新判在这一版判到的片数）。
fn non_rightmost_containers_under_a_knob(pool: &Pool<SparseBlockDevice>, knobs: Knobs) -> u64 {
    let output = &pool.output;
    let non_rightmost = |count: usize| u64::try_from(count.saturating_sub(1)).expect("小");
    let mut judged = 0;
    if knobs.tombstone > 0 {
        judged += non_rightmost(output.tombstone_containers.len());
    }
    if knobs.lineage > 0 {
        judged += non_rightmost(output.lineage_containers.len());
    }
    if knobs.retirement > 0 {
        judged += non_rightmost(output.retirement_containers.len());
    }
    judged
}

fn one_seed(base: &Pool<SparseBlockDevice>, seed: u64, operations: u64, mode: &str) -> String {
    let started = std::time::Instant::now();
    let mut random = SeededRandomSource::from_seed(seed);
    let knobs = knobs_of_the_seed(&mut random, mode);
    let mut pool = base.clone();
    pool.allocator.set_deletion_switches(knobs.switches());
    pool.create(u64::from(INODE_LEAF_RECORDS) + 1 + random.below(8));
    let mut versions: Vec<(InstanceGeneration, u64)> = vec![(pool.instance, pool.txg())];
    let mut counts = std::collections::BTreeMap::<&'static str, u64>::new();
    let (mut checks, mut violating, mut held_9_22, mut held_9_37) = (0u64, 0u64, 0u64, 0u64);
    let (mut judged_non_rightmost, mut steps_with_three_or_more) = (0u64, 0u64);
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
        let judged_now = non_rightmost_containers_under_a_knob(&pool, knobs);
        judged_non_rightmost += judged_now;
        steps_with_three_or_more += u64::from(judged_now >= 2);
        held_9_22 += u64::from(verdict_name(&verdicts, "I-9.22") == "Holds");
        held_9_37 += u64::from(verdict_name(&verdicts, "I-9.37") == "Holds");
    }
    format!(
        "OPUS_K1R seed={seed} mode={mode} knobs={knobs:?} ops={counts:?} checks={checks} violating={violating} held_I-9.22={held_9_22} held_I-9.37={held_9_37} non_rightmost_judged={judged_non_rightmost} steps_with_two_or_more_non_rightmost={steps_with_three_or_more} final={} first_violation={first_violation} elapsed={:.1}s",
        pool.tree_summary(),
        started.elapsed().as_secs_f64()
    )
}

#[test]
fn opus_probe_b57_random_knob_histories() {
    let first = env_u64("OPUS_SEED_FIRST", 1);
    let last = env_u64("OPUS_SEED_LAST", 2);
    let operations = env_u64("OPUS_OPS", 20);
    let threads = env_u64("OPUS_THREADS", 2);
    let mode = std::env::var("OPUS_MODE").unwrap_or_else(|_| "all".to_string());
    let base = Pool::after_the_first_file();
    let seeds: Vec<u64> = (first..=last).collect();
    let lines: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|thread_index| {
                let (base, seeds, mode) = (&base, &seeds, mode.as_str());
                scope.spawn(move || {
                    seeds
                        .iter()
                        .copied()
                        .filter(|seed| (seed - first) % threads == thread_index)
                        .map(|seed| {
                            let line = one_seed(base, seed, operations, mode);
                            eprintln!("{line}");
                            line
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
    });
    eprintln!("OPUS_K1R_DONE seeds={}", lines.len());
}
