//! 整理的一趟：停着时起不起跑、回退门、一趟的开头、每一批之前的判定、一批推进之后的记账，取下一批或收尾。

use super::*;

/// 一次整理运行推到哪为止。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompactionRunReach {
    /// 显式入口与定时兜底起跑的那一趟：推到停机原因或发布预算用完。
    UntilAStopOrTheBudget,
    /// 后台入口取走用户改动之后记下的「该跑了」（判决 G8 丁）：至多一批。一批 = 搬了单元的那一次发布（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F2）；不搬单元的发布不算批：上一条意图的收尾（一个没搬就删意图、R 已按记录全空只删意图、丢意图）与签发。所以一趟可以发
    /// 「收尾 + 签发 + 一批」三次；签发之后计划里只剩这次发布本来就重写的节点时，那一次也是一个没搬就删意图，全局差值为真就再签再搬。
    /// 一趟发几次的上界是这一趟的发布预算 `B_gc`（[`publish_budget_of_a_compaction_run`]），每签一条之前照判签发前判与「上一条无产出就不签」
    /// （判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G4、G8；判决 `research/prompts/m4-segment-code-r3-main-verification.md`
    /// 第三节 J3 核过：与 G8 丁「至多一批」、G8 甲「至多签 `B_gc` 条」一致）。
    AtMostOneBatchAfterAUserChange,
    /// 可写挂载续做（D8（核心索引结构） 已定项 5 第 2 条；判决 G1）：只推盘上那一条未完成的意图，推到它删了、停机原因或预算用完，不签新的。
    ContinueTheLiveIntentAtTheMount,
    /// 后台入口因删除队列不空起跑的那一趟（[`BackgroundCompactionTrigger::DeletionQueueNotEmpty`]）：走空队列就停，不推整理。
    OnlyTheDeletionQueue,
}

/// 整理的一次发布做的是哪一种（[`CompactionStep::Published`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompactionPublishKind {
    /// 签发那次发布：意图写进 logged_ops。
    IntentIssued,
    /// 搬一批，意图照抄。
    Batch,
    /// 最后一批：搬完 R 按记录全空，同一次发布删意图（D8（核心索引结构） 已定项 5）。
    BatchCompletingTheIntent,
    /// R 已按记录全空（别的发布腾空的），单独一次发布删意图。
    IntentCompletedWithoutABatch,
    /// 丢意图：单独一次发布删它（攻方 K-c 的隔离槽、区域扫描拒了 R）。
    IntentDropped,
    /// 丢意图：R 里一个能搬的都没有、却还有未释放的记录；丢完这一趟停（[`CompactionRunStop::NothingMovableButTheRegionIsNotEmpty`]）。
    IntentDroppedWithNothingMovable,
    /// 丢意图：整理停下时单独一次发布删它（[`drop_the_live_intent_on_a_stop`]），丢完这一趟停在 `stop`：一批被空间拒的暂停
    /// （[`CompactionRunStop::PausedShortOfSpace`]）、一批出错放弃（[`CompactionRunStop::BatchAbandonedAfterABackgroundComputeError`]）、
    /// 停着而不补跑（[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
    IntentDroppedOnAStop { stop: CompactionRunStop },
}

/// 推一步整理的结局：发成一次整理发布，或照判据停。
pub(super) enum CompactionStep {
    Published {
        version: Box<TransactionOutput>,
        kind: CompactionPublishKind,
    },
    Stopped(CompactionRunStop),
}

/// 整理此刻停着而不补跑（D26（后台整理与放置回收） 已定项 13 第 4 条：GC 因一批出错停着、池子每块盘的已分配占它的容量都不超过补跑阈值）：
/// 交回真；没停着或要在 CPU 上补跑交回假。只判，分配器与盘一样都不动：停着时还有活意图的由一趟开头那一判单发一次删意图（[`drop_the_live_intent_on_a_stop`]）；
/// 后台入口判出停着而有活意图时起跑那一趟（[`start_while_the_compaction_stays_stopped`]），显式入口每一趟都走到那一判。
pub(super) fn compaction_stays_stopped(
    allocator: &PoolAllocator,
    background_compute: &BackgroundComputeOfThisMount,
) -> bool {
    match background_compute.whether_the_task_runs_the_next_batch(
        BackgroundTask::CompactionRelocation,
        AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator(allocator),
    ) {
        WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => true,
        WhetherTheTaskRunsTheNextBatch::AsTheSettingsSay
        | WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(_) => false,
    }
}

