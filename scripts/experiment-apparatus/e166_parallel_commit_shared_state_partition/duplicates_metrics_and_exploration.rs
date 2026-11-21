//! 资源的重复数、状态度量与汇总、调度的枚举与记忆化探索、整轮模拟与串行跑。

use super::*;

// ── 重复数：一个调度跑完之后，每个被发出的项数「最终持有它的已提交事务个数」，记 持有数 − 1 再求和 ──────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Resource {
    UserDataSlot,
    CommitGeneratedSlot,
    InodeNumber,
    TreeIdentifier,
    TransactionNumber,
    JsnCounter,
    JournalRingSlot,
    CheckpointTxg,
    RootRingSlot,
}

pub(super) const RESOURCE_COUNT: usize = 9;
pub(super) const ALL_RESOURCES: [Resource; RESOURCE_COUNT] = [
    Resource::UserDataSlot,
    Resource::CommitGeneratedSlot,
    Resource::InodeNumber,
    Resource::TreeIdentifier,
    Resource::TransactionNumber,
    Resource::JsnCounter,
    Resource::JournalRingSlot,
    Resource::CheckpointTxg,
    Resource::RootRingSlot,
];

impl Resource {
    pub(super) fn label(self) -> &'static str {
        match self {
            Resource::UserDataSlot => "U",
            Resource::CommitGeneratedSlot => "G",
            Resource::InodeNumber => "I",
            Resource::TreeIdentifier => "T",
            Resource::TransactionNumber => "X",
            Resource::JsnCounter => "J",
            Resource::JournalRingSlot => "JS",
            Resource::CheckpointTxg => "TXG",
            Resource::RootRingSlot => "RS",
        }
    }

    pub(super) fn index(self) -> usize {
        ALL_RESOURCES
            .iter()
            .position(|candidate| *candidate == self)
            .expect("每个资源都在 ALL_RESOURCES 里")
    }
}

pub(super) fn push_held_keys(model: &Model, resource: Resource, held: &Holdings, keys: &mut Vec<u64>) {
    match resource {
        Resource::UserDataSlot => {
            if model.user_pairs_counted_per_device() {
                for device in 0..model.device_count {
                    let pair = held.user_pairs[device];
                    if pair != NO_PAIR {
                        keys.push((u64::try_from(device).expect("盘号很小") << 8) | u64::from(pair));
                    }
                }
            } else if held.user_pairs[0] != NO_PAIR {
                keys.push(u64::from(held.user_pairs[0]));
            }
        }
        Resource::CommitGeneratedSlot => {
            if held.commit_slot_start != NO_SLOT {
                for node in 0..COMMIT_GENERATED_NODES_PER_PUBLISH {
                    keys.push(u64::from(held.commit_slot_start + node));
                }
            }
        }
        Resource::InodeNumber => {
            for number in held.inode_start..held.inode_start + held.inode_count {
                keys.push(u64::from(number));
            }
            for number in held.extra_inode_start..held.extra_inode_start + held.extra_inode_count {
                keys.push(u64::from(number));
            }
        }
        Resource::TreeIdentifier => {
            if held.tree_start != 0 {
                for offset in 0..TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION {
                    keys.push(u64::from(held.tree_start + offset));
                }
            }
        }
        Resource::TransactionNumber => {
            if held.transaction_number != 0 {
                keys.push(u64::from(held.transaction_number));
            }
        }
        Resource::JsnCounter => {
            if held.counter != 0 {
                keys.push(u64::from(held.counter));
            }
        }
        Resource::JournalRingSlot => {
            if held.counter != 0 {
                keys.push(u64::from(journal_ring_slot(held.counter, model.journal_ring_slots)));
            }
        }
        Resource::CheckpointTxg => {
            if held.txg != 0 {
                keys.push(u64::from(held.txg));
            }
        }
        Resource::RootRingSlot => {
            if held.txg != 0 {
                keys.push(root_ring_slot_key(held.txg, model.root_slots_per_region));
            }
        }
    }
}

