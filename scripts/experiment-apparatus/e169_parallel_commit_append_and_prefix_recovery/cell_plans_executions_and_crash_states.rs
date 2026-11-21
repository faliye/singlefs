//! 格的规划、交错与顺序约束、执行的放置、按写持久化状态收集崩溃状态。

use super::*;

/// 一格的布局（登记 5.1 的写怎么排、5.2 的臂、5.4 的历史；txg 与半截发布的读法见第十二节修订）。
pub(super) fn plan_cell(condition: PublishCondition, append_arm: AppendArm, history: &History, discipline: AppendDiscipline) -> CellLayout {
    let reserves_jsn_at_acquisition = append_arm == AppendArm::YiReservedWhole && discipline == AppendDiscipline::Kept;
    let narrow_void_leaves_fillers = reserves_jsn_at_acquisition;
    let mut reserved_first_jsns: Vec<u64> = Vec::new();
    let mut next_reserved_jsn = 1u64;
    for record_count in &history.record_counts {
        reserved_first_jsns.push(next_reserved_jsn);
        next_reserved_jsn += u64::from(*record_count);
    }
    let reserved_first_of = |transaction: u32| -> Option<u64> {
        if reserves_jsn_at_acquisition {
            Some(reserved_first_jsns[transaction_position(transaction)])
        } else {
            None
        }
    };
    let fillers_of = |transaction: u32| -> Option<RecordStream> {
        let first = reserved_first_of(transaction)?;
        let record_count = history.record_count_of(transaction);
        match history.void_timing_of(transaction) {
            Some(VoidTiming::NarrowBeforeAnyWrite) if narrow_void_leaves_fillers => filler_stream(transaction, 1, record_count, first, false),
            Some(VoidTiming::NarrowBeforeAnyWrite) | None => None,
            Some(VoidTiming::WideAfterFirstRecord) => filler_stream(transaction, 2, record_count, first, true),
        }
    };
    let mut transactions: Vec<TransactionLayout> = (1..=history.transaction_count())
        .map(|number| TransactionLayout {
            number,
            planned_records: history.record_count_of(number),
            voided: history.void_timing_of(number),
            txg: None,
        })
        .collect();
    let mut publishes: Vec<PublishLayout> = Vec::new();
    match condition {
        PublishCondition::SharedPublish => {
            let mut unit_transactions = Vec::new();
            let mut record_streams = Vec::new();
            for transaction in 1..=history.transaction_count() {
                let timing = history.void_timing_of(transaction);
                if let Some(stream) = own_record_stream(transaction, history.record_count_of(transaction), timing, reserved_first_of(transaction)) {
                    unit_transactions.push(transaction);
                    record_streams.push(stream);
                }
                if let Some(stream) = fillers_of(transaction) {
                    record_streams.push(stream);
                }
                transactions[transaction_position(transaction)].txg = match timing {
                    Some(VoidTiming::NarrowBeforeAnyWrite) => None,
                    Some(VoidTiming::WideAfterFirstRecord) | None => Some(1),
                };
            }
            publishes.push(PublishLayout { identity: 1, txg: 1, is_abandoned: false, unit_transactions, record_streams });
        }
        // 多飞与串落的布局相同：每个没作废的事务自成一次发布，txg 按事务号连号（txg 从上一版现算，作废·窄的事务没开发布）；
        // 乙整段的占位写在作废事务之后紧接着的那个发布里，没有下一个时另开一次空发布（登记 5.2「没有永久的洞」）。
        // 两者只差执行怎么排：串落一次一个发布，多飞发布之间任意交错（`multi_flight_executions`）。
        PublishCondition::SerialPublishes | PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered => {
            let mut completed_publishes = 0u64;
            let mut identity = 0u32;
            let mut pending_fillers: Vec<RecordStream> = Vec::new();
            for transaction in 1..=history.transaction_count() {
                let timing = history.void_timing_of(transaction);
                match timing {
                    Some(VoidTiming::NarrowBeforeAnyWrite) => {
                        if let Some(stream) = fillers_of(transaction) {
                            pending_fillers.push(stream);
                        }
                    }
                    Some(VoidTiming::WideAfterFirstRecord) => {
                        identity += 1;
                        let wide_void_own_stream = own_record_stream(transaction, history.record_count_of(transaction), timing, reserved_first_of(transaction))
                            .expect("作废·宽的事务发出了第一条记录");
                        publishes.push(PublishLayout {
                            identity,
                            txg: completed_publishes + 1,
                            is_abandoned: true,
                            unit_transactions: vec![transaction],
                            record_streams: vec![wide_void_own_stream],
                        });
                        transactions[transaction_position(transaction)].txg = Some(completed_publishes + 1);
                        if let Some(stream) = fillers_of(transaction) {
                            pending_fillers.push(stream);
                        }
                    }
                    None => {
                        identity += 1;
                        completed_publishes += 1;
                        let mut record_streams: Vec<RecordStream> = std::mem::take(&mut pending_fillers);
                        record_streams.push(
                            own_record_stream(transaction, history.record_count_of(transaction), None, reserved_first_of(transaction))
                                .expect("没作废的事务有自己的记录流"),
                        );
                        publishes.push(PublishLayout {
                            identity,
                            txg: completed_publishes,
                            is_abandoned: false,
                            unit_transactions: vec![transaction],
                            record_streams,
                        });
                        transactions[transaction_position(transaction)].txg = Some(completed_publishes);
                    }
                }
            }
            if !pending_fillers.is_empty() {
                identity += 1;
                completed_publishes += 1;
                publishes.push(PublishLayout {
                    identity,
                    txg: completed_publishes,
                    is_abandoned: false,
                    unit_transactions: Vec::new(),
                    record_streams: pending_fillers,
                });
            }
        }
    }
    CellLayout { transactions, publishes }
}

