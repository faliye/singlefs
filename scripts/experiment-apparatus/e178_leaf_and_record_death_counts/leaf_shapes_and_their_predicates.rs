//! 叶容器的形状、形状上要成立的句子与判法、形状目录与稀疏布局。

// ---------------------------------------------------------------------------------------------
// 树的形态与三个谓词（登记 5.6；第十一节 V7 在每个发布出去的版本上判同一组谓词）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ShapeLeaf {
    pub(super) container_number: u64,
    pub(super) record_keys: Vec<u64>,
}

impl ShapeLeaf {
    fn largest_key(&self) -> Option<u64> {
        self.record_keys.iter().copied().max()
    }

    fn smallest_key(&self) -> Option<u64> {
        self.record_keys.iter().copied().min()
    }
}

/// 一串叶容器加根里每条条目的分隔 key（分隔 key 与叶一一对应）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TreeShape {
    pub(super) leaves: Vec<ShapeLeaf>,
    pub(super) separators: Vec<u64>,
}

/// I-9.13：每片类型 2 容器记录数 ≥ 1。
pub(super) fn every_leaf_holds_a_record(shape: &TreeShape) -> bool {
    shape.leaves.iter().all(|leaf| !leaf.record_keys.is_empty())
}

/// I-9.4 的第一、二、四句（今天的写法与 N1 的写法共用）：记录号 ≥ 容器号；沿叶序容器号严格递增；同一片里记录号沿记录序严格递增。
fn container_number_sentences_shared_by_both_readings_hold(shape: &TreeShape) -> bool {
    let records_not_below_the_container_number = shape
        .leaves
        .iter()
        .all(|leaf| leaf.record_keys.iter().all(|key| *key >= leaf.container_number));
    let numbers_increase_along_the_leaf_order =
        shape.leaves.windows(2).all(|adjacent| adjacent[0].container_number < adjacent[1].container_number);
    let records_increase_inside_each_leaf = shape
        .leaves
        .iter()
        .all(|leaf| leaf.record_keys.windows(2).all(|adjacent_keys| adjacent_keys[0] < adjacent_keys[1]));
    records_not_below_the_container_number && numbers_increase_along_the_leaf_order && records_increase_inside_each_leaf
}

/// 今天的 I-9.4（四句）：第三句是「左容器的最大 key < 右容器号」（`crates/singlefs-checker/src/walk.rs:2768-2789`、`:3067`、`:3079`）。
fn container_numbers_hold_today(shape: &TreeShape) -> bool {
    if !container_number_sentences_shared_by_both_readings_hold(shape) {
        return false;
    }
    shape.leaves.windows(2).all(|adjacent| {
        let right_leaf = &adjacent[1];
        match adjacent[0].largest_key() {
            Some(left_largest_key) => left_largest_key < right_leaf.container_number,
            None => true,
        }
    })
}

/// N1 的 I-9.4：第三句换成「左容器的最大 key < 右容器的分隔 key」，其余三句不动（登记 5.3 的 N1 那一行）。
fn container_numbers_hold_with_raised_separators(shape: &TreeShape) -> bool {
    if !container_number_sentences_shared_by_both_readings_hold(shape) {
        return false;
    }
    shape
        .leaves
        .windows(2)
        .zip(shape.separators.iter().skip(1))
        .all(|(adjacent, right_separator)| match adjacent[0].largest_key() {
            Some(left_largest_key) => left_largest_key < *right_separator,
            None => true,
        })
}

/// I-9.12：分隔 key 严格递增；第 i 条 ≤ 第 i 个孩子的最小 key，且 > 第 i−1 个孩子的最大 key（`walk.rs:3245-3266`）。
fn separators_fall_outside_the_child_ranges(shape: &TreeShape) -> bool {
    shape.separators.iter().enumerate().all(|(position, separator)| {
        let not_above_its_child = shape.leaves[position].smallest_key().is_none_or(|smallest| *separator <= smallest);
        if position == 0 {
            return not_above_its_child;
        }
        let above_the_previous_separator = *separator > shape.separators[position - 1];
        let above_the_previous_child = shape.leaves[position - 1].largest_key().is_none_or(|largest| *separator > largest);
        not_above_its_child && above_the_previous_separator && above_the_previous_child
    })
}

