//! m4-admission-delta-r1 攻方腿的计数模型（只在攻方模型上量过、被攻过零轮）。tools/rerun.sh 把这份拷进冻结副本的
//! crates/singlefs-core/tests/ 下、按 singlefs-core 的测试目标编。量的是冻结副本 crates/singlefs-core/src/code_two_tree.rs 的
//! `plan_the_tree_after_this_publish`（码 2 树的规划器，记账树、deadlist 树、中央映射树三棵共用）：
//! - 候选 A3 的字面式子「这一次进出的 key 数 × 树高」（正文第三节）是不是一次发布重写节点数的上界；
//! - 攻方推的式子「插的 key 数 × 2 × 之后的树高 + 删的 key 数 × 之前的树高」是不是上界（推的，只打出违反的次数，不断言）；
//! - 候选 A4（不改规划算法）下，只改一把 key 的发布，规划器交回的形状有几个节点、照抄的节点里有几个换了位置（位置就是角色身份）。

use std::collections::BTreeSet;

use singlefs_core::code_two_tree::{
    plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey,
    CodeTwoTreeNodeCapacity, CodeTwoTreeNodeOrigin, CodeTwoTreePlan, CodeTwoTreeShape,
};

/// 记账树的 key 宽（统计量 2 + 树 8 + 设备 4 + 代 8），只让代段取 `value`：key 的全序就是 `value` 的序。
fn accounting_key(value: u64) -> CodeTwoTreeKey {
    let mut bytes = Vec::with_capacity(22);
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&value.to_le_bytes());
    CodeTwoTreeKey::new(&bytes, CodeTwoKeyFieldWidths::ACCOUNTING)
}

fn planned(
    previous: &CodeTwoTreeShape,
    values: &BTreeSet<u64>,
    capacity: CodeTwoTreeNodeCapacity,
) -> CodeTwoTreePlan {
    let keys: BTreeSet<CodeTwoTreeKey> = values.iter().copied().map(accounting_key).collect();
    plan_the_tree_after_this_publish(previous, &keys, capacity)
        .expect("树高远低于 256 层，上一版的形状是规划器自己交回的")
}

/// 线性同余取样：只为可复现。
struct SampleStream(u64);

impl SampleStream {
    fn below(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % bound
    }
}

#[test]
fn one_key_into_a_full_root_leaf_rewrites_three_nodes_where_keys_times_height_gives_one() {
    for leaf_entries in [1_usize, 2, 3, 8, 64, 255] {
        let capacity = CodeTwoTreeNodeCapacity {
            leaf_entries,
            internal_entries: 4,
        };
        let full: BTreeSet<u64> = (0..u64::try_from(leaf_entries).expect("容量")).collect();
        let before = planned(&CodeTwoTreeShape::default(), &full, capacity).shape;
        assert_eq!(before.height(), 1, "装满一片叶的树只有根叶");
        let mut after = full.clone();
        after.insert(u64::try_from(leaf_entries).expect("容量"));
        let plan = planned(&before, &after, capacity);
        let rewritten = u64::try_from(plan.rewritten_positions().len()).expect("节点数");
        let keys_in_and_out: u64 = 1;
        let literal_bound = keys_in_and_out * before.height();
        println!(
            "name=full_root_leaf_one_insert leaf_entries={leaf_entries} height_before={} height_after={} rewritten={rewritten} literal_bound={literal_bound}",
            before.height(),
            plan.shape.height()
        );
        assert_eq!(rewritten, 3, "根叶装满再插一把：左半重写、右半新写、长出新根");
        assert!(rewritten > literal_bound);
    }
}

