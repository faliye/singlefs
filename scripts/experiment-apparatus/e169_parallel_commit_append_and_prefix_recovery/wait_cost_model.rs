//! 等待代价的节点、场景与计划、关键路径、轨迹与按场景求等待量及增长判定。

use super::*;

// ===== 第三段：fsync 等待模型（登记 5.7；离散事件，抽象单位，不是本机计时；读法见登记第十二节修订 3、4） =====

/// 登记 5.7 的抽象单位换成整数：1 个抽象单位 = 1000 个整数单位（w = 0.05 → 50），算术全用整数。
pub(super) const WAIT_UNITS_PER_ABSTRACT_UNIT: u64 = 1000;
/// 登记 5.7：k ∈ {1, 2, 4, 8, 16, 32, 64}。
const WAIT_TRANSACTION_COUNTS: [u32; 7] = [1, 2, 4, 8, 16, 32, 64];
/// 登记 5.7：每个事务 r 相同，r ∈ {1, 2}。
pub(super) const WAIT_RECORD_COUNTS: [u32; 2] = [1, 2];
/// 登记第六节 Q2-3：ρ = W_max(64) ÷ W_max(8)。
pub(super) const GROWTH_RATIO_LARGE_TRANSACTION_COUNT: u32 = 64;
pub(super) const GROWTH_RATIO_SMALL_TRANSACTION_COUNT: u32 = 8;
/// 登记第六节 Q2-3 的门槛：ρ ≥ 2 判「随 k 增长」，ρ ≤ 1.25 判「有界」；写成（分子，分母），按整数乘法比。
pub(super) const REGISTERED_GROWS_THRESHOLD: (u64, u64) = (2, 1);
pub(super) const REGISTERED_BOUNDED_THRESHOLD: (u64, u64) = (5, 4);
/// 登记 5.7 只给了甲两条臂（追加者）与乙整段（预留）的等待规则，对照丁没有，第三段只跑这三条臂（修订 4 第 1 条）。
pub(super) const WAIT_APPEND_ARMS: [AppendArm; 3] = [AppendArm::JiaWhole, AppendArm::JiaInterleaved, AppendArm::YiReservedWhole];
pub(super) const WAIT_CONDITIONS: [PublishCondition; 4] = [
    PublishCondition::SerialPublishes,
    PublishCondition::SharedPublish,
    PublishCondition::MultiFlightRootsOrdered,
    PublishCondition::MultiFlightRootsUnordered,
];

/// 登记 5.7 的两个代价取样点（拍的，方向相反）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum CostPoint {
    /// P1 设备主导：w = 0.05、b = 1、h = g = 0.005、Δ = 5。
    DeviceDominated,
    /// P2 CPU 主导：w = 0.05、b = 1、h = g = 0.5、Δ = 5。
    CpuDominated,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct WaitCosts {
    pub(super) write: u64,
    pub(super) barrier_or_forced_unit_access: u64,
    pub(super) cpu_step: u64,
    pub(super) slow_head_delay: u64,
}

impl CostPoint {
    pub(super) const ALL: [CostPoint; 2] = [CostPoint::DeviceDominated, CostPoint::CpuDominated];
    pub(super) fn label(self) -> &'static str {
        match self {
            CostPoint::DeviceDominated => "p1_device_dominated",
            CostPoint::CpuDominated => "p2_cpu_dominated",
        }
    }
    pub(super) fn costs(self) -> WaitCosts {
        match self {
            CostPoint::DeviceDominated => WaitCosts { write: 50, barrier_or_forced_unit_access: 1000, cpu_step: 5, slow_head_delay: 5000 },
            CostPoint::CpuDominated => WaitCosts { write: 50, barrier_or_forced_unit_access: 1000, cpu_step: 500, slow_head_delay: 5000 },
        }
    }
}

/// 登记 5.7 的两种准备时间剖面。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum PreparationProfile {
    /// 等速：每个事务的计划在时刻 0 就绪。
    Uniform,
    /// 慢头：T1 在时刻 Δ 就绪，其余在 0。
    SlowHead,
}

