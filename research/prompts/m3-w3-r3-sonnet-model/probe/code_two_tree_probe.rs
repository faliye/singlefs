
/// m3-w3-r3 正推腿的探针（只在副本里）：拿真的 `plan_the_tree_after_this_publish` 在记账树的 key 形状上逐发布走，
/// 每个发布打一行 `W3COST …`：重写的节点数、树高、节点总数、叶数。给 `cost_model.py` 里 Python 复刻的规划器对账。
#[cfg(test)]
mod w3_probe {
    use super::*;

    const STAT_TREE_NONE: u64 = 0;
    const INODE_TREE: u64 = 65537;
    const NO_DEVICE: u32 = 0xFFFF_FFFF;

    fn key(stat: u16, tree: u64, device: u32, generation: u64) -> CodeTwoTreeKey {
        let mut bytes = Vec::with_capacity(22);
        bytes.extend_from_slice(&stat.to_le_bytes());
        bytes.extend_from_slice(&tree.to_le_bytes());
        bytes.extend_from_slice(&device.to_le_bytes());
        bytes.extend_from_slice(&generation.to_le_bytes());
        CodeTwoTreeKey::new(&bytes, CodeTwoKeyFieldWidths::ACCOUNTING)
    }

    /// 一代的 15 行：每盘带设备维的六项（统计量 1、2、3、5、10、11），池级三行（4、6，水位 12 记在 inode 树上）。
    fn rows_of_one_generation(generation: u64) -> Vec<CodeTwoTreeKey> {
        let mut rows = Vec::new();
        for device in 0..2u32 {
            for stat in [1u16, 2, 3, 5, 10, 11] {
                rows.push(key(stat, STAT_TREE_NONE, device, generation));
            }
        }
        rows.push(key(4, STAT_TREE_NONE, NO_DEVICE, generation));
        rows.push(key(6, STAT_TREE_NONE, NO_DEVICE, generation));
        rows.push(key(12, INODE_TREE, NO_DEVICE, generation));
        rows
    }

    fn describe(plan: &CodeTwoTreePlan) -> (usize, u64, usize, usize) {
        let rewritten = plan.origins.iter().filter(|origin| **origin == CodeTwoTreeNodeOrigin::RewrittenThisPublish).count();
        let leaves = plan.shape.nodes().iter().filter(|node| matches!(node.contents, CodeTwoTreeNodeContents::Leaf { .. })).count();
        (rewritten, plan.shape.height(), plan.shape.nodes().len(), leaves)
    }

    /// 模式：每发布 k（从 1 起）是不是水位变了（留台阶行）：`change_every` = 1 每发布都变；3 表示 k % 3 != 0 的变。
    /// 走法：第 k 发布 = 删上一代的 15 行（上一代水位行若是台阶行则留着）、插这一代的 15 行。
    fn run(leaf: usize, internal: usize, publishes: u64, change_every: u64, label: &str, print_every: u64) {
        let capacity = CodeTwoTreeNodeCapacity { leaf_entries: leaf, internal_entries: internal };
        let mut shape = CodeTwoTreeShape::default();
        let mut keys: BTreeSet<CodeTwoTreeKey> = BTreeSet::new();
        let mut previous_step_row_is_kept = false;
        let mut previous_generation = 0u64;
        for k in 1..=publishes {
            let generation = 2 + k; // 第一代从 3 起
            let changed = change_every == 1 || k % change_every != 0;
            if k > 1 {
                for row in rows_of_one_generation(previous_generation) {
                    let is_watermark = matches!(&row, r if *r == key(12, INODE_TREE, NO_DEVICE, previous_generation));
                    if is_watermark && previous_step_row_is_kept {
                        continue;
                    }
                    keys.remove(&row);
                }
            }
            for row in rows_of_one_generation(generation) {
                keys.insert(row);
            }
            let plan = plan_the_tree_after_this_publish(&shape, &keys, capacity).expect("规划");
            let (rewritten, height, nodes, leaves) = describe(&plan);
            if k <= 4 || k % print_every == 0 || k == publishes {
                println!("W3COST {label} leaf={leaf} internal={internal} change_every={change_every} publish={k} rewritten={rewritten} height={height} nodes={nodes} leaves={leaves} keys={}", keys.len());
            }
            shape = plan.shape;
            // 这一代的水位行：这一代水位变了就是台阶行，下一发布留着它。
            previous_step_row_is_kept = changed;
            previous_generation = generation;
        }
    }

