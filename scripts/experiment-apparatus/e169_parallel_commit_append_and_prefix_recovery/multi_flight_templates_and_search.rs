//! 多在途的发布模板、多在途规则下的执行合并、配置搜索与多在途崩溃状态的枚举。

use super::*;

// ===== 第二段：多飞（发布之间任意交错） =====

/// 第二段 k = 3 只跑这两种 r（登记 5.6 第二段）。
const SEGMENT_TWO_THREE_TRANSACTION_RECORD_COUNTS: [&[u32]; 2] = [&[1, 1, 1], &[2, 1, 1]];
/// 停机条款 T3：多飞 k = 3 的配置数超过 10⁸ 就停在 k = 2（登记第十一节）。
pub(super) const CONFIGURATION_LIMIT: u64 = 100_000_000;
/// 一份配置里装得下的发布数：k ≤ 3，加乙整段作废最后一个事务时另开的那次空发布。
const MAXIMUM_PUBLISHES_IN_A_CONFIGURATION: usize = 4;
/// 一份配置里一次发布装得下的记录数：多飞一次发布至多是一个事务的 2 条加前一个作废事务的 2 条占位。
const MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION: usize = 4;

/// 第二段的历史（登记 5.6 第二段）：k = 2 的 H0、H1 全部；k = 3 只取 r = (1, 1, 1) 与 (2, 1, 1) 的 H0、H1。H2 放在段末、够判后不跑（登记第六节「附带」）。
pub(super) fn segment_two_histories() -> Vec<History> {
    all_histories()
        .into_iter()
        .filter(|history| history.family().is_legal())
        .filter(|history| history.transaction_count() == 2 || SEGMENT_TWO_THREE_TRANSACTION_RECORD_COUNTS.contains(&history.record_counts.as_slice()))
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub(super) enum TemplateOperation {
    Fixed(Operation),
    /// 记录的 jsn 在发出时才定（发 jsn 的臂）；发布内序号与末条标志按这次发布的记录次序先算好（`publish_template`）。
    Record { planned: PlannedRecord, ordinal_within_publish: u32, ends_its_publish: bool },
}

/// 一次发布按发布内次序排好的操作（D16（发布语义） 已定项 7 的写序）。
#[derive(Clone, Debug)]
pub(super) struct PublishTemplate {
    pub(super) identity: u32,
    pub(super) txg: u64,
    pub(super) unit_transactions: Vec<u32>,
    pub(super) operations: Vec<TemplateOperation>,
    root_slot_position: usize,
    system_configuration_position: usize,
    final_barrier_position: usize,
}

impl PublishTemplate {
    fn own_record_positions_of(&self, transaction: u32) -> Vec<usize> {
        self.operations
            .iter()
            .enumerate()
            .filter(|(_, operation)| match operation {
                TemplateOperation::Record { planned, .. } => planned.transaction == transaction && planned.transaction != 0,
                TemplateOperation::Fixed(_) => false,
            })
            .map(|(operation_position, _)| operation_position)
            .collect()
    }
    fn own_transactions(&self) -> BTreeSet<u32> {
        self.operations
            .iter()
            .filter_map(|operation| match operation {
                TemplateOperation::Record { planned, .. } if planned.transaction != 0 => Some(planned.transaction),
                TemplateOperation::Record { .. } | TemplateOperation::Fixed(_) => None,
            })
            .collect()
    }
    /// 这一位置是这次发布的第几条记录（从 0 起）；不是记录时为 None。
    fn record_index_at(&self, operation_position: usize) -> Option<usize> {
        match self.operations[operation_position] {
            TemplateOperation::Record { .. } => Some(self.operations[..operation_position].iter().filter(|operation| matches!(operation, TemplateOperation::Record { .. })).count()),
            TemplateOperation::Fixed(_) => None,
        }
    }
}

/// 一次发布的操作序列。发布内序号与末条标志照 `place_records_of_publish` 的口径（按 jsn 排名、jsn 最大那条带末条）：
/// 发 jsn 的臂里一次发布的记录按发布内次序先后拿号，名次就是发布内次序；预留的臂按预留号排名。
fn publish_template(publish: &PublishLayout, order: &[usize]) -> PublishTemplate {
    assert!(!publish.is_abandoned, "多飞不跑作废·宽（第二段 H2 够判后未跑）");
    let mut operations: Vec<TemplateOperation> = Vec::new();
    for transaction in &publish.unit_transactions {
        operations.push(TemplateOperation::Fixed(Operation::Write {
            content: WriteContent::TransactionUnit { transaction: *transaction },
            states: WriteDurabilityStates::NewOrMissing,
        }));
    }
    operations.push(TemplateOperation::Fixed(Operation::Write {
        content: WriteContent::SharedUnit { publish: publish.identity },
        states: WriteDurabilityStates::NewOrMissing,
    }));
    // B1：单元与共享单元之后
    operations.push(TemplateOperation::Fixed(Operation::Barrier));
    let mut cursor = vec![0usize; publish.record_streams.len()];
    let planned_in_order: Vec<PlannedRecord> = order
        .iter()
        .map(|stream_index| {
            let planned = publish.record_streams[*stream_index].records[cursor[*stream_index]];
            cursor[*stream_index] += 1;
            planned
        })
        .collect();
    let rank_keys: Vec<u64> = planned_in_order
        .iter()
        .enumerate()
        .map(|(position_in_publish, planned)| planned.reserved_jsn.unwrap_or(u64::try_from(position_in_publish).expect("一次发布至多 4 条记录")))
        .collect();
    let largest_key = rank_keys.iter().copied().max();
    for (position_in_publish, planned) in planned_in_order.iter().enumerate() {
        let key = rank_keys[position_in_publish];
        let ordinal = 1 + rank_keys.iter().filter(|other_key| **other_key < key).count();
        operations.push(TemplateOperation::Record {
            planned: *planned,
            ordinal_within_publish: u32::try_from(ordinal).expect("一次发布至多 4 条记录"),
            ends_its_publish: Some(key) == largest_key && !planned.successor_planned_at_issue,
        });
    }
    // B2：记录之后
    operations.push(TemplateOperation::Fixed(Operation::Barrier));
    let root_slot_position = operations.len();
    let root_slot = WriteContent::RootSlot { publish: publish.identity, txg: publish.txg };
    if ROOT_SLOT_WRITE_CLOSES_ITS_SEGMENT {
        operations.push(TemplateOperation::Fixed(Operation::ForcedUnitAccessWrite { content: root_slot }));
    } else {
        operations.push(TemplateOperation::Fixed(Operation::Write { content: root_slot, states: WriteDurabilityStates::NewOrMissing }));
    }
    let system_configuration_position = operations.len();
    operations.push(TemplateOperation::Fixed(Operation::Write {
        content: WriteContent::SystemConfiguration { publish: publish.identity },
        states: SYSTEM_CONFIGURATION_DURABILITY,
    }));
    let final_barrier_position = operations.len();
    // B3：系统配置轮换之后
    operations.push(TemplateOperation::Fixed(Operation::Barrier));
    PublishTemplate {
        identity: publish.identity,
        txg: publish.txg,
        unit_transactions: publish.unit_transactions.clone(),
        operations,
        root_slot_position,
        system_configuration_position,
        final_barrier_position,
    }
}

/// 每次发布各选一种允许的记录次序，全部组合（各发布的选择号）。
fn every_order_choice(orders: &[Vec<Vec<usize>>]) -> Vec<Vec<usize>> {
    let mut choices: Vec<Vec<usize>> = vec![Vec::new()];
    for publish_orders in orders {
        choices = choices
            .into_iter()
            .flat_map(|prefix| {
                (0..publish_orders.len()).map(move |choice| {
                    let mut extended = prefix.clone();
                    extended.push(choice);
                    extended
                })
            })
            .collect();
    }
    choices
}

fn multi_flight_templates(layout: &CellLayout, orders: &[Vec<Vec<usize>>], choice: &[usize]) -> Vec<PublishTemplate> {
    let templates: Vec<PublishTemplate> = layout
        .publishes
        .iter()
        .enumerate()
        .map(|(publish_position, publish)| publish_template(publish, &orders[publish_position][choice[publish_position]]))
        .collect();
    assert!(templates.windows(2).all(|pair| pair[0].txg < pair[1].txg), "模板按 txg 升序排（根序的「前一次发布」就是前一个模板）");
    templates
}

/// 多飞的发布之间的次序约束（登记 5.1 根序一行；5.2 甲两条臂的跨事务约束在多飞下落在不同发布之间）。
#[derive(Clone, Copy, Debug)]
pub(super) struct MultiFlightRules {
    roots_in_txg_order: bool,
    pub(super) append_arm: AppendArm,
    pub(super) discipline: AppendDiscipline,
}

impl MultiFlightRules {
    pub(super) fn of(condition: PublishCondition, append_arm: AppendArm, discipline: AppendDiscipline) -> MultiFlightRules {
        let roots_in_txg_order = match condition {
            PublishCondition::MultiFlightRootsOrdered => true,
            PublishCondition::MultiFlightRootsUnordered => false,
            PublishCondition::SerialPublishes | PublishCondition::SharedPublish => panic!("只有多飞两个条件的发布之间任意交错"),
        };
        MultiFlightRules { roots_in_txg_order, append_arm, discipline }
    }
}

/// 第 `publish_position` 次发布的下一个操作，在各发布已发出到 `issued_counts` 时能不能发出。
fn next_operation_may_issue(templates: &[PublishTemplate], issued_counts: &[usize], publish_position: usize, rules: MultiFlightRules) -> bool {
    let template = &templates[publish_position];
    let next_position = issued_counts[publish_position];
    // 多飞·根序：F 与 C 都按 txg 先后发出。
    if rules.roots_in_txg_order && publish_position > 0 {
        let previous = &templates[publish_position - 1];
        let previous_issued = issued_counts[publish_position - 1];
        let waits_for_previous_root_slot = next_position == template.root_slot_position && previous_issued <= previous.root_slot_position;
        let waits_for_previous_system_configuration = next_position == template.system_configuration_position && previous_issued <= previous.system_configuration_position;
        if waits_for_previous_root_slot || waits_for_previous_system_configuration {
            return false;
        }
    }
    let starting_transaction = match template.operations[next_position] {
        TemplateOperation::Record { planned, .. } if planned.transaction != 0 && planned.origin == (RecordOrigin::OwnRecord { index_within_transaction: 1 }) => planned.transaction,
        TemplateOperation::Record { .. } | TemplateOperation::Fixed(_) => return true,
    };
    if rules.discipline == AppendDiscipline::RemovedForPositiveControl {
        return true;
    }
    for (other_position, other) in templates.iter().enumerate().filter(|(other_position, _)| *other_position != publish_position) {
        for earlier_transaction in other.own_transactions().into_iter().filter(|candidate| *candidate < starting_transaction) {
            let positions = other.own_record_positions_of(earlier_transaction);
            let earlier_first = *positions.first().expect("没作废的事务至少一条记录");
            let earlier_last = *positions.last().expect("没作废的事务至少一条记录");
            let earlier_cleared = match rules.append_arm {
                // 甲整段：发出 Ti 的第一条之前，每个 j < i 的 Tj 记录全部已发出
                AppendArm::JiaWhole => issued_counts[other_position] > earlier_last,
                // 甲交错：Ti 的第一条在每个 j < i 的 Tj 的第一条之后发出
                AppendArm::JiaInterleaved => issued_counts[other_position] > earlier_first,
                AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => true,
            };
            if !earlier_cleared {
                return false;
            }
        }
    }
    true
}

fn every_allowed_merge(templates: &[PublishTemplate], rules: MultiFlightRules) -> Vec<Vec<usize>> {
    let lengths: Vec<usize> = templates.iter().map(|template| template.operations.len()).collect();
    let total: usize = lengths.iter().sum();
    let mut merges = Vec::new();
    let mut issued_counts = vec![0usize; templates.len()];
    let mut current = Vec::with_capacity(total);
    extend_allowed_merges(templates, rules, &lengths, &mut issued_counts, &mut current, total, &mut merges);
    merges
}

fn extend_allowed_merges(templates: &[PublishTemplate], rules: MultiFlightRules, lengths: &[usize], issued_counts: &mut [usize], current: &mut Vec<usize>, total: usize, merges: &mut Vec<Vec<usize>>) {
    if current.len() == total {
        merges.push(current.clone());
        return;
    }
    let mut extended = false;
    for publish_position in 0..templates.len() {
        if issued_counts[publish_position] < lengths[publish_position] && next_operation_may_issue(templates, issued_counts, publish_position, rules) {
            extended = true;
            issued_counts[publish_position] += 1;
            current.push(publish_position);
            extend_allowed_merges(templates, rules, lengths, issued_counts, current, total, merges);
            current.pop();
            issued_counts[publish_position] -= 1;
        }
    }
    assert!(extended, "多飞的次序约束不会卡死：按 txg 先后把各发布做完总走得通");
}

/// 直接枚举的一个执行：先按全局发出次序给 jsn，再在每次发布里按 jsn 排名定发布内序号与末条（与 `place_records_of_publish` 同一口径），最后按全局次序排出操作。
fn execution_from_merge(layout: &CellLayout, templates: &[PublishTemplate], merge: &[usize]) -> Execution {
    let mut cursors = vec![0usize; templates.len()];
    let mut next_issue_jsn = 1u64;
    let mut record_jsns: BTreeMap<(usize, usize), u64> = BTreeMap::new();
    for publish_position in merge {
        let operation_position = cursors[*publish_position];
        cursors[*publish_position] += 1;
        if let TemplateOperation::Record { planned, .. } = templates[*publish_position].operations[operation_position] {
            let jsn = match planned.reserved_jsn {
                Some(reserved) => reserved,
                None => {
                    let issue_jsn = next_issue_jsn;
                    next_issue_jsn += 1;
                    issue_jsn
                }
            };
            record_jsns.insert((*publish_position, operation_position), jsn);
        }
    }
    let mut record_contents: BTreeMap<(usize, usize), RecordContent> = BTreeMap::new();
    for (publish_position, template) in templates.iter().enumerate() {
        let jsns_here: Vec<(usize, u64)> = record_jsns
            .range((publish_position, 0)..(publish_position + 1, 0))
            .map(|((_, operation_position), jsn)| (*operation_position, *jsn))
            .collect();
        let mut jsns_ascending: Vec<u64> = jsns_here.iter().map(|(_, jsn)| *jsn).collect();
        jsns_ascending.sort_unstable();
        let planned_at = |operation_position: usize| match template.operations[operation_position] {
            TemplateOperation::Record { planned, .. } => planned,
            TemplateOperation::Fixed(_) => panic!("这一位置是记录"),
        };
        let end_jsn = jsns_here
            .iter()
            .max_by_key(|(_, jsn)| *jsn)
            .filter(|(operation_position, _)| !planned_at(*operation_position).successor_planned_at_issue)
            .map(|(_, jsn)| *jsn);
        for (operation_position, jsn) in &jsns_here {
            let planned = planned_at(*operation_position);
            let ordinal = 1 + jsns_ascending.iter().filter(|smaller| *smaller < jsn).count();
            record_contents.insert(
                (publish_position, *operation_position),
                RecordContent {
                    jsn: *jsn,
                    txg: template.txg,
                    publish: template.identity,
                    transaction: planned.transaction,
                    is_commit: planned.is_commit,
                    ends_its_publish: Some(*jsn) == end_jsn,
                    ordinal_within_publish: u32::try_from(ordinal).expect("一次发布至多 4 条记录"),
                    origin: planned.origin,
                },
            );
        }
    }
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
                Some(VoidTiming::WideAfterFirstRecord) => panic!("多飞不跑作废·宽（第二段 H2 够判后未跑）"),
                None => None,
            },
        })
        .collect();
    let mut operations: Vec<Operation> = Vec::with_capacity(merge.len());
    let mut operation_cursors = vec![0usize; templates.len()];
    for publish_position in merge {
        let template = &templates[*publish_position];
        let operation_position = operation_cursors[*publish_position];
        operation_cursors[*publish_position] += 1;
        let operation_index = operations.len();
        match template.operations[operation_position] {
            TemplateOperation::Fixed(operation) => operations.push(operation),
            TemplateOperation::Record { planned, .. } => {
                operations.push(Operation::Write {
                    content: WriteContent::JournalRecord(record_contents[&(*publish_position, operation_position)]),
                    states: WriteDurabilityStates::NewOrMissing,
                });
                if planned.transaction != 0 {
                    timelines[transaction_position(planned.transaction)].issued_record_operation_indices.push(operation_index);
                }
            }
        }
        if operation_position == template.final_barrier_position {
            for transaction in &template.unit_transactions {
                if layout.transactions[transaction_position(*transaction)].voided.is_none() {
                    timelines[transaction_position(*transaction)].final_barrier_operation_index = Some(operation_index);
                }
            }
        }
    }
    Execution { operations, timelines }
}