/// 一批项里的重复数：每一项的持有数 − 1 求和。
pub(super) fn duplicates_among(keys: &mut [u64]) -> u32 {
    keys.sort_unstable();
    let mut duplicates = 0u32;
    let mut run_start = 0;
    while run_start < keys.len() {
        let mut run_end = run_start + 1;
        while run_end < keys.len() && keys[run_end] == keys[run_start] {
            run_end += 1;
        }
        let holders = run_end - run_start;
        duplicates += u32::try_from(holders - 1).expect("持有数很小");
        run_start = run_end;
    }
    duplicates
}

fn highest_inode_number_held(held: &Holdings) -> Option<u8> {
    let first_range = (held.inode_count > 0).then(|| held.inode_start + held.inode_count - 1);
    let second_range = (held.extra_inode_count > 0).then(|| held.extra_inode_start + held.extra_inode_count - 1);
    first_range.into_iter().chain(second_range).max()
}

/// 一个状态上谓词消费的几个量（登记第八节「轨迹」）。
pub(super) struct StateMetrics {
    /// 已提交事务之间各资源的重复数。
    pub(super) duplicates: [u32; RESOURCE_COUNT],
    /// inode 号水位余量 = 共享水位 − 已提交事务持有的最大 inode 号 − 1（I-9.6）。
    inode_margin: i32,
    busy_regions: u32,
    unused_reserved_inodes: u32,
}

pub(super) fn state_metrics(model: &Model, state: &ModelState) -> StateMetrics {
    let committed: Vec<&TransactionState> = state.transactions[..model.transaction_count]
        .iter()
        .filter(|transaction| transaction.committed)
        .collect();
    let mut duplicates = [0u32; RESOURCE_COUNT];
    let mut keys: Vec<u64> = Vec::with_capacity(64);
    if !committed.is_empty() {
        for resource in ALL_RESOURCES {
            keys.clear();
            for transaction in &committed {
                push_held_keys(model, resource, &transaction.items, &mut keys);
            }
            duplicates[resource.index()] = duplicates_among(&mut keys);
        }
    }
    let highest_held = committed
        .iter()
        .filter_map(|transaction| highest_inode_number_held(&transaction.items))
        .max()
        .unwrap_or(INITIAL_INODE_WATERMARK - 1);
    StateMetrics {
        duplicates,
        inode_margin: i32::from(state.shared.inode_watermark) - i32::from(highest_held) - 1,
        busy_regions: state.shared.region_busy.count_ones(),
        unused_reserved_inodes: state.transactions[..model.transaction_count]
            .iter()
            .map(|transaction| u32::from(transaction.inode_holes))
            .sum(),
    }
}

/// 全部调度（或一个子树）上的汇总：每个量对每个叶（完整调度）算一次，再求和 / 取最大；
/// 谓词量（轨迹）按「每个调度每一步之后」算，求峰值与为正的步数。
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct Aggregate {
    pub(super) schedules: u128,
    pub(super) duplicate_total: [u128; RESOURCE_COUNT],
    pub(super) duplicate_schedules: [u128; RESOURCE_COUNT],
    pub(super) duplicate_maximum_single_schedule: [u32; RESOURCE_COUNT],
    pub(super) retries_total: u128,
    pub(super) retries_schedules: u128,
    pub(super) retries_maximum_schedule: u32,
    pub(super) retries_maximum_transaction: u32,
    pub(super) fallbacks_total: u128,
    pub(super) truncated: u128,
    pub(super) deadlocks: u128,
    pub(super) duplicate_peak: [u32; RESOURCE_COUNT],
    pub(super) duplicate_positive_steps: [u128; RESOURCE_COUNT],
    pub(super) inode_margin_minimum: i32,
    pub(super) inode_margin_negative_steps: u128,
    pub(super) inode_margin_final_minimum: i32,
    pub(super) busy_regions_peak: u32,
    pub(super) busy_regions_positive_steps: u128,
    pub(super) busy_regions_final_maximum: u32,
    pub(super) unused_reserved_peak: u32,
    pub(super) unused_reserved_positive_steps: u128,
    pub(super) unused_reserved_final_maximum: u32,
}