// ===== 执行：记录流的发出次序 =====

fn interleavings(stream_lengths: &[usize]) -> Vec<Vec<usize>> {
    let total: usize = stream_lengths.iter().sum();
    let mut results = Vec::new();
    let mut current = Vec::with_capacity(total);
    let mut taken = vec![0usize; stream_lengths.len()];
    extend_interleavings(stream_lengths, &mut taken, &mut current, total, &mut results);
    results
}

fn extend_interleavings(stream_lengths: &[usize], taken: &mut [usize], current: &mut Vec<usize>, total: usize, results: &mut Vec<Vec<usize>>) {
    if current.len() == total {
        results.push(current.clone());
        return;
    }
    for stream_index in 0..stream_lengths.len() {
        if taken[stream_index] < stream_lengths[stream_index] {
            taken[stream_index] += 1;
            current.push(stream_index);
            extend_interleavings(stream_lengths, taken, current, total, results);
            current.pop();
            taken[stream_index] -= 1;
        }
    }
}

fn order_is_allowed(order: &[usize], streams: &[RecordStream], append_arm: AppendArm, discipline: AppendDiscipline) -> bool {
    let first_position = |stream_index: usize| order.iter().position(|chosen| *chosen == stream_index).expect("每条流至少一条记录");
    let last_position = |stream_index: usize| order.iter().rposition(|chosen| *chosen == stream_index).expect("每条流至少一条记录");
    for (stream_index, stream) in streams.iter().enumerate() {
        if let Some(leader_transaction) = stream.must_follow_first_record_of {
            if let Some(leader_index) = streams.iter().position(|candidate| !candidate.is_filler && candidate.transaction == leader_transaction) {
                if first_position(stream_index) < first_position(leader_index) {
                    return false;
                }
            }
        }
    }
    if discipline == AppendDiscipline::RemovedForPositiveControl {
        return true;
    }
    for (earlier_index, earlier) in streams.iter().enumerate() {
        for (later_index, later) in streams.iter().enumerate() {
            if earlier.is_filler || later.is_filler || earlier.transaction >= later.transaction {
                continue;
            }
            let later_first = first_position(later_index);
            let earlier_first = first_position(earlier_index);
            let earlier_last = last_position(earlier_index);
            let violates = match append_arm {
                AppendArm::JiaWhole => later_first < earlier_last,
                AppendArm::JiaInterleaved => later_first < earlier_first,
                AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => false,
            };
            if violates {
                return false;
            }
        }
    }
    true
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum VoidPoint {
    FromTheStart,
    AfterOperation(usize),
}

#[derive(Clone, Debug)]
pub(super) struct TransactionTimeline {
    pub(super) number: u32,
    pub(super) planned_records: u32,
    pub(super) issued_record_operation_indices: Vec<usize>,
    pub(super) final_barrier_operation_index: Option<usize>,
    pub(super) voided_from: Option<VoidPoint>,
}

#[derive(Clone, Debug)]
pub(super) struct Execution {
    pub(super) operations: Vec<Operation>,
    pub(super) timelines: Vec<TransactionTimeline>,
}

fn place_records_of_publish(publish: &PublishLayout, order: &[usize], next_issue_jsn: &mut u64) -> Vec<(PlannedRecord, RecordContent)> {
    let mut cursor = vec![0usize; publish.record_streams.len()];
    let mut issued: Vec<(PlannedRecord, u64)> = Vec::new();
    for stream_index in order {
        let planned = publish.record_streams[*stream_index].records[cursor[*stream_index]];
        cursor[*stream_index] += 1;
        let jsn = match planned.reserved_jsn {
            Some(reserved) => reserved,
            None => {
                let issue_jsn = *next_issue_jsn;
                *next_issue_jsn += 1;
                issue_jsn
            }
        };
        issued.push((planned, jsn));
    }
    let mut jsns_ascending: Vec<u64> = issued.iter().map(|(_, jsn)| *jsn).collect();
    jsns_ascending.sort_unstable();
    // 末条标志在一次发布 jsn 最大的那条上；那一条发出时本发布还有计划中的记录（作废·宽的第一条），它不知道自己是末条。
    let end_jsn = issued
        .iter()
        .max_by_key(|(_, jsn)| *jsn)
        .filter(|(planned, _)| !planned.successor_planned_at_issue)
        .map(|(_, jsn)| *jsn);
    issued
        .iter()
        .map(|(planned, jsn)| {
            let ordinal = jsns_ascending.iter().position(|candidate| candidate == jsn).expect("jsn 在本发布里") + 1;
            (
                *planned,
                RecordContent {
                    jsn: *jsn,
                    txg: publish.txg,
                    publish: publish.identity,
                    transaction: planned.transaction,
                    is_commit: planned.is_commit,
                    ends_its_publish: Some(*jsn) == end_jsn,
                    ordinal_within_publish: u32::try_from(ordinal).expect("一次发布至多 6 条记录"),
                    origin: planned.origin,
                },
            )
        })
        .collect()
}

fn execution_for(layout: &CellLayout, orders: &[Vec<Vec<usize>>], choice: &[usize], barriers: PublishBarriers) -> Execution {
    let mut operations: Vec<Operation> = Vec::new();
    let mut next_issue_jsn = 1u64;
    let mut timelines: Vec<TransactionTimeline> = layout
        .transactions
        .iter()
        .map(|transaction| TransactionTimeline {
            number: transaction.number,
            planned_records: transaction.planned_records,
            issued_record_operation_indices: Vec::new(),
            final_barrier_operation_index: None,
            voided_from: match transaction.voided {
                Some(VoidTiming::NarrowBeforeAnyWrite) => Some(VoidPoint::FromTheStart),
                Some(VoidTiming::WideAfterFirstRecord) | None => None,
            },
        })
        .collect();
    for (publish_position, publish) in layout.publishes.iter().enumerate() {
        let order = &orders[publish_position][choice[publish_position]];
        for transaction in &publish.unit_transactions {
            operations.push(Operation::Write {
                content: WriteContent::TransactionUnit { transaction: *transaction },
                states: WriteDurabilityStates::NewOrMissing,
            });
        }
        operations.push(Operation::Write {
            content: WriteContent::SharedUnit { publish: publish.identity },
            states: WriteDurabilityStates::NewOrMissing,
        });
        if barriers == PublishBarriers::AllThree {
            // B1：单元与共享单元之后
            operations.push(Operation::Barrier);
        }
        for (planned, content) in place_records_of_publish(publish, order, &mut next_issue_jsn) {
            let operation_index = operations.len();
            operations.push(Operation::Write {
                content: WriteContent::JournalRecord(content),
                states: WriteDurabilityStates::NewOrMissing,
            });
            if planned.transaction != 0 {
                let timeline = &mut timelines[transaction_position(planned.transaction)];
                timeline.issued_record_operation_indices.push(operation_index);
                if layout.transactions[transaction_position(planned.transaction)].voided == Some(VoidTiming::WideAfterFirstRecord) {
                    timeline.voided_from = Some(VoidPoint::AfterOperation(operation_index));
                }
            }
        }
        if publish.is_abandoned {
            continue;
        }
        if barriers == PublishBarriers::AllThree {
            // B2：记录之后
            operations.push(Operation::Barrier);
        }
        let root_slot = WriteContent::RootSlot { publish: publish.identity, txg: publish.txg };
        if ROOT_SLOT_WRITE_CLOSES_ITS_SEGMENT {
            operations.push(Operation::ForcedUnitAccessWrite { content: root_slot });
        } else {
            operations.push(Operation::Write { content: root_slot, states: WriteDurabilityStates::NewOrMissing });
        }
        operations.push(Operation::Write {
            content: WriteContent::SystemConfiguration { publish: publish.identity },
            states: SYSTEM_CONFIGURATION_DURABILITY,
        });
        let final_barrier_index = operations.len();
        // B3：系统配置轮换之后
        operations.push(Operation::Barrier);
        for transaction in &publish.unit_transactions {
            if layout.transactions[transaction_position(*transaction)].voided.is_none() {
                timelines[transaction_position(*transaction)].final_barrier_operation_index = Some(final_barrier_index);
            }
        }
    }
    Execution { operations, timelines }
}

/// 每次发布里允许的记录发出次序（发布内的臂约束）。
pub(super) fn allowed_record_orders(layout: &CellLayout, append_arm: AppendArm, discipline: AppendDiscipline) -> Vec<Vec<Vec<usize>>> {
    let orders: Vec<Vec<Vec<usize>>> = layout
        .publishes
        .iter()
        .map(|publish| {
            let lengths: Vec<usize> = publish.record_streams.iter().map(|stream| stream.records.len()).collect();
            interleavings(&lengths)
                .into_iter()
                .filter(|order| order_is_allowed(order, &publish.record_streams, append_arm, discipline))
                .collect()
        })
        .collect();
    assert!(orders.iter().all(|choices| !choices.is_empty()), "每次发布至少有一种允许的发出次序");
    orders
}

pub(super) fn build_executions(layout: &CellLayout, append_arm: AppendArm, discipline: AppendDiscipline, barriers: PublishBarriers) -> Vec<Execution> {
    let orders = allowed_record_orders(layout, append_arm, discipline);
    let mut executions = Vec::new();
    let mut choice = vec![0usize; orders.len()];
    loop {
        executions.push(execution_for(layout, &orders, &choice, barriers));
        let mut position = choice.len();
        loop {
            if position == 0 {
                return executions;
            }
            position -= 1;
            choice[position] += 1;
            if choice[position] < orders[position].len() {
                break;
            }
            choice[position] = 0;
        }
    }
}

// ===== 崩溃状态与 fsync 返回 =====

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum DurableForm {
    New,
    Torn,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum ReturnBranch {
    /// 析取支 ①：小于它的事务全部已追加。
    EveryEarlierAppended,
    /// 析取支 ②：至少一个小于它的事务是靠「已作废」放行的。
    SomeEarlierPassedAsVoided,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) struct CrashState {
    pub(super) durable: Vec<(WriteContent, DurableForm)>,
    pub(super) returned: Vec<(u32, ReturnBranch)>,
}

pub(super) fn is_voided_by(timeline: &TransactionTimeline, issued_through: Option<usize>) -> bool {
    match timeline.voided_from {
        None => false,
        Some(VoidPoint::FromTheStart) => true,
        Some(VoidPoint::AfterOperation(operation_index)) => issued_through.is_some_and(|issued| issued >= operation_index),
    }
}

fn all_records_issued(timeline: &TransactionTimeline, issued_through: Option<usize>) -> bool {
    let planned = usize::try_from(timeline.planned_records).expect("r 只取 1、2");
    timeline.issued_record_operation_indices.len() == planned
        && timeline
            .issued_record_operation_indices
            .iter()
            .all(|operation_index| issued_through.is_some_and(|issued| *operation_index <= issued))
}

pub(super) fn is_appended(timeline: &TransactionTimeline, completed_through: Option<usize>, issued_through: Option<usize>, reading: AppendedReading) -> bool {
    if !all_records_issued(timeline, issued_through) {
        return false;
    }
    match reading {
        AppendedReading::IssuedOnly => true,
        AppendedReading::IssuedAndCovered => {
            let last_record_index = timeline.issued_record_operation_indices.iter().copied().max().expect("r ≥ 1");
            // completed_through 是一个闭段点：它在最后一条记录之后，就有一道完成了的屏障或 FUA 罩住那些记录。
            completed_through.is_some_and(|completed| last_record_index < completed)
        }
    }
}

/// 登记第一节 1.2「fsync 返回」：① 本事务所在发布的最后一道屏障已完成；② 每个 j < i 已作废或已追加。
fn returned_transactions(execution: &Execution, completed_through: Option<usize>, issued_through: Option<usize>, reading: AppendedReading, rule: ReturnRule) -> Vec<(u32, ReturnBranch)> {
    let mut returned = Vec::new();
    for timeline in &execution.timelines {
        if is_voided_by(timeline, issued_through) {
            continue;
        }
        let own_condition = match rule {
            ReturnRule::AfterOwnPublishLastBarrier => timeline
                .final_barrier_operation_index
                .is_some_and(|barrier| completed_through.is_some_and(|completed| barrier <= completed)),
            ReturnRule::EarlyAfterOwnRecordsIssued => all_records_issued(timeline, issued_through),
        };
        if !own_condition {
            continue;
        }
        let mut passed_as_voided = false;
        let mut blocked = false;
        for earlier in execution.timelines.iter().filter(|candidate| candidate.number < timeline.number) {
            if is_appended(earlier, completed_through, issued_through, reading) {
                continue;
            }
            if is_voided_by(earlier, issued_through) {
                passed_as_voided = true;
                continue;
            }
            blocked = true;
            break;
        }
        if blocked {
            continue;
        }
        let branch = if passed_as_voided { ReturnBranch::SomeEarlierPassedAsVoided } else { ReturnBranch::EveryEarlierAppended };
        returned.push((timeline.number, branch));
    }
    returned
}

/// 一个执行的全部崩溃状态并进 `states`；返回第七节 A7 不成立的崩溃点个数（返回集合里有事务的最后一道屏障还没完成）。
pub(super) fn collect_crash_states(execution: &Execution, reading: AppendedReading, rule: ReturnRule, states: &mut HashSet<CrashState>) -> u64 {
    let operations = &execution.operations;
    let mut boundaries: Vec<Option<usize>> = vec![None];
    boundaries.extend(
        operations
            .iter()
            .enumerate()
            .filter(|(_, operation)| operation.closes_a_segment())
            .map(|(operation_index, _)| Some(operation_index)),
    );
    let mut returned_before_last_barrier_points = 0u64;
    for (boundary_position, completed_through) in boundaries.iter().copied().enumerate() {
        let segment_first = completed_through.map_or(0, |completed| completed + 1);
        let segment_end_exclusive = match boundaries.get(boundary_position + 1) {
            Some(next_closer) => next_closer.expect("第一个之后都是闭段点") + 1,
            None => operations.len(),
        };
        let base: Vec<(WriteContent, DurableForm)> = operations[..segment_first]
            .iter()
            .filter_map(Operation::written_content)
            .map(|content| (content, DurableForm::New))
            .collect();
        let mut issue_candidates: Vec<Option<usize>> = vec![completed_through];
        issue_candidates.extend((segment_first..segment_end_exclusive).map(Some));
        #[allow(clippy::type_complexity, reason = "每组就是（签发到哪一笔，这一笔之后各笔的返回分支），只在这个循环里按位取用，命名结构体不会让哪一步更好验")]
        let mut groups: Vec<(Option<usize>, Vec<(u32, ReturnBranch)>)> = Vec::new();
        for issued_through in issue_candidates {
            let returned = returned_transactions(execution, completed_through, issued_through, reading, rule);
            let extends_last_group = groups.last().is_some_and(|(_, last_returned)| *last_returned == returned);
            if extends_last_group {
                groups.last_mut().expect("刚看过非空").0 = issued_through;
            } else {
                groups.push((issued_through, returned));
            }
        }
        for (issued_through, returned) in groups {
            for (number, _) in &returned {
                let timeline = &execution.timelines[transaction_position(*number)];
                let barrier_done = timeline
                    .final_barrier_operation_index
                    .is_some_and(|barrier| completed_through.is_some_and(|completed| barrier <= completed));
                if !barrier_done {
                    returned_before_last_barrier_points += 1;
                }
            }
            let segment_writes: Vec<(WriteContent, WriteDurabilityStates)> = (segment_first..segment_end_exclusive)
                .filter(|operation_index| issued_through.is_some_and(|issued| *operation_index <= issued))
                .filter_map(|operation_index| operations[operation_index].durability_choices())
                .collect();
            insert_every_durability_choice(&base, &segment_writes, &returned, states);
        }
    }
    returned_before_last_barrier_points
}

pub(super) fn insert_every_durability_choice(base: &[(WriteContent, DurableForm)], segment_writes: &[(WriteContent, WriteDurabilityStates)], returned: &[(u32, ReturnBranch)], states: &mut HashSet<CrashState>) {
    let mut digits = vec![0usize; segment_writes.len()];
    loop {
        let mut durable: Vec<(WriteContent, DurableForm)> = base.to_vec();
        for (position, (content, durability)) in segment_writes.iter().enumerate() {
            if let Some(form) = durability.forms()[digits[position]] {
                durable.push((*content, form));
            }
        }
        durable.sort();
        states.insert(CrashState { durable, returned: returned.to_vec() });
        let mut position = digits.len();
        loop {
            if position == 0 {
                return;
            }
            position -= 1;
            digits[position] += 1;
            if digits[position] < segment_writes[position].1.forms().len() {
                break;
            }
            digits[position] = 0;
        }
    }
}