    /// 先一次性把「前 H 个发布都留了台阶行」那一刻的 key 集合从空形状装进去（按 key 升序逐把插，与逐发布往右端追加是同一串插入，
    /// 因为每发布都变时台阶行从不删），再接着逐发布走 `extra` 个：看真规划器在产品容量下树高变化的那几个发布。
    fn run_bulk(leaf: usize, internal: usize, prebuilt: u64, extra: u64, label: &str) {
        let capacity = CodeTwoTreeNodeCapacity { leaf_entries: leaf, internal_entries: internal };
        let mut keys: BTreeSet<CodeTwoTreeKey> = BTreeSet::new();
        for generation in 3..=(prebuilt + 1) {
            keys.insert(key(12, INODE_TREE, NO_DEVICE, generation)); // 前 H−1 代的水位行（都是台阶行，留着）
        }
        let mut previous_generation = 2 + prebuilt;
        for row in rows_of_one_generation(previous_generation) {
            keys.insert(row);
        }
        let plan = plan_the_tree_after_this_publish(&CodeTwoTreeShape::default(), &keys, capacity).expect("规划");
        let (rewritten, height, nodes, leaves) = describe(&plan);
        println!("W3COST {label} leaf={leaf} internal={internal} change_every=1 publish={prebuilt} rewritten={rewritten} height={height} nodes={nodes} leaves={leaves} keys={} (bulk)", keys.len());
        let mut shape = plan.shape;
        for k in (prebuilt + 1)..=(prebuilt + extra) {
            let generation = 2 + k;
            for row in rows_of_one_generation(previous_generation) {
                if row == key(12, INODE_TREE, NO_DEVICE, previous_generation) {
                    continue; // 上一代的水位行是台阶行，留着
                }
                keys.remove(&row);
            }
            for row in rows_of_one_generation(generation) {
                keys.insert(row);
            }
            let plan = plan_the_tree_after_this_publish(&shape, &keys, capacity).expect("规划");
            let (rewritten, height, nodes, leaves) = describe(&plan);
            println!("W3COST {label} leaf={leaf} internal={internal} change_every=1 publish={k} rewritten={rewritten} height={height} nodes={nodes} leaves={leaves} keys={}", keys.len());
            shape = plan.shape;
            previous_generation = generation;
        }
    }

    #[test]
    fn w3_cost_probe() {
        let which = std::env::var("W3_PROBE").unwrap_or_default();
        if which.contains("small") {
            run(7, 4, 3000, 1, "small-every-publish", 1);
            run(10, 3, 3000, 3, "small-two-of-three", 1);
            run(5, 2, 3000, 1, "tiny-every-publish", 1);
        }
        if which.contains("real") {
            run(477, 150, 6000, 1, "real-every-publish", 250);
            run(477, 150, 6000, 3, "real-two-of-three", 250);
        }
        if which.contains("bulk") {
            run_bulk(477, 150, 462, 4, "real-bulk-462");
            run_bulk(477, 150, 36_071, 5, "real-bulk-36071");
            run_bulk(477, 150, 100_000, 3, "real-bulk-100000");
            run_bulk(477, 150, 2_742_507, 5, "real-bulk-2742507");
        }
        if which.contains("big") {
            run(477, 150, 40000, 1, "real-every-publish-40k", 2000);
        }
    }
}
