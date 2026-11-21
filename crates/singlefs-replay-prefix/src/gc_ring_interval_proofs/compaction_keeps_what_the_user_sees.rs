//! G7：整理对用户看得见的东西的效应。用户看得见的那一层（观察、用户操作、根环步骤到这一层的投影），
//! 整理那份与不整理那份历史的对应关系与它一步一步保持的引理，以及 `theorem_compaction_keeps_what_the_user_sees`。
use super::*;

verus! {

// ===== G7：整理对用户看得见的东西的效应 =====
// D13 已定项 18 性质那一句：整理不改现行内容；
// 回退候选集是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k（k 为这一段整理占的发布数），按版本号点名。
// 这一层只留用户看得见的：根的逻辑内容与名字、内存 F、持久 F、现行内容、根环容量与下一个版本号。整理那几步里不发布的（写目的地、换映射、回收）
// 与读者不改这一层，整理的发布在这一层上是「推一个不带版本号的根」；见 lemma_run_ring_steps_project_onto_the_observed_operations。
// 历史里可以有崩溃（两份同时崩溃，G7 连崩溃一起成立）：崩溃让内存 F 回到持久 F、现行回到最后一个根，
// 所以持久 F 之上的根也要一层一层对得上，两份最后一个根的内容也要相同。整理的发布只在现行内容与最后一个根相同时发
// （compaction_publish_is_accepted；穷举 g7_named_roots.py 同样只在这时插整理的发布）：不然它把用户还没落成根的改动先落盘，
// 崩溃之后整理那份的现行比不整理那份新。根环那一层的整理的发布带着同一道门（run_ring_step_is_enabled），
// lemma_run_ring_steps_project_onto_the_observed_operations 说两道门相同；门拦下的那一段历史在
// lemma_a_compaction_publish_is_refused_while_the_head_is_ahead_of_the_last_root。
// 只说两份都接受了每一步的历史：同一步两份都拒的是两边的空操作，删掉它不改结论；不整理那份接受、整理那份拒的
// 就是候选移出窗口本身，之后两份不再是同一串用户历史。整理那份接受、不整理那份拒的从不出现
// （lemma_compaction_never_admits_an_operation_the_plain_history_refuses）。
pub struct RingObservation {
    pub roots: Seq<ObservedRoot>,
    pub in_memory_floor: nat,
    pub persisted_floor: nat,
    pub head_logical: Map<u64, Seq<Seq<u8>>>,
    pub window: nat,
    pub next_user_version: nat,
}

pub enum UserOperation {
    Publish { logical: Map<u64, Seq<Seq<u8>>> },
    Checkpoint,
    Rollback { version: nat },
    RaiseFloor { version: nat },
    Crash,
}

pub open spec fn observation_of(
    roots: Seq<ObservedRoot>,
    in_memory_floor: nat,
    persisted_floor: nat,
    head_logical: Map<u64, Seq<Seq<u8>>>,
    window: nat,
    next_user_version: nat,
) -> RingObservation {
    RingObservation { roots, in_memory_floor, persisted_floor, head_logical, window, next_user_version }
}

pub open spec fn observe_the_run_ring(state: RunRingState) -> RingObservation {
    observation_of(
        observed_roots_of(state.roots),
        state.in_memory_floor,
        state.persisted_floor,
        state.head.logical,
        state.window,
        state.next_user_version,
    )
}

pub open spec fn observed_floor_after_a_checkpoint(observation: RingObservation) -> nat {
    if observation.roots.len() + 1 - observation.in_memory_floor > observation.window {
        (observation.in_memory_floor + 1) as nat
    } else {
        observation.in_memory_floor
    }
}

// 整理的发布落成根之后的持久 F 与内存 F：persisted_floor_after_a_compaction_publish、in_memory_floor_after_a_compaction_publish 在这一层上的写法。
pub open spec fn observed_persisted_floor_after_a_compaction_publish(observation: RingObservation) -> nat {
    if observation.roots.len() + 1 - observation.persisted_floor > observation.window {
        (observation.persisted_floor + 1) as nat
    } else {
        observation.persisted_floor
    }
}

pub open spec fn observed_in_memory_floor_after_a_compaction_publish(observation: RingObservation) -> nat {
    if observation.in_memory_floor >= observed_persisted_floor_after_a_compaction_publish(observation) {
        observation.in_memory_floor
    } else {
        observed_persisted_floor_after_a_compaction_publish(observation)
    }
}

pub open spec fn user_operation_is_accepted(observation: RingObservation, operation: UserOperation) -> bool {
    match operation {
        UserOperation::Publish { logical: _ } => true,
        UserOperation::Checkpoint => true,
        UserOperation::Rollback { version } => version_is_a_candidate(
            observation.roots,
            observation.in_memory_floor,
            version,
        ),
        UserOperation::RaiseFloor { version } => version_is_a_candidate(
            observation.roots,
            observation.in_memory_floor,
            version,
        ),
        UserOperation::Crash => true,
    }
}

pub open spec fn observation_after_user_operation(observation: RingObservation, operation: UserOperation) -> RingObservation {
    match operation {
        UserOperation::Publish { logical } => observation_of(
            observation.roots,
            observation.in_memory_floor,
            observation.persisted_floor,
            logical,
            observation.window,
            observation.next_user_version,
        ),
        UserOperation::Checkpoint => observation_of(
            observation.roots.push(
                ObservedRoot {
                    logical: observation.head_logical,
                    name: RootName::UserVersion { version: observation.next_user_version },
                },
            ),
            observed_floor_after_a_checkpoint(observation),
            observed_floor_after_a_checkpoint(observation),
            observation.head_logical,
            observation.window,
            observation.next_user_version + 1,
        ),
        UserOperation::Rollback { version } => observation_of(
            observation.roots,
            observation.in_memory_floor,
            observation.persisted_floor,
            observation.roots[index_of_the_candidate_version(
                observation.roots,
                observation.in_memory_floor,
                version,
            )].logical,
            observation.window,
            observation.next_user_version,
        ),
        UserOperation::RaiseFloor { version } => observation_of(
            observation.roots,
            index_of_the_candidate_version(observation.roots, observation.in_memory_floor, version) as nat,
            observation.persisted_floor,
            observation.head_logical,
            observation.window,
            observation.next_user_version,
        ),
        // 崩溃：内存 F 回到持久 F，现行回到最后一个根。
        UserOperation::Crash => observation_of(
            observation.roots,
            observation.persisted_floor,
            observation.persisted_floor,
            observation.roots[observation.roots.len() - 1].logical,
            observation.window,
            observation.next_user_version,
        ),
    }
}

pub open spec fn observation_after_a_compaction_publish(observation: RingObservation) -> RingObservation {
    observation_of(
        observation.roots.push(ObservedRoot { logical: observation.head_logical, name: RootName::CompactionPublish }),
        observed_in_memory_floor_after_a_compaction_publish(observation),
        observed_persisted_floor_after_a_compaction_publish(observation),
        observation.head_logical,
        observation.window,
        observation.next_user_version,
    )
}

// 整理的发布只在现行内容与最后一个根相同时发：整理不把用户还没落成根的改动先落盘。
pub open spec fn compaction_publish_is_accepted(observation: RingObservation) -> bool {
    &&& observation.roots.len() >= 1
    &&& observation.head_logical == observation.roots[observation.roots.len() - 1].logical
}

// 根环的每一步在用户看得见的那一层上是什么：不发布的步骤（写、回收、搬迁、读者取得与释放）不改这一层；用户发布、两种发布、
// 按版本号回退、只在内存里抬 F 与崩溃各是这一层的一步，根环允许的这一层也接受。整理的发布、回退与抬 F 三样，根环允许的恰是这一层接受的：
// G7 要「整理那份也接受每一步」，其中整理的发布那一条（compaction_publish_is_accepted）由此从根环的门推出，不是另加的假设。
pub proof fn lemma_run_ring_steps_project_onto_the_observed_operations(state: RunRingState, step: RunRingStep)
    ensures
        match step {
            RunRingStep::Write { slot: _, bytes: _ } => observe_the_run_ring(run_ring_next(state, step)) == observe_the_run_ring(state),
            RunRingStep::Reclaim { slot: _ } => observe_the_run_ring(run_ring_next(state, step)) == observe_the_run_ring(state),
            RunRingStep::Relocate { key: _, destination: _ } => observe_the_run_ring(run_ring_next(state, step))
                == observe_the_run_ring(state),
            RunRingStep::UserPublish { new_view } => observe_the_run_ring(run_ring_next(state, step))
                == observation_after_user_operation(
                observe_the_run_ring(state),
                UserOperation::Publish { logical: new_view.logical },
            ),
            RunRingStep::UserCheckpoint => observe_the_run_ring(run_ring_next(state, step)) == observation_after_user_operation(
                observe_the_run_ring(state),
                UserOperation::Checkpoint,
            ),
            RunRingStep::CompactionCheckpoint => {
                &&& run_ring_step_is_enabled(state, step) == compaction_publish_is_accepted(observe_the_run_ring(state))
                &&& observe_the_run_ring(run_ring_next(state, step)) == observation_after_a_compaction_publish(
                    observe_the_run_ring(state),
                )
            },
            RunRingStep::Rollback { version } => {
                &&& run_ring_step_is_enabled(state, step) == user_operation_is_accepted(
                    observe_the_run_ring(state),
                    UserOperation::Rollback { version },
                )
                &&& (run_ring_step_is_enabled(state, step) ==> observe_the_run_ring(run_ring_next(state, step))
                    == observation_after_user_operation(observe_the_run_ring(state), UserOperation::Rollback { version }))
            },
            RunRingStep::RaiseFloorInMemory { version } => {
                &&& run_ring_step_is_enabled(state, step) == user_operation_is_accepted(
                    observe_the_run_ring(state),
                    UserOperation::RaiseFloor { version },
                )
                &&& observe_the_run_ring(run_ring_next(state, step)) == observation_after_user_operation(
                    observe_the_run_ring(state),
                    UserOperation::RaiseFloor { version },
                )
            },
            RunRingStep::AcquireReader => observe_the_run_ring(run_ring_next(state, step)) == observe_the_run_ring(state),
            RunRingStep::ReleaseReader { index: _ } => observe_the_run_ring(run_ring_next(state, step))
                == observe_the_run_ring(state),
            RunRingStep::Crash => state.roots.len() >= 1 ==> observe_the_run_ring(run_ring_next(state, step))
                == observation_after_user_operation(observe_the_run_ring(state), UserOperation::Crash),
        },
{
    let after = run_ring_next(state, step);
    lemma_observed_roots_keep_each_root(state.roots);
    match step {
        RunRingStep::UserCheckpoint => {
            assert(observed_roots_of(after.roots) =~= observed_roots_of(state.roots).push(
                ObservedRoot { logical: state.head.logical, name: RootName::UserVersion { version: state.next_user_version } },
            ));
        },
        RunRingStep::CompactionCheckpoint => {
            assert(observed_roots_of(after.roots) =~= observed_roots_of(state.roots).push(
                ObservedRoot { logical: state.head.logical, name: RootName::CompactionPublish },
            ));
            if state.roots.len() >= 1 {
                assert(observed_roots_of(state.roots)[state.roots.len() - 1] == observed_root_of(
                    state.roots[state.roots.len() - 1],
                ));
            }
        },
        RunRingStep::Rollback { version } => {
            if run_ring_step_is_enabled(state, step) {
                let target = index_of_user_version(state.roots, state.in_memory_floor, version);
                assert(state.in_memory_floor <= target < state.roots.len());
                assert(observed_roots_of(state.roots)[target] == observed_root_of(state.roots[target]));
            }
        },
        RunRingStep::Crash => {
            if state.roots.len() >= 1 {
                assert(observed_roots_of(state.roots)[state.roots.len() - 1] == observed_root_of(
                    state.roots[state.roots.len() - 1],
                ));
            }
        },
        RunRingStep::Write { slot: _, bytes: _ } => {},
        RunRingStep::Reclaim { slot: _ } => {},
        RunRingStep::Relocate { key: _, destination: _ } => {},
        RunRingStep::UserPublish { new_view: _ } => {},
        RunRingStep::RaiseFloorInMemory { version: _ } => {},
        RunRingStep::AcquireReader => {},
        RunRingStep::ReleaseReader { index: _ } => {},
    }
}

pub open spec fn observation_is_shaped(observation: RingObservation) -> bool {
    &&& observation.roots.len() >= 1
    &&& observation.persisted_floor <= observation.in_memory_floor
    &&& observation.in_memory_floor < observation.roots.len()
    &&& observation.window >= 1
    &&& observation.roots.len() - observation.in_memory_floor <= observation.window
    &&& observation.roots.len() - observation.persisted_floor <= observation.window
}

pub open spec fn version_of_root(root: ObservedRoot) -> nat {
    match root.name {
        RootName::UserVersion { version } => version,
        RootName::CompactionPublish => 0,
    }
}

// 不整理那份的每个根都是用户发布的，版本号按根环次序严格升、都小于下一个版本号：按版本号点名恰点到一个根。
pub open spec fn every_root_is_a_user_version_in_order(observation: RingObservation) -> bool {
    &&& forall|index: int| 0 <= index < observation.roots.len() ==> (#[trigger] observation.roots[index]).name is UserVersion
    &&& forall|index: int| 0 <= index < observation.roots.len() ==> version_of_root(#[trigger] observation.roots[index])
        < observation.next_user_version
    &&& forall|earlier: int, later: int|
        #![trigger observation.roots[earlier], observation.roots[later]]
        0 <= earlier < later < observation.roots.len() ==> version_of_root(observation.roots[earlier]) < version_of_root(
            observation.roots[later],
        )
}

// 抬 F 能抬到的最高下标（最后一个根）。
pub open spec fn raise_floor_ceiling(observation: RingObservation) -> int {
    observation.roots.len() - 1
}

// 一层 F 上的对应（内存 F 那一层是回退候选，持久 F 那一层是崩溃之后的回退候选）：oldest 是不整理那份里第一个在整理那份这一层 F 之上的根。
pub open spec fn floors_correspond(
    without_compaction_floor: nat,
    with_compaction_floor: nat,
    without_compaction_root_count: nat,
    with_compaction_root_count: nat,
    position: Seq<int>,
    oldest: int,
    compaction_publishes: nat,
) -> bool {
    // 整理那份这一层 F 之上带版本号的根，恰是不整理那份这一层 F 之上从 oldest 起的那一截（后缀）
    &&& without_compaction_floor <= oldest <= without_compaction_root_count
    &&& forall|index: int|
        0 <= index < position.len() ==> (with_compaction_floor <= #[trigger] position[index] <==> oldest <= index)
    // 至多少 k 个
    &&& oldest - without_compaction_floor <= compaction_publishes
    // 归纳用：不整理那份这一层 F 之上的根数不多于整理那份（整理那份多出来的都是整理的发布）
    &&& without_compaction_root_count - without_compaction_floor <= with_compaction_root_count - with_compaction_floor
}

// 不整理那份与整理那份之间的对应：position 把不整理那份的每个根送到整理那份里同内容、同版本号的根，按原次序；
// oldest_still_shown 是不整理那份里第一个在整理那份回退候选（内存 F 之上）里的根，oldest_still_protected 是第一个在整理那份持久 F 之上的根。
// 各条的意思写在行尾那一句里。
pub open spec fn compaction_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
) -> bool {
    &&& observation_is_shaped(without_compaction)
    &&& observation_is_shaped(with_compaction)
    &&& every_root_is_a_user_version_in_order(without_compaction)
    // 整理不改现行内容
    &&& with_compaction.head_logical == without_compaction.head_logical
    // 崩溃之后重现的内容相同：两份最后一个根的内容相同
    &&& with_compaction.roots[with_compaction.roots.len() - 1].logical == without_compaction.roots[without_compaction.roots.len()
        - 1].logical
    &&& with_compaction.window == without_compaction.window
    &&& with_compaction.next_user_version == without_compaction.next_user_version
    // 抬 F 的上限恰多 k
    &&& raise_floor_ceiling(with_compaction) == raise_floor_ceiling(without_compaction) + compaction_publishes
    // 不整理那份的根按原次序、同内容同版本号出现在整理那份里；整理那份里带版本号的根都是这样来的
    &&& position.len() == without_compaction.roots.len()
    &&& forall|index: int|
        0 <= index < position.len() ==> 0 <= #[trigger] position[index] < with_compaction.roots.len()
            && with_compaction.roots[position[index]] == without_compaction.roots[index]
    &&& forall|earlier: int, later: int|
        #![trigger position[earlier], position[later]]
        0 <= earlier < later < position.len() ==> position[earlier] < position[later]
    &&& forall|with_index: int|
        0 <= with_index < with_compaction.roots.len() && (#[trigger] with_compaction.roots[with_index]).name is UserVersion
            ==> exists|index: int| 0 <= index < position.len() && #[trigger] position[index] == with_index
    // 回退候选（内存 F 之上）是不整理那份的后缀、至多少 k 个
    &&& floors_correspond(
        without_compaction.in_memory_floor,
        with_compaction.in_memory_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_shown,
        compaction_publishes,
    )
    // 持久 F 之上（崩溃之后的回退候选）同样是不整理那份的后缀、至多少 k 个
    &&& floors_correspond(
        without_compaction.persisted_floor,
        with_compaction.persisted_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_protected,
        compaction_publishes,
    )
}

pub open spec fn compaction_keeps_what_the_user_sees(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
) -> bool {
    exists|position: Seq<int>, oldest_still_shown: int, oldest_still_protected: int|
        #[trigger] compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        )
}

// 严格升的下标序列，两项之间的差不小于它们在序列里隔开的距离。
pub proof fn lemma_strictly_increasing_positions_spread_out(position: Seq<int>, earlier: int, later: int)
    requires
        0 <= earlier <= later < position.len(),
        forall|this_index: int, that_index: int|
            #![trigger position[this_index], position[that_index]]
            0 <= this_index < that_index < position.len() ==> position[this_index] < position[that_index],
    ensures
        position[later] - position[earlier] >= later - earlier,
    decreases later - earlier,
{
    if earlier < later {
        lemma_strictly_increasing_positions_spread_out(position, earlier, later - 1);
        assert(position[later - 1] < position[later]);
    }
}

// 从一层 F 上的对应推出的三件计数：oldest 不超过整理那份这一层的 F；整理那份这一层之上至少装得下不整理那份从 oldest 起的根；
// 两个数相等时整理那份这一层之上最旧的根就是 oldest 那个根。
pub proof fn lemma_corresponding_floors_count(
    without_compaction_floor: nat,
    with_compaction_floor: nat,
    without_compaction_root_count: nat,
    with_compaction_root_count: nat,
    position: Seq<int>,
    oldest: int,
    compaction_publishes: nat,
)
    requires
        floors_correspond(
            without_compaction_floor,
            with_compaction_floor,
            without_compaction_root_count,
            with_compaction_root_count,
            position,
            oldest,
            compaction_publishes,
        ),
        position.len() == without_compaction_root_count,
        forall|index: int| 0 <= index < position.len() ==> 0 <= #[trigger] position[index] < with_compaction_root_count,
        forall|earlier: int, later: int|
            #![trigger position[earlier], position[later]]
            0 <= earlier < later < position.len() ==> position[earlier] < position[later],
    ensures
        oldest <= with_compaction_floor,
        with_compaction_root_count - with_compaction_floor >= without_compaction_root_count - oldest,
        oldest < without_compaction_root_count && with_compaction_root_count - with_compaction_floor
            == without_compaction_root_count - oldest ==> position[oldest] == with_compaction_floor,
        oldest < without_compaction_root_count ==> position[oldest] >= with_compaction_floor,
{
    let length = without_compaction_root_count as int;
    if oldest > 0 {
        lemma_strictly_increasing_positions_spread_out(position, 0, oldest - 1);
        assert(position[0] >= 0);
        assert(!(oldest <= oldest - 1));
        assert(position[oldest - 1] < with_compaction_floor);
    }
    if oldest < length {
        lemma_strictly_increasing_positions_spread_out(position, oldest, length - 1);
        assert(position[length - 1] < with_compaction_root_count);
        assert(position[oldest] >= with_compaction_floor);
    }
}

// 整理那份这一层的 F 挤上一格时，oldest 跟着走：最旧的那个被挤到 F 之下、而它正是 oldest 那个根时加一，否则不变。
pub open spec fn oldest_after_the_floor_moves(position: Seq<int>, oldest: int, floor_before: nat, floor_after: nat) -> int {
    if floor_after != floor_before && oldest < position.len() && position[oldest] == floor_before {
        oldest + 1
    } else {
        oldest
    }
}

// 整理那份多一个根（整理的发布，k 加一）、这一层 F 至多挤上一格：这一层的对应照旧，oldest 照 oldest_after_the_floor_moves 走。
pub proof fn lemma_a_floor_moving_up_by_at_most_one_keeps_the_correspondence(
    without_compaction_floor: nat,
    with_compaction_floor: nat,
    with_compaction_floor_after: nat,
    without_compaction_root_count: nat,
    with_compaction_root_count: nat,
    position: Seq<int>,
    oldest: int,
    compaction_publishes: nat,
)
    requires
        floors_correspond(
            without_compaction_floor,
            with_compaction_floor,
            without_compaction_root_count,
            with_compaction_root_count,
            position,
            oldest,
            compaction_publishes,
        ),
        with_compaction_floor <= with_compaction_floor_after <= with_compaction_floor + 1,
        position.len() == without_compaction_root_count,
        forall|index: int| 0 <= index < position.len() ==> 0 <= #[trigger] position[index] < with_compaction_root_count,
        forall|earlier: int, later: int|
            #![trigger position[earlier], position[later]]
            0 <= earlier < later < position.len() ==> position[earlier] < position[later],
    ensures
        floors_correspond(
            without_compaction_floor,
            with_compaction_floor_after,
            without_compaction_root_count,
            with_compaction_root_count + 1,
            position,
            oldest_after_the_floor_moves(position, oldest, with_compaction_floor, with_compaction_floor_after),
            compaction_publishes + 1,
        ),
{
    lemma_corresponding_floors_count(
        without_compaction_floor,
        with_compaction_floor,
        without_compaction_root_count,
        with_compaction_root_count,
        position,
        oldest,
        compaction_publishes,
    );
    let oldest_after = oldest_after_the_floor_moves(position, oldest, with_compaction_floor, with_compaction_floor_after);
    assert forall|index: int| 0 <= index < position.len() implies (with_compaction_floor_after <= #[trigger] position[index]
        <==> oldest_after <= index) by {
        if with_compaction_floor_after != with_compaction_floor && oldest < position.len() && position[oldest]
            == with_compaction_floor {
            if index > oldest {
                assert(position[oldest] < position[index]);
            } else if index < oldest {
                assert(position[index] < position[oldest]);
            }
        } else if with_compaction_floor_after != with_compaction_floor {
            if index > oldest {
                assert(position[oldest] < position[index]);
            }
        }
    }
}

// 不整理那份按版本号点到的根，在整理那份里就是它的对应根；整理那份接受了的点名，不整理那份也接受。
pub proof fn lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
    version: nat,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
        version_is_a_candidate(with_compaction.roots, with_compaction.in_memory_floor, version),
    ensures
        version_is_a_candidate(without_compaction.roots, without_compaction.in_memory_floor, version),
        index_of_the_candidate_version(with_compaction.roots, with_compaction.in_memory_floor, version) == position[index_of_the_candidate_version(
            without_compaction.roots,
            without_compaction.in_memory_floor,
            version,
        )],
        oldest_still_shown <= index_of_the_candidate_version(without_compaction.roots, without_compaction.in_memory_floor, version),
{
    let with_index = index_of_the_candidate_version(with_compaction.roots, with_compaction.in_memory_floor, version);
    assert(with_compaction.roots[with_index].name is UserVersion);
    let index = choose|index: int| 0 <= index < position.len() && #[trigger] position[index] == with_index;
    assert(without_compaction.roots[index] == with_compaction.roots[with_index]);
    assert(oldest_still_shown <= index);
    assert(without_compaction.roots[index].name == (RootName::UserVersion { version }));
    assert(version_is_a_candidate(without_compaction.roots, without_compaction.in_memory_floor, version));
    let without_index = index_of_the_candidate_version(without_compaction.roots, without_compaction.in_memory_floor, version);
    assert(without_compaction.roots[without_index].name == (RootName::UserVersion { version }));
    if without_index < index {
        assert(version_of_root(without_compaction.roots[without_index]) < version_of_root(without_compaction.roots[index]));
    } else if index < without_index {
        assert(version_of_root(without_compaction.roots[index]) < version_of_root(without_compaction.roots[without_index]));
    }
    assert(without_index == index);
}

// 整理那份接受了的点名，不整理那份也接受（有界穷举里的 b_accepts_a_rejects = 0）。
pub proof fn lemma_compaction_never_admits_an_operation_the_plain_history_refuses(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    operation: UserOperation,
)
    requires
        compaction_keeps_what_the_user_sees(without_compaction, with_compaction, compaction_publishes),
        user_operation_is_accepted(with_compaction, operation),
    ensures
        user_operation_is_accepted(without_compaction, operation),
{
    let (position, oldest_still_shown, oldest_still_protected) = choose|
        position: Seq<int>,
        oldest_still_shown: int,
        oldest_still_protected: int,
    |
        #[trigger] compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        );
    match operation {
        UserOperation::Rollback { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
        },
        UserOperation::RaiseFloor { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
        },
        UserOperation::Publish { logical: _ } => {},
        UserOperation::Checkpoint => {},
        UserOperation::Crash => {},
    }
}

// 整理的发布（只在整理那份里，现行内容与最后一个根相同）：k 加一，对应照旧；两层 F 各自至多挤上一格，
// oldest_still_shown 与 oldest_still_protected 各照 oldest_after_the_floor_moves 走。整理的发布要是带内存 F，持久 F 一次能跳过几个根，这里就断。
pub proof fn lemma_a_compaction_publish_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
        compaction_publish_is_accepted(with_compaction),
    ensures
        compaction_relation(
            without_compaction,
            observation_after_a_compaction_publish(with_compaction),
            compaction_publishes + 1,
            position,
            oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_in_memory_floor_after_a_compaction_publish(with_compaction),
            ),
            oldest_after_the_floor_moves(
                position,
                oldest_still_protected,
                with_compaction.persisted_floor,
                observed_persisted_floor_after_a_compaction_publish(with_compaction),
            ),
        ),
{
    let after = observation_after_a_compaction_publish(with_compaction);
    lemma_a_floor_moving_up_by_at_most_one_keeps_the_correspondence(
        without_compaction.in_memory_floor,
        with_compaction.in_memory_floor,
        after.in_memory_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_shown,
        compaction_publishes,
    );
    lemma_a_floor_moving_up_by_at_most_one_keeps_the_correspondence(
        without_compaction.persisted_floor,
        with_compaction.persisted_floor,
        after.persisted_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_protected,
        compaction_publishes,
    );
    assert forall|index: int| 0 <= index < position.len() implies 0 <= #[trigger] position[index] < after.roots.len()
        && after.roots[position[index]] == without_compaction.roots[index] by {
        assert(after.roots[position[index]] == with_compaction.roots[position[index]]);
    }
    assert forall|with_index: int|
        0 <= with_index < after.roots.len() && (#[trigger] after.roots[with_index]).name is UserVersion implies exists|index: int|
        0 <= index < position.len() && #[trigger] position[index] == with_index by {
        if with_index < with_compaction.roots.len() {
            assert(after.roots[with_index] == with_compaction.roots[with_index]);
        } else {
            assert(after.roots[with_index].name == RootName::CompactionPublish);
        }
    }
    assert(after.roots[after.roots.len() - 1].logical == with_compaction.head_logical);
}

// 两份同时崩溃：内存 F 回到持久 F、现行回到最后一个根。崩溃之后的回退候选就是持久 F 那一层的对应：整理那份照样是不整理那份的后缀、至多少 k 个，
// 现行内容照样相同。
pub proof fn lemma_a_crash_of_both_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
    ensures
        compaction_relation(
            observation_after_user_operation(without_compaction, UserOperation::Crash),
            observation_after_user_operation(with_compaction, UserOperation::Crash),
            compaction_publishes,
            position,
            oldest_still_protected,
            oldest_still_protected,
        ),
{
    let without_after = observation_after_user_operation(without_compaction, UserOperation::Crash);
    let with_after = observation_after_user_operation(with_compaction, UserOperation::Crash);
    assert(without_after.roots == without_compaction.roots);
    assert(with_after.roots == with_compaction.roots);
}

// 用户的一步、两份都接受：对应关系照旧（发布落成根时 position 多一项，两层 F 一起照 oldest_after_the_floor_moves 走；
// 抬 F 时 oldest_still_shown 就是点到的那个根；崩溃时两层都取持久 F 那一层的）。
pub proof fn lemma_an_operation_accepted_by_both_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
    operation: UserOperation,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
        user_operation_is_accepted(without_compaction, operation),
        user_operation_is_accepted(with_compaction, operation),
    ensures
        compaction_keeps_what_the_user_sees(
            observation_after_user_operation(without_compaction, operation),
            observation_after_user_operation(with_compaction, operation),
            compaction_publishes,
        ),
{
    let without_after = observation_after_user_operation(without_compaction, operation);
    let with_after = observation_after_user_operation(with_compaction, operation);
    match operation {
        UserOperation::Publish { logical: _ } => {
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                oldest_still_shown,
                oldest_still_protected,
            ));
        },
        UserOperation::Rollback { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
            let without_index = index_of_the_candidate_version(
                without_compaction.roots,
                without_compaction.in_memory_floor,
                version,
            );
            assert(with_compaction.roots[position[without_index]] == without_compaction.roots[without_index]);
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                oldest_still_shown,
                oldest_still_protected,
            ));
        },
        UserOperation::RaiseFloor { version } => {
            lemma_a_version_named_in_the_compacted_history_is_the_corresponding_root(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected, version,
            );
            let without_index = index_of_the_candidate_version(
                without_compaction.roots,
                without_compaction.in_memory_floor,
                version,
            );
            let length = without_compaction.roots.len() as int;
            lemma_strictly_increasing_positions_spread_out(position, without_index, length - 1);
            assert(position[length - 1] < with_compaction.roots.len());
            assert forall|index: int| 0 <= index < position.len() implies (with_after.in_memory_floor <= #[trigger] position[index]
                <==> without_index <= index) by {
                if index > without_index {
                    assert(position[without_index] < position[index]);
                } else if index < without_index {
                    assert(position[index] < position[without_index]);
                }
            }
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                without_index,
                oldest_still_protected,
            ));
        },
        UserOperation::Checkpoint => {
            lemma_a_user_checkpoint_keeps_the_relation(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected,
            );
            let oldest_after = oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_floor_after_a_checkpoint(with_compaction),
            );
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position.push(with_compaction.roots.len() as int),
                oldest_after,
                oldest_after,
            ));
        },
        UserOperation::Crash => {
            lemma_a_crash_of_both_keeps_the_relation(
                without_compaction, with_compaction, compaction_publishes, position, oldest_still_shown,
                oldest_still_protected,
            );
            assert(compaction_relation(
                without_after,
                with_after,
                compaction_publishes,
                position,
                oldest_still_protected,
                oldest_still_protected,
            ));
        },
    }
}