impl Aggregate {
    fn empty() -> Aggregate {
        Aggregate {
            schedules: 0,
            duplicate_total: [0; RESOURCE_COUNT],
            duplicate_schedules: [0; RESOURCE_COUNT],
            duplicate_maximum_single_schedule: [0; RESOURCE_COUNT],
            retries_total: 0,
            retries_schedules: 0,
            retries_maximum_schedule: 0,
            retries_maximum_transaction: 0,
            fallbacks_total: 0,
            truncated: 0,
            deadlocks: 0,
            duplicate_peak: [0; RESOURCE_COUNT],
            duplicate_positive_steps: [0; RESOURCE_COUNT],
            inode_margin_minimum: i32::MAX,
            inode_margin_negative_steps: 0,
            inode_margin_final_minimum: i32::MAX,
            busy_regions_peak: 0,
            busy_regions_positive_steps: 0,
            busy_regions_final_maximum: 0,
            unused_reserved_peak: 0,
            unused_reserved_positive_steps: 0,
            unused_reserved_final_maximum: 0,
        }
    }

    fn merge(&mut self, other: &Aggregate) {
        self.schedules += other.schedules;
        self.retries_total += other.retries_total;
        self.retries_schedules += other.retries_schedules;
        self.retries_maximum_schedule = self.retries_maximum_schedule.max(other.retries_maximum_schedule);
        self.retries_maximum_transaction = self.retries_maximum_transaction.max(other.retries_maximum_transaction);
        self.fallbacks_total += other.fallbacks_total;
        self.truncated += other.truncated;
        self.deadlocks += other.deadlocks;
        for resource in 0..RESOURCE_COUNT {
            self.duplicate_total[resource] += other.duplicate_total[resource];
            self.duplicate_schedules[resource] += other.duplicate_schedules[resource];
            self.duplicate_maximum_single_schedule[resource] =
                self.duplicate_maximum_single_schedule[resource].max(other.duplicate_maximum_single_schedule[resource]);
            self.duplicate_peak[resource] = self.duplicate_peak[resource].max(other.duplicate_peak[resource]);
            self.duplicate_positive_steps[resource] += other.duplicate_positive_steps[resource];
        }
        self.inode_margin_minimum = self.inode_margin_minimum.min(other.inode_margin_minimum);
        self.inode_margin_negative_steps += other.inode_margin_negative_steps;
        self.inode_margin_final_minimum = self.inode_margin_final_minimum.min(other.inode_margin_final_minimum);
        self.busy_regions_peak = self.busy_regions_peak.max(other.busy_regions_peak);
        self.busy_regions_positive_steps += other.busy_regions_positive_steps;
        self.busy_regions_final_maximum = self.busy_regions_final_maximum.max(other.busy_regions_final_maximum);
        self.unused_reserved_peak = self.unused_reserved_peak.max(other.unused_reserved_peak);
        self.unused_reserved_positive_steps += other.unused_reserved_positive_steps;
        self.unused_reserved_final_maximum = self.unused_reserved_final_maximum.max(other.unused_reserved_final_maximum);
    }

    /// 一个完整调度（叶）：期末各量。
    fn add_leaf(&mut self, metrics: &StateMetrics, retries_of_each_transaction: &[u8], fallbacks: u32) {
        self.schedules += 1;
        for resource in 0..RESOURCE_COUNT {
            let duplicates = metrics.duplicates[resource];
            self.duplicate_total[resource] += u128::from(duplicates);
            if duplicates > 0 {
                self.duplicate_schedules[resource] += 1;
            }
            self.duplicate_maximum_single_schedule[resource] = self.duplicate_maximum_single_schedule[resource].max(duplicates);
        }
        let retries_of_schedule: u32 = retries_of_each_transaction.iter().map(|retries| u32::from(*retries)).sum();
        self.retries_total += u128::from(retries_of_schedule);
        if retries_of_schedule > 0 {
            self.retries_schedules += 1;
        }
        self.retries_maximum_schedule = self.retries_maximum_schedule.max(retries_of_schedule);
        let most_retries_of_one_transaction =
            retries_of_each_transaction.iter().copied().max().map_or(0, u32::from);
        self.retries_maximum_transaction = self.retries_maximum_transaction.max(most_retries_of_one_transaction);
        self.fallbacks_total += u128::from(fallbacks);
        self.inode_margin_final_minimum = self.inode_margin_final_minimum.min(metrics.inode_margin);
        self.busy_regions_final_maximum = self.busy_regions_final_maximum.max(metrics.busy_regions);
        self.unused_reserved_final_maximum = self.unused_reserved_final_maximum.max(metrics.unused_reserved_inodes);
    }