#[test]
fn keys_times_height_is_exceeded_over_sampled_histories() {
    let capacities = [(2_usize, 2_usize), (3, 3), (4, 4), (8, 8), (16, 16)];
    let mut publishes_judged: u64 = 0;
    let mut literal_bound_exceeded: u64 = 0;
    let mut largest_excess_over_the_literal_bound: i64 = i64::MIN;
    let mut corrected_bound_exceeded: u64 = 0;
    for (leaf_entries, internal_entries) in capacities {
        let capacity = CodeTwoTreeNodeCapacity {
            leaf_entries,
            internal_entries,
        };
        // 取样的历史：ascending 每次发布按升序插一把；其余四条每次删 0–2 把上一版有的、插 1–4 把上一版没有的。
        for history in 0..5_u64 {
            let mut stream = SampleStream(history * 7919 + u64::try_from(leaf_entries).expect("容量"));
            let mut values: BTreeSet<u64> = BTreeSet::new();
            let mut shape = CodeTwoTreeShape::default();
            let mut next_ascending: u64 = 0;
            for _publish in 0..400 {
                let mut after = values.clone();
                let (inserted, deleted) = if history == 0 {
                    after.insert(next_ascending);
                    next_ascending += 1;
                    (1_u64, 0_u64)
                } else {
                    let mut deleted = 0;
                    if values.len() > 1 {
                        let deletions = stream.below(3).min(u64::try_from(values.len() - 1).expect("条数"));
                        for _ in 0..deletions {
                            let index = usize::try_from(stream.below(u64::try_from(values.len()).expect("条数"))).expect("下标");
                            let victim = *values.iter().nth(index).expect("下标小于条数");
                            if after.remove(&victim) {
                                deleted += 1;
                            }
                        }
                    }
                    let insertions = 1 + stream.below(4);
                    let mut inserted = 0;
                    while inserted < insertions {
                        let candidate = stream.below(100_000);
                        if !values.contains(&candidate) && after.insert(candidate) {
                            inserted += 1;
                        }
                    }
                    (inserted, deleted)
                };
                let plan = planned(&shape, &after, capacity);
                let rewritten = u64::try_from(plan.rewritten_positions().len()).expect("节点数");
                let height_before = shape.height();
                let height_after = plan.shape.height();
                if height_before > 0 {
                    publishes_judged += 1;
                    let literal_bound = (inserted + deleted) * height_before;
                    if rewritten > literal_bound {
                        literal_bound_exceeded += 1;
                    }
                    let excess = i64::try_from(rewritten).expect("节点数") - i64::try_from(literal_bound).expect("上界");
                    largest_excess_over_the_literal_bound = largest_excess_over_the_literal_bound.max(excess);
                    let corrected_bound = inserted * 2 * height_after + deleted * height_before;
                    if rewritten > corrected_bound {
                        corrected_bound_exceeded += 1;
                        println!(
                            "name=corrected_bound_violation leaf_entries={leaf_entries} internal_entries={internal_entries} history={history} inserted={inserted} deleted={deleted} height_before={height_before} height_after={height_after} rewritten={rewritten} corrected_bound={corrected_bound}"
                        );
                    }
                }
                shape = plan.shape;
                values = after;
            }
        }
    }
    println!(
        "name=keys_times_height_sweep capacities=5 histories_per_capacity=5 publishes_per_history=400 publishes_judged={publishes_judged} literal_bound_exceeded={literal_bound_exceeded} largest_excess_over_the_literal_bound={largest_excess_over_the_literal_bound} corrected_bound_exceeded={corrected_bound_exceeded}"
    );
    assert!(literal_bound_exceeded > 0, "字面式子在取样里一次都没被超过：攻方这一格没打中");
}

#[test]
fn a_one_key_publish_gets_back_a_shape_holding_every_node_of_the_tree() {
    let capacity = CodeTwoTreeNodeCapacity {
        leaf_entries: 64,
        internal_entries: 64,
    };
    for total_keys in [1_000_u64, 10_000, 100_000] {
        let values: BTreeSet<u64> = (0..total_keys).map(|value| value * 2).collect();
        let tree = planned(&CodeTwoTreeShape::default(), &values, capacity).shape;
        // 插一把奇数 key，落在最左那片叶：它右边每一层的节点都可能挪位置。
        let mut after = values.clone();
        after.insert(1);
        let plan = planned(&tree, &after, capacity);
        let carried_at_a_new_position = plan
            .shape
            .nodes()
            .iter()
            .zip(&plan.origins)
            .filter(|(node, origin)| match origin {
                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => *previous_position != node.position,
                CodeTwoTreeNodeOrigin::RewrittenThisPublish => false,
            })
            .count();
        println!(
            "name=one_key_publish_output_size total_keys={total_keys} height={} nodes_before={} nodes_in_the_returned_shape={} origins_returned={} rewritten={} replaced={} carried_at_a_new_position={carried_at_a_new_position} input_key_set={}",
            plan.shape.height(),
            tree.nodes().len(),
            plan.shape.nodes().len(),
            plan.origins.len(),
            plan.rewritten_positions().len(),
            plan.replaced_previous_nodes.len(),
            after.len()
        );
        assert!(plan.shape.nodes().len() >= tree.nodes().len());
        assert!(
            u64::try_from(plan.rewritten_positions().len()).expect("节点数") <= 2 * plan.shape.height()
        );
    }
}
