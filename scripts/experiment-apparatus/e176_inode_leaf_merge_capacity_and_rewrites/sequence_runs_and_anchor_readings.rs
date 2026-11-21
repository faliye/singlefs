//! 一条序列的整趟、窗口前缀的对齐、各种锚点的读数与 crates 单测的读数。

use super::*;

/// 第一段的两个公共窗口：主窗口（M1、M2 主、M3 主）与 f = 59 那一个（M1、M2 主、M3 f = 59），下标指进 [`SEGMENT_ONE_ARMS`]。
pub const SEGMENT_ONE_WINDOWS: [(&str, [usize; 3]); 2] = [("main", [0, 1, 2]), ("floor59", [0, 1, 3])];
/// 第二段的三个公共窗口（登记修订 3）：主窗口（M1、M2 主、M3 主）；借到刚好够下限那一个（M1、M2 主、M3 借到刚好够下限）；
/// 先查左边那一个（M1、M2 先查左边、M3 先查左边）。下标指进 [`SEGMENT_TWO_ARMS`]。
pub const SEGMENT_TWO_WINDOWS: [(&str, [usize; 3]); 3] = [("main", [0, 1, 2]), ("just_enough", [0, 1, 3]), ("left_first", [0, 4, 5])];

/// 一条序列喂给一组臂变体，逐步同走：每条臂各用一个生成器实例、各算各的操作流指纹，直到每条臂都停下（被拒或吃满上限）。
#[must_use]
pub fn run_sequence(
    sequence: RegisteredSequence,
    arms: &[ArmVariant],
    window_triples: &[(&'static str, [usize; 3])],
    operations_per_arm_at_most: u64,
) -> SequenceResult {
    let mut runs: Vec<ArmRun> = arms
        .iter()
        .map(|arm_variant| ArmRun {
            variant: *arm_variant,
            tree: InodeLeafTree::empty(arm_variant.policy, LeafContainerLimit::OneRootNode),
            generator: WorkloadGenerator::new(sequence.workload, sequence.seed),
            hasher: StreamingSha256::new(),
            hasher_before_this_step: StreamingSha256::new(),
            trajectory: ArmTrajectory {
                arm: arm_variant.name,
                stop: ArmStop::ReachedOperationCap,
                operations_accepted: 0,
                live_inodes_at_stop: 0,
                containers_peak: 0,
                publishes_at_root_capacity: 0,
                containers_final: 0,
                fill_smallest: (1, 1),
                fill_final: (0, 1),
                merges: 0,
                borrows: 0,
                third_sentence_false_peak: 0,
                third_sentence_false_positive_publishes: 0,
                third_sentence_false_final: 0,
                consumed_prefix_sha256: String::new(),
            },
            is_running: true,
        })
        .collect();
    let mut windows: Vec<ComparisonWindow> = window_triples.iter().map(|(label, arm_indices)| ComparisonWindow::opened(label, *arm_indices)).collect();
    let mut step: u64 = 0;
    while runs.iter().any(|run| run.is_running) {
        let any_window_open = windows.iter().any(|window| window.is_open);
        let mut outcomes: Vec<StepOutcome> = Vec::with_capacity(runs.len());
        for run in &mut runs {
            if !run.is_running {
                outcomes.push(StepOutcome::NotTaken);
                continue;
            }
            if run.trajectory.operations_accepted == operations_per_arm_at_most {
                run.is_running = false;
                run.trajectory.stop = ArmStop::ReachedOperationCap;
                run.trajectory.live_inodes_at_stop = run.tree.live_inode_count();
                outcomes.push(StepOutcome::NotTaken);
                continue;
            }
            if any_window_open {
                run.hasher_before_this_step = run.hasher.clone();
            }
            let operation = run.generator.next_operation();
            outcomes.push(apply_operation(run, operation, sequence.name, step));
        }
        for window in windows.iter_mut().filter(|window| window.is_open) {
            record_step_in_window(window, &runs, &outcomes, step);
        }
        step += 1;
    }
    let trajectories = runs
        .into_iter()
        .map(|run| {
            let mut trajectory = run.trajectory;
            trajectory.containers_final = run.tree.containers.len();
            trajectory.merges = run.tree.merges;
            trajectory.borrows = run.tree.borrows;
            trajectory.consumed_prefix_sha256 = run.hasher.hexadecimal_digest();
            trajectory
        })
        .collect();
    SequenceResult { sequence, trajectories, windows }
}

/// 第十一节作废条款：公共窗口内三条臂的操作流前缀指纹必须相同，逐步的操作也必须相同。不同就当场停下（这一段产物不算）。
pub fn assert_window_prefixes_agree(result: &SequenceResult) {
    for window in &result.windows {
        assert!(
            window.operations_identical
                && window.prefix_sha256[0] == window.prefix_sha256[1]
                && window.prefix_sha256[0] == window.prefix_sha256[2],
            "作废：序列 {} 窗口 {} 三条臂的操作流前缀不同：{:?}",
            result.sequence.name,
            window.label,
            window.prefix_sha256
        );
    }
}

/// 一个锚点格的读数：格名、臂、读到的（规范写法）、登记的答案（第七节；没有登记答案的写 None，只报不判）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorReading {
    pub cell: &'static str,
    pub arm: &'static str,
    pub observed: String,
    pub expected: Option<&'static str>,
}

impl AnchorReading {
    #[must_use]
    pub fn matches(&self) -> bool {
        self.expected.is_none_or(|expected| expected == self.observed)
    }
}

fn joined_sizes(tree: &InodeLeafTree) -> String {
    tree.containers.iter().map(|container| container.inode_numbers.len().to_string()).collect::<Vec<_>>().join("/")
}

/// 插入 `first_inode`、`first_inode` + 1 … 直到被拒，交回被拒的那个号。
fn insert_until_refused(tree: &mut InodeLeafTree, first_inode: u64) -> u64 {
    let mut inode = first_inode;
    loop {
        match tree.insert(inode) {
            InsertOutcome::Accepted => inode += 1,
            InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { .. } => return inode,
        }
    }
}

/// A1：插入 1, 2, … 直到被拒。
#[must_use]
pub fn anchor_insert_until_refused(arm: ArmVariant) -> String {
    let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
    let refused_inode = insert_until_refused(&mut tree, 1);
    format!("refused_insert={refused_inode} live={} containers={}", live_inodes_at_refusal(&tree), tree.containers.len())
}

/// A2：插入 1 … 31 455；对 k = 0 … 134 按号升序删 233k + 2 … 233k + 233；再插入直到被拒。
#[must_use]
pub fn anchor_one_record_per_leaf_then_insert_until_refused(arm: ArmVariant) -> String {
    let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
    for inode in 1..=31_455_u64 {
        assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "A2 建满 135 片之前不会被拒");
    }
    for leaf in 0..135_u64 {
        for inode in (233 * leaf + 2)..=(233 * leaf + 233) {
            let rewrites = tree.remove(inode);
            check_structure_after_publish(&tree, &rewrites.content_rewritten_positions).unwrap_or_else(|violation| panic!("A2 臂 {} 删 {inode}：{violation}", arm.name));
        }
    }
    let refused_inode = insert_until_refused(&mut tree, 31_456);
    format!(
        "refused_insert={refused_inode} live={} containers={} merges={} borrows={}",
        live_inodes_at_refusal(&tree),
        tree.containers.len(),
        tree.merges,
        tree.borrows
    )
}

