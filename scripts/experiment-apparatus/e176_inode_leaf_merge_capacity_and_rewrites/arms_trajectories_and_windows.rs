//! 各条臂的变体参数、一条臂的轨迹与窗口里的重写、操作的施加与逐步记账。

use super::*;

/// 一条臂的变体：名字（产物里的臂字段）与它的合并策略。第一段跑 M1、M2 主、M3 主、M3 f = 59 四个；第二段跑 M1、M2 主、M3 主与另三个变体。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArmVariant {
    pub name: &'static str,
    pub policy: MergePolicy,
}

pub const ARM_DROP_EMPTY_ONLY: ArmVariant = ArmVariant { name: "M1", policy: MergePolicy::DropEmptyOnly };
pub const ARM_ABSORB_RIGHT_MAIN: ArmVariant =
    ArmVariant { name: "M2", policy: MergePolicy::AbsorbRightWhenTheyFit { order: NeighbourCheckOrder::RightPairFirst } };
pub const ARM_ABSORB_AND_BORROW_MAIN: ArmVariant = ArmVariant {
    name: "M3",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::RightPairFirst,
        floor: BORROW_FLOOR_MAIN,
        amount: BorrowAmount::HalfTheDifference,
    },
};
pub const ARM_ABSORB_AND_BORROW_FLOOR_59: ArmVariant = ArmVariant {
    name: "M3_floor59",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::RightPairFirst,
        floor: BORROW_FLOOR_SMALL,
        amount: BorrowAmount::HalfTheDifference,
    },
};
pub const ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR: ArmVariant = ArmVariant {
    name: "M3_just_enough",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::RightPairFirst,
        floor: BORROW_FLOOR_MAIN,
        amount: BorrowAmount::JustEnoughForTheFloor,
    },
};
pub const ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST: ArmVariant =
    ArmVariant { name: "M2_left_first", policy: MergePolicy::AbsorbRightWhenTheyFit { order: NeighbourCheckOrder::LeftPairFirst } };
/// 第二段「先查哪一对」那个旋钮在 M3 上的取值：下限与借多少取主旋钮，只把先查的那一对换成左边（跑前登记第五节、第八节）。
pub const ARM_ABSORB_AND_BORROW_LEFT_PAIR_FIRST: ArmVariant = ArmVariant {
    name: "M3_left_first",
    policy: MergePolicy::AbsorbRightAndBorrowFromRight {
        order: NeighbourCheckOrder::LeftPairFirst,
        floor: BORROW_FLOOR_MAIN,
        amount: BorrowAmount::HalfTheDifference,
    },
};

/// 第一段的四个臂变体，次序就是产物里的次序；下标 0 是 M1。
pub const SEGMENT_ONE_ARMS: [ArmVariant; 4] = [ARM_DROP_EMPTY_ONLY, ARM_ABSORB_RIGHT_MAIN, ARM_ABSORB_AND_BORROW_MAIN, ARM_ABSORB_AND_BORROW_FLOOR_59];
/// 第二段的六个臂变体：主窗口那三条（M1、M2 主、M3 主）当参照，加登记第五节第二段那三个变体；次序就是产物里的次序，下标 0 是 M1、1 是 M2 主、2 是 M3 主。
pub const SEGMENT_TWO_ARMS: [ArmVariant; 6] =
    [ARM_DROP_EMPTY_ONLY, ARM_ABSORB_RIGHT_MAIN, ARM_ABSORB_AND_BORROW_MAIN, ARM_ABSORB_AND_BORROW_JUST_ENOUGH_FOR_THE_FLOOR, ARM_ABSORB_RIGHT_LEFT_PAIR_FIRST, ARM_ABSORB_AND_BORROW_LEFT_PAIR_FIRST];

/// 一条臂在一条序列上为什么停下。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmStop {
    /// 第 `refused_operation_number` 个操作（从 1 数）是一次被拒的插入（叶容器数会超过 135）。
    Refused { refused_operation_number: u64 },
    /// 吃满 2 × 10⁷ 个操作还没被拒：「未撞到」。
    ReachedOperationCap,
}

/// 一条臂在一条序列上的轨迹（第八节），到它自己停下为止。填充率存成 (活数, 片数 × 233) 两个整数，比较按交叉相乘。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmTrajectory {
    pub arm: &'static str,
    pub stop: ArmStop,
    pub operations_accepted: u64,
    pub live_inodes_at_stop: u64,
    pub containers_peak: usize,
    pub publishes_at_root_capacity: u64,
    pub containers_final: usize,
    pub fill_smallest: (u64, u64),
    pub fill_final: (u64, u64),
    pub merges: u64,
    pub borrows: u64,
    pub third_sentence_false_peak: u64,
    pub third_sentence_false_positive_publishes: u64,
    pub third_sentence_false_final: u64,
    pub consumed_prefix_sha256: String,
}

/// 公共窗口里一条臂每次 unlink 的重写片数：两种记法的和、单次最大、分布（0、1、2、3–9、10–134 各几次）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowRewrites {
    pub content_sum: u64,
    pub position_sum: u64,
    pub content_largest: usize,
    pub position_largest: usize,
    pub content_histogram: [u64; 5],
    pub position_histogram: [u64; 5],
}

