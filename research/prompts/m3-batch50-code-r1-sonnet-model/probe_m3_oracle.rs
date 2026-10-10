//! 草稿：把 crates 的 inode_tree 删除/插入逐步对拍 E176 模型的 M3 臂（先右后左、f = 117、均分借），并在 crates 一侧单独核几条性质。
#[path = "e176_model.rs"]
#[allow(dead_code)]
mod model;

use model::{BorrowAmount, InodeLeafTree, InsertOutcome, LeafContainerLimit, MergePolicy, NeighbourCheckOrder};
use singlefs_core::address::{CheckpointTxg, InodeNumber, TreeIdentifier};
use singlefs_core::inode_tree::{
    remove_record_from_leaf_containers, write_records_into_leaf_containers, InodeLeafContainer, InodeLeafContainerOrigin,
};
use singlefs_core::records::InodeRecord;
use std::collections::BTreeSet;

thread_local! { static EMPTIED: std::cell::Cell<u64> = std::cell::Cell::new(0); }
const TREE: TreeIdentifier = TreeIdentifier(65537);

fn record(inode: u64, txg: u64) -> InodeRecord {
    InodeRecord { inode, object_birth: CheckpointTxg(txg), nlink: 1, size: 0, change_count: txg, write_time_seconds: 0 }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 { self.next() % n }
}

fn shape_model(t: &InodeLeafTree) -> Vec<(u64, usize, u64, u64)> {
    t.containers.iter().map(|c| (c.container_number, c.inode_numbers.len(), c.smallest_inode_number(), c.largest_inode_number())).collect()
}
fn shape_crates(cs: &[InodeLeafContainer]) -> Vec<(u64, usize, u64, u64)> {
    cs.iter().map(|c| (c.identity.container, c.records.len(), c.records.first().unwrap().inode, c.records.last().unwrap().inode)).collect()
}