    /// 一个状态（某个调度某一步之后）上谓词消费的量，对这个状态之下的全部 `leaves_below` 个调度各算一步。
    fn add_state_metrics(&mut self, metrics: &StateMetrics, leaves_below: u128) {
        for resource in 0..RESOURCE_COUNT {
            self.duplicate_peak[resource] = self.duplicate_peak[resource].max(metrics.duplicates[resource]);
            if metrics.duplicates[resource] > 0 {
                self.duplicate_positive_steps[resource] += leaves_below;
            }
        }
        self.inode_margin_minimum = self.inode_margin_minimum.min(metrics.inode_margin);
        if metrics.inode_margin < 0 {
            self.inode_margin_negative_steps += leaves_below;
        }
        self.busy_regions_peak = self.busy_regions_peak.max(metrics.busy_regions);
        if metrics.busy_regions > 0 {
            self.busy_regions_positive_steps += leaves_below;
        }
        self.unused_reserved_peak = self.unused_reserved_peak.max(metrics.unused_reserved_inodes);
        if metrics.unused_reserved_inodes > 0 {
            self.unused_reserved_positive_steps += leaves_below;
        }
    }
}

// ── 调度枚举：逐叶 + 按状态记忆化两种 ───────────────────────────────────────────

/// 记忆化状态数的上限。第一段全部格里最大的一格（P设去锁、D = 2、n = 3）实测 1 330 186 个状态、进程峰值 2.5 GiB；
/// 上限取它的 3 倍，约 7.7 GiB，低于变异跑道的 10G 上限。撞上限说明状态空间变了一个量级（例：变异 V10 让标占一直不撤），
/// 当场断言失败，不让进程无界吃内存。
const MEMO_STATE_LIMIT: usize = 4_000_000;

pub(super) struct Explorer<'model> {
    model: &'model Model,
    memo: Option<HashMap<ModelState, Aggregate>>,
}