/// 后台入口判出整理停着而不补跑（[`compaction_stays_stopped`]）之后起不起跑：现行那一版里有活意图，而上一次用户改动给的那一份 `B_gc`
/// 还剩（或全空段低于底额、前台写不进时破例至少 1 次，同后台入口别的起跑）⇒ 起跑一趟
/// （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]），那一趟开头判出同一个停着，开关照水位判时过了回退门才单发一次删意图
/// （[`drop_the_live_intent_on_a_stop`]；破例那一档 [`CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite`] 不过门，门关着照发），
/// 之后停在 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；
/// 没有活意图、或那一份扣完而前台写得进 ⇒ 不起跑、一个写都不发，「该跑了」留着。只判，分配器与盘一样都不动。
pub(super) fn start_while_the_compaction_stays_stopped(
    allocator: &PoolAllocator,
    current: &PoolVersion,
    root_ring_slots_per_region: u64,
    publishes_since_the_last_user_change: u64,
) -> BackgroundCompactionStart {
    let has_a_live_intent = current.version_of_trees().is_some_and(|version_of_trees| {
        live_compaction_intent_of_the_entries(&version_of_trees.logged_operations).is_some()
    });
    let foreground_cannot_write = foreground_cannot_write_under_the_switches(allocator);
    let is_the_budget_spent = publishes_left_of_the_budget_given_by_the_last_user_change(
        root_ring_slots_per_region,
        publishes_since_the_last_user_change,
    ) == 0;
    if !has_a_live_intent || (is_the_budget_spent && !foreground_cannot_write) {
        return BackgroundCompactionStart::NotStarted(
            BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
        );
    }
    BackgroundCompactionStart::Starts {
        trigger: BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        reach: CompactionRunReach::AtMostOneBatchAfterAUserChange,
        budget: if foreground_cannot_write {
            CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
        } else {
            CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange
        },
        batches_published_earlier_in_the_trip: 0,
    }
}

/// 整理一趟（显式入口、可写挂载续做共用；后台入口另经 [`run_compaction_on_with_the_region_check_of_the_intent_deletion`]）：先走空删除续做（G8 次序），
/// 再照预算一次一次推（[`advance_compaction_by_one_publish`]）。预算 `B_gc` 数的是整理的每一次发布（签发、搬一批、删意图：每一次都用掉一个根槽）。
/// 每一批之前不看调用方。discard 由调用方在这一趟之后发。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、这次挂载的后台计算状态"
)]
pub(crate) fn run_compaction_on<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
    background_compute: &mut BackgroundComputeOfThisMount,
) -> CompactionRun {
    run_compaction_on_with_the_region_check_of_the_intent_deletion(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        reach,
        CompactionRunBudget::SinceTheLastUserChange,
        PacingOfTheRun::NotPaced,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        background_compute,
        &mut InTheMiddleOfEachBackgroundBatch::NothingHappens,
    )
}

/// 回退门（D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」，D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：
/// 下一次发布写的那一槽与它之后留的余量（[`crate::compaction_plan::root_ring_slots_left_by_a_compaction_publish`]）里的根一走，根环里不同用户状态就少于 4 个而且比此刻少，这次整理发布就不发。
/// 此刻本来就不到 4 个、走的那几条又都不是某个状态的最后一条的，不是这次发布挤掉的，照发。读不通不发（[`CompactionRunStop::RollbackStatesOfTheRingUnreadable`]）。
/// 只读，读根环与每条有效根的树表；按位置条目读单元读到只坏一份的进别的读那份账（D4（校验和位置） 已定项 4「可写挂载期间别的读」）。
pub(super) fn stop_of_the_rollback_gate<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    allocator: &PoolAllocator,
    current: &PoolVersion,
) -> Option<CompactionRunStop> {
    match allocator.reading_and_counting_the_other_unit_reads_with_one_copy_failing(
        &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
        |reader| rollback_states_of_the_ring(reader, parameters, current.root()),
    ) {
        Err(_unreadable) => Some(CompactionRunStop::RollbackStatesOfTheRingUnreadable),
        Ok(states) => (states.states_after_the_next_publish_and_the_margin
            < ROLLBACK_STATES_KEPT_BY_THE_COMPACTION
            && states.states_after_the_next_publish_and_the_margin < states.states_now)
            .then_some(
                CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
                    states_now: states.states_now,
                    states_after_the_publish_and_the_margin: states
                        .states_after_the_next_publish_and_the_margin,
                },
            ),
    }
}

/// 回退门保住的不同用户状态个数（D16（发布语义） 已定项 1）。
const ROLLBACK_STATES_KEPT_BY_THE_COMPACTION: u64 = 4;

