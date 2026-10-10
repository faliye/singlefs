//! 攻方探针（副本，不入库）：随机 建 inode / unlink / 管理员回退 / 崩了再挂 / 覆盖写，逐版判 M3、R3、退役完备、水位不重发，逐步跑池级 checker。
#![allow(dead_code, clippy::all)]
mod common_admission;

use common_admission::*;
use singlefs_core::address::{CheckpointTxg, InodeNumber};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use std::collections::{BTreeMap, BTreeSet};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn id(c: &singlefs_core::unit::PackedIdentity) -> String {
    format!("{:?}", c)
}

fn ids(v: &TransactionOutput) -> Vec<String> {
    v.inode_leaf_containers.iter().map(|c| id(&c.contents.identity)).collect()
}

fn retire_records(v: &TransactionOutput) -> Vec<(String, u64)> {
    v.retirement_containers
        .iter()
        .flat_map(|c| c.contents.records.iter().map(|r| (id(&r.retired), r.death_txg.0)))
        .collect()
}

/// 只看一版自己的形状
fn shape_issues(v: &TransactionOutput, out: &mut Vec<String>) {
    let leaves = &v.inode_leaf_containers;
    let mut prev_max: Option<u64> = None;
    for (i, c) in leaves.iter().enumerate() {
        let r = &c.contents.records;
        if r.is_empty() || r.len() > 233 {
            out.push(format!("leaf {i} has {} records", r.len()));
        }
        if r.windows(2).any(|w| w[0].inode >= w[1].inode) {
            out.push(format!("leaf {i} keys not increasing"));
        }
        if let Some(min) = r.first().map(|x| x.inode) {
            if c.contents.separator_key_in_the_parent_entry > min {
                out.push(format!("leaf {i} separator {} > min {min}", c.contents.separator_key_in_the_parent_entry));
            }
            if let Some(pm) = prev_max {
                if min <= pm {
                    out.push(format!("leaf {i} min {min} <= left max {pm}"));
                }
                if c.contents.separator_key_in_the_parent_entry <= pm {
                    out.push(format!("leaf {i} separator {} <= left max {pm}", c.contents.separator_key_in_the_parent_entry));
                }
            }
        }
        prev_max = r.last().map(|x| x.inode).or(prev_max);
    }
}

/// 非回退的一次发布 p → v
fn transition_issues(p: &TransactionOutput, v: &TransactionOutput, out: &mut Vec<String>, stats: &mut BTreeMap<&'static str, u64>) {
    let txg = v.root.checkpoint_txg.0;
    let pi = ids(p);
    let vi = ids(v);
    let mut expected: Vec<String> = pi.iter().filter(|x| !vi.contains(x)).cloned().collect();
    expected.sort();
    let mut written: Vec<String> = retire_records(v).into_iter().filter(|(_, d)| *d == txg).map(|(i, _)| i).collect();
    written.sort();
    if expected != written {
        out.push(format!("txg {txg}: retired expected {expected:?} written {written:?}"));
    }
    let pr = retire_records(p);
    let vr = retire_records(v);
    if vr.len() < pr.len() || vr[..pr.len()] != pr[..] {
        out.push(format!("txg {txg}: retire tree not append-only"));
    }
    *stats.entry("retired").or_default() += expected.len() as u64;
    // R3 与 M3
    let multi = v.inode_leaf_containers.len() > 1;
    let n = v.inode_leaf_containers.len();
    for (i, c) in v.inode_leaf_containers.iter().enumerate() {
        if i + 1 < n && c.contents.records.len() < 117 {
            out.push(format!("txg {txg}: non-rightmost leaf {i} of {n} has {} < 117", c.contents.records.len()));
        }
    }
    for c in &v.inode_leaf_containers {
        if let Some(pc) = p.inode_leaf_containers.iter().find(|x| id(&x.contents.identity) == id(&c.contents.identity)) {
            let same = pc.contents == c.contents;
            let same_ptr = pc.pointer == c.pointer;
            if !same && same_ptr {
                out.push(format!("txg {txg}: leaf {} content changed but pointer copied", id(&c.contents.identity)));
            }
            if same && !same_ptr {
                *stats.entry("rewritten_unchanged").or_default() += 1;
            }
            if same && same_ptr {
                *stats.entry("carried").or_default() += 1;
            }
            let is_rightmost = id(&v.inode_leaf_containers.last().unwrap().contents.identity) == id(&c.contents.identity);
            if c.contents.records.len() < pc.contents.records.len() && c.contents.records.len() < 117 && multi && !is_rightmost {
                out.push(format!("txg {txg}: leaf {} lost records and has {} < 117", id(&c.contents.identity), c.contents.records.len()));
            }
            if c.contents.records.len() < pc.contents.records.len() {
                *stats.entry("leaf_lost").or_default() += 1;
            }
            if c.contents.separator_key_in_the_parent_entry != pc.contents.separator_key_in_the_parent_entry {
                *stats.entry("separator_changed").or_default() += 1;
            }
        }
    }
}

