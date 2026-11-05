//! 攻方腿 s2 的容器小树差分：同一份种子的发布序列喂给改前（HEAD）与改后（冻结副本）的 core，逐步打出结局，两边逐行比。
use std::fmt::Debug;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{CheckpointTxg, InodeNumber, TreeIdentifier};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::container_tree::*;
use singlefs_core::inode_tree::*;
use singlefs_core::records::{InodeRecord, InodeRecordTruncationField};

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
    fn below(&mut self, bound: u64) -> u64 { self.next() % bound.max(1) }
}

struct Out(std::io::BufWriter<std::fs::File>, u64);
impl Out {
    fn line<T: Debug>(&mut self, name: &str, value: &T) {
        self.1 += 1;
        let shown = format!("{value:?}");
        if shown.len() > 300 {
            writeln!(self.0, "{} {} len{} crc{:08x}", self.1, name, shown.len(), crc32_castagnoli(shown.as_bytes())).unwrap();
        } else {
            writeln!(self.0, "{} {} {}", self.1, name, shown).unwrap();
        }
        self.0.flush().unwrap();
    }
}

fn caught<T>(call: impl FnOnce() -> T) -> Result<T, &'static str> {
    catch_unwind(AssertUnwindSafe(call)).map_err(|_| "PANIC")
}

fn shapes(out: &mut Out, random: &mut Random) {
    for _ in 0..200000 {
        let levels = random.below(5) as usize;
        let mut children: Vec<Vec<usize>> = Vec::new();
        for _ in 0..levels {
            let nodes = random.below(5) as usize;
            children.push((0..nodes).map(|_| match random.below(12) { 0 => 0, 1 => usize::MAX, 2 => usize::MAX / 2 + 1, _ => 1 + random.below(4) as usize }).collect());
        }
        // 一半的样本按上一层的个数把下一层凑成一致，好走到收下的分支。
        if random.below(2) == 0 {
            for level in 1..children.len() {
                let below = children[level - 1].len();
                let nodes = children[level].len().max(1);
                let mut split = vec![below / nodes; nodes];
                if let Some(last) = split.last_mut() { *last += below % nodes; }
                children[level] = split;
            }
            if let Some(top) = children.last_mut() { if random.below(3) != 0 { let sum = top.len(); *top = vec![sum]; } }
        }
        let built = caught(|| ContainerTreeShape::from_children_of_each_node_at_each_level(children.clone()));
        out.line("shape", &(&children, &built));
        if let Ok(Some(shape)) = built {
            let level = shape.root_level();
            out.line("shape_facts", &(level, shape.internal_levels(), caught(|| shape.containers()), (0..=level + 1).map(|l| caught(|| shape.nodes_at_level(l))).collect::<Vec<_>>(),
                caught(|| shape.positions_in_bump_order()), shape.root_position()));
            for _ in 0..4 {
                let position = ContainerTreeNodePosition { level: random.below(u64::from(level) + 2) as u8, index_in_level: random.below(6) as u32 };
                out.line("shape_at", &(position, shape.has_a_node_at(position), shape.is_the_root(position),
                    if shape.has_a_node_at(position) { Some(caught(|| shape.children_of(position))) } else { None }));
            }
            let count = caught(|| shape.containers()).unwrap_or(0);
            let asked: Vec<usize> = (0..random.below(4)).map(|_| random.below((count as u64).saturating_add(2)) as usize).collect();
            // 改后的 ancestors_of_containers 按全部容器数开一份标记表（O(容器数)），容器数上了百万的形状上不问（见报告 T7）。
            if count <= 1_000_000 {
                out.line("ancestors", &(&asked, caught(|| shape.ancestors_of_containers(&asked))));
            } else {
                out.line("ancestors_skipped", &count);
            }
            let entries = 1 + random.below(5) as usize;
            out.line("opened", &(entries, caught(|| shape.after_a_container_is_opened_at_the_right_end(entries))));
        }
    }
}

fn appends(out: &mut Out, random: &mut Random) {
    for history in 0..1500 {
        let per_container = 1 + random.below(4) as usize;
        let node_entries = match random.below(5) {
            0 => ContainerTreeNodeEntries::FromTheNodeFormat,
            _ => ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(2 + random.below(3) as usize).unwrap()),
        };
        let tree = TreeIdentifier(1000 + history);
        let mut containers: Vec<RecordContainer<u64>> = Vec::new();
        let mut shape = ContainerTreeShape::zero_root();
        let mut next_record = 0u64;
        for publish in 0..(20 + random.below(60)) {
            let count = random.below(6) as usize;
            let records: Vec<u64> = (0..count).map(|_| { next_record += 1; next_record }).collect();
            let txg = CheckpointTxg(10 + publish);
            let result = caught(|| append_records_to_the_open_container(&containers, &shape, &records, txg, tree, 7, per_container, node_entries));
            match result {
                Ok(Ok(after)) => {
                    out.line("append", &(history, publish, &after));
                    out.line("append_facts", &(after.is_the_root_rewritten(), after.internal_levels(), after.grew_a_level_this_publish(),
                        after.previous_root_is_carried_down_unchanged(), caught(|| after.rewritten_nodes()), caught(|| after.node_origins())));
                    if count == 1 {
                        let single = caught(|| append_record_to_the_open_container(&containers, &shape, records[0], txg, tree, 7, per_container, node_entries));
                        out.line("append_one", &single.map(|r| r.map(|a| a == after)));
                    }
                    let carried = caught(|| ContainerTreeAfterThisPublish::carried(after.containers.clone(), after.shape.clone()));
                    out.line("carried", &carried.map(|c| (c.is_the_root_rewritten(), c.rewritten_nodes(), c.node_origins(), c.grew_a_level_this_publish())));
                    containers = after.containers;
                    shape = after.shape;
                }
                other => { out.line("append_refused", &(history, publish, &other)); break; }
            }
        }
    }
}

