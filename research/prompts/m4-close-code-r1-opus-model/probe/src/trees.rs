// 容器小树的差分格：与 suite.rs 一样 include! 进两个模块（`c` 指改前或改后的 core）。
use c::container_tree::{
    append_records_to_the_open_container, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests, ContainerTreeNodePosition,
    ContainerTreeShape, RecordContainer,
};
use c::inode_tree::{remove_record_from_leaf_containers, write_records_into_leaf_containers, InodeLeafContainer};
use c::records::{InodeRecord, InodeRecordTruncationField};

fn raw_levels(r: &mut Rng) -> Vec<Vec<usize>> {
    if r.below(3) == 0 {
        (0..r.below(4)).map(|_| (0..r.below(5)).map(|_| r.below(5) as usize).collect()).collect()
    } else {
        // 自下而上造一份合法形状：层级 1 的节点各挂 1..=6 片容器，往上每层把下一层的节点分组，直到只剩一个根
        let mut levels: Vec<Vec<usize>> = Vec::new();
        let mut below = 1 + r.below(12) as usize;
        let first: Vec<usize> = (0..below).map(|_| 1 + r.below(6) as usize).collect();
        levels.push(first);
        while below > 1 {
            let mut groups = Vec::new();
            let mut left = below;
            while left > 0 { let g = (1 + r.below(4) as usize).min(left); groups.push(g); left -= g; }
            below = groups.len();
            levels.push(groups);
            if levels.len() > 6 { break; }
        }
        if r.below(8) == 0 { let l = r.below(levels.len() as u64) as usize; let i = r.below(levels[l].len() as u64) as usize; levels[l][i] = r.below(3) as usize; }
        levels
    }
}

pub fn shape_case(seed: u64) -> String {
    let mut r = Rng(seed | 1);
    let r = &mut r;
    let levels = raw_levels(r);
    let mut out = vec![guard(|| ContainerTreeShape::from_children_of_each_node_at_each_level(levels.clone()), dbg)];
    let Some(shape) = ContainerTreeShape::from_children_of_each_node_at_each_level(levels.clone()) else { return out.join(" | ") };
    let rl = shape.root_level();
    out.push(format!("{} {} {} {:?} {:?}", rl, shape.internal_levels(), shape.containers(), shape.positions_in_bump_order(), shape.root_position()));
    for l in 0..=(rl.saturating_add(1)) { out.push(guard(|| shape.nodes_at_level(l), dbg)); }
    for _ in 0..8 {
        let level = r.below(u64::from(rl) + 2) as u8;
        let index = r.below(8) as u32;
        let p = ContainerTreeNodePosition { level, index_in_level: index };
        out.push(guard(|| (shape.has_a_node_at(p), shape.is_the_root(p)), dbg));
        if shape.has_a_node_at(p) || r.below(4) == 0 { out.push(guard(|| shape.children_of(p), dbg)); }
    }
    let reach = shape.containers() as u64 + if r.below(6) == 0 { 2 } else { 0 };
    let containers: Vec<usize> = (0..r.below(5)).map(|_| r.below(reach) as usize).collect();
    out.push(guard(|| shape.ancestors_of_containers(&containers), dbg));
    let epn = match r.below(5) { 0 => r.below(3) as usize, 1 => 2, 2 => 3, 3 => 2 + r.below(6) as usize, _ => 2031 };
    let mut s = shape.clone();
    for _ in 0..(1 + r.below(30)) {
        let next = guard(|| s.after_a_container_is_opened_at_the_right_end(epn), dbg);
        out.push(next);
        match catch_unwind(AssertUnwindSafe(|| s.after_a_container_is_opened_at_the_right_end(epn))) { Ok(n) => s = n, Err(_) => break }
    }
    out.join(" | ")
}