/// A3–A6 那种手造场景：插入 1 … `inserted`，再按给定次序删；交回两种记法的合计、末态片数、并 / 借次数、末态 I-9.4 第三句为假的对数、末态各片记录数。
#[must_use]
pub fn anchor_scenario_tree(arm: ArmVariant, inserted: u64, deleted: &[u64]) -> (String, InodeLeafTree) {
    let mut tree = InodeLeafTree::empty(arm.policy, LeafContainerLimit::OneRootNode);
    for inode in 1..=inserted {
        assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "手造场景插不到 135 片");
    }
    let (mut content_total, mut position_total) = (0_usize, 0_usize);
    for inode in deleted {
        let rewrites = tree.remove(*inode);
        check_structure_after_publish(&tree, &rewrites.content_rewritten_positions).unwrap_or_else(|violation| panic!("场景 臂 {} 删 {inode}：{violation}", arm.name));
        content_total += rewrites.content_notation_count();
        position_total += rewrites.position_notation_count();
    }
    let observed = format!(
        "content={content_total} position={position_total} containers={} merges={} borrows={} third_sentence_false={} sizes={}",
        tree.containers.len(),
        tree.merges,
        tree.borrows,
        tree.adjacent_pairs_where_left_largest_reaches_right_container_number(),
        joined_sizes(&tree)
    );
    (observed, tree)
}