fn live_inodes(v: &TransactionOutput) -> Vec<u64> {
    v.inode_leaf_containers
        .iter()
        .flat_map(|c| c.contents.records.iter())
        .filter(|r| r.nlink != 0)
        .map(|r| r.inode)
        .collect()
}

fn all_numbers(v: &TransactionOutput) -> Vec<(u64, u64)> {
    v.inode_leaf_containers
        .iter()
        .flat_map(|c| c.contents.records.iter())
        .map(|r| (r.inode, r.object_birth.0))
        .collect()
}

fn cap_switches(pool: &mut PoolUnderTest<singlefs_harness::memory_pool::SparseBlockDevice>) {
    if std::env::var("CHURN_CAPPED").is_err() { return; }
    use singlefs_core::container_tree::{ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests};
    use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
    use singlefs_core::retirement_tree::{RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests};
    use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
    use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
    let tomb: u64 = std::env::var("CHURN_TOMB").ok().map(|x| x.parse().unwrap()).unwrap_or(100);
    let entries: usize = std::env::var("CHURN_ENTRIES").ok().map(|x| x.parse().unwrap()).unwrap_or(2);
    pool.session.as_mut().unwrap().allocator.set_deletion_switches(DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: TombstoneContainerCapacity::CappedForTests(TombstoneRecordsPerContainerCappedForTests::new(tomb).unwrap()),
        lineage_container_capacity: LineageContainerCapacity::CappedForTests(LineageRecordsPerContainerCappedForTests::new(1).unwrap()),
        retirement_container_capacity: RetirementContainerCapacity::CappedForTests(RetirementRecordsPerContainerCappedForTests::new(1).unwrap()),
        container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(entries).unwrap()),
    });
}