/// 同 [`run_compaction_on`]，另带三样：这一趟的发布预算按 `budget_rule` 算；每一批之前照 `pacing` 问调用方（后台入口：前台有 fsync 在等、
/// 有别的前台入口在等会话锁、写带宽到了上限就不发下一批，D26（后台整理与放置回收） 已定项 13 第 6 条）；一批腾空了 R、照同一批删意图再发时删意图那一支的核照 `region_check` 判
/// （[`MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`]），
/// 每一批读完后台计算设置之后照 `in_the_middle_of_each_batch` 做（产品路径什么都不做，
/// [`MountedSession::run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`]）。整理因一批出错停着而每块盘都不超过补跑阈值时，
/// 从判出的那一刻起只发一次删活意图的发布（有活意图时），之后一个写都不发（[`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]）。
/// 每一批是三段首尾相接、中间不放锁（[`advance_compaction_by_one_publish`]）：放锁的三段入口
/// （[`MountedSessionWithPendingChanges::take_the_input_of_a_background_compaction_batch`]）与这里共用一趟的四块（开一趟、每批之前那几判、取一批的输入、一批之后记账）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、预算按哪一条算、每批之前看不看调用方、删意图那一支的核怎么判、这次挂载的后台计算状态、一批读完设置之后做什么"
)]
pub(super) fn run_compaction_on_with_the_region_check_of_the_intent_deletion<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
    budget_rule: CompactionRunBudget,
    pacing: PacingOfTheRun<'_>,
    region_check: RegionCheckOfTheIntentDeletion,
    background_compute: &mut BackgroundComputeOfThisMount,
    in_the_middle_of_each_batch: &mut InTheMiddleOfEachBackgroundBatch<'_>,
) -> CompactionRun {
    let mut run = match begin_a_compaction_run(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        reach,
        budget_rule,
        pacing,
        region_check,
        background_compute,
    ) {
        Ok(run) => run,
        Err(finished) => return *finished,
    };
    // 循环：每一轮发成一次整理发布或停。迭代上界是预算（每发成一次计一次，到 budget 停）；跨轮携带的是现行那一版与这一趟的账（[`CompactionRunUnderWay`]）。
    // 上一条完成的意图住会话（`CompactionOfThisMount::last_completed_intent`）：跨趟留着，签下一条之前那一判（攻方 K-h）读它。
    loop {
        run = match judge_before_the_next_compaction_batch(
            run, parameters, devices, allocator, compaction, current, pacing,
        ) {
            Ok(run) => run,
            Err(finished) => return *finished,
        };
        let advanced = advance_compaction_by_one_publish(
            parameters,
            devices,
            allocator,
            compaction,
            current,
            instance,
            &run,
            background_compute,
            in_the_middle_of_each_batch,
        );
        run = match note_the_compaction_batch_advanced(
            run,
            advanced,
            allocator,
            compaction,
            current,
            devices.len(),
        ) {
            Ok(run) => run,
            Err(finished) => return *finished,
        };
    }
}