fn run(seed: u64, steps: u64, mode: u64) -> (u64, u64, u64, u64, u64) {
    let mut rng = Lcg(seed);
    let policy = MergePolicy::AbsorbRightAndBorrowFromRight { order: NeighbourCheckOrder::RightPairFirst, floor: 117, amount: BorrowAmount::HalfTheDifference };
    let mut m = InodeLeafTree::empty(policy, LeafContainerLimit::OneRootNode);
    let mut cs: Vec<InodeLeafContainer> = Vec::new();
    let mut live: Vec<u64> = Vec::new();
    let mut next_inode = 1u64;
    let (mut deletes, mut merges, mut borrows, mut retired_total, mut max_retired) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut emptied_with_right_neighbour = 0u64; let mut emptied_leftmost_with_others = 0u64;
    let mut txg = 3u64; let mut refused = 0u64;
    // 阶段：增长 / 随机删 / 顺序删 / 删一端，随 mode 与步数切换
    for step in 0..steps {
        txg += 1;
        let phase = (step / 1500 + mode) % 4;
        let want_create = match phase { 0 => rng.below(100) < 80, 1 => rng.below(100) < 20, 2 => rng.below(100) < 50, _ => rng.below(100) < 8 };
        let create = (want_create || live.is_empty()) && live.len() < 31000;
        if create {
            let burst = 1 + rng.below(if phase == 0 { 40 } else { 3 });
            for _ in 0..burst {
                if live.len() >= 31000 { break; }
                let inode = next_inode; next_inode += 1;
                let mi = m.insert(inode);
                let ci = write_records_into_leaf_containers(&cs, &[record(inode, txg)], CheckpointTxg(txg), TREE);
                match (mi, ci) {
                    (InsertOutcome::Accepted, Ok(after)) => { cs = after.containers; live.push(inode); }
                    (InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { containers_it_would_take: a },
                     Err(singlefs_core::inode_tree::InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers: b, capacity: 135 })) if a == b => {
                        next_inode -= 1; refused += 1; break;
                    }
                    other => panic!("insert diverges at step {step}: {other:?}"),
                }
            }
        } else {
            let idx = match phase { 1 => rng.below(live.len() as u64) as usize, 2 => 0, 3 => (rng.below(live.len().min(7) as u64)) as usize, _ => rng.below(live.len() as u64) as usize };
            let inode = live.remove(idx);
            deletes += 1;
            let before = cs.clone();
            if let Some(h) = before.iter().position(|c| c.records.iter().any(|r| r.inode == inode)) {
                if before[h].records.len() == 1 && h + 1 < before.len() { emptied_with_right_neighbour += 1; if h == 0 { emptied_leftmost_with_others += 1; } }
            }
            let mm = m.remove(inode);
            let after = remove_record_from_leaf_containers(&before, InodeNumber(inode)).expect("在树里");
            merges = m.merges; borrows = m.borrows;
            // 1 形态逐片相同
            assert_eq!(shape_model(&m), shape_crates(&after.containers), "shape step {step} inode {inode}");
            // 2 退役集 = 上一版身份集 − 这一版身份集；每次删除至多一片
            let prev_ids: BTreeSet<u64> = before.iter().map(|c| c.identity.container).collect();
            let new_ids: BTreeSet<u64> = after.containers.iter().map(|c| c.identity.container).collect();
            let gone: BTreeSet<u64> = prev_ids.difference(&new_ids).copied().collect();
            let retired: BTreeSet<u64> = after.retired.iter().map(|r| r.identity.container).collect();
            assert_eq!(gone, retired, "retired set step {step}");
            assert_eq!(after.retired.len(), mm.containers_dropped, "drop count step {step}");
            for r in &after.retired { assert_eq!(before[r.previous_index.position()].identity, r.identity); }
            retired_total += after.retired.len() as u64; max_retired = max_retired.max(after.retired.len() as u64);
            // 3 R3：重写清单 = 内容真变了的片（含分隔 key 变了的）；其余 CarriedFrom 指向上一版同身份、同内容、同分隔 key 的片
            let content_changed: Vec<usize> = after.containers.iter().enumerate().filter(|(_, c)| {
                before.iter().find(|p| p.identity == c.identity).map_or(true, |p| p.records != c.records || p.separator_key_in_the_parent_entry != c.separator_key_in_the_parent_entry)
            }).map(|(i, _)| i).collect();
            let rewritten: Vec<usize> = after.rewritten.iter().map(|i| i.position()).collect();
            assert_eq!(rewritten, content_changed, "rewritten list step {step}");
            assert_eq!(rewritten, mm.content_rewritten_positions, "rewritten vs model content step {step}");
            for (pos, origin) in after.origins.iter().enumerate() {
                if let InodeLeafContainerOrigin::CarriedFrom(prev) = origin {
                    assert_eq!(before[prev.position()], after.containers[pos], "carried container changed step {step}");
                }
            }
            cs = after.containers;
        }
        // 4 性质：分隔 key、无相邻可并对、容器号与记录次序
        let mut prev_sep = 0u64; let mut prev_max = 0u64;
        for (i, c) in cs.iter().enumerate() {
            let sep = c.separator_key_in_the_parent_entry;
            let min = c.records.first().unwrap().inode; let max = c.records.last().unwrap().inode;
            assert!(sep <= min, "sep > min at {i}");
            assert!(sep >= c.identity.container, "sep < number at {i}");
            if i > 0 { assert!(sep > prev_sep && sep > prev_max, "sep order/ > left max at {i}: sep {sep} prev_sep {prev_sep} prev_max {prev_max}"); assert!(c.identity.container > cs[i-1].identity.container); }
            assert!(min >= c.identity.container);
            assert!(c.records.windows(2).all(|w| w[0].inode < w[1].inode));
            prev_sep = sep; prev_max = max;
            if i + 1 < cs.len() { assert!(c.records.len() + cs[i + 1].records.len() > 233, "adjacent pair fits at {i} step {step}"); }
        }
    }
    EMPTIED.with(|e| { e.set(e.get() + emptied_with_right_neighbour * 1000 + emptied_leftmost_with_others); });
    (deletes, merges, borrows, retired_total, max_retired.max(0) + 0 * refused)
}