fn histogram_bucket(rewritten: usize) -> usize {
    match rewritten {
        0 => 0,
        1 => 1,
        2 => 2,
        3..=9 => 3,
        10..=134 => 4,
        _ => panic!("一次发布重写 {rewritten} 片：摘过片之后至多 134 片，没摘时内容记法至多 2 片"),
    }
}

/// 累计重写差 S(t) = Σ(臂 − M1)（第八节），t 走公共窗口里的最后一批：峰值、单次差 > 0 的次数、期末值。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CumulativeDifference {
    pub running: i64,
    pub peak: i64,
    pub positive_single_differences: u64,
}

impl CumulativeDifference {
    fn add(&mut self, single_difference: i64) {
        self.running += single_difference;
        self.peak = self.peak.max(self.running);
        if single_difference > 0 {
            self.positive_single_differences += 1;
        }
    }
}

/// 一个公共窗口：三条臂（下标指进这条序列的臂表，第 0 个是 M1）从头到三条里最早一次不接受操作之前。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComparisonWindow {
    pub label: &'static str,
    pub arm_indices: [usize; 3],
    pub is_open: bool,
    pub operations: u64,
    pub unlinks: u64,
    pub ended_by_arm: Option<&'static str>,
    pub prefix_sha256: [String; 3],
    pub operations_identical: bool,
    pub rewrites: [WindowRewrites; 3],
    /// [第 1 条臂 − M1 位置记法, 第 1 条 − M1 内容记法, 第 2 条 − M1 位置记法, 第 2 条 − M1 内容记法]
    pub cumulative: [CumulativeDifference; 4],
}

impl ComparisonWindow {
    pub(super) fn opened(label: &'static str, arm_indices: [usize; 3]) -> Self {
        Self {
            label,
            arm_indices,
            is_open: true,
            operations: 0,
            unlinks: 0,
            ended_by_arm: None,
            prefix_sha256: [String::new(), String::new(), String::new()],
            operations_identical: true,
            rewrites: [WindowRewrites::default(), WindowRewrites::default(), WindowRewrites::default()],
            cumulative: [CumulativeDifference::default(); 4],
        }
    }
}

/// 一条序列跑完的全部结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceResult {
    pub sequence: RegisteredSequence,
    pub trajectories: Vec<ArmTrajectory>,
    pub windows: Vec<ComparisonWindow>,
}

/// 一条臂在一步里吃到了什么。
#[derive(Clone, Debug)]
pub(super) enum StepOutcome {
    NotTaken,
    Refused(Operation),
    Created(Operation),
    Unlinked(Operation, usize, usize),
}

/// 一条臂跑的时候带着的状态。
pub(super) struct ArmRun {
    pub(super) variant: ArmVariant,
    pub(super) tree: InodeLeafTree,
    pub(super) generator: WorkloadGenerator,
    pub(super) hasher: StreamingSha256,
    pub(super) hasher_before_this_step: StreamingSha256,
    pub(super) trajectory: ArmTrajectory,
    pub(super) is_running: bool,
}

/// 读法 1（跑前登记第一节）：撞上上限时的活数 = 最后一个被接受的版本里的记录条数，被拒那一条不算。序列与锚点格共用这一处。
#[must_use]
#[allow(clippy::let_and_return, reason = "中间那个名字是变异表 V4 的锚点：它说清这里不算被拒那一条")]
pub fn live_inodes_at_refusal(tree_after_the_refused_insert: &InodeLeafTree) -> u64 {
    let live_inodes_not_counting_the_refused_insert = tree_after_the_refused_insert.live_inode_count();
    live_inodes_not_counting_the_refused_insert
}

fn fill_is_smaller(candidate: (u64, u64), current: (u64, u64)) -> bool {
    u128::from(candidate.0) * u128::from(current.1) < u128::from(current.0) * u128::from(candidate.1)
}