pub fn append_case(seed: u64) -> String {
    let mut r = Rng(seed | 1);
    let r = &mut r;
    let per = match r.below(4) { 0 => 1usize, 1 => 2, 2 => 1 + r.below(8) as usize, _ => r.below(3) as usize };
    let node_entries = if r.below(5) == 0 { ContainerTreeNodeEntries::FromTheNodeFormat } else {
        match ContainerTreeNodeEntriesCappedForTests::new(r.below(6) as usize) { Some(cap) => ContainerTreeNodeEntries::CappedForTests(cap), None => ContainerTreeNodeEntries::FromTheNodeFormat } };
    let tree = c::address::TreeIdentifier(1 + r.below(5));
    let start_number = if r.below(5) == 0 { u64::MAX - r.below(4) } else { 0 };
    let mut containers: Vec<RecordContainer<u64>> = Vec::new();
    let mut shape = ContainerTreeShape::zero_root();
    let mut out = Vec::new();
    if start_number > 0 {
        containers.push(RecordContainer { identity: PackedIdentity { birth_tree: tree, record_type: 5, container: start_number, container_birth: CheckpointTxg(1) }, records: vec![7; per.max(1)] });
        shape = ContainerTreeShape::from_children_of_each_node_at_each_level(vec![vec![1]]).expect("一片容器的形状");
    }
    for publish in 0..(1 + r.below(12)) {
        let records: Vec<u64> = (0..r.below(10)).map(|_| r.below(100)).collect();
        let res = guard(|| append_records_to_the_open_container(&containers, &shape, &records, CheckpointTxg(10 + publish), tree, 5, per, node_entries), |x| match x {
            Ok(a) => format!("Ok({:?} rewritten_nodes={:?} origins={:?} root_rewritten={} grew={} carried_down={} levels={})", a, a.rewritten_nodes(), a.node_origins(), a.is_the_root_rewritten(), a.grew_a_level_this_publish(), a.previous_root_is_carried_down_unchanged(), a.internal_levels()),
            Err(e) => format!("Err({e:?})"),
        });
        out.push(res);
        match catch_unwind(AssertUnwindSafe(|| append_records_to_the_open_container(&containers, &shape, &records, CheckpointTxg(10 + publish), tree, 5, per, node_entries))) {
            Ok(Ok(a)) => { containers = a.containers; shape = a.shape; }
            _ => break,
        }
    }
    out.join(" | ")
}

/// 一次写或删的结果压成一行：头一个词（Ok / Err / PANIC）、去掉 panic 那句话之后整串的 FNV、有没有并掉一片（退役叶）。
fn digest(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut inside = false;
    for ch in text.chars() { match ch { '⟦' => inside = true, '⟧' => inside = false, _ if !inside => plain.push(ch), _ => {} } }
    let head: String = plain.chars().take_while(|ch| ch.is_alphanumeric()).collect();
    format!("{head}:{:016x}{}", fnv(plain.as_bytes()), if plain.contains("RetiredInodeLeafContainer {") { ":RetiredInodeLeafContainer {" } else { "" })
}

fn inode_record(inode: u64, r: &mut Rng) -> InodeRecord {
    InodeRecord { inode, object_birth: CheckpointTxg(1 + r.below(3)), nlink: r.below(3) as u32, size: r.below(1 << 20), change_count: r.below(9), write_time_seconds: r.below(9), truncation_field: InodeRecordTruncationField::NOT_WRITTEN }
}