/// 一趟开头那一段（[`run_compaction_on_with_the_region_check_of_the_intent_deletion`] 进第一批之前）：开关叫停、整理停着时的删意图、
/// 预算、删除续做。走到整理批之前交回这一趟的账；这一趟在这里就停了交回那一趟。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、预算按哪一条算、每批之前看不看调用方、删意图那一支的核怎么判、这次挂载的后台计算状态"
)]
fn begin_a_compaction_run<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
    budget_rule: CompactionRunBudget,
    pacing: PacingOfTheRun<'_>,
    region_check: RegionCheckOfTheIntentDeletion,
    background_compute: &mut BackgroundComputeOfThisMount,
) -> Result<CompactionRunUnderWay, Box<CompactionRun>> {
    compaction.observations.runs_started += 1;
    let was_frozen_when_this_call_came_in = allocator.frozen_publish().is_some();
    let switches_of_the_run: CompactionSwitches = allocator.compaction().switches;
    let batch_units = match switches_of_the_run.batch_size {
        CompactionBatchSize::OfTheMount(batch_units)
        | CompactionBatchSize::ForcedForTests(batch_units) => batch_units,
    };
    let (span_of_the_call, batches_published_earlier_in_the_trip) = match pacing {
        PacingOfTheRun::NotPaced => (SpanOfTheBackgroundCall::WholeTrip, 0),
        PacingOfTheRun::PacedByTheCaller {
            span,
            batches_published_earlier_in_the_trip,
            ..
        } => (span, batches_published_earlier_in_the_trip),
    };
    // 回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）。
    let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })
        && matches!(
            switches_of_the_run.run_forcing,
            CompactionRunForcing::FollowTheWatermarks
        );
    let mut run = CompactionRunUnderWay {
        versions_published: Vec::new(),
        background_compute_of_each_batch: Vec::new(),
        deletion_steps_published: 0,
        publishes_of_the_run: 0,
        batches_published: 0,
        budget: 0,
        budget_rule,
        reach,
        span_of_the_call,
        batches_published_earlier_in_the_trip,
        switches_of_the_run,
        was_frozen_when_this_call_came_in,
        batch_units,
        is_the_rollback_gate_judged,
        region_check,
        write_bandwidth_limit_of_the_batch: write_bandwidth_limit_of_the_pacing(pacing),
        is_the_batch_the_exception: false,
    };
    match switches_of_the_run.run_forcing {
        CompactionRunForcing::ForcedToStopForTests => {
            return Err(run.finish(
                compaction,
                current.root().checkpoint_txg,
                CompactionRunOutcome::Stopped(CompactionRunStop::ForcedToStopForTests),
            ));
        }
        CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStartForTests => {
        }
    }
    // D26（后台整理与放置回收） 已定项 13 第 4 条：整理因一批出错停着，等一次通过校验的切换请求解除；停着而池子每块盘的已分配占它的容量都不超过补跑阈值时
    // 删除续做不走：还有活意图（补跑那几批推着的、挂载续做的、停下那一刻没删成的）而这一趟的预算没用完，就单发一次只删意图的发布
    // （[`drop_the_live_intent_on_a_stop`]），之后一个写都不发、当场停；没有活意图或预算用完就一个写都不发。超过阈值就照常推、每一批在 CPU 上补跑。
    // 删意图排在删除续做之前，队列不空时那一版照抄队列，续做等解除之后。后台入口照水位判时那一次删意图先过回退门（下面那一判）。
    // 没停着（补跑、停下已解除、只放弃那一批的出错）而会话记着回锁时被回退门压住的那一条删意图的，在取下一批的输入之前删
    // （[`take_the_input_of_one_compaction_batch`]），回退门与破例在每批之前那几判里判（[`judge_before_the_next_compaction_batch`]）。
    // 挂着切到一张卡都用不上的那一组时，先在这里换上它（一趟开头不在一批之内），与挂载那一刻一样当场判出停着。
    background_compute.put_the_switch_requested_from_the_next_batch_in_effect();
    let budget = match budget_rule {
        CompactionRunBudget::SinceTheLastUserChange => publish_budget_of_a_compaction_run(
            parameters.geometry.root_ring_slots_per_region.count(),
            compaction.publishes_since_the_last_user_change,
        ),
        CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange => {
            publishes_left_of_the_budget_given_by_the_last_user_change(
                parameters.geometry.root_ring_slots_per_region.count(),
                compaction.publishes_since_the_last_user_change,
            )
        }
        CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite => {
            publishes_left_of_the_budget_given_by_the_last_user_change(
                parameters.geometry.root_ring_slots_per_region.count(),
                compaction.publishes_since_the_last_user_change,
            )
            .max(1)
        }
    };
    run.budget = budget;
    if compaction_stays_stopped(allocator, background_compute) {
        let stop = CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError;
        // 停着时单发的删意图也是一次整理发布，同样过回退门（D26（后台整理与放置回收） 已定项 13 第 5 条）：门关着就不发，意图与 R 排他留到门开；
        // 全空段低于底额、前台写不进时这一次破例，同循环里每趟的第一次发布。
        let is_this_drop_the_exception =
            budget_rule == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite;
        if budget != 0 && is_the_rollback_gate_judged && !is_this_drop_the_exception {
            if let Some(stop_of_the_gate) =
                stop_of_the_rollback_gate(parameters, &*devices, allocator, current)
            {
                return Err(run.finish(
                    compaction,
                    current.root().checkpoint_txg,
                    CompactionRunOutcome::Stopped(stop_of_the_gate),
                ));
            }
        }
        let dropped_on_the_stop = if budget == 0 {
            Ok(CompactionStep::Stopped(stop))
        } else {
            drop_the_live_intent_on_a_stop(
                parameters, devices, allocator, compaction, current, instance, stop,
            )
        };
        match dropped_on_the_stop {
            Ok(CompactionStep::Published { version, kind: _ }) => {
                // 这一次删意图也是一次整理发布：一批发成，让路的两个计数都清零（D26（后台整理与放置回收） 已定项 13 第 6 条）。
                compaction.clear_the_yields_in_a_row();
                compaction.publishes_since_the_last_user_change += 1;
                compaction.observations.root_ring_slots_used += 1;
                compaction.note_the_bytes_written_by_a_compaction_publish(
                    write_bandwidth_limit_of_the_pacing(pacing),
                    bytes_written_by_a_compaction_publish(&version, devices.len()),
                );
                run.versions_published.push(*version);
            }
            Ok(CompactionStep::Stopped(_nothing_to_drop)) => {}
            Err(refused) => {
                note_the_kind_of_a_publish_frozen_by_this_call(
                    allocator,
                    was_frozen_when_this_call_came_in,
                    KindOfTheFrozenPublish::CompactionPublish,
                );
                return Err(run.finish(
                    compaction,
                    current.root().checkpoint_txg,
                    CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),
                ));
            }
        }
        return Err(run.finish(
            compaction,
            current.root().checkpoint_txg,
            CompactionRunOutcome::Stopped(stop),
        ));
    }
    // 次序（判决 G8）：删除续做在整理批之前。放锁的后台入口一次调用至多走一步、发了就返回，队列长的时候前台在步与步之间等得到锁。
    if !deletion_queue_of_the_version_is_empty(current) {
        let mut walked = Arc::clone(
            current
                .shared_version_of_trees()
                .expect("队列不空的一版是有树的一版"),
        );
        let walk = match span_of_the_call {
            SpanOfTheBackgroundCall::WholeTrip => publish_the_queue_until_it_is_empty(
                parameters,
                devices,
                allocator,
                &mut walked,
                instance,
            ),
            SpanOfTheBackgroundCall::AtMostOnePublish => publish_at_most_one_step_of_the_queue(
                parameters,
                devices,
                allocator,
                &mut walked,
                instance,
            ),
        };
        *current = PoolVersion::of_a_shared_version_of_trees(walked);
        run.deletion_steps_published =
            u64::try_from(walk.steps_published.len()).expect("一趟的删除步数");
        // 整理这一趟的版本列表按值交回（[`CompactionRun::versions_published`]）：走空的各步只有最后一步还与 `current` 共享，拷那一份。
        run.versions_published
            .extend(walk.steps_published.into_iter().map(Arc::unwrap_or_clone));
        if let Err(cause) = walk.stopped_by {
            return Err(run.finish(
                compaction,
                current.root().checkpoint_txg,
                CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(cause)),
            ));
        }
        if span_of_the_call == SpanOfTheBackgroundCall::AtMostOnePublish
            && (reach != CompactionRunReach::OnlyTheDeletionQueue
                || !deletion_queue_of_the_version_is_empty(current))
        {
            return Err(run.finish(
                compaction,
                current.root().checkpoint_txg,
                CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone),
            ));
        }
    }
    if reach == CompactionRunReach::OnlyTheDeletionQueue {
        return Err(run.finish(
            compaction,
            current.root().checkpoint_txg,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish,
            ),
        ));
    }
    Ok(run)
}