/// 把一个操作喂给一条臂：插入被拒就记停下（树不变、被拒那一条不算）；接受了就进操作流指纹、跑丙类断言、更新轨迹。
pub(super) fn apply_operation(run: &mut ArmRun, operation: Operation, sequence_name: &str, step: u64) -> StepOutcome {
    let (outcome, touched_positions) = match operation {
        Operation::Create(inode) => match run.tree.insert(inode) {
            InsertOutcome::Accepted => (StepOutcome::Created(operation), vec![run.tree.containers.len() - 1]),
            InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { .. } => {
                let live_inodes_when_refused = live_inodes_at_refusal(&run.tree);
                run.is_running = false;
                run.trajectory.stop = ArmStop::Refused { refused_operation_number: step + 1 };
                run.trajectory.live_inodes_at_stop = live_inodes_when_refused;
                return StepOutcome::Refused(operation);
            }
        },
        Operation::Unlink(inode) => {
            let rewrites = run.tree.remove(inode);
            let bound = content_notation_bound_per_removal(&run.tree);
            assert!(
                rewrites.content_notation_count() <= bound,
                "F4 丙类断言红：序列 {sequence_name} 臂 {} 第 {} 个操作 {operation:?} 内容记法重写 {} 片，上界 {bound}",
                run.variant.name,
                step + 1,
                rewrites.content_notation_count()
            );
            let content = rewrites.content_notation_count();
            let position = rewrites.position_notation_count();
            (StepOutcome::Unlinked(operation, content, position), rewrites.content_rewritten_positions)
        }
    };
    run.hasher.absorb_bytes(&operation.fingerprint_bytes());
    run.trajectory.operations_accepted += 1;
    let third_sentence_false_pairs = check_structure_after_publish(&run.tree, &touched_positions).unwrap_or_else(|violation| {
        panic!("F4 丙类断言红：序列 {sequence_name} 臂 {} 第 {} 个操作 {operation:?}：{violation}", run.variant.name, step + 1)
    });
    let containers = run.tree.containers.len();
    let trajectory = &mut run.trajectory;
    trajectory.containers_peak = trajectory.containers_peak.max(containers);
    if containers == ROOT_NODE_LEAF_CONTAINER_CAPACITY {
        trajectory.publishes_at_root_capacity += 1;
    }
    if containers > 0 {
        let fill = (run.tree.live_inode_count(), u64::try_from(containers * LEAF_RECORD_CAPACITY).expect("片数 × 233 装得进 u64"));
        if fill_is_smaller(fill, trajectory.fill_smallest) {
            trajectory.fill_smallest = fill;
        }
        trajectory.fill_final = fill;
    }
    trajectory.third_sentence_false_peak = trajectory.third_sentence_false_peak.max(third_sentence_false_pairs);
    if third_sentence_false_pairs > 0 {
        trajectory.third_sentence_false_positive_publishes += 1;
    }
    trajectory.third_sentence_false_final = third_sentence_false_pairs;
    outcome
}

fn operation_of(outcome: &StepOutcome) -> Option<Operation> {
    match outcome {
        StepOutcome::NotTaken => None,
        StepOutcome::Refused(operation) | StepOutcome::Created(operation) | StepOutcome::Unlinked(operation, _, _) => Some(*operation),
    }
}

fn accepted(outcome: &StepOutcome) -> bool {
    match outcome {
        StepOutcome::NotTaken | StepOutcome::Refused(_) => false,
        StepOutcome::Created(_) | StepOutcome::Unlinked(..) => true,
    }
}

/// 窗口还开着时记一步：有一条臂没接受这一步的操作就关窗（这一步不算），记三条臂已有的操作流指纹；否则三条臂都删了就记重写片数。
pub(super) fn record_step_in_window(window: &mut ComparisonWindow, runs: &[ArmRun], outcomes: &[StepOutcome], step: u64) {
    let triple_outcomes = window.arm_indices.map(|arm_index| &outcomes[arm_index]);
    if let Some(slot) = triple_outcomes.iter().position(|outcome| !accepted(outcome)) {
        window.is_open = false;
        window.operations = step;
        window.ended_by_arm = Some(runs[window.arm_indices[slot]].variant.name);
        for (slot_index, arm_index) in window.arm_indices.iter().enumerate() {
            let hasher = if accepted(&outcomes[*arm_index]) { &runs[*arm_index].hasher_before_this_step } else { &runs[*arm_index].hasher };
            window.prefix_sha256[slot_index] = hasher.hexadecimal_digest();
        }
        return;
    }
    let operations = triple_outcomes.map(operation_of);
    if operations[0] != operations[1] || operations[0] != operations[2] {
        window.operations_identical = false;
    }
    let StepOutcome::Unlinked(_, drop_empty_only_content, drop_empty_only_position) = triple_outcomes[0] else { return };
    window.unlinks += 1;
    for (slot_index, outcome) in triple_outcomes.iter().enumerate() {
        let StepOutcome::Unlinked(_, content, position) = outcome else {
            window.operations_identical = false;
            continue;
        };
        let rewrites = &mut window.rewrites[slot_index];
        rewrites.content_sum += u64::try_from(*content).expect("片数装得进 u64");
        rewrites.position_sum += u64::try_from(*position).expect("片数装得进 u64");
        rewrites.content_largest = rewrites.content_largest.max(*content);
        rewrites.position_largest = rewrites.position_largest.max(*position);
        rewrites.content_histogram[histogram_bucket(*content)] += 1;
        rewrites.position_histogram[histogram_bucket(*position)] += 1;
        if slot_index > 0 {
            let position_difference = i64::try_from(*position).expect("片数") - i64::try_from(*drop_empty_only_position).expect("片数");
            let content_difference = i64::try_from(*content).expect("片数") - i64::try_from(*drop_empty_only_content).expect("片数");
            window.cumulative[(slot_index - 1) * 2].add(position_difference);
            window.cumulative[(slot_index - 1) * 2 + 1].add(content_difference);
        }
    }
}

#[cfg(test)]
#[path = "arms_trajectories_and_windows_tests.rs"]
mod tests;