pub fn inode_tree_case(seed: u64) -> String {
    let mut r = Rng(seed | 1);
    let r = &mut r;
    let tree = c::address::TreeIdentifier(3);
    let mut leaves: Vec<InodeLeafContainer> = Vec::new();
    let mut next_inode = 1 + r.below(3);
    let mut live: Vec<u64> = Vec::new();
    let mut out = Vec::new();
    for publish in 0..(1 + r.below(25)) {
        let txg = CheckpointTxg(5 + publish);
        if r.below(3) == 0 && !live.is_empty() {
            // 一串删除（每删一条是一次发布）：删得多才走得到借位与合并
            let longest_burst = if r.below(2) == 0 { 8 } else { 150 };
            let burst = 1 + r.below(longest_burst);
            let mut stopped = false;
            for _ in 0..burst {
                if live.is_empty() { break; }
                let pick = if r.below(16) == 0 { 999_999 } else if r.below(2) == 0 { live[0] } else { live[r.below(live.len() as u64) as usize] };
                let res = guard(|| remove_record_from_leaf_containers(&leaves, InodeNumber(pick)), dbg);
                out.push(format!("rm {pick}: {}", digest(&res)));
                match catch_unwind(AssertUnwindSafe(|| remove_record_from_leaf_containers(&leaves, InodeNumber(pick)))) {
                    Ok(Ok(a)) => { leaves = a.containers; live.retain(|x| *x != pick); }
                    Ok(Err(_)) => {}
                    Err(_) => { stopped = true; break; }
                }
            }
            if stopped { break; }
            let _ = txg;
        } else {
            let n = match r.below(4) { 0 => 233 + r.below(3) as usize, 1 => 117 + r.below(3) as usize, _ => r.below(40) as usize };
            let mut records: Vec<InodeRecord> = Vec::new();
            // 大多是往最右追加；也混进改已有的记录与乱序
            if r.below(4) == 0 && !live.is_empty() { let v = live[r.below(live.len() as u64) as usize]; records.push(inode_record(v, r)); }
            for _ in 0..n { next_inode += 1 + if r.below(10) == 0 { r.below(5) } else { 0 }; records.push(inode_record(next_inode, r)); }
            if r.below(15) == 0 && records.len() > 1 { records.swap(0, 1); }
            let res = guard(|| write_records_into_leaf_containers(&leaves, &records, txg, tree), dbg);
            out.push(format!("wr {}: {}", records.len(), digest(&res)));
            match catch_unwind(AssertUnwindSafe(|| write_records_into_leaf_containers(&leaves, &records, txg, tree))) {
                Ok(Ok(a)) => { leaves = a.containers; for x in &records { if !live.contains(&x.inode) { live.push(x.inode); } } }
                Ok(Err(_)) => {}
                Err(_) => break,
            }
        }
    }
    for leaf in &leaves { out.push(guard(|| (leaf.container_number(), leaf.smallest_inode_number(), leaf.largest_inode_number(), leaf.separator_key(), leaf.free_record_slots()), dbg)); }
    out.join(" | ")
}

pub fn capacity_case(seed: u64) -> String {
    let mut r = Rng(seed | 1);
    let k = match r.below(3) { 0 => r.below(4), 1 => r.below(2500), _ => r.edgy() };
    let t = c::tombstone_tree::TombstoneRecordsPerContainerCappedForTests::new(k).map(|x| { let cap = c::tombstone_tree::TombstoneContainerCapacity::CappedForTests(x); (x.records(), cap.records_per_container(), cap.name()) });
    let l = c::lineage_tree::LineageRecordsPerContainerCappedForTests::new(k).map(|x| { let cap = c::lineage_tree::LineageContainerCapacity::CappedForTests(x); (x.records(), cap.records_per_container(), cap.name()) });
    let rt = c::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(k).map(|x| { let cap = c::retirement_tree::RetirementContainerCapacity::CappedForTests(x); (x.records(), cap.records_per_container(), cap.name()) });
    let full = (
        c::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat.records_per_container(),
        c::lineage_tree::LineageContainerCapacity::FromTheContainerFormat.records_per_container(),
        c::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat.records_per_container(),
        c::container_tree::containers_one_container_tree_root_holds(),
        c::inode_tree::leaf_containers_one_root_node_holds(),
        c::inode_tree::records_below_which_a_leaf_borrows_from_its_right_neighbour(),
        ContainerTreeNodeEntriesCappedForTests::new(k as usize).map(|x| x.entries()),
    );
    let position = k as usize;
    let pos = guard(|| (c::tombstone_tree::TombstoneContainerIndexInTree::of_position(position).position(), c::lineage_tree::LineageContainerIndexInTree::of_position(position).position(),
        c::retirement_tree::RetirementContainerIndexInTree::of_position(position).position(), c::inode_tree::InodeLeafContainerIndexInTree::of_position(position).position()), dbg);
    format!("{t:?} {l:?} {rt:?} {full:?} {pos}")
}