/// 每一批之前那几判（取这一批的输入之前）：预算、推到哪为止、放锁的入口至多一次发布、后台入口问调用方（让路与它的下限、写带宽）、回退门。
/// 放这一批就交回这一趟的账（记下这一批照的写带宽上限与是不是破例）；不放就照停的那一条收尾交回那一趟。回锁那一刻不问调用方：
/// 放锁的入口回锁发成之后再走这一判，前三样里至多一次发布那一样先停。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：这一趟的账、参数、盘表、分配器、会话这一侧的整理状态、现行那一版、每批之前看不看调用方"
)]
pub(super) fn judge_before_the_next_compaction_batch<Device: BlockDevice>(
    mut run: CompactionRunUnderWay,
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    allocator: &PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &PoolVersion,
    pacing: PacingOfTheRun<'_>,
) -> Result<CompactionRunUnderWay, Box<CompactionRun>> {
    if run.publishes_of_the_run >= run.budget {
        let budget = run.budget;
        return Err(run.finish(
            compaction,
            current.root().checkpoint_txg,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget }),
        ));
    }
    if run.reach == CompactionRunReach::AtMostOneBatchAfterAUserChange
        && run.batches_published + run.batches_published_earlier_in_the_trip >= 1
    {
        return Err(run.finish(
            compaction,
            current.root().checkpoint_txg,
            CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone),
        ));
    }
    if run.span_of_the_call == SpanOfTheBackgroundCall::AtMostOnePublish
        && run.publishes_of_the_run == 1
    {
        return Err(run.finish(
            compaction,
            current.root().checkpoint_txg,
            CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone),
        ));
    }
    // 全空段低于底额、前台写不进时这一趟的第一次发布破例：不判回退门（前台等它腾段），也不为前台让路（前台已写不进，让路换不来什么）。
    // 会话记着回锁时被回退门压住的那一条删意图时，这一次发的就是删它的那一次（[`take_the_input_of_one_compaction_batch`]）：破例照回锁那一刻同一个判法，
    // 此刻在现行那一版上判全空段低于底额、前台写不进（[`foreground_cannot_write_under_the_switches`]）。
    let is_this_publish_the_exception = (run.budget_rule
        == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
        && run.publishes_of_the_run == 0)
        || (stop_of_the_abandoned_batch_whose_intent_is_still_to_drop(compaction, current)
            .is_some()
            && foreground_cannot_write_under_the_switches(allocator));
    run.is_the_batch_the_exception = is_this_publish_the_exception;
    // D26（后台整理与放置回收） 已定项 13 第 6 条：后台入口每一批之前问调用方，前台有入口在等会话锁、写带宽到了上限就不发下一批。
    let write_bandwidth_limit_of_the_batch = match pacing {
        PacingOfTheRun::NotPaced => CompactionWriteBandwidthLimit::Unlimited,
        PacingOfTheRun::PacedByTheCaller {
            write_bandwidth_limit,
            yield_floor,
            caller,
            ..
        } => {
            let foreground_waiting = match caller.answer_before_the_next_batch() {
                AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting => {
                    Some(ForegroundEntryWaitingForTheSessionLock::Fsync)
                }
                AnswerOfTheCallerBeforeTheNextBatch::ForegroundEntryOtherThanAnFsyncIsWaitingForTheSessionLock => {
                    Some(ForegroundEntryWaitingForTheSessionLock::EntryOtherThanAnFsync)
                }
                AnswerOfTheCallerBeforeTheNextBatch::CallerIsStoppingItsThread => {
                    return Err(run.finish(
                        compaction,
                        current.root().checkpoint_txg,
                        CompactionRunOutcome::Stopped(
                            CompactionRunStop::YieldedToTheCallerStoppingItsThread,
                        ),
                    ));
                }
                AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting => None,
            };
            if let Some(waiting) = foreground_waiting.filter(|_| !is_this_publish_the_exception) {
                match compaction.judge_a_yield_under_the_floor(
                    yield_floor,
                    waiting,
                    caller.clock_reading(),
                ) {
                    ForegroundYieldUnderTheFloor::Yields => {
                        return Err(run.finish(
                            compaction,
                            current.root().checkpoint_txg,
                            CompactionRunOutcome::Stopped(waiting.stop_of_the_yield()),
                        ));
                    }
                    ForegroundYieldUnderTheFloor::DoesNotYieldForTheFloorIsReached => {}
                }
            }
            // 走到这里这一次取输入没让路（前台没有入口在等、破例、或下限到了）：「连着让路」断了，两个计数都清零
            // （D26（后台整理与放置回收） 已定项 13 第 6 条），后面写带宽或回退门停下、回锁没发成都不改这一条。
            compaction.break_the_yields_in_a_row_by_a_batch_not_yielding();
            if let Err(stop) = compaction.admit_the_next_batch_under_the_write_bandwidth_limit(
                write_bandwidth_limit,
                caller.clock_reading(),
            ) {
                return Err(run.finish(
                    compaction,
                    current.root().checkpoint_txg,
                    CompactionRunOutcome::Stopped(stop),
                ));
            }
            write_bandwidth_limit
        }
    };
    run.write_bandwidth_limit_of_the_batch = write_bandwidth_limit_of_the_batch;
    if run.is_the_rollback_gate_judged && !is_this_publish_the_exception {
        if let Some(stop) = stop_of_the_rollback_gate(parameters, devices, allocator, current) {
            return Err(run.finish(
                compaction,
                current.root().checkpoint_txg,
                CompactionRunOutcome::Stopped(stop),
            ));
        }
    }
    Ok(run)
}