/// 多飞 k = 2 的直接枚举（登记 5.6 第二段「k = 2 直接枚举执行」）：每种发布内记录次序 × 发布之间全部允许的交错。
fn multi_flight_executions(layout: &CellLayout, rules: MultiFlightRules) -> Vec<Execution> {
    let orders = allowed_record_orders(layout, rules.append_arm, rules.discipline);
    let mut executions = Vec::new();
    for choice in every_order_choice(&orders) {
        let templates = multi_flight_templates(layout, &orders, &choice);
        for merge in every_allowed_merge(&templates, rules) {
            executions.push(execution_from_merge(layout, &templates, &merge));
        }
    }
    executions
}

/// 按配置去重的搜索（登记 5.6 第二段「k = 3 按配置去重」）。一份配置 = 各发布已发出到哪、最后一道完成的闭段点那一刻各发布发出到哪、
/// 有没有一个已发出没完成的闭段点、已发出记录的 jsn。同一份配置往后的崩溃状态与可走的下一步都相同，所以全部执行的崩溃状态并集 = 走到过的每份配置的崩溃状态并集。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct SearchConfiguration {
    issued: [u8; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
    completed: [u8; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
    closer_pending: bool,
    /// 每次发布第几条记录拿到的 jsn（0 = 还没发出）。
    record_jsns: [[u8; MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION]; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
    next_issue_jsn: u8,
}

#[derive(Clone, Debug)]
struct SearchSummary {
    paths_to_the_end: u64,
    /// 每个事务从这份配置走到底，「前面尚未追加且未作废」的事务数为正的闭段点最多几个。
    positive_closers_to_the_end: Vec<u64>,
}

/// 一个事务在多飞模板里的位置：自己的发布、自己的记录在那次发布里的位置。
#[derive(Clone, Debug)]
struct TransactionPlacement {
    pub(super) number: u32,
    is_voided: bool,
    pub(super) publish_position: Option<usize>,
    record_positions: Vec<usize>,
}

pub(super) struct ConfigurationSearch<'search> {
    templates: &'search [PublishTemplate],
    placements: Vec<TransactionPlacement>,
    rules: MultiFlightRules,
    pub(super) reading: AppendedReading,
    pub(super) return_rule: ReturnRule,
    pub(super) states: HashSet<CrashState>,
    memo: HashMap<SearchConfiguration, SearchSummary>,
    pub(super) returned_before_last_barrier_points: u64,
    limit_exceeded: bool,
}

impl<'search> ConfigurationSearch<'search> {
    pub(super) fn new(layout: &CellLayout, templates: &'search [PublishTemplate], rules: MultiFlightRules, reading: AppendedReading, return_rule: ReturnRule) -> ConfigurationSearch<'search> {
        assert!(templates.len() <= MAXIMUM_PUBLISHES_IN_A_CONFIGURATION, "一份配置装得下的发布数");
        assert!(
            templates.iter().all(|template| template.operations.iter().filter(|operation| matches!(operation, TemplateOperation::Record { .. })).count() <= MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION),
            "一份配置装得下的记录数"
        );
        let placements = layout
            .transactions
            .iter()
            .map(|transaction| {
                let publish_position = templates.iter().position(|template| template.own_transactions().contains(&transaction.number));
                TransactionPlacement {
                    number: transaction.number,
                    is_voided: match transaction.voided {
                        Some(VoidTiming::NarrowBeforeAnyWrite) => true,
                        Some(VoidTiming::WideAfterFirstRecord) => panic!("多飞不跑作废·宽（第二段 H2 够判后未跑）"),
                        None => false,
                    },
                    publish_position,
                    record_positions: publish_position.map_or(Vec::new(), |position| templates[position].own_record_positions_of(transaction.number)),
                }
            })
            .collect();
        ConfigurationSearch {
            templates,
            placements,
            rules,
            reading,
            return_rule,
            states: HashSet::new(),
            memo: HashMap::new(),
            returned_before_last_barrier_points: 0,
            limit_exceeded: false,
        }
    }

    pub(super) fn start() -> SearchConfiguration {
        SearchConfiguration {
            issued: [0; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
            completed: [0; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
            closer_pending: false,
            record_jsns: [[0; MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION]; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
            next_issue_jsn: 1,
        }
    }

    fn finished(&self) -> SearchConfiguration {
        let mut configuration = ConfigurationSearch::start();
        for (publish_position, template) in self.templates.iter().enumerate() {
            let length = u8::try_from(template.operations.len()).expect("一次发布的操作至多十几个");
            configuration.issued[publish_position] = length;
            configuration.completed[publish_position] = length;
        }
        configuration
    }

    fn content_at(&self, configuration: &SearchConfiguration, publish_position: usize, operation_position: usize) -> Option<(WriteContent, WriteDurabilityStates)> {
        let template = &self.templates[publish_position];
        match template.operations[operation_position] {
            TemplateOperation::Fixed(operation) => operation.durability_choices(),
            TemplateOperation::Record { planned, ordinal_within_publish, ends_its_publish } => {
                let record_index = template.record_index_at(operation_position).expect("这一位置是记录");
                let jsn = configuration.record_jsns[publish_position][record_index];
                assert!(jsn > 0, "已发出的记录都拿到了 jsn");
                Some((
                    WriteContent::JournalRecord(RecordContent {
                        jsn: u64::from(jsn),
                        txg: template.txg,
                        publish: template.identity,
                        transaction: planned.transaction,
                        is_commit: planned.is_commit,
                        ends_its_publish,
                        ordinal_within_publish,
                        origin: planned.origin,
                    }),
                    WriteDurabilityStates::NewOrMissing,
                ))
            }
        }
    }

    fn is_appended(&self, placement: &TransactionPlacement, configuration: &SearchConfiguration) -> bool {
        let publish_position = placement.publish_position.expect("没作废的事务有自己的发布");
        let last_record = *placement.record_positions.last().expect("没作废的事务至少一条记录");
        match self.reading {
            AppendedReading::IssuedOnly => usize::from(configuration.issued[publish_position]) > last_record,
            AppendedReading::IssuedAndCovered => usize::from(configuration.completed[publish_position]) > last_record,
        }
    }

    /// 登记第一节 1.2「fsync 返回」在一份配置上：① 本事务所在发布的 B3 已完成；② 每个 j < i 已追加或已作废。
    pub(super) fn returned(&self, configuration: &SearchConfiguration) -> Vec<(u32, ReturnBranch)> {
        let mut returned = Vec::new();
        for placement in self.placements.iter().filter(|placement| !placement.is_voided) {
            let publish_position = placement.publish_position.expect("没作废的事务有自己的发布");
            let own_condition = match self.return_rule {
                ReturnRule::AfterOwnPublishLastBarrier => usize::from(configuration.completed[publish_position]) > self.templates[publish_position].final_barrier_position,
                ReturnRule::EarlyAfterOwnRecordsIssued => usize::from(configuration.issued[publish_position]) > *placement.record_positions.last().expect("至少一条记录"),
            };
            if !own_condition {
                continue;
            }
            let mut passed_as_voided = false;
            let mut blocked = false;
            for earlier in self.placements.iter().filter(|candidate| candidate.number < placement.number) {
                if earlier.is_voided {
                    passed_as_voided = true;
                    continue;
                }
                if !self.is_appended(earlier, configuration) {
                    blocked = true;
                    break;
                }
            }
            if !blocked {
                returned.push((placement.number, if passed_as_voided { ReturnBranch::SomeEarlierPassedAsVoided } else { ReturnBranch::EveryEarlierAppended }));
            }
        }
        returned
    }

    /// 登记 8.1 第一行：每个没作废的事务前面「尚未追加且未作废」的事务数（作废的事务记 0、不进轨迹）。
    pub(super) fn pending(&self, configuration: &SearchConfiguration) -> Vec<Option<u64>> {
        self.placements
            .iter()
            .map(|placement| {
                if placement.is_voided {
                    return None;
                }
                let count = self
                    .placements
                    .iter()
                    .filter(|earlier| earlier.number < placement.number && !earlier.is_voided && !self.is_appended(earlier, configuration))
                    .count();
                Some(u64::try_from(count).expect("k ≤ 3"))
            })
            .collect()
    }

    fn insert_states_of(&mut self, configuration: &SearchConfiguration) {
        let mut base: Vec<(WriteContent, DurableForm)> = Vec::new();
        let mut segment_writes: Vec<(WriteContent, WriteDurabilityStates)> = Vec::new();
        for publish_position in 0..self.templates.len() {
            for operation_position in 0..usize::from(configuration.completed[publish_position]) {
                if let Some((content, _)) = self.content_at(configuration, publish_position, operation_position) {
                    base.push((content, DurableForm::New));
                }
            }
            for operation_position in usize::from(configuration.completed[publish_position])..usize::from(configuration.issued[publish_position]) {
                if let Some(choice) = self.content_at(configuration, publish_position, operation_position) {
                    segment_writes.push(choice);
                }
            }
        }
        let returned = self.returned(configuration);
        for (number, _) in &returned {
            let placement = &self.placements[transaction_position(*number)];
            let publish_position = placement.publish_position.expect("返回的事务有自己的发布");
            if usize::from(configuration.completed[publish_position]) <= self.templates[publish_position].final_barrier_position {
                self.returned_before_last_barrier_points += 1;
            }
        }
        insert_every_durability_choice(&base, &segment_writes, &returned, &mut self.states);
    }

    /// 下一步：有已发出没完成的闭段点时只能等它完成；否则任一发布发出它的下一个操作（照多飞的次序约束）。
    fn successors(&self, configuration: &SearchConfiguration) -> Vec<(SearchConfiguration, bool)> {
        if configuration.closer_pending {
            let mut next = *configuration;
            next.completed = next.issued;
            next.closer_pending = false;
            return vec![(next, true)];
        }
        let issued_counts: Vec<usize> = (0..self.templates.len()).map(|publish_position| usize::from(configuration.issued[publish_position])).collect();
        let mut successors = Vec::new();
        for (publish_position, template) in self.templates.iter().enumerate() {
            let next_position = issued_counts[publish_position];
            if next_position >= template.operations.len() || !next_operation_may_issue(self.templates, &issued_counts, publish_position, self.rules) {
                continue;
            }
            let mut next = *configuration;
            next.issued[publish_position] += 1;
            match template.operations[next_position] {
                TemplateOperation::Record { planned, .. } => {
                    let jsn = match planned.reserved_jsn {
                        Some(reserved) => u8::try_from(reserved).expect("jsn 至多 7"),
                        None => {
                            let issue_jsn = next.next_issue_jsn;
                            next.next_issue_jsn += 1;
                            issue_jsn
                        }
                    };
                    next.record_jsns[publish_position][template.record_index_at(next_position).expect("这一位置是记录")] = jsn;
                }
                TemplateOperation::Fixed(operation) => {
                    next.closer_pending = operation.closes_a_segment();
                }
            }
            successors.push((next, false));
        }
        successors
    }

    fn visit(&mut self, configuration: SearchConfiguration) -> SearchSummary {
        if let Some(summary) = self.memo.get(&configuration) {
            return summary.clone();
        }
        let transaction_count = self.placements.len();
        if u64::try_from(self.memo.len()).expect("配置数有限") >= CONFIGURATION_LIMIT {
            self.limit_exceeded = true;
            return SearchSummary { paths_to_the_end: 0, positive_closers_to_the_end: vec![0; transaction_count] };
        }
        self.insert_states_of(&configuration);
        let pending_here = self.pending(&configuration);
        let successors = self.successors(&configuration);
        let mut summary = SearchSummary { paths_to_the_end: 0, positive_closers_to_the_end: vec![0; transaction_count] };
        if successors.is_empty() {
            let finished = self.finished();
            assert!(
                !configuration.closer_pending && configuration.issued == finished.issued && configuration.completed == finished.completed,
                "走不动的配置只能是全部发出且完成（次序约束不会卡死）"
            );
            summary.paths_to_the_end = 1;
        }
        for (next, completes_a_closer) in successors {
            let pending_next = self.pending(&next);
            for (after, before) in pending_next.iter().zip(&pending_here) {
                assert!(after <= before, "「前面尚未追加且未作废」的事务数沿每一步只减不增");
            }
            let next_summary = self.visit(next);
            summary.paths_to_the_end += next_summary.paths_to_the_end;
            for (transaction_index, pending_of_transaction) in pending_next.iter().enumerate().take(transaction_count) {
                let gained = u64::from(completes_a_closer && pending_of_transaction.is_some_and(|value| value > 0));
                summary.positive_closers_to_the_end[transaction_index] = summary.positive_closers_to_the_end[transaction_index].max(next_summary.positive_closers_to_the_end[transaction_index] + gained);
            }
        }
        self.memo.insert(configuration, summary.clone());
        summary
    }
}

/// 一格多飞的崩溃状态集合与它附带的计数。
pub(super) struct EnumeratedCrashStates {
    pub(super) states: HashSet<CrashState>,
    pub(super) executions: u64,
    pub(super) configurations: Option<u64>,
    pub(super) returned_before_last_barrier_points: u64,
    pub(super) pending: PendingTrajectory,
    pub(super) configuration_limit_exceeded: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum MultiFlightEnumeration {
    DirectExecutions,
    ByConfiguration,
}

impl MultiFlightEnumeration {
    pub(super) fn label(self) -> &'static str {
        match self {
            MultiFlightEnumeration::DirectExecutions => "direct",
            MultiFlightEnumeration::ByConfiguration => "by_configuration",
        }
    }
    /// 登记 5.6 第二段：k = 2 直接枚举执行，k = 3 按配置去重。
    pub(super) fn registered_for(history: &History) -> MultiFlightEnumeration {
        match history.transaction_count() {
            2 => MultiFlightEnumeration::DirectExecutions,
            3 => MultiFlightEnumeration::ByConfiguration,
            other => panic!("第二段只跑 k = 2、3，这里是 {other}"),
        }
    }
}

pub(super) fn enumerate_multi_flight(request: &CellRequest, layout: &CellLayout, enumeration: MultiFlightEnumeration) -> EnumeratedCrashStates {
    assert!(request.barriers == PublishBarriers::AllThree, "多飞不跑拿掉屏障的对照");
    let rules = MultiFlightRules::of(request.condition, request.append_arm, request.discipline);
    match enumeration {
        MultiFlightEnumeration::DirectExecutions => {
            let executions = multi_flight_executions(layout, rules);
            let mut states = HashSet::new();
            let mut returned_before_last_barrier_points = 0u64;
            let mut pending = PendingTrajectory::default();
            for execution in &executions {
                returned_before_last_barrier_points += collect_crash_states(execution, request.reading, request.return_rule, &mut states);
                pending.absorb(&pending_trajectory_of(execution, request.reading));
            }
            EnumeratedCrashStates {
                states,
                executions: u64::try_from(executions.len()).expect("执行数有限"),
                configurations: None,
                returned_before_last_barrier_points,
                pending,
                configuration_limit_exceeded: false,
            }
        }
        MultiFlightEnumeration::ByConfiguration => {
            let orders = allowed_record_orders(layout, request.append_arm, request.discipline);
            let mut enumerated = EnumeratedCrashStates {
                states: HashSet::new(),
                executions: 0,
                configurations: Some(0),
                returned_before_last_barrier_points: 0,
                pending: PendingTrajectory::default(),
                configuration_limit_exceeded: false,
            };
            for choice in every_order_choice(&orders) {
                let templates = multi_flight_templates(layout, &orders, &choice);
                let mut search = ConfigurationSearch::new(layout, &templates, rules, request.reading, request.return_rule);
                let start = ConfigurationSearch::start();
                let summary = search.visit(start);
                let pending_at_start = search.pending(&start);
                let pending_at_finish = search.pending(&search.finished());
                let mut trajectory = PendingTrajectory::default();
                for (transaction_index, at_start) in pending_at_start.iter().enumerate() {
                    let (Some(initial), Some(final_value)) = (at_start, pending_at_finish[transaction_index]) else {
                        continue;
                    };
                    // 沿每一步只减不增（`visit` 里逐步断言），所以第一次为 0 之后一直是 0；期末不为 0 就是从没清空过，每条路径都算一次。
                    trajectory.absorb(&PendingTrajectory {
                        initial_peak: *initial,
                        peak_after_drain: 0,
                        positive_closers_peak: summary.positive_closers_to_the_end[transaction_index],
                        final_peak: final_value,
                        never_drained: if final_value > 0 { summary.paths_to_the_end } else { 0 },
                    });
                }
                enumerated.states.extend(search.states.drain());
                enumerated.executions += summary.paths_to_the_end;
                enumerated.configurations = enumerated.configurations.map(|count| count + u64::try_from(search.memo.len()).expect("配置数有限"));
                enumerated.returned_before_last_barrier_points += search.returned_before_last_barrier_points;
                enumerated.pending.absorb(&trajectory);
                enumerated.configuration_limit_exceeded |= search.limit_exceeded;
            }
            enumerated
        }
    }
}

// ===== 恢复：丙0 逐判转写与三个候选 =====

pub(super) struct StateView {
    pub(super) records_by_jsn: Vec<RecordContent>,
    pub(super) durable_new: BTreeSet<WriteContent>,
    pub(super) chosen_root: Option<(u32, u64)>,
    pub(super) returned: Vec<(u32, ReturnBranch)>,
}