// 用户的发布落成根（两份都做）：position 多一项指到整理那份的新根，内存 F 那一层的 oldest_still_shown 照 oldest_after_the_floor_moves 走，
// 持久 F 跟上内存 F，持久那一层取同一个。不整理那份挤掉最旧的候选时它的候选是满的，整理那份的候选不少于它、也满、也挤；
// oldest_still_shown 等于不整理那份的 F 时，整理那份最旧的候选就是 oldest_still_shown 那个根（lemma_corresponding_floors_count 的第三条）。
pub proof fn lemma_a_user_checkpoint_keeps_the_relation(
    without_compaction: RingObservation,
    with_compaction: RingObservation,
    compaction_publishes: nat,
    position: Seq<int>,
    oldest_still_shown: int,
    oldest_still_protected: int,
)
    requires
        compaction_relation(
            without_compaction,
            with_compaction,
            compaction_publishes,
            position,
            oldest_still_shown,
            oldest_still_protected,
        ),
    ensures
        compaction_relation(
            observation_after_user_operation(without_compaction, UserOperation::Checkpoint),
            observation_after_user_operation(with_compaction, UserOperation::Checkpoint),
            compaction_publishes,
            position.push(with_compaction.roots.len() as int),
            oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_floor_after_a_checkpoint(with_compaction),
            ),
            oldest_after_the_floor_moves(
                position,
                oldest_still_shown,
                with_compaction.in_memory_floor,
                observed_floor_after_a_checkpoint(with_compaction),
            ),
        ),
{
    lemma_corresponding_floors_count(
        without_compaction.in_memory_floor,
        with_compaction.in_memory_floor,
        without_compaction.roots.len(),
        with_compaction.roots.len(),
        position,
        oldest_still_shown,
        compaction_publishes,
    );
    let without_after = observation_after_user_operation(without_compaction, UserOperation::Checkpoint);
    let with_after = observation_after_user_operation(with_compaction, UserOperation::Checkpoint);
    let new_root = ObservedRoot {
        logical: without_compaction.head_logical,
        name: RootName::UserVersion { version: without_compaction.next_user_version },
    };
    let position_after = position.push(with_compaction.roots.len() as int);
    let oldest_still_shown_after = oldest_after_the_floor_moves(
        position,
        oldest_still_shown,
        with_compaction.in_memory_floor,
        observed_floor_after_a_checkpoint(with_compaction),
    );
    let without_length = without_compaction.roots.len() as int;
    let with_length = with_compaction.roots.len() as int;
    let without_floor = without_compaction.in_memory_floor as int;
    let with_floor = with_compaction.in_memory_floor as int;
    assert(with_after.roots[with_length] == new_root);
    assert(without_after.roots[without_length] == new_root);
    assert forall|index: int| 0 <= index < without_after.roots.len() implies (#[trigger] without_after.roots[index]).name is UserVersion
        && version_of_root(without_after.roots[index]) < without_after.next_user_version by {
        if index < without_length {
            assert(without_after.roots[index] == without_compaction.roots[index]);
        }
    }
    assert forall|earlier: int, later: int|
        #![trigger without_after.roots[earlier], without_after.roots[later]]
        0 <= earlier < later < without_after.roots.len() implies version_of_root(without_after.roots[earlier])
        < version_of_root(without_after.roots[later]) by {
        assert(without_after.roots[earlier] == without_compaction.roots[earlier]);
        if later < without_length {
            assert(without_after.roots[later] == without_compaction.roots[later]);
        }
    }
    assert forall|index: int| 0 <= index < position_after.len() implies 0 <= #[trigger] position_after[index]
        < with_after.roots.len() && with_after.roots[position_after[index]] == without_after.roots[index] by {
        if index < without_length {
            assert(position_after[index] == position[index]);
            assert(with_after.roots[position[index]] == with_compaction.roots[position[index]]);
            assert(without_after.roots[index] == without_compaction.roots[index]);
        }
    }
    assert forall|earlier: int, later: int|
        #![trigger position_after[earlier], position_after[later]]
        0 <= earlier < later < position_after.len() implies position_after[earlier] < position_after[later] by {
        assert(position_after[earlier] == position[earlier]);
        if later < without_length {
            assert(position_after[later] == position[later]);
        }
    }
    assert forall|with_index: int|
        0 <= with_index < with_after.roots.len() && (#[trigger] with_after.roots[with_index]).name is UserVersion implies exists|index: int|
        0 <= index < position_after.len() && #[trigger] position_after[index] == with_index by {
        if with_index < with_length {
            assert(with_after.roots[with_index] == with_compaction.roots[with_index]);
            let index = choose|index: int| 0 <= index < position.len() && #[trigger] position[index] == with_index;
            assert(position_after[index] == position[index]);
        } else {
            assert(position_after[without_length] == with_index);
        }
    }
    assert forall|index: int| 0 <= index < position_after.len() implies (with_after.in_memory_floor
        <= #[trigger] position_after[index] <==> oldest_still_shown_after <= index) by {
        if index < without_length {
            assert(position_after[index] == position[index]);
            if with_after.in_memory_floor != with_floor && oldest_still_shown < position.len() && position[oldest_still_shown] == with_floor {
                if index > oldest_still_shown {
                    assert(position[oldest_still_shown] < position[index]);
                } else if index < oldest_still_shown {
                    assert(position[index] < position[oldest_still_shown]);
                }
            } else if with_after.in_memory_floor != with_floor {
                if index > oldest_still_shown {
                    assert(position[oldest_still_shown] < position[index]);
                }
            }
        }
    }
    if without_after.in_memory_floor != without_floor {
        // 不整理那份挤掉了最旧的候选：它的候选是满的，整理那份也是满的、也挤；oldest_still_shown 等于不整理那份的 F 时
        // 整理那份最旧的候选就是 oldest_still_shown 那个根。
        assert(with_after.in_memory_floor != with_floor);
        if oldest_still_shown == without_floor {
            assert(position[oldest_still_shown] == with_floor);
        }
    }
    assert(floors_correspond(
        without_after.in_memory_floor,
        with_after.in_memory_floor,
        without_after.roots.len(),
        with_after.roots.len(),
        position_after,
        oldest_still_shown_after,
        compaction_publishes,
    ));
    assert(compaction_relation(
        without_after,
        with_after,
        compaction_publishes,
        position_after,
        oldest_still_shown_after,
        oldest_still_shown_after,
    ));
}

// G7 的历史：一串事件，用户的一步（含崩溃）两份都做，整理的发布只在整理那份里做。
pub enum ObservedEvent {
    User { operation: UserOperation },
    CompactionPublish,
}

pub open spec fn observation_without_compaction(start: RingObservation, events: Seq<ObservedEvent>, count: int) -> RingObservation
    decreases count,
{
    if count <= 0 {
        start
    } else {
        let before = observation_without_compaction(start, events, count - 1);
        match events[count - 1] {
            ObservedEvent::User { operation } => observation_after_user_operation(before, operation),
            ObservedEvent::CompactionPublish => before,
        }
    }
}

pub open spec fn observation_with_compaction(start: RingObservation, events: Seq<ObservedEvent>, count: int) -> RingObservation
    decreases count,
{
    if count <= 0 {
        start
    } else {
        let before = observation_with_compaction(start, events, count - 1);
        match events[count - 1] {
            ObservedEvent::User { operation } => observation_after_user_operation(before, operation),
            ObservedEvent::CompactionPublish => observation_after_a_compaction_publish(before),
        }
    }
}

// k：前 count 个事件里整理占的发布数。
pub open spec fn compaction_publishes_among(events: Seq<ObservedEvent>, count: int) -> nat
    decreases count,
{
    if count <= 0 {
        0
    } else {
        match events[count - 1] {
            ObservedEvent::User { operation: _ } => compaction_publishes_among(events, count - 1),
            ObservedEvent::CompactionPublish => compaction_publishes_among(events, count - 1) + 1,
        }
    }
}

pub open spec fn event_is_accepted_by_both(start: RingObservation, events: Seq<ObservedEvent>, index: int) -> bool {
    match events[index] {
        ObservedEvent::User { operation } => user_operation_is_accepted(
            observation_without_compaction(start, events, index),
            operation,
        ) && user_operation_is_accepted(observation_with_compaction(start, events, index), operation),
        ObservedEvent::CompactionPublish => compaction_publish_is_accepted(observation_with_compaction(start, events, index)),
    }
}

// G7：从同一个起点出发（每个根都是用户发布的、版本号按次序），同一串用户历史（可以有崩溃）两份都接受了每一步，整理那份另插 k 次整理的发布；
// 前 count 个事件之后，现行内容相同，整理那份的回退候选（按版本号）是不整理那份候选的后缀、至多少 k 个，抬 F 的上限恰多 k；
// 持久 F 之上的根同样是后缀、至多少 k 个，所以下一次崩溃之后的回退候选也是。
pub proof fn theorem_compaction_keeps_what_the_user_sees(start: RingObservation, events: Seq<ObservedEvent>, count: nat)
    requires
        observation_is_shaped(start),
        every_root_is_a_user_version_in_order(start),
        count <= events.len(),
        forall|index: int| 0 <= index < events.len() ==> #[trigger] event_is_accepted_by_both(start, events, index),
    ensures
        compaction_keeps_what_the_user_sees(
            observation_without_compaction(start, events, count as int),
            observation_with_compaction(start, events, count as int),
            compaction_publishes_among(events, count as int),
        ),
    decreases count,
{
    if count == 0 {
        let identity = Seq::new(start.roots.len(), |index: int| index);
        assert forall|with_index: int|
            0 <= with_index < start.roots.len() && (#[trigger] start.roots[with_index]).name is UserVersion implies exists|index: int|
            0 <= index < identity.len() && #[trigger] identity[index] == with_index by {
            assert(identity[with_index] == with_index);
        }
        assert(compaction_relation(
            start,
            start,
            0,
            identity,
            start.in_memory_floor as int,
            start.persisted_floor as int,
        ));
    } else {
        let previous = (count - 1) as nat;
        theorem_compaction_keeps_what_the_user_sees(start, events, previous);
        let without_before = observation_without_compaction(start, events, previous as int);
        let with_before = observation_with_compaction(start, events, previous as int);
        let publishes_before = compaction_publishes_among(events, previous as int);
        let (position, oldest_still_shown, oldest_still_protected) = choose|
            position: Seq<int>,
            oldest_still_shown: int,
            oldest_still_protected: int,
        |
            #[trigger] compaction_relation(
                without_before,
                with_before,
                publishes_before,
                position,
                oldest_still_shown,
                oldest_still_protected,
            );
        assert(event_is_accepted_by_both(start, events, previous as int));
        match events[previous as int] {
            ObservedEvent::User { operation } => {
                lemma_an_operation_accepted_by_both_keeps_the_relation(
                    without_before, with_before, publishes_before, position, oldest_still_shown, oldest_still_protected,
                    operation,
                );
            },
            ObservedEvent::CompactionPublish => {
                lemma_a_compaction_publish_keeps_the_relation(
                    without_before, with_before, publishes_before, position, oldest_still_shown, oldest_still_protected,
                );
                assert(compaction_relation(
                    without_before,
                    observation_after_a_compaction_publish(with_before),
                    publishes_before + 1,
                    position,
                    oldest_after_the_floor_moves(
                        position,
                        oldest_still_shown,
                        with_before.in_memory_floor,
                        observed_in_memory_floor_after_a_compaction_publish(with_before),
                    ),
                    oldest_after_the_floor_moves(
                        position,
                        oldest_still_protected,
                        with_before.persisted_floor,
                        observed_persisted_floor_after_a_compaction_publish(with_before),
                    ),
                ));
            },
        }
    }
}

} // verus!