/// 一批推完之后记账（[`run_compaction_on_with_the_region_check_of_the_intent_deletion`] 每一轮的后半）：发成一次就扣预算、记根槽与写带宽、
/// 搬了单元的记成一批，前台让路的连续计数清零；丢意图而停、照判据停、没发成的照那一条收尾交回那一趟，别的交回这一趟的账接着推。
pub(super) fn note_the_compaction_batch_advanced(
    mut run: CompactionRunUnderWay,
    advanced: CompactionPublishAdvanced,
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &PoolVersion,
    devices: usize,
) -> Result<CompactionRunUnderWay, Box<CompactionRun>> {
    let CompactionPublishAdvanced {
        step,
        background_compute_of_the_batch,
        txg_the_step_was_judged_on,
    } = advanced;
    run.background_compute_of_each_batch
        .extend(background_compute_of_the_batch);
    match step {
        Ok(CompactionStep::Published { version, kind }) => {
            run.publishes_of_the_run += 1;
            compaction.publishes_since_the_last_user_change += 1;
            compaction.observations.root_ring_slots_used += 1;
            compaction.note_the_bytes_written_by_a_compaction_publish(
                run.write_bandwidth_limit_of_the_batch,
                bytes_written_by_a_compaction_publish(&version, devices),
            );
            // 让路下限「一批发成两个都清零」（D26（后台整理与放置回收） 已定项 13 第 6 条）的一批是一次整理发布：签发、搬一批、删意图都清零。
            compaction.clear_the_yields_in_a_row();
            match kind {
                // 这一趟与观测数的批数（`batches_published`）只数搬了单元的那一次，与上面清零的口径不同：没搬单元的发布不记进批数
                // （删意图的那一批计划里的单元都是这次发布本来就重写的节点时，它就是一次只删意图的发布）。
                CompactionPublishKind::Batch | CompactionPublishKind::BatchCompletingTheIntent
                    if !version.units_relocated_by_this_publish.is_empty() =>
                {
                    run.batches_published += 1;
                    compaction.observations.batches_published += 1;
                    let relocated = u64::try_from(version.units_relocated_by_this_publish.len())
                        .expect("一批的单元数");
                    compaction.observations.units_relocated += relocated;
                    compaction.observations.units_relocated_by_the_last_batch = relocated;
                }
                CompactionPublishKind::Batch
                | CompactionPublishKind::BatchCompletingTheIntent
                | CompactionPublishKind::IntentIssued
                | CompactionPublishKind::IntentCompletedWithoutABatch
                | CompactionPublishKind::IntentDropped
                | CompactionPublishKind::IntentDroppedWithNothingMovable
                | CompactionPublishKind::IntentDroppedOnAStop { .. } => {}
            }
            run.versions_published.push(*version);
            let stop_after_the_drop = match kind {
                CompactionPublishKind::IntentDroppedWithNothingMovable => {
                    Some(CompactionRunStop::NothingMovableButTheRegionIsNotEmpty)
                }
                CompactionPublishKind::IntentDroppedOnAStop { stop } => Some(stop),
                CompactionPublishKind::IntentIssued
                | CompactionPublishKind::Batch
                | CompactionPublishKind::BatchCompletingTheIntent
                | CompactionPublishKind::IntentCompletedWithoutABatch
                | CompactionPublishKind::IntentDropped => None,
            };
            if let Some(stop) = stop_after_the_drop {
                return Err(run.finish(
                    compaction,
                    current.root().checkpoint_txg,
                    CompactionRunOutcome::Stopped(stop),
                ));
            }
            Ok(run)
        }
        Ok(CompactionStep::Stopped(stop)) => Err(run.finish(
            compaction,
            txg_the_step_was_judged_on,
            CompactionRunOutcome::Stopped(stop),
        )),
        Err(refused) => {
            note_the_kind_of_a_publish_frozen_by_this_call(
                allocator,
                run.was_frozen_when_this_call_came_in,
                KindOfTheFrozenPublish::CompactionPublish,
            );
            Err(run.finish(
                compaction,
                current.root().checkpoint_txg,
                CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),
            ))
        }
    }
}