/// C1：不设 135 上限的 M1，按号 1 … 10⁶ 建满，再按号升序删 (n − 1) mod 10 ≠ 0 的每一条；交回叶数、活数、⌈活数 ÷ 233⌉、比值（四舍五入到两位）。
#[must_use]
pub fn anchor_c1() -> String {
    let mut tree = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::Unlimited);
    for inode in 1..=1_000_000_u64 {
        assert_eq!(tree.insert(inode), InsertOutcome::Accepted, "C1 不设上限");
    }
    for inode in (1..=1_000_000_u64).filter(|inode| (inode - 1) % 10 != 0) {
        let _ = tree.remove(inode);
    }
    let leaves = u64::try_from(tree.containers.len()).expect("叶数装得进 u64");
    let live = tree.live_inode_count();
    let ideal = live.div_ceil(u64::try_from(LEAF_RECORD_CAPACITY).expect("233"));
    let ratio_in_hundredths = (leaves * 200 + ideal) / (ideal * 2);
    format!("leaves={leaves} live={live} ideal={ideal} ratio={}.{:02}", ratio_in_hundredths / 100, ratio_in_hundredths % 100)
}

pub(super) fn ascending_run(first: u64, last: u64) -> Vec<u64> {
    (first..=last).collect()
}

/// `crates/` 单测钉住的数（第七节乙类末段），模型这边写本地常量，对拍 bin 回比 `crates/`：
/// 插入 1 … 500 各片记录数与容器号；插入 1 … 233 × 135 + 1 拒在第 136 片、上限 135；三片 [1 … 233][234][467] 删 234 的重写清单与摘片数。
#[must_use]
pub fn crates_unit_test_readings() -> Vec<AnchorReading> {
    let mut five_hundred = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::OneRootNode);
    for inode in 1..=500_u64 {
        assert_eq!(five_hundred.insert(inode), InsertOutcome::Accepted, "500 条装得下");
    }
    let container_numbers = five_hundred.containers.iter().map(|container| container.container_number.to_string()).collect::<Vec<_>>().join("/");
    let mut full = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::OneRootNode);
    let mut refusal = String::from("never");
    for inode in 1..=(233 * 135 + 1_u64) {
        if let InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { containers_it_would_take } = full.insert(inode) {
            refusal = format!("refused_inode={inode} refused_containers={containers_it_would_take} capacity={ROOT_NODE_LEAF_CONTAINER_CAPACITY}");
            break;
        }
    }
    let mut three = InodeLeafTree::from_containers(
        MergePolicy::DropEmptyOnly,
        LeafContainerLimit::OneRootNode,
        vec![
            LeafContainer { container_number: 1, inode_numbers: ascending_run(1, 233) },
            LeafContainer { container_number: 234, inode_numbers: vec![234] },
            LeafContainer { container_number: 467, inode_numbers: vec![467] },
        ],
    );
    let rewrites = three.remove(234);
    vec![
        AnchorReading {
            cell: "crates_insert_500",
            arm: "M1",
            observed: format!("sizes={} container_numbers={container_numbers}", joined_sizes(&five_hundred)),
            expected: Some("sizes=233/233/34 container_numbers=1/234/467"),
        },
        AnchorReading { cell: "crates_insert_31456", arm: "M1", observed: refusal, expected: Some("refused_inode=31456 refused_containers=136 capacity=135") },
        AnchorReading {
            cell: "crates_drop_middle",
            arm: "M1",
            observed: format!("position_rewritten={:?} dropped={}", rewrites.position_rewritten_positions, rewrites.containers_dropped),
            expected: Some("position_rewritten=[1] dropped=1"),
        },
    ]
}