impl<'model> Explorer<'model> {
    pub(super) fn new(model: &'model Model, use_memo: bool) -> Explorer<'model> {
        Explorer { model, memo: if use_memo { Some(HashMap::new()) } else { None } }
    }

    fn explore(&mut self, state: &ModelState) -> Aggregate {
        let model = self.model;
        // 记忆化的键 = 整个状态：各事务已重算次数也在里面（截断判据要它）。
        let memo_key = *state;
        if let Some(table) = &self.memo {
            if let Some(found) = table.get(&memo_key) {
                return found.clone();
            }
        }
        let mut aggregate = Aggregate::empty();
        let live: Vec<usize> = (0..model.transaction_count)
            .filter(|transaction_index| !state.transactions[*transaction_index].committed)
            .collect();
        if live.is_empty() {
            let metrics = state_metrics(model, state);
            let retries: Vec<u8> = state.transactions[..model.transaction_count]
                .iter()
                .map(|transaction| transaction.retries)
                .collect();
            let fallbacks: u32 = state.transactions[..model.transaction_count]
                .iter()
                .map(|transaction| u32::from(transaction.fallbacks))
                .sum();
            aggregate.add_leaf(&metrics, &retries, fallbacks);
            aggregate.add_state_metrics(&metrics, 1);
        } else {
            let enabled_indices: Vec<usize> = live
                .into_iter()
                .filter(|transaction_index| is_step_enabled(model, state, *transaction_index))
                .collect();
            if enabled_indices.is_empty() {
                aggregate.deadlocks += 1;
            }
            for transaction_index in enabled_indices {
                let mut next = *state;
                match apply_step(model, &mut next, transaction_index) {
                    StepResult::Applied => {
                        let child = self.explore(&next);
                        aggregate.merge(&child);
                    }
                    StepResult::Truncated => aggregate.truncated += 1,
                }
            }
            if aggregate.schedules > 0 {
                let metrics = state_metrics(model, state);
                aggregate.add_state_metrics(&metrics, aggregate.schedules);
            }
        }
        if let Some(table) = &mut self.memo {
            assert!(
                table.len() < MEMO_STATE_LIMIT,
                "记忆化的状态数超过上限 {MEMO_STATE_LIMIT}：模型的状态空间比登记预期的大一个量级，不再往下吃内存"
            );
            table.insert(memo_key, aggregate.clone());
        }
        aggregate
    }
}

pub(super) struct Enumeration {
    pub(super) aggregate: Aggregate,
}

pub(super) fn enumerate_schedules(model: &Model, use_memo: bool) -> Enumeration {
    let mut explorer = Explorer::new(model, use_memo);
    let aggregate = explorer.explore(&initial_state(model));
    Enumeration { aggregate}
}

// ── Q-9：同步轮次 ────────────────────────────────────────────────────────────

fn round_order(transaction_count: usize) -> Vec<usize> {
    (0..transaction_count).collect()
}

const ROUND_LIMIT: usize = 64;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct RoundsOutcome {
    /// 全部提交用了几轮；死锁或截断时没有。
    pub(super) rounds: Option<u32>,
    pub(super) commits_per_round: Vec<u32>,
}

/// 登记第六节 Q-9：全部 n 个事务第 1 轮同时开始；每一轮里按步序号从小到大逐个相位，相位内按 `round_order` 的次序
/// 让每个停在这一相位且使能的事务走一步；失败回到前面的事务下一轮才再走。
pub(super) fn simulate_rounds(model: &Model) -> RoundsOutcome {
    simulate_rounds_in_order(model, round_order)
}

pub(super) fn simulate_rounds_in_order(model: &Model, order_of_transactions: fn(usize) -> Vec<usize>) -> RoundsOutcome {
    let mut state = initial_state(model);
    let mut commits_per_round: Vec<u32> = Vec::new();
    loop {
        let committed_before = state.transactions[..model.transaction_count]
            .iter()
            .filter(|transaction| transaction.committed)
            .count();
        if committed_before == model.transaction_count {
            return RoundsOutcome {
                rounds: Some(u32::try_from(commits_per_round.len()).expect("轮数很小")),
                commits_per_round,
            };
        }
        for phase in 0..model.steps.len() {
            for transaction_index in order_of_transactions(model.transaction_count) {
                let transaction = &state.transactions[transaction_index];
                if !transaction.committed
                    && usize::from(transaction.program_counter) == phase
                    && is_step_enabled(model, &state, transaction_index)
                    && apply_step(model, &mut state, transaction_index) == StepResult::Truncated
                {
                    return RoundsOutcome { rounds: None, commits_per_round };
                }
            }
        }
        let committed_after = state.transactions[..model.transaction_count]
            .iter()
            .filter(|transaction| transaction.committed)
            .count();
        if committed_after == committed_before || commits_per_round.len() >= ROUND_LIMIT {
            return RoundsOutcome { rounds: None, commits_per_round };
        }
        commits_per_round.push(u32::try_from(committed_after - committed_before).expect("提交数很小"));
    }
}

// ── 串行基线 S（`crates/` 今天）：n 个事务按下标一个接一个跑完 ───────────────────────────

pub(super) fn run_serially(model: &Model) -> ModelState {
    let mut state = initial_state(model);
    for transaction_index in 0..model.transaction_count {
        while !state.transactions[transaction_index].committed {
            assert_eq!(apply_step(model, &mut state, transaction_index), StepResult::Applied);
        }
    }
    state
}

/// Q1乙 的批发布者：全部事务提交之后一步发 G、checkpoint_txg（连同根环槽）与树 ID。
pub(super) fn batch_publisher_holdings(shared_after_transactions: &SharedState) -> Holdings {
    let mut view = *shared_after_transactions;
    let mut held = NOTHING_HELD;
    take_resources(
        &mut view,
        TransactionShape { user_units: 0, new_inodes: 0 },
        GROUP_COMMIT_SLOTS | GROUP_ROOT_RING | GROUP_TREE,
        &mut held,
    );
    held
}