/// 放锁的三段入口推一趟推到哪：要算一批（交回这一趟的账与这一批的输入），或在锁里就推完了。封闭集合。
pub(super) enum CompactionBatchTakenOrRunFinished {
    RunFinished(Box<CompactionRun>),
    BatchTaken {
        run: CompactionRunUnderWay,
        input: Box<CompactionBatchInput>,
    },
}

/// 放锁的三段入口推一趟（[`MountedSession::take_the_background_batch_that_starts`]）：开一趟、每批之前那几判、取一批的输入，与锁里推完一趟
/// （[`run_compaction_on_with_the_region_check_of_the_intent_deletion`]）同几块；推到要算一批就交回，不在这里算。
/// 迭代上界是 2：放锁的入口一次调用至多一次发布，取输入那一刻整理停着而删了意图、或删了回锁时被回退门压住的那一条意图
/// （[`take_the_input_of_one_compaction_batch`] 在锁里推完的那两种）之后这一趟就停。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、预算按哪一条算、每批之前问调用方、这次挂载的后台计算状态"
)]
pub(super) fn take_the_next_compaction_batch_or_finish_the_run<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
    budget_rule: CompactionRunBudget,
    pacing: PacingOfTheRun<'_>,
    background_compute: &mut BackgroundComputeOfThisMount,
) -> CompactionBatchTakenOrRunFinished {
    let mut run = match begin_a_compaction_run(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        reach,
        budget_rule,
        pacing,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        background_compute,
    ) {
        Ok(run) => run,
        Err(finished) => return CompactionBatchTakenOrRunFinished::RunFinished(finished),
    };
    loop {
        run = match judge_before_the_next_compaction_batch(
            run, parameters, devices, allocator, compaction, current, pacing,
        ) {
            Ok(run) => run,
            Err(finished) => return CompactionBatchTakenOrRunFinished::RunFinished(finished),
        };
        match take_the_input_of_one_compaction_batch(
            parameters,
            devices,
            allocator,
            compaction,
            current,
            instance,
            &run,
            background_compute,
            &mut InTheMiddleOfEachBackgroundBatch::NothingHappens,
        ) {
            CompactionBatchTaken::Input(input) => {
                return CompactionBatchTakenOrRunFinished::BatchTaken { run, input }
            }
            CompactionBatchTaken::AdvancedInTheLock(advanced) => {
                run = match note_the_compaction_batch_advanced(
                    run,
                    *advanced,
                    allocator,
                    compaction,
                    current,
                    devices.len(),
                ) {
                    Ok(run) => run,
                    Err(finished) => {
                        return CompactionBatchTakenOrRunFinished::RunFinished(finished)
                    }
                };
            }
        }
    }
}