/// 第七节甲、乙类全部锚点格，加上第五节「阳性对照对每条臂、每个旋钮取值都跑」里 f = 59 那一个变体（登记没给它答案，只报不判）。
#[must_use]
pub fn anchor_readings() -> Vec<AnchorReading> {
    let mut readings = Vec::new();
    for arm in [ARM_DROP_EMPTY_ONLY, ARM_ABSORB_RIGHT_MAIN, ARM_ABSORB_AND_BORROW_MAIN, ARM_ABSORB_AND_BORROW_FLOOR_59] {
        readings.push(AnchorReading { cell: "A1", arm: arm.name, observed: anchor_insert_until_refused(arm), expected: Some("refused_insert=31456 live=31455 containers=135") });
    }
    let one_record_per_leaf_expected: [(ArmVariant, Option<&'static str>); 4] = [
        (ARM_DROP_EMPTY_ONLY, Some("refused_insert=31688 live=367 containers=135 merges=0 borrows=0")),
        (ARM_ABSORB_RIGHT_MAIN, Some("refused_insert=62776 live=31455 containers=135 merges=134 borrows=0")),
        (ARM_ABSORB_AND_BORROW_MAIN, Some("refused_insert=62776 live=31455 containers=135 merges=134 borrows=918")),
        (ARM_ABSORB_AND_BORROW_FLOOR_59, None),
    ];
    for (arm, expected) in one_record_per_leaf_expected {
        readings.push(AnchorReading { cell: "A2", arm: arm.name, observed: anchor_one_record_per_leaf_then_insert_until_refused(arm), expected });
    }
    let deleted_inodes_1_to_117 = ascending_run(1, 117);
    let deleted_inodes_1_to_233 = ascending_run(1, 233);
    let deleted_inodes_234_to_350_and_467_to_583: Vec<u64> = ascending_run(234, 350).into_iter().chain(ascending_run(467, 583)).collect();
    let deleted_inodes_1_to_117_and_234_to_349: Vec<u64> = ascending_run(1, 117).into_iter().chain(ascending_run(234, 349)).collect();
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let scenarios: [(&'static str, u64, &[u64], [(ArmVariant, Option<&'static str>); 4]); 4] = [
        ("A3", 467, &deleted_inodes_1_to_117, [
            (ARM_DROP_EMPTY_ONLY, Some("content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1")),
            (ARM_ABSORB_RIGHT_MAIN, Some("content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1")),
            (ARM_ABSORB_AND_BORROW_MAIN, Some("content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176")),
            (ARM_ABSORB_AND_BORROW_FLOOR_59, None),
        ]),
        ("A4", 700, &deleted_inodes_1_to_233, [
            (ARM_DROP_EMPTY_ONLY, Some("content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1")),
            (ARM_ABSORB_RIGHT_MAIN, Some("content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1")),
            (ARM_ABSORB_AND_BORROW_MAIN, Some("content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1")),
            (ARM_ABSORB_AND_BORROW_FLOOR_59, None),
        ]),
        ("A5", 933, &deleted_inodes_234_to_350_and_467_to_583, [
            (ARM_DROP_EMPTY_ONLY, Some("content=234 position=234 containers=5 merges=0 borrows=0 third_sentence_false=0 sizes=233/116/116/233/1")),
            (ARM_ABSORB_RIGHT_MAIN, Some("content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1")),
            (ARM_ABSORB_AND_BORROW_MAIN, Some("content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1")),
            (ARM_ABSORB_AND_BORROW_FLOOR_59, None),
        ]),
        ("A6", 466, &deleted_inodes_1_to_117_and_234_to_349, [
            (ARM_DROP_EMPTY_ONLY, Some("content=233 position=233 containers=2 merges=0 borrows=0 third_sentence_false=0 sizes=116/117")),
            (ARM_ABSORB_RIGHT_MAIN, Some("content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233")),
            (ARM_ABSORB_AND_BORROW_MAIN, Some("content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233")),
            (ARM_ABSORB_AND_BORROW_FLOOR_59, None),
        ]),
    ];
    for (cell, inserted, deleted, arms) in scenarios {
        for (arm, expected) in arms {
            let (observed, tree) = anchor_scenario_tree(arm, inserted, deleted);
            readings.push(AnchorReading { cell, arm: arm.name, observed, expected });
            if cell == "A3" && arm == ARM_ABSORB_AND_BORROW_MAIN {
                let boundary = format!(
                    "leaf0_largest={} leaf1_smallest={}",
                    tree.containers[0].largest_inode_number(),
                    tree.containers[1].smallest_inode_number()
                );
                readings.push(AnchorReading { cell: "A3_boundary", arm: arm.name, observed: boundary, expected: Some("leaf0_largest=291 leaf1_smallest=292") });
            }
        }
    }
    let (just_enough_borrow_observed, _) = anchor_scenario_tree(ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR, 467, &deleted_inodes_1_to_117);
    readings.push(AnchorReading {
        cell: "A3",
        arm: ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR.name,
        observed: just_enough_borrow_observed,
        expected: Some("content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=117/233"),
    });
    let (left_pair_first_observed, _) = anchor_scenario_tree(ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST, 933, &deleted_inodes_234_to_350_and_467_to_583);
    readings.push(AnchorReading {
        cell: "A5",
        arm: ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST.name,
        observed: left_pair_first_observed,
        expected: Some("content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1"),
    });
    readings.push(AnchorReading { cell: "C1", arm: "M1_unlimited", observed: anchor_c1(), expected: Some("leaves=4292 live=100000 ideal=430 ratio=9.98") });
    readings.extend(crates_unit_test_readings());
    readings
}