#[test]
fn m3_crates_vs_e176_model() {
    let seeds: u64 = std::env::var("PROBE_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(40);
    let steps: u64 = std::env::var("PROBE_STEPS").ok().and_then(|s| s.parse().ok()).unwrap_or(9000);
    let mut totals = (0, 0, 0, 0, 0u64);
    for seed in 1..=seeds {
        let r = run(seed * 7919, steps, seed % 4);
        totals = (totals.0 + r.0, totals.1 + r.1, totals.2 + r.2, totals.3 + r.3, totals.4.max(r.4));
    }
    println!("PROBE emptied_with_right_neighbour*1000+leftmost={}", EMPTIED.with(|e| e.get()));
    println!("PROBE deletes={} merges={} borrows={} retired_total={} max_retired_per_delete={}", totals.0, totals.1, totals.2, totals.3, totals.4);
}

/// 定向形：三片各 117 条（相邻两片之和 234，一对都不并），再从中间那片删一条：左对、右对同时装得下，先右后左与先左后右的结果不同。
#[test]
fn directed_three_117_both_pairs_fit_after_one_delete() {
    let policy = MergePolicy::AbsorbRightAndBorrowFromRight { order: NeighbourCheckOrder::RightPairFirst, floor: 117, amount: BorrowAmount::HalfTheDifference };
    let mut m = InodeLeafTree::empty(policy, LeafContainerLimit::OneRootNode);
    let mut cs: Vec<InodeLeafContainer> = Vec::new();
    let mut txg = 3u64;
    for inode in 1..=699u64 {
        txg += 1;
        assert!(matches!(m.insert(inode), InsertOutcome::Accepted));
        cs = write_records_into_leaf_containers(&cs, &[record(inode, txg)], CheckpointTxg(txg), TREE).unwrap().containers;
    }
    let del = |m: &mut InodeLeafTree, cs: &mut Vec<InodeLeafContainer>, inode: u64| {
        m.remove(inode);
        *cs = remove_record_from_leaf_containers(cs, InodeNumber(inode)).unwrap().containers;
    };
    for inode in (1..=116u64).chain(234..=349).chain(467..=582) { del(&mut m, &mut cs, inode); }
    assert_eq!(shape_crates(&cs).iter().map(|s| s.1).collect::<Vec<_>>(), vec![117, 117, 117]);
    del(&mut m, &mut cs, 400);
    let (a, b) = (shape_model(&m), shape_crates(&cs));
    println!("DIRECTED model(right-first)={:?} crates={:?}", a.iter().map(|s| (s.0, s.1)).collect::<Vec<_>>(), b.iter().map(|s| (s.0, s.1)).collect::<Vec<_>>());
    assert_eq!(a, b);
}

/// 不满足「没有相邻可并对」的树（旧 M1 池留下的：每片 2 条、135 片）：删一条引起的连锁吸收有多长、重写几片。
#[test]
fn adversarial_cascade_on_a_tree_that_breaks_the_no_adjacent_fit_invariant() {
    use singlefs_core::unit::{PackedIdentity, PACKED_TYPE_INODE};
    let mut cs: Vec<InodeLeafContainer> = Vec::new();
    for leaf in 0..135u64 {
        let a = leaf * 2 + 1;
        cs.push(InodeLeafContainer {
            identity: PackedIdentity { birth_tree: TREE, record_type: PACKED_TYPE_INODE, container: a, container_birth: CheckpointTxg(3) },
            separator_key_in_the_parent_entry: a,
            records: vec![record(a, 3), record(a + 1, 3)],
        });
    }
    // 删第 40 片的第 1 条（inode 79）：该片剩 1 条，向右吸收 2 条的邻居，一路并到 233 条为止
    let after = remove_record_from_leaf_containers(&cs, InodeNumber(79)).unwrap();
    let sizes: Vec<usize> = after.containers.iter().map(|c| c.records.len()).collect();
    println!("CASCADE retired={} rewritten={} leaves_after={} survivor_sizes_first_few={:?}", after.retired.len(), after.rewritten.len(), after.containers.len(), &sizes[..sizes.len().min(6)]);
    // 删最左一片的第 1 条（inode 1）：剩 1 条，同样连并
    let after0 = remove_record_from_leaf_containers(&cs, InodeNumber(1)).unwrap();
    println!("CASCADE-LEFTMOST retired={} rewritten={}", after0.retired.len(), after0.rewritten.len());
}