impl PreparationProfile {
    pub(super) const ALL: [PreparationProfile; 2] = [PreparationProfile::Uniform, PreparationProfile::SlowHead];
    pub(super) fn label(self) -> &'static str {
        match self {
            PreparationProfile::Uniform => "uniform",
            PreparationProfile::SlowHead => "slow_head",
        }
    }
    fn ready_time_of(self, transaction: u32, costs: &WaitCosts) -> u64 {
        match self {
            PreparationProfile::SlowHead if transaction == 1 => costs.slow_head_delay,
            PreparationProfile::SlowHead | PreparationProfile::Uniform => 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaitServer {
    /// 设备：一个先进先出的服务台。
    Device,
    /// 甲两条臂的追加者（CPU 单服务台，每条记录耗 h）。
    Appender,
    /// 乙整段的预留（CPU 单服务台，每个事务耗 g）。
    Reserver,
}

impl WaitServer {
    pub(super) fn position(self) -> usize {
        match self {
            WaitServer::Device => 0,
            WaitServer::Appender => 1,
            WaitServer::Reserver => 2,
        }
    }
    /// 关键路径上「同一服务台的前一个」是哪一类前驱（读法 ④）。
    fn queue_role(self) -> PredecessorRole {
        match self {
            WaitServer::Device => PredecessorRole::DeviceQueue,
            WaitServer::Appender | WaitServer::Reserver => PredecessorRole::Cpu,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaitStepKind {
    UnitWrite,
    SharedUnitWrite,
    RecordWrite,
    SystemConfigurationWrite,
    Barrier,
    RootSlotForcedUnitAccess,
    AppenderStep,
    ReservationStep,
}

/// 关键路径分量（登记 5.7 最后一条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PathComponent {
    BarrierOrForcedUnitAccess,
    DeviceWrite,
    CpuStep,
}

impl WaitStepKind {
    fn server(self) -> WaitServer {
        match self {
            WaitStepKind::UnitWrite
            | WaitStepKind::SharedUnitWrite
            | WaitStepKind::RecordWrite
            | WaitStepKind::SystemConfigurationWrite
            | WaitStepKind::Barrier
            | WaitStepKind::RootSlotForcedUnitAccess => WaitServer::Device,
            WaitStepKind::AppenderStep => WaitServer::Appender,
            WaitStepKind::ReservationStep => WaitServer::Reserver,
        }
    }
    /// 读法 ③：R2 的「被一道完成的屏障或 FUA 罩住」。
    fn covers_earlier_writes(self) -> bool {
        match self {
            WaitStepKind::Barrier | WaitStepKind::RootSlotForcedUnitAccess => true,
            WaitStepKind::UnitWrite
            | WaitStepKind::SharedUnitWrite
            | WaitStepKind::RecordWrite
            | WaitStepKind::SystemConfigurationWrite
            | WaitStepKind::AppenderStep
            | WaitStepKind::ReservationStep => false,
        }
    }
    fn path_component(self) -> PathComponent {
        match self {
            WaitStepKind::Barrier | WaitStepKind::RootSlotForcedUnitAccess => PathComponent::BarrierOrForcedUnitAccess,
            WaitStepKind::UnitWrite | WaitStepKind::SharedUnitWrite | WaitStepKind::RecordWrite | WaitStepKind::SystemConfigurationWrite => PathComponent::DeviceWrite,
            WaitStepKind::AppenderStep | WaitStepKind::ReservationStep => PathComponent::CpuStep,
        }
    }
}

fn wait_step_cost(kind: WaitStepKind, costs: &WaitCosts) -> u64 {
    match kind {
        WaitStepKind::UnitWrite | WaitStepKind::SharedUnitWrite | WaitStepKind::RecordWrite | WaitStepKind::SystemConfigurationWrite => costs.write,
        WaitStepKind::Barrier | WaitStepKind::RootSlotForcedUnitAccess => costs.barrier_or_forced_unit_access,
        WaitStepKind::AppenderStep => costs.cpu_step,
        WaitStepKind::ReservationStep => costs.cpu_step,
    }
}

/// 关键路径回追时前驱的类别；声明次序就是同刻时的先后（读法 ④：同一发布 > 设备队列 > CPU；跨发布的次序约束与就绪时刻是修订 4 第 3 条补的位置）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum PredecessorRole {
    SamePublish,
    DeviceQueue,
    CrossPublish,
    Cpu,
    ReadySource,
}

#[derive(Clone, Debug)]
struct WaitNode {
    kind: WaitStepKind,
    pub(super) publish_position: usize,
    /// 同刻时第一个键：事务号（发布级的操作取这次发布里最小的事务号）。
    pub(super) transaction: u32,
    /// 同刻时第二个键：在事务（或发布）里的先后。
    order_within_transaction: u32,
    predecessors: Vec<(usize, PredecessorRole)>,
    /// 这一步最早能发出的时刻（事务就绪，或预留从时刻 0 起）。
    ready_source: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct WaitScenario {
    pub(super) condition: PublishCondition,
    pub(super) append_arm: AppendArm,
    pub(super) profile: PreparationProfile,
    pub(super) point: CostPoint,
    pub(super) record_count: u32,
    pub(super) transaction_count: u32,
}

struct WaitPlan {
    nodes: Vec<WaitNode>,
    transaction_publish: Vec<usize>,
    /// 每个事务自己的记录写，按记录序。
    transaction_record_writes: Vec<Vec<usize>>,
    publish_final_barrier: Vec<usize>,
}

/// 登记 5.7 的写序与次序约束（读法 ①：同一事务内同步，同一次发布里不同事务的 U 各自就绪即发出）。
fn plan_wait_scenario(scenario: &WaitScenario) -> WaitPlan {
    let costs = scenario.point.costs();
    let transaction_count = scenario.transaction_count;
    let record_count = scenario.record_count;
    let publish_groups: Vec<Vec<u32>> = match scenario.condition {
        PublishCondition::SharedPublish => vec![(1..=transaction_count).collect()],
        PublishCondition::SerialPublishes | PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered => {
            (1..=transaction_count).map(|transaction| vec![transaction]).collect()
        }
    };
    let roots_in_txg_order = scenario.condition == PublishCondition::MultiFlightRootsOrdered;
    let mut nodes: Vec<WaitNode> = Vec::new();
    let mut add = |node: WaitNode| -> usize {
        nodes.push(node);
        nodes.len() - 1
    };
    let reservation_of: Vec<Option<usize>> = (1..=transaction_count)
        .map(|transaction| match scenario.append_arm {
            AppendArm::YiReservedWhole => Some(add(WaitNode {
                kind: WaitStepKind::ReservationStep,
                publish_position: 0,
                transaction,
                order_within_transaction: 0,
                predecessors: Vec::new(),
                ready_source: Some(0),
            })),
            AppendArm::JiaWhole | AppendArm::JiaInterleaved | AppendArm::DingNoDiscipline => None,
        })
        .collect();
    let mut transaction_publish = vec![0usize; transaction_position(transaction_count + 1)];
    let mut transaction_record_writes: Vec<Vec<usize>> = vec![Vec::new(); transaction_position(transaction_count + 1)];
    let mut appender_steps: Vec<Vec<usize>> = vec![Vec::new(); transaction_position(transaction_count + 1)];
    let mut publish_final_barrier: Vec<usize> = Vec::new();
    let mut previous_root_slot: Option<usize> = None;
    let mut previous_system_configuration: Option<usize> = None;
    for (publish_position, group) in publish_groups.iter().enumerate() {
        let publish_transaction = group[0];
        let publish_node = |kind: WaitStepKind, order_within_transaction: u32, predecessors: Vec<(usize, PredecessorRole)>| WaitNode {
            kind,
            publish_position,
            transaction: publish_transaction,
            order_within_transaction,
            predecessors,
            ready_source: None,
        };
        let mut units = Vec::new();
        for transaction in group {
            transaction_publish[transaction_position(*transaction)] = publish_position;
            let mut predecessors = Vec::new();
            // 串落：Ti 的第一个操作在 Ti−1 的 B3 完成之后发出。
            if scenario.condition == PublishCondition::SerialPublishes {
                if let Some(previous_final_barrier) = publish_final_barrier.last() {
                    predecessors.push((*previous_final_barrier, PredecessorRole::CrossPublish));
                }
            }
            units.push(add(WaitNode {
                kind: WaitStepKind::UnitWrite,
                publish_position,
                transaction: *transaction,
                order_within_transaction: 0,
                predecessors,
                ready_source: Some(scenario.profile.ready_time_of(*transaction, &costs)),
            }));
        }
        let shared_unit = add(publish_node(WaitStepKind::SharedUnitWrite, 1, units.iter().map(|unit| (*unit, PredecessorRole::SamePublish)).collect()));
        let barrier_after_units = add(publish_node(WaitStepKind::Barrier, 2, vec![(shared_unit, PredecessorRole::SamePublish)]));
        let mut publish_record_writes = Vec::new();
        for transaction in group {
            for record_index in 1..=record_count {
                let previous_write = transaction_record_writes[transaction_position(*transaction)].last().copied();
                let mut write_predecessors = Vec::new();
                match scenario.append_arm {
                    AppendArm::JiaWhole | AppendArm::JiaInterleaved => {
                        let mut step_predecessors = vec![(barrier_after_units, PredecessorRole::SamePublish)];
                        if let Some(previous) = previous_write {
                            step_predecessors.push((previous, PredecessorRole::SamePublish));
                        }
                        if record_index == 1 {
                            for earlier in 1..*transaction {
                                let earlier_steps = &appender_steps[transaction_position(earlier)];
                                let gate = match scenario.append_arm {
                                    AppendArm::JiaWhole => *earlier_steps.last().expect("前面的事务已经排好"),
                                    AppendArm::JiaInterleaved => *earlier_steps.first().expect("前面的事务已经排好"),
                                    AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => panic!("只有甲两条臂有追加者"),
                                };
                                step_predecessors.push((gate, PredecessorRole::Cpu));
                            }
                        }
                        let step = add(WaitNode {
                            kind: WaitStepKind::AppenderStep,
                            publish_position,
                            transaction: *transaction,
                            order_within_transaction: record_index,
                            predecessors: step_predecessors,
                            ready_source: None,
                        });
                        appender_steps[transaction_position(*transaction)].push(step);
                        write_predecessors.push((step, PredecessorRole::Cpu));
                    }
                    AppendArm::YiReservedWhole => {
                        write_predecessors.push((barrier_after_units, PredecessorRole::SamePublish));
                        write_predecessors.push((reservation_of[transaction_position(*transaction)].expect("乙整段每个事务都预留"), PredecessorRole::Cpu));
                        if let Some(previous) = previous_write {
                            write_predecessors.push((previous, PredecessorRole::SamePublish));
                        }
                    }
                    AppendArm::DingNoDiscipline => panic!("第三段不跑对照丁"),
                }
                let write = add(WaitNode {
                    kind: WaitStepKind::RecordWrite,
                    publish_position,
                    transaction: *transaction,
                    order_within_transaction: 2 + record_index,
                    predecessors: write_predecessors,
                    ready_source: None,
                });
                transaction_record_writes[transaction_position(*transaction)].push(write);
                publish_record_writes.push(write);
            }
        }
        let barrier_after_records = add(publish_node(WaitStepKind::Barrier, 5, publish_record_writes.iter().map(|write| (*write, PredecessorRole::SamePublish)).collect()));
        let mut root_slot_predecessors = vec![(barrier_after_records, PredecessorRole::SamePublish)];
        let mut system_configuration_predecessors = Vec::new();
        // 多飞·根序：F_i 在 F_{i−1} 完成之后、C_i 在 C_{i−1} 完成之后发出。
        if roots_in_txg_order {
            if let Some(previous) = previous_root_slot {
                root_slot_predecessors.push((previous, PredecessorRole::CrossPublish));
            }
            if let Some(previous) = previous_system_configuration {
                system_configuration_predecessors.push((previous, PredecessorRole::CrossPublish));
            }
        }
        let root_slot = add(publish_node(WaitStepKind::RootSlotForcedUnitAccess, 6, root_slot_predecessors));
        system_configuration_predecessors.insert(0, (root_slot, PredecessorRole::SamePublish));
        let system_configuration = add(publish_node(WaitStepKind::SystemConfigurationWrite, 7, system_configuration_predecessors));
        let final_barrier = add(publish_node(WaitStepKind::Barrier, 8, vec![(system_configuration, PredecessorRole::SamePublish)]));
        previous_root_slot = Some(root_slot);
        previous_system_configuration = Some(system_configuration);
        publish_final_barrier.push(final_barrier);
    }
    WaitPlan { nodes, transaction_publish, transaction_record_writes, publish_final_barrier }
}

struct WaitTimeline {
    ready: Vec<u64>,
    start: Vec<u64>,
    complete: Vec<u64>,
    duration: Vec<u64>,
    /// 同一服务台上紧挨在前面处理的那一步。
    server_previous: Vec<Option<usize>>,
    /// 设备上的处理次序。
    device_order: Vec<usize>,
    pub(super) merged_barriers: u64,
}

/// 离散事件：每个服务台先进先出（按发出时刻，同刻按事务号、再按事务里的先后，读法 ②）；一步在「它发出」与「同一服务台前一步完成」两者较晚时开始。
fn simulate_wait(plan: &WaitPlan, costs: &WaitCosts) -> WaitTimeline {
    let node_count = plan.nodes.len();
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    let mut unresolved: Vec<usize> = plan.nodes.iter().map(|node| node.predecessors.len()).collect();
    for (node_index, node) in plan.nodes.iter().enumerate() {
        for (predecessor, _) in &node.predecessors {
            successors[*predecessor].push(node_index);
        }
    }
    let mut ready: Vec<Option<u64>> = vec![None; node_count];
    let mut start = vec![0u64; node_count];
    let mut complete: Vec<Option<u64>> = vec![None; node_count];
    let mut duration = vec![0u64; node_count];
    let mut server_previous: Vec<Option<usize>> = vec![None; node_count];
    let mut device_order: Vec<usize> = Vec::new();
    let mut merged_barriers = 0u64;
    let mut queues: [BTreeSet<(u64, u32, u32, usize)>; 3] = [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
    let mut in_progress: [Option<usize>; 3] = [None; 3];
    let mut busy_until = [0u64; 3];
    let mut last_on_server: [Option<usize>; 3] = [None; 3];
    let ready_time_of = |node_index: usize, completed_times: &[Option<u64>]| -> u64 {
        let node = &plan.nodes[node_index];
        let from_predecessors = node.predecessors.iter().map(|(predecessor, _)| completed_times[*predecessor].expect("前驱都完成了")).max();
        let from_source = node.ready_source;
        from_predecessors.into_iter().chain(from_source).max().expect("每一步不是有前驱就是有就绪时刻")
    };
    for node_index in 0..node_count {
        if unresolved[node_index] == 0 {
            let time = ready_time_of(node_index, &complete);
            ready[node_index] = Some(time);
            let node = &plan.nodes[node_index];
            queues[node.kind.server().position()].insert((time, node.transaction, node.order_within_transaction, node_index));
        }
    }
    loop {
        let next_completion = in_progress.iter().flatten().map(|node_index| (complete[*node_index].expect("在做的那一步有完成时刻"), *node_index)).min();
        let next_start = (0..3)
            .filter(|server| in_progress[*server].is_none())
            .filter_map(|server| queues[server].first().map(|(ready_time, ..)| ((*ready_time).max(busy_until[server]), server)))
            .min();
        // 同一时刻先把完成的放出去，再开始新的一步：同刻发出的几步才按事务号排得到。
        match (next_completion, next_start) {
            (None, None) => break,
            (Some((completion_time, node_index)), start_option) if start_option.is_none_or(|(start_time, _)| completion_time <= start_time) => {
                let server = plan.nodes[node_index].kind.server().position();
                in_progress[server] = None;
                for successor in &successors[node_index] {
                    unresolved[*successor] -= 1;
                    if unresolved[*successor] == 0 {
                        let time = ready_time_of(*successor, &complete);
                        ready[*successor] = Some(time);
                        let node = &plan.nodes[*successor];
                        queues[node.kind.server().position()].insert((time, node.transaction, node.order_within_transaction, *successor));
                    }
                }
            }
            (_, Some((start_time, server))) => {
                let (_, _, _, node_index) = queues[server].pop_first().expect("刚看过非空");
                let kind = plan.nodes[node_index].kind;
                // 读法 ⑤：连着的屏障（设备上紧挨在前的一步也是屏障，中间没有写）不占时间，跨发布也合并。
                let follows_a_barrier = last_on_server[server].is_some_and(|previous| plan.nodes[previous].kind == WaitStepKind::Barrier);
                let step_duration = if kind == WaitStepKind::Barrier && follows_a_barrier { 0 } else { wait_step_cost(kind, costs) };
                merged_barriers += u64::from(kind == WaitStepKind::Barrier && step_duration == 0);
                start[node_index] = start_time;
                duration[node_index] = step_duration;
                complete[node_index] = Some(start_time + step_duration);
                server_previous[node_index] = last_on_server[server];
                last_on_server[server] = Some(node_index);
                busy_until[server] = start_time + step_duration;
                in_progress[server] = Some(node_index);
                if server == WaitServer::Device.position() {
                    device_order.push(node_index);
                }
            }
            (Some(_), None) => panic!("有在做的一步却没放出去"),
        }
    }
    assert!(unresolved.iter().all(|count| *count == 0), "每一步的前驱都完成了：次序约束没有卡死");
    WaitTimeline {
        ready: ready.into_iter().map(|time| time.expect("每一步都发出了")).collect(),
        start,
        complete: complete.into_iter().map(|time| time.expect("每一步都做完了")).collect(),
        duration,
        server_previous,
        device_order,
        merged_barriers,
    }
}

/// 关键路径上的一个点：一步的完成，或一步的发出（R1 的「已发出」）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaitPathPoint {
    CompletionOf(usize),
    IssueOf(usize),
}

/// 读法 ③：设备上在这条写之后处理的第一道屏障或 FUA（先进先出，它就是这条写完成之后最早完成的那一道，跨发布也算）。
fn first_covering_operation_after(plan: &WaitPlan, timeline: &WaitTimeline, write: usize) -> usize {
    let position = timeline.device_order.iter().position(|node_index| *node_index == write).expect("这条写在设备上做过");
    *timeline.device_order[position + 1..]
        .iter()
        .find(|node_index| plan.nodes[**node_index].kind.covers_earlier_writes())
        .expect("每次发布最后都有屏障")
}

/// D13（验证路线） 已定项 2 条件 ② 的「已追加」，每个事务一项：（时刻，关键路径从哪个点往回追）。
fn appended_events(plan: &WaitPlan, timeline: &WaitTimeline, reading: AppendedReading) -> Vec<(u64, WaitPathPoint)> {
    plan.transaction_record_writes
        .iter()
        .map(|writes| match reading {
            AppendedReading::IssuedOnly => {
                let last_issued = *writes.iter().max_by_key(|write| (timeline.ready[**write], **write)).expect("r ≥ 1");
                (timeline.ready[last_issued], WaitPathPoint::IssueOf(last_issued))
            }
            AppendedReading::IssuedAndCovered => {
                let last_record = *writes.iter().max_by_key(|write| timeline.start[**write]).expect("r ≥ 1");
                let cover = first_covering_operation_after(plan, timeline, last_record);
                (timeline.complete[cover], WaitPathPoint::CompletionOf(cover))
            }
        })
        .collect()
}

fn point_role(plan: &WaitPlan, point: WaitPathPoint, own_publish: usize) -> PredecessorRole {
    let node_index = match point {
        WaitPathPoint::CompletionOf(node_index) | WaitPathPoint::IssueOf(node_index) => node_index,
    };
    if plan.nodes[node_index].publish_position == own_publish {
        PredecessorRole::SamePublish
    } else {
        PredecessorRole::CrossPublish
    }
}

/// 一个事务的返回：时刻与它的候选前驱（本发布 B3 的完成；每个 j < i 的「已追加」）。
fn return_candidates(plan: &WaitPlan, timeline: &WaitTimeline, transaction: u32, appended: &[(u64, WaitPathPoint)]) -> Vec<(u64, PredecessorRole, Option<WaitPathPoint>)> {
    let own_publish = plan.transaction_publish[transaction_position(transaction)];
    let final_barrier = plan.publish_final_barrier[own_publish];
    let mut candidates = vec![(timeline.complete[final_barrier], PredecessorRole::SamePublish, Some(WaitPathPoint::CompletionOf(final_barrier)))];
    for (time, point) in &appended[..transaction_position(transaction)] {
        candidates.push((*time, point_role(plan, *point, own_publish), Some(*point)));
    }
    candidates
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CriticalPath {
    pub(super) barriers_or_forced_unit_access: u64,
    pub(super) device_writes: u64,
    pub(super) cpu_steps: u64,
    pub(super) merged_barriers: u64,
    pub(super) tie_steps: u64,
    pub(super) source_time: u64,
    pub(super) duration_sum: u64,
}

/// 读法 ④：取定下这一刻的那个前驱；几个前驱同刻时按角色的声明次序取，同一步以几个角色出现时按最靠前的那个算，同刻的步数照报。
fn choose_predecessor(candidates: &[(u64, PredecessorRole, Option<WaitPathPoint>)], expected_time: u64, path: &mut CriticalPath) -> Option<WaitPathPoint> {
    let latest = candidates.iter().map(|(time, ..)| *time).max().expect("至少一个候选");
    assert_eq!(latest, expected_time, "这一刻由某个前驱定下");
    let mut at_latest: Vec<(PredecessorRole, Option<WaitPathPoint>)> = candidates.iter().filter(|(time, ..)| *time == latest).map(|(_, role, point)| (*role, *point)).collect();
    at_latest.sort_by_key(|(role, _)| *role);
    let mut distinct_points: Vec<Option<WaitPathPoint>> = Vec::new();
    for (_, point) in &at_latest {
        if !distinct_points.contains(point) {
            distinct_points.push(*point);
        }
    }
    path.tie_steps += u64::from(distinct_points.len() > 1);
    at_latest[0].1
}

fn critical_path_of(plan: &WaitPlan, timeline: &WaitTimeline, transaction: u32, appended: &[(u64, WaitPathPoint)], return_time: u64) -> CriticalPath {
    let mut path = CriticalPath::default();
    let mut point = choose_predecessor(&return_candidates(plan, timeline, transaction, appended), return_time, &mut path);
    let mut time_at_point = return_time;
    while let Some(current) = point {
        let (node_index, include_server_queue) = match current {
            WaitPathPoint::CompletionOf(node_index) => (node_index, true),
            WaitPathPoint::IssueOf(node_index) => (node_index, false),
        };
        let node = &plan.nodes[node_index];
        let expected_time = if include_server_queue {
            assert_eq!(timeline.complete[node_index], time_at_point, "路径上的完成时刻接得上");
            let step_duration = timeline.duration[node_index];
            path.duration_sum += step_duration;
            match (node.kind.path_component(), step_duration) {
                (PathComponent::BarrierOrForcedUnitAccess, 0) => path.merged_barriers += 1,
                (PathComponent::BarrierOrForcedUnitAccess, _) => path.barriers_or_forced_unit_access += 1,
                (PathComponent::DeviceWrite, _) => path.device_writes += 1,
                (PathComponent::CpuStep, _) => path.cpu_steps += 1,
            }
            timeline.start[node_index]
        } else {
            timeline.ready[node_index]
        };
        let mut candidates: Vec<(u64, PredecessorRole, Option<WaitPathPoint>)> = node
            .predecessors
            .iter()
            .map(|(predecessor, role)| (timeline.complete[*predecessor], *role, Some(WaitPathPoint::CompletionOf(*predecessor))))
            .collect();
        if let Some(source_time) = node.ready_source {
            candidates.push((source_time, PredecessorRole::ReadySource, None));
        }
        if include_server_queue {
            if let Some(previous) = timeline.server_previous[node_index] {
                candidates.push((timeline.complete[previous], node.kind.server().queue_role(), Some(WaitPathPoint::CompletionOf(previous))));
            }
        }
        point = choose_predecessor(&candidates, expected_time, &mut path);
        time_at_point = expected_time;
    }
    path.source_time = time_at_point;
    path
}

/// 登记 8.1 的轨迹：（初始值，吃光初始供给之后的峰值，值为正的事件步数，期末值）。事件步 = 时刻 0 与每一步的完成时刻（去重），截到期末那一刻为止（修订 4 第 6 条）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WaitTrajectory {
    pub(super) initial: u64,
    pub(super) peak_after_drain: u64,
    pub(super) positive_steps: u64,
    pub(super) final_value: u64,
}

fn trajectory_over(event_times: &[u64], end_time: u64, value_at: &dyn Fn(u64) -> u64) -> WaitTrajectory {
    let mut trajectory = WaitTrajectory { initial: value_at(0), peak_after_drain: 0, positive_steps: 0, final_value: 0 };
    let mut drained = false;
    for time in event_times.iter().copied().filter(|time| *time <= end_time) {
        let value = value_at(time);
        trajectory.positive_steps += u64::from(value > 0);
        if drained {
            trajectory.peak_after_drain = trajectory.peak_after_drain.max(value);
        }
        drained |= value == 0;
    }
    trajectory.final_value = value_at(end_time);
    trajectory
}

#[derive(Clone, Debug)]
pub(super) struct WaitResult {
    pub(super) return_times: Vec<u64>,
    pub(super) latest_return_time: u64,
    pub(super) latest_transaction: u32,
    pub(super) path: CriticalPath,
    pub(super) pending: WaitTrajectory,
    /// 乙整段：已预留而还没写的槽里 jsn 最小那个与已写的最大 jsn 之差（甲两条臂为 None）。
    pub(super) reserved_lag: Option<WaitTrajectory>,
    pub(super) merged_barriers_on_device: u64,
}

impl WaitResult {
    pub(super) fn return_time_sum(&self) -> u64 {
        self.return_times.iter().sum()
    }
}

pub(super) fn evaluate_wait(scenario: &WaitScenario, reading: AppendedReading) -> WaitResult {
    let plan = plan_wait_scenario(scenario);
    let costs = scenario.point.costs();
    let timeline = simulate_wait(&plan, &costs);
    let appended = appended_events(&plan, &timeline, reading);
    let return_times: Vec<u64> = (1..=scenario.transaction_count)
        .map(|transaction| return_candidates(&plan, &timeline, transaction, &appended).iter().map(|(time, ..)| *time).max().expect("至少本发布的 B3"))
        .collect();
    let latest_return_time = *return_times.iter().max().expect("k ≥ 1");
    // 同时最晚返回的取事务号最大的那个（修订 4 第 2 条）。
    let latest_transaction = (1..=scenario.transaction_count).rev().find(|transaction| return_times[transaction_position(*transaction)] == latest_return_time).expect("有最晚的");
    let path = critical_path_of(&plan, &timeline, latest_transaction, &appended, latest_return_time);
    assert_eq!(path.duration_sum + path.source_time, latest_return_time, "关键路径上各步耗时之和加起点时刻等于返回时刻");
    let mut event_times: Vec<u64> = std::iter::once(0).chain(timeline.complete.iter().copied()).collect();
    event_times.sort_unstable();
    event_times.dedup();
    let appended_times: Vec<u64> = appended[..transaction_position(latest_transaction)].iter().map(|(time, _)| *time).collect();
    let pending = trajectory_over(&event_times, latest_return_time, &|time| u64::try_from(appended_times.iter().filter(|appended_time| **appended_time > time).count()).expect("至多 63"));
    let reserved_lag = match scenario.append_arm {
        AppendArm::YiReservedWhole => {
            let record_count = u64::from(scenario.record_count);
            // (jsn, 预留完成时刻, 发出时刻)：Ti 预留 [(i − 1)·r + 1, i·r]。
            let slots: Vec<(u64, u64, u64)> = (1..=scenario.transaction_count)
                .flat_map(|transaction| {
                    let reservation = plan.nodes.iter().position(|node| node.kind == WaitStepKind::ReservationStep && node.transaction == transaction).expect("乙整段每个事务都预留");
                    let reserved_at = timeline.complete[reservation];
                    plan.transaction_record_writes[transaction_position(transaction)]
                        .iter()
                        .enumerate()
                        .map(move |(record_position, write)| (u64::from(transaction - 1) * record_count + u64::try_from(record_position).expect("r ≤ 2") + 1, reserved_at, *write))
                        .collect::<Vec<_>>()
                })
                .map(|(jsn, reserved_at, write)| (jsn, reserved_at, timeline.ready[write]))
                .collect();
            let lag_at = |time: u64| -> u64 {
                let largest_written = slots.iter().filter(|(_, _, issued_at)| *issued_at <= time).map(|(jsn, ..)| *jsn).max();
                let smallest_reserved_unwritten = slots.iter().filter(|(_, reserved_at, issued_at)| *reserved_at <= time && *issued_at > time).map(|(jsn, ..)| *jsn).min();
                match (largest_written, smallest_reserved_unwritten) {
                    (Some(written), Some(unwritten)) => written.saturating_sub(unwritten),
                    (Some(_), None) | (None, Some(_)) | (None, None) => 0,
                }
            };
            Some(trajectory_over(&event_times, latest_return_time, &lag_at))
        }
        AppendArm::JiaWhole | AppendArm::JiaInterleaved | AppendArm::DingNoDiscipline => None,
    };
    WaitResult { return_times, latest_return_time, latest_transaction, path, pending, reserved_lag, merged_barriers_on_device: timeline.merged_barriers }
}

pub(super) fn format_wait_units(units: u64) -> String {
    format!("{}.{:03}", units / WAIT_UNITS_PER_ABSTRACT_UNIT, units % WAIT_UNITS_PER_ABSTRACT_UNIT)
}

/// 分数按四位小数四舍五入打出来（只给人看；判定用整数乘法比）。
pub(super) fn format_ratio(numerator: u64, denominator: u64) -> String {
    let scaled = (numerator * 10_000 + denominator / 2) / denominator;
    format!("{}.{:04}", scaled / 10_000, scaled % 10_000)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum GrowthJudgement {
    Grows,
    Bounded,
    Between,
}

impl GrowthJudgement {
    pub(super) fn label(self) -> &'static str {
        match self {
            GrowthJudgement::Grows => "grows",
            GrowthJudgement::Bounded => "bounded",
            GrowthJudgement::Between => "between_not_judged",
        }
    }
}

/// ρ = large ÷ small 与两道门槛比：ρ ≥ grows ⇔ large·grows分母 ≥ grows分子·small。
pub(super) fn growth_judgement(small_transaction_count_latest_return: u64, large_transaction_count_latest_return: u64, grows_threshold: (u64, u64), bounded_threshold: (u64, u64)) -> GrowthJudgement {
    if large_transaction_count_latest_return * grows_threshold.1 >= grows_threshold.0 * small_transaction_count_latest_return {
        GrowthJudgement::Grows
    } else if large_transaction_count_latest_return * bounded_threshold.1 <= bounded_threshold.0 * small_transaction_count_latest_return {
        GrowthJudgement::Bounded
    } else {
        GrowthJudgement::Between
    }
}

/// 一格等待模型的键：（Q1 条件，Q2 臂，剖面，取样点，r，读法，k）。
pub(super) type WaitKey = (PublishCondition, AppendArm, PreparationProfile, CostPoint, u32, AppendedReading, u32);
/// 一条判决的键：等待格的键去掉 k。
pub(super) type WaitVerdictKey = (PublishCondition, AppendArm, PreparationProfile, CostPoint, u32, AppendedReading);

fn wait_scenario_of(key: WaitKey) -> WaitScenario {
    let (condition, append_arm, profile, point, record_count, _, transaction_count) = key;
    WaitScenario { condition, append_arm, profile, point, record_count, transaction_count }
}

pub(super) fn wait_grid() -> BTreeMap<WaitKey, WaitResult> {
    let mut grid = BTreeMap::new();
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for profile in PreparationProfile::ALL {
                for point in CostPoint::ALL {
                    for record_count in WAIT_RECORD_COUNTS {
                        for reading in AppendedReading::ALL {
                            for transaction_count in WAIT_TRANSACTION_COUNTS {
                                let key = (condition, arm, profile, point, record_count, reading, transaction_count);
                                grid.insert(key, evaluate_wait(&wait_scenario_of(key), reading));
                            }
                        }
                    }
                }
            }
        }
    }
    grid
}

/// 登记第七节 B6：k = 1、r = 1：甲两条臂 W = 4w + 4b + h，乙整段 W = 4w + 4b（g ≤ 2w + b），慢头各加 Δ。
pub(super) fn wait_anchor_single_transaction(arm: AppendArm, profile: PreparationProfile, point: CostPoint) -> u64 {
    let costs = point.costs();
    let appender = match arm {
        AppendArm::JiaWhole | AppendArm::JiaInterleaved => costs.cpu_step,
        AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => 0,
    };
    4 * costs.write + 4 * costs.barrier_or_forced_unit_access + appender + profile.ready_time_of(1, &costs)
}