fn record(random: &mut Random, inode: u64) -> InodeRecord {
    InodeRecord {
        inode,
        object_birth: CheckpointTxg(random.below(100)),
        nlink: 1,
        size: random.next() >> random.below(64),
        change_count: random.below(1000),
        write_time_seconds: random.below(1000),
        truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
    }
}

fn inode_publishes(out: &mut Out, random: &mut Random) {
    for history in 0..1200 {
        let tree = TreeIdentifier(5);
        let mut containers: Vec<InodeLeafContainer> = Vec::new();
        let mut live: Vec<u64> = Vec::new();
        let mut next_inode = 1 + random.below(3);
        let steps = 10 + random.below(50);
        for step in 0..steps {
            let txg = CheckpointTxg(100 + step);
            let kind = random.below(10);
            if kind < 5 || live.is_empty() {
                // 写：一批新号接在末尾（偶尔整批几百条，跨片），连同几条已有号的覆写；偶尔塞一条不升序或重复的号。
                let fresh = match random.below(6) { 0 => 200 + random.below(400), 1 => 0, _ => random.below(8) } as usize;
                let mut chosen: Vec<u64> = (0..random.below(4)).filter_map(|_| if live.is_empty() { None } else { Some(live[random.below(live.len() as u64) as usize]) }).collect();
                for _ in 0..fresh { chosen.push(next_inode); next_inode += 1 + random.below(3) * random.below(2); }
                chosen.sort();
                chosen.dedup();
                if random.below(40) == 0 && chosen.len() >= 2 { chosen.swap(0, 1); }
                if random.below(40) == 0 && !chosen.is_empty() { let first = chosen[0]; chosen.push(first); }
                let records: Vec<InodeRecord> = chosen.iter().map(|&inode| record(random, inode)).collect();
                let result = caught(|| write_records_into_leaf_containers(&containers, &records, txg, tree));
                out.line("inode_write", &(history, step, &chosen, &result));
                if let Ok(Ok(after)) = result {
                    out.line("inode_write_facts", &(after.is_the_root_rewritten(), after.retired_identities()));
                    for inode in &chosen { if !live.contains(inode) { live.push(*inode); } }
                    live.sort();
                    containers = after.containers;
                }
            } else {
                // 删：多半删一个在的号，偶尔删一个不在的号；偶尔一口气删掉一片的大半。
                let rounds = if random.below(8) == 0 { 1 + random.below(150) } else { 1 };
                for _ in 0..rounds {
                    if live.is_empty() { break; }
                    let inode = if random.below(15) == 0 { next_inode + 5 } else { live[random.below(live.len() as u64) as usize] };
                    let result = caught(|| remove_record_from_leaf_containers(&containers, InodeNumber(inode)));
                    out.line("inode_remove", &(history, step, inode, &result));
                    if let Ok(Ok(after)) = result {
                        out.line("inode_remove_facts", &(after.is_the_root_rewritten(), after.retired_identities()));
                        live.retain(|existing| *existing != inode);
                        containers = after.containers;
                    }
                }
            }
            out.line("inode_state", &containers.iter().map(|c| (c.container_number(), c.smallest_inode_number(), c.largest_inode_number(), c.separator_key(), caught(|| c.free_record_slots()))).collect::<Vec<_>>());
        }
    }
    out.line("inode_constants", &(leaf_containers_one_root_node_holds(), records_below_which_a_leaf_borrows_from_its_right_neighbour(), containers_one_container_tree_root_holds()));
}

#[test]
fn container_tree_outputs_for_the_differential() {
    let path = std::env::var("ATTACK_OUT").expect("ATTACK_OUT");
    let mut out = Out(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), 0);
    std::panic::set_hook(Box::new(|_| {}));
    let mut random = Random(0xD1B5_4A32_D192_ED03);
    shapes(&mut out, &mut random);
    // 各段各用一份种子：形状那一段在改前改后收下的集合不同（层级 1 求和越过 usize，报告 T7），不让它带偏后面两段的取样。
    appends(&mut out, &mut Random(0xA076_1D64_78BD_642F));
    inode_publishes(&mut out, &mut Random(0xE703_7ED1_A0B4_28DB));
    out.0.flush().unwrap();
}