/// 路由对不对：对形态里每个 key 取「最后一个分隔 key ≤ 目标的条目」，落到的是不是装它的那一片。
fn routing_finds_every_key(shape: &TreeShape) -> bool {
    shape.leaves.iter().enumerate().all(|(leaf_position, leaf)| {
        leaf.record_keys.iter().all(|key| {
            let routed = shape.separators.iter().rposition(|separator| *separator <= *key);
            routed == Some(leaf_position)
        })
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PredicateVerdicts {
    pub(super) today_container_numbers_hold: bool,
    pub(super) raised_separator_container_numbers_hold: bool,
    pub(super) separators_hold: bool,
    pub(super) routing_holds: bool,
}

pub(super) fn judge_the_shape(shape: &TreeShape) -> PredicateVerdicts {
    PredicateVerdicts {
        today_container_numbers_hold: container_numbers_hold_today(shape),
        raised_separator_container_numbers_hold: container_numbers_hold_with_raised_separators(shape),
        separators_hold: separators_fall_outside_the_child_ranges(shape),
        routing_holds: routing_finds_every_key(shape),
    }
}

pub(super) const fn red_or_green(holds: bool) -> &'static str {
    if holds {
        "green"
    } else {
        "red"
    }
}

/// 5.6 的一个形态。
#[derive(Clone, Debug)]
pub(super) struct NamedShape {
    pub(super) family: &'static str,
    pub(super) name: String,
    pub(super) shape: TreeShape,
}

fn leaf_of(container_number: u64, record_keys: &[u64]) -> ShapeLeaf {
    ShapeLeaf { container_number, record_keys: record_keys.to_vec() }
}

fn two_leaves(left: ShapeLeaf, right: ShapeLeaf, separators: [u64; 2]) -> TreeShape {
    TreeShape { leaves: vec![left, right], separators: separators.to_vec() }
}

/// 登记 5.6 的全表：K（现有，逐形抄 `crates/` 的测试）、S（分裂时号给错）、BN1、BN2（借之后）、DV（方向错）。
pub(super) fn catalogue_of_shapes() -> Vec<NamedShape> {
    let mut shapes = Vec::new();
    let named = |family: &'static str, name: String, shape: TreeShape| NamedShape { family, name, shape };
    // K：`checker_narrow_invariants_and_abandoned_roots.rs:858`（两叶、分隔 key 15）、`checker_known_bad_images.rs:1064-1070`（单叶号 5）、
    // `:10599`（同号两条）、`pool_checker_judges_the_deletion_state_of_every_walked_version.rs:1363`（倒序）。单叶那几形的根条目分隔 key 照旧是 1。
    shapes.push(named("K", "K-0".to_string(), two_leaves(leaf_of(1, &[1, 10]), leaf_of(11, &[20, 30]), [1, 15])));
    shapes.push(named("K", "K-1".to_string(), TreeShape { leaves: vec![leaf_of(5, &[1])], separators: vec![1] }));
    shapes.push(named("K", "K-2".to_string(), two_leaves(leaf_of(1, &[1, 10]), leaf_of(8, &[20, 30]), [1, 15])));
    shapes.push(named("K", "K-3".to_string(), two_leaves(leaf_of(1, &[1, 10]), leaf_of(0, &[20, 30]), [1, 15])));
    shapes.push(named("K", "K-4".to_string(), TreeShape { leaves: vec![leaf_of(1, &[1, 1])], separators: vec![1] }));
    shapes.push(named("K", "K-5".to_string(), TreeShape { leaves: vec![leaf_of(1, &[1, 3, 2])], separators: vec![1] }));
    // S：左 {1, 10} 号 1、分隔 key 1；右 {20, 30} 正确号 20；错号 m 两种分隔 key（= m、= 20）；m = 20 是合法对照。
    for wrong_number in [0, 1, 2, 9, 10, 11, 19, 20, 21, 30, 31] {
        for (separator_name, separator) in [("separator_equals_number", wrong_number), ("separator_twenty", 20)] {
            shapes.push(named(
                "S",
                format!("S-m{wrong_number}-{separator_name}"),
                two_leaves(leaf_of(1, &[1, 10]), leaf_of(wrong_number, &[20, 30]), [1, separator]),
            ));
        }
    }
    // BN1：左 {1, 10, 20} 号 1；右 {30} 号 20；分隔 key s；再在 s = 30 下换右号 m。
    for separator in [20, 25, 30, 31] {
        shapes.push(named(
            "BN1",
            format!("BN1-s{separator}"),
            two_leaves(leaf_of(1, &[1, 10, 20]), leaf_of(20, &[30]), [1, separator]),
        ));
    }
    for right_number in [0, 1, 10, 21, 31] {
        shapes.push(named(
            "BN1",
            format!("BN1-s30-m{right_number}"),
            two_leaves(leaf_of(1, &[1, 10, 20]), leaf_of(right_number, &[30]), [1, 30]),
        ));
    }
    // BN2：左 {1, 10, 20} 号 1；右 {30} 号 = 分隔 key = m。
    for right_number in [20, 25, 30, 31] {
        shapes.push(named(
            "BN2",
            format!("BN2-m{right_number}"),
            two_leaves(leaf_of(1, &[1, 10, 20]), leaf_of(right_number, &[30]), [1, right_number]),
        ));
    }
    // DV：右从左借（分隔 key 20 与 10）、右吸收左（单片，保留右身份与它的分隔 key）。
    for separator in [20, 10] {
        shapes.push(named(
            "DV",
            format!("DV-right-borrows-from-left-s{separator}"),
            two_leaves(leaf_of(1, &[1]), leaf_of(20, &[10, 20, 30]), [1, separator]),
        ));
    }
    shapes.push(named(
        "DV",
        "DV-right-absorbs-left".to_string(),
        TreeShape { leaves: vec![leaf_of(20, &[1, 10, 20, 30])], separators: vec![20] },
    ));
    shapes
}

/// 键布局的第二个取样点（第八节 G-K）：key 乘 100；夹在两个 key 之间的号与分隔 key 取两个 key 乘 100 之后的中点；0 不变。
fn scaled_by_one_hundred(value: u64, sorted_keys: &[u64]) -> u64 {
    if value == 0 || sorted_keys.contains(&value) {
        return value * 100;
    }
    let below = sorted_keys.iter().copied().filter(|key| *key < value).max();
    let above = sorted_keys.iter().copied().filter(|key| *key > value).min();
    match (below, above) {
        (Some(lower_key), Some(upper_key)) => (lower_key * 100 + upper_key * 100) / 2,
        (None, Some(_)) | (Some(_), None) | (None, None) => value * 100,
    }
}

pub(super) fn sparse_layout_of(shape: &TreeShape) -> TreeShape {
    let mut keys: Vec<u64> = shape.leaves.iter().flat_map(|leaf| leaf.record_keys.iter().copied()).collect();
    keys.sort_unstable();
    keys.dedup();
    TreeShape {
        leaves: shape
            .leaves
            .iter()
            .map(|leaf| ShapeLeaf {
                container_number: scaled_by_one_hundred(leaf.container_number, &keys),
                record_keys: leaf.record_keys.iter().map(|key| key * 100).collect(),
            })
            .collect(),
        separators: shape.separators.iter().map(|separator| scaled_by_one_hundred(*separator, &keys)).collect(),
    }
}

#[cfg(test)]
#[path = "leaf_shapes_and_their_predicates_tests.rs"]
mod tests;