/// 第二段的三个臂变体（登记第五节第二段那一行），次序就是锚点行里的次序。
pub const SEGMENT_TWO_VARIANTS: [ArmVariant; 3] = [ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR, ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST, ARM_ABSORB_AND_BORROW_LEFT_PAIR_FIRST];

/// 第二段的阳性对照（登记修订 3）：第一段的全部锚点格（M1、M2 主、M3 主在第二段照样进窗口），加第二段三个变体在 A1 … A6 上还没有读数的格。
/// A1 只插入、臂不起作用，三个变体都按 A1 那一格的登记答案判；A3 借到刚好够下限、A5 M2 先查左边两格第一段里已有、带登记答案；其余只报不判。
#[must_use]
pub fn segment_two_anchor_readings() -> Vec<AnchorReading> {
    let mut readings = anchor_readings();
    for arm in SEGMENT_TWO_VARIANTS {
        readings.push(AnchorReading { cell: "A1", arm: arm.name, observed: anchor_insert_until_refused(arm), expected: Some("refused_insert=31456 live=31455 containers=135") });
    }
    for arm in SEGMENT_TWO_VARIANTS {
        readings.push(AnchorReading { cell: "A2", arm: arm.name, observed: anchor_one_record_per_leaf_then_insert_until_refused(arm), expected: None });
    }
    let deleted_inodes_1_to_117 = ascending_run(1, 117);
    let deleted_inodes_1_to_233 = ascending_run(1, 233);
    let deleted_inodes_234_to_350_and_467_to_583: Vec<u64> = ascending_run(234, 350).into_iter().chain(ascending_run(467, 583)).collect();
    let deleted_inodes_1_to_117_and_234_to_349: Vec<u64> = ascending_run(1, 117).into_iter().chain(ascending_run(234, 349)).collect();
    let scenarios: [(&'static str, u64, &[u64]); 4] = [("A3", 467, &deleted_inodes_1_to_117), ("A4", 700, &deleted_inodes_1_to_233), ("A5", 933, &deleted_inodes_234_to_350_and_467_to_583), ("A6", 466, &deleted_inodes_1_to_117_and_234_to_349)];
    for (cell, inserted, deleted) in scenarios {
        for arm in SEGMENT_TWO_VARIANTS {
            let has_a_segment_one_reading = (cell == "A3" && arm == ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR) || (cell == "A5" && arm == ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST);
            if has_a_segment_one_reading {
                continue;
            }
            let (observed, _) = anchor_scenario_tree(arm, inserted, deleted);
            readings.push(AnchorReading { cell, arm: arm.name, observed, expected: None });
        }
    }
    readings
}

#[cfg(test)]
#[path = "sequence_runs_and_anchor_readings_tests.rs"]
mod tests;