/// 停在 `stop` 上时这一趟没做完、是节奏或回锁截住的：放锁的入口把这一趟留着接着推，用户改动之后那一趟把「该跑了」放回去
/// （[`MountedSession::after_the_background_run`]）。穷举：加一条停因要在这里定它算不算。
pub(super) const fn stop_holds_back_the_trip_without_ending_it(stop: CompactionRunStop) -> bool {
    match stop {
        CompactionRunStop::YieldedToAForegroundFsync
        | CompactionRunStop::YieldedToAForegroundEntryWaitingForTheSessionLock
        | CompactionRunStop::YieldedToTheCallerStoppingItsThread
        | CompactionRunStop::WriteBandwidthLimitReached { .. }
        | CompactionRunStop::BatchPrunedToNothingAtTheRelock { .. }
        | CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock
        | CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock => true,
        CompactionRunStop::ForcedToStopForTests
        | CompactionRunStop::NoEligibleSegment
        | CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. }
        | CompactionRunStop::NoYieldFromTheLastIntent { .. }
        | CompactionRunStop::PublishBudgetSpent { .. }
        | CompactionRunStop::PausedShortOfSpace
        | CompactionRunStop::IssueRefusedShortOfSpace
        | CompactionRunStop::NoLiveIntentLeftToContinue
        | CompactionRunStop::NothingMovableButTheRegionIsNotEmpty
        | CompactionRunStop::OneBatchAfterTheUserChangeDone
        | CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
        | CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
        | CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. }
        | CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. }
        | CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
        | CompactionRunStop::RollbackStatesOfTheRingUnreadable
        | CompactionRunStop::OnePublishOfTheTripDone
        | CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish
        | CompactionRunStop::NotRunWhileABatchOfThisSessionIsOutOfTheLock => false,
    }
}

/// 停在 `stop` 上是给前台让路（前台有入口在等会话锁、调用方在停它的线程）：全空段低于底额、前台写不进时这样停的一趟不留着
/// （[`MountedSession::after_the_background_run`]）。穷举：加一条停因要在这里定它算不算。
pub(super) const fn stop_yields_to_the_foreground(stop: CompactionRunStop) -> bool {
    match stop {
        CompactionRunStop::YieldedToAForegroundFsync
        | CompactionRunStop::YieldedToAForegroundEntryWaitingForTheSessionLock
        | CompactionRunStop::YieldedToTheCallerStoppingItsThread => true,
        CompactionRunStop::ForcedToStopForTests
        | CompactionRunStop::NoEligibleSegment
        | CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. }
        | CompactionRunStop::NoYieldFromTheLastIntent { .. }
        | CompactionRunStop::PublishBudgetSpent { .. }
        | CompactionRunStop::PausedShortOfSpace
        | CompactionRunStop::IssueRefusedShortOfSpace
        | CompactionRunStop::NoLiveIntentLeftToContinue
        | CompactionRunStop::NothingMovableButTheRegionIsNotEmpty
        | CompactionRunStop::OneBatchAfterTheUserChangeDone
        | CompactionRunStop::WriteBandwidthLimitReached { .. }
        | CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
        | CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
        | CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. }
        | CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. }
        | CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
        | CompactionRunStop::RollbackStatesOfTheRingUnreadable
        | CompactionRunStop::OnePublishOfTheTripDone
        | CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish
        | CompactionRunStop::BatchPrunedToNothingAtTheRelock { .. }
        | CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock
        | CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock
        | CompactionRunStop::NotRunWhileABatchOfThisSessionIsOutOfTheLock => false,
    }
}