fn run(seed: u64, steps: usize, width: HistoryDeviceWidth) -> (Vec<String>, BTreeMap<&'static str, u64>) {
    let mut rng = Rng(seed);
    let mut pool = start_plain(width);
    cap_switches(&mut pool);
    let mut issues = Vec::new();
    let mut stats: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut seen_numbers: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
    let mut recent_roots: Vec<(singlefs_core::address::InstanceGeneration, CheckpointTxg)> = Vec::new();
    let check_every = std::env::var("CHURN_CHECK_EVERY").ok().map(|s| s.parse().unwrap()).unwrap_or(1usize);
    for step in 0..steps {
        let before = pool.current_file_version().clone();
        let live: Vec<u64> = live_inodes(&before).into_iter().filter(|n| *n != 1).collect();
        let dice = rng.below(100);
        let mut versions: Vec<TransactionOutput> = Vec::new();
        let mut is_rollback = false;
        let mut is_deletion = false;
        let mut op = String::new();
        if dice < 30 || live.is_empty() {
            let k = [1u64, 5, 40, 117, 233, 300, 466][rng.below(7) as usize];
            pool.write_time_seconds += 1;
            let w = pool.write_time_seconds;
            op = format!("create {k}");
            match pool.publish(UserChange::NewInodes { count: k, write_time_seconds: w }, None) {
                Ok(p) => versions = p.versions_published,
                Err(e) => op.push_str(&format!(" refused {:.60}", format!("{e:?}"))),
            }
        } else if dice < 75 {
            // 挑一片叶，从里面连删 m 条（逐条一次 unlink，每条一次调用）
            let leaves = &before.inode_leaf_containers;
            let li = rng.below(leaves.len() as u64) as usize;
            let mut in_leaf: Vec<u64> = leaves[li].contents.records.iter().filter(|r| r.nlink != 0 && r.inode != 1).map(|r| r.inode).collect();
            let m = [1usize, 10, 60, 115, 116, 117, 118, 200, 233][rng.below(9) as usize].min(in_leaf.len());
            let mode = rng.below(3);
            op = format!("unlink-burst leaf {li} m {m} mode {mode}");
            is_deletion = true;
            for j in 0..m {
                if in_leaf.is_empty() { break; }
                let pick = match mode { 0 => 0, 1 => in_leaf.len() - 1, _ => rng.below(in_leaf.len() as u64) as usize };
                let n = in_leaf.remove(pick);
                pool.write_time_seconds += 1;
                let w = pool.write_time_seconds;
                match pool.publish(UserChange::Unlink { inode: InodeNumber(n), write_time_seconds: w }, None) {
                    Ok(p) => versions.extend(p.versions_published),
                    Err(e) => {
                        issues.push(format!("seed {seed} step {step} burst {j}: unlink {n} refused {e:?}"));
                        break;
                    }
                }
            }
        } else if dice < 87 && !recent_roots.is_empty() {
            let (instance, txg) = recent_roots[rng.below(recent_roots.len() as u64) as usize];
            op = format!("rollback to {}/{}", instance.0, txg.0);
            let mut session = pool.session.take().unwrap();
            let mut cur = session.current.file_version().unwrap().clone();
            match roll_back_by_a_forward_publish(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut cur, RollbackTarget { instance, checkpoint_txg: txg }) {
                Ok(_) => {
                    session.current = PoolVersion::of_a_version_of_trees(cur.clone());
                    versions = vec![cur];
                    is_rollback = true;
                }
                Err(e) => op.push_str(&format!(" refused {:.50}", format!("{e:?}"))),
            }
            pool.session = Some(session);
        } else if dice < 92 {
            op = "crash+mount".into();
            match pool.crash_and_mount_writable() {
                Ok(_) => { cap_switches(&mut pool); }
                Err(e) => {
                    issues.push(format!("seed {seed} step {step}: mount failed {e:?}"));
                    return (issues, stats);
                }
            }
            recent_roots.clear();
        } else {
            op = "overwrite".into();
            let _ = pool.overwrite(1000 + (rng.below(1000) as usize));
        }
        // 逐版判
        let mut prev = before.clone();
        for v in &versions {
            let mut local = Vec::new();
            shape_issues(v, &mut local);
            if !is_rollback {
                transition_issues(&prev, v, &mut local, &mut stats);
            } else {
                let vr = retire_records(v);
                let pr = retire_records(&prev);
                if vr != pr {
                    local.push("rollback changed the retire tree".into());
                }
            }
            // 水位与台阶行
            let wm = v.inode_number_watermark();
            if let Some(maxn) = all_numbers(v).iter().map(|x| x.0).max() {
                if wm <= maxn { local.push(format!("watermark {wm} <= max inode {maxn}")); }
            }
            if wm < prev.inode_number_watermark() { local.push(format!("watermark went down {} -> {wm}", prev.inode_number_watermark())); }
            let own = v.accounting_entries.iter().filter(|e| e.statistic == singlefs_core::records::STATISTIC_INODE_WATERMARK && e.generation == v.root.checkpoint_txg).count();
            if own != 1 { local.push(format!("own watermark rows {own}")); }
            let ps = prev.inode_number_watermark_steps();
            let vs = v.inode_number_watermark_steps();
            if vs.len() < ps.len() || vs[..ps.len()] != ps[..] { local.push(format!("steps not a prefix-extension: prev {:?} v {:?}", ps.iter().map(|e|(e.generation.0,e.value)).collect::<Vec<_>>(), vs.iter().map(|e|(e.generation.0,e.value)).collect::<Vec<_>>())); }
            if vs.last().map(|e| e.value) != Some(wm) && !vs.is_empty() { local.push(format!("last step value != watermark {wm}")); }
            // 删除类发布的新写槽数对 W
            if is_deletion {
                let new_slots: u64 = v.units.iter().filter(|u| v.rewritten.contains(&u.identity)).map(|u| (u.bytes.len() as u64).div_ceil(singlefs_format::SLOT_BYTES)).sum();
                let s = pool.parameters.geometry.root_ring_slots_per_region;
                let w = singlefs_core::admission::ring_window_on_one_device(&pool.session().allocator, s, 144, prev.retirement_tree_internal_levels()).0 / singlefs_format::SLOT_BYTES / (3 * s.count() + 1);
                let e = stats.entry("max_deletion_new_slots").or_default();
                if new_slots > *e { *e = new_slots; }
                let e = stats.entry("W").or_default();
                *e = w;
                if new_slots > w { local.push(format!("deletion publish new slots {new_slots} > W {w}")); }
            }
            for (n, b) in all_numbers(v) {
                let births = seen_numbers.entry(n).or_default();
                if !births.is_empty() && !births.contains(&b) {
                    local.push(format!("inode number {n} reissued with birth {b}, earlier births {births:?}"));
                }
                births.insert(b);
            }
            for l in local {
                issues.push(format!("seed {seed} step {step} ({op}) txg {}: {l}", v.root.checkpoint_txg.0));
            }
            prev = v.clone();
        }
        let cur = pool.current_file_version().clone();
        recent_roots.push((cur.root.instance, cur.root.checkpoint_txg));
        if recent_roots.len() > 6 {
            recent_roots.remove(0);
        }
        if step % check_every == 0 || step + 1 == steps {
            let violations = checker_violations_on(&pool.image());
            *stats.entry("checker_runs").or_default() += 1;
            for x in violations {
                issues.push(format!("seed {seed} step {step} ({op}) CHECKER {x:.300}"));
            }
        }
        *stats.entry("steps").or_default() += 1;
        { let h = cur.retirement_tree_internal_levels(); let e = stats.entry("max_retire_internal_levels").or_default(); if h > *e { *e = h; } }
        { let n = cur.retirement_containers.len() as u64; let e = stats.entry("max_retire_containers").or_default(); if n > *e { *e = n; } }
        if std::env::var("CHURN_TRACE").is_ok() {
            println!("seed {seed} step {step} {op} leaves={:?}", cur.inode_leaf_containers.iter().map(|c| c.contents.records.len()).collect::<Vec<_>>());
        }
    }
    (issues, stats)
}

#[test]
fn churn() {
    let seeds: Vec<u64> = std::env::var("CHURN_SEEDS").map(|s| {
        let (a, b) = s.split_once("..").unwrap();
        (a.parse().unwrap()..b.parse().unwrap()).collect()
    }).unwrap_or((1..3).collect());
    let steps: usize = std::env::var("CHURN_STEPS").ok().map(|s| s.parse().unwrap()).unwrap_or(40);
    let mut total: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut all = 0;
    for seed in seeds {
        let (issues, stats) = run(seed, steps, HistoryDeviceWidth::UnitAreaOf384Slots);
        for (k, v) in stats {
            if k == "W" || k.starts_with("max_") {
                let e = total.entry(k).or_default();
                if v > *e { *e = v; }
            } else {
                *total.entry(k).or_default() += v;
            }
        }
        all += issues.len();
        for i in issues.iter().take(20) {
            println!("ISSUE {i}");
        }
    }
    println!("TOTAL issues={all} stats={total:?}");
}
