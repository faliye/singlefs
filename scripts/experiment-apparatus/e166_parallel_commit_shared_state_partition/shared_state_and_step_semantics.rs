//! 共享状态、持有的号与槽、事务状态、模型状态、取资源的各条规则与一步一步的应用。

use super::*;

// ── 状态 ─────────────────────────────────────────────────────────────────────

/// 每个池一份、各事务共用的状态（登记第三节清单里本登记量的那几样；ACC 不建模）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct SharedState {
    /// 同步臂的用户数据空闲表：第 p 位 = 第 p 个对已被占（各盘同步变化，只记一份）。
    user_taken: u16,
    /// P设 的逐盘空闲表。
    device_taken: [u16; MAXIMUM_DEVICES],
    /// 提交内生块池的 bump 游标（非分区臂用；分区臂用 `region_cursor`）。
    commit_cursor: u8,
    region_cursor: [u8; MAXIMUM_REGIONS],
    /// 被在飞事务占着的区（按位）。
    pub(super) region_busy: u8,
    pub(super) inode_watermark: u8,
    pub(super) tree_watermark: TreeIdentifierNumber,
    /// BASE 的「有没有文件」位：视图里没有文件的发布才发十二个树 ID。
    has_file: bool,
    highest_transaction: u8,
    pub(super) counter: u8,
    pub(super) txg: u8,
    /// 提交版本号：O整 整版校验用；O读 在 Q1甲 / Q1丙 下当 BASE 版本号读。
    version: u8,
}

const EMPTY_SHARED_STATE: SharedState = SharedState {
    user_taken: 0,
    device_taken: [0; MAXIMUM_DEVICES],
    commit_cursor: 0,
    region_cursor: [0; MAXIMUM_REGIONS],
    region_busy: 0,
    inode_watermark: 0,
    tree_watermark: 0,
    has_file: false,
    highest_transaction: 0,
    counter: 0,
    txg: 0,
    version: 0,
};

/// 一个事务拿到的号与槽。0（号）或 `NO_PAIR` / `NO_SLOT`（槽）表示没拿。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Holdings {
    pub(super) user_pairs: [u8; MAXIMUM_DEVICES],
    pub(super) commit_slot_start: u8,
    pub(super) inode_start: u8,
    pub(super) inode_count: u8,
    pub(super) extra_inode_start: u8,
    pub(super) extra_inode_count: u8,
    pub(super) tree_start: TreeIdentifierNumber,
    pub(super) transaction_number: u8,
    pub(super) counter: u8,
    pub(super) txg: u8,
}

pub(super) const NOTHING_HELD: Holdings = Holdings {
    user_pairs: [NO_PAIR; MAXIMUM_DEVICES],
    commit_slot_start: NO_SLOT,
    inode_start: 0,
    inode_count: 0,
    extra_inode_start: 0,
    extra_inode_count: 0,
    tree_start: 0,
    transaction_number: 0,
    counter: 0,
    txg: 0,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct TransactionState {
    pub(super) program_counter: u8,
    /// 冲突重算次数：O 臂校验失败回到第一步；P设 的 CHECK 判各盘不一致后回到第一个 DEV 步。
    pub(super) retries: u8,
    pub(super) fallbacks: u8,
    pub(super) committed: bool,
    /// 乐观臂与「去锁」阳性对照里这个事务自己的整份拷贝（或拷贝里属于它那几组的字段）。
    view: SharedState,
    seen_version: u8,
    pub(super) items: Holdings,
    claimed_regions: u8,
    device_read_value: u8,
    pub(super) inode_holes: u8,
    needs_interval_fallback: bool,
}

const FRESH_TRANSACTION: TransactionState = TransactionState {
    program_counter: 0,
    retries: 0,
    fallbacks: 0,
    committed: false,
    view: EMPTY_SHARED_STATE,
    seen_version: 0,
    items: NOTHING_HELD,
    claimed_regions: 0,
    device_read_value: NO_PAIR,
    inode_holes: 0,
    needs_interval_fallback: false,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct ModelState {
    pub(super) shared: SharedState,
    pub(super) transactions: [TransactionState; MAXIMUM_TRANSACTIONS],
}

pub(super) fn initial_state(model: &Model) -> ModelState {
    let mut region_cursor = [0; MAXIMUM_REGIONS];
    for (region, cursor) in region_cursor.iter_mut().enumerate().take(model.region_count) {
        *cursor = model.region_slot_range(region).0;
    }
    let shared = SharedState {
        user_taken: 0,
        device_taken: [0; MAXIMUM_DEVICES],
        commit_cursor: 0,
        region_cursor,
        region_busy: 0,
        inode_watermark: INITIAL_INODE_WATERMARK,
        tree_watermark: if model.initial_has_file {
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH
        } else {
            TREE_IDENTIFIER_WATERMARK_AT_MKFS
        },
        has_file: model.initial_has_file,
        highest_transaction: INITIAL_HIGHEST_TRANSACTION_NUMBER,
        counter: INITIAL_JSN_COUNTER,
        txg: INITIAL_CHECKPOINT_TXG,
        version: 0,
    };
    let mut transactions = [FRESH_TRANSACTION; MAXIMUM_TRANSACTIONS];
    for unused in transactions.iter_mut().skip(model.transaction_count) {
        unused.committed = true;
    }
    ModelState { shared, transactions }
}

// ── 号与槽的发放规则 ─────────────────────────────────────────────────────────

/// 用户数据落点：起点槽号最小、两槽都空的对。`crates/singlefs-core/src/allocator.rs:732`（`lowest_user_data_slot`）。
pub(super) fn lowest_free_pair(taken_pairs: u16, first_pair: usize, end_pair: usize) -> Option<u8> {
    (first_pair..end_pair)
        .find(|pair| taken_pairs & (1 << pair) == 0)
        .map(small)
}

/// 按分组从 `view` 里取本事务要的号与槽，推进 `view`，记进 `held`（只改它那几组对应的字段）。
/// 同一条规则既用在锁内直接取、乐观臂拷贝上取，也用在串行管道上。
pub(super) fn take_resources(view: &mut SharedState, shape: TransactionShape, groups: u8, held: &mut Holdings) {
    if groups & GROUP_USER != 0 && shape.user_units > 0 {
        let pair = lowest_free_pair(view.user_taken, 0, USER_DATA_PAIR_COUNT).expect("16 个对够 n ≤ 4 个事务用");
        view.user_taken |= 1 << pair;
        held.user_pairs = [pair; MAXIMUM_DEVICES];
    }
    if groups & GROUP_COMMIT_SLOTS != 0 {
        // 提交内生块从开放段 bump，游标挪到落点之后。`crates/singlefs-core/src/allocator.rs:1494`、`:1522`。
        held.commit_slot_start = view.commit_cursor;
        view.commit_cursor += COMMIT_GENERATED_NODES_PER_PUBLISH;
        assert!(view.commit_cursor <= COMMIT_GENERATED_POOL_SLOTS);
    }
    if groups & GROUP_INODE != 0 && shape.new_inodes > 0 {
        // inode 号从上一版水位起连号，新水位 = 上一版 + 新建数。`crates/singlefs-core/src/transaction.rs:4839`、`:4848`、`:6266`–`6270`。
        held.inode_start = view.inode_watermark;
        held.inode_count = shape.new_inodes;
        view.inode_watermark += shape.new_inodes;
    }
    if groups & GROUP_TREE != 0 {
        if !view.has_file {
            held.tree_start = view.tree_watermark;
            view.tree_watermark += TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION;
        }
        view.has_file = true;
    }
    if groups & GROUP_TRANSACTION_NUMBER != 0 {
        // 事务号从上一版加一。`crates/singlefs-core/src/transaction.rs:4733`。
        view.highest_transaction += 1;
        held.transaction_number = view.highest_transaction;
    }
    if groups & GROUP_JOURNAL != 0 {
        // jsn 计数器从上一版加一。`crates/singlefs-core/src/transaction.rs:4732`。
        view.counter += 1;
        held.counter = view.counter;
    }
    if groups & GROUP_ROOT_RING != 0 {
        // checkpoint_txg 从上一版加一。`crates/singlefs-core/src/transaction.rs:3420`。
        view.txg += 1;
        held.txg = view.txg;
    }
}

/// 把 `source` 里属于 `groups` 的字段整份写进 `destination`（覆盖，不合并：这就是丢更新发生的地方）。
fn copy_group_fields(destination: &mut SharedState, source: &SharedState, groups: u8) {
    if groups & GROUP_USER != 0 {
        destination.user_taken = source.user_taken;
    }
    if groups & GROUP_COMMIT_SLOTS != 0 {
        destination.commit_cursor = source.commit_cursor;
    }
    if groups & GROUP_INODE != 0 {
        destination.inode_watermark = source.inode_watermark;
    }
    if groups & GROUP_TREE != 0 {
        destination.tree_watermark = source.tree_watermark;
        destination.has_file = source.has_file;
    }
    if groups & GROUP_TRANSACTION_NUMBER != 0 {
        destination.highest_transaction = source.highest_transaction;
    }
    if groups & GROUP_JOURNAL != 0 {
        destination.counter = source.counter;
    }
    if groups & GROUP_ROOT_RING != 0 {
        destination.txg = source.txg;
    }
}

/// 根环槽 = (txg mod 3, (txg div 3) mod S)。`crates/singlefs-core/src/root_ring.rs:114`–`115`。
pub(super) fn root_ring_slot_key(txg: u8, slots_per_region: u32) -> u64 {
    let region = u32::from(txg) % ROOT_RING_REGIONS;
    let slot = (u32::from(txg) / ROOT_RING_REGIONS) % slots_per_region;
    (u64::from(region) << 32) | u64::from(slot)
}

/// journal 环槽 = (计数器 − 1) mod 环槽数。`crates/singlefs-core/src/journal.rs:196`–`201`。
pub(super) fn journal_ring_slot(counter: u8, ring_slots: u32) -> u32 {
    (u32::from(counter) - 1) % ring_slots
}

// ── 步的使能与施加 ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum StepResult {
    Applied,
    /// P设 的 CHECK 判不一致而重算已到上限：这一枝记一次「截断」，不进调度数。
    Truncated,
}

pub(super) fn pair_range_mask(first_pair: usize, end_pair: usize) -> u16 {
    u16::try_from((1u32 << end_pair) - (1u32 << first_pair)).expect("16 个对装得进 u16")
}

fn lowest_free_region(model: &Model, shared: &SharedState) -> Option<usize> {
    (0..model.region_count).find(|region| shared.region_busy & (1 << region) == 0)
}

/// 本事务在自己认领的区里还缺用户数据落点或提交内生块落点（区装不下）。
fn region_shortage(model: &Model, shape: TransactionShape, held: &Holdings) -> bool {
    let missing_user_pair =
        model.transaction_groups & GROUP_USER != 0 && shape.user_units > 0 && held.user_pairs[0] == NO_PAIR;
    let missing_commit_slots = model.transaction_groups & GROUP_COMMIT_SLOTS != 0 && held.commit_slot_start == NO_SLOT;
    missing_user_pair || missing_commit_slots
}

pub(super) fn is_step_enabled(model: &Model, state: &ModelState, transaction_index: usize) -> bool {
    let transaction = &state.transactions[transaction_index];
    let shape = model.shapes[transaction_index];
    match model.steps[usize::from(transaction.program_counter)] {
        StepKind::RegionClaim => {
            model.variant == ArmVariant::PositiveControl || lowest_free_region(model, &state.shared).is_some()
        }
        StepKind::RegionFallback => {
            !region_shortage(model, shape, &transaction.items) || lowest_free_region(model, &state.shared).is_some()
        }
        StepKind::WholeLocked
        | StepKind::WholeSnapshot
        | StepKind::WholeWriteBack
        | StepKind::OutsideSnapshot
        | StepKind::InsideLocked
        | StepKind::InsideSnapshot
        | StepKind::InsideWriteBack
        | StepKind::OutsideWriteBack
        | StepKind::RegionCommit
        | StepKind::DeviceClaim
        | StepKind::DeviceTake(_)
        | StepKind::DeviceRead(_)
        | StepKind::DeviceWrite(_)
        | StepKind::DeviceCheck
        | StepKind::DeviceCommit
        | StepKind::OptimisticSnapshot
        | StepKind::OptimisticCommit => true,
    }
}

fn restart_transaction(transaction: &mut TransactionState) {
    transaction.program_counter = 0;
    transaction.view = EMPTY_SHARED_STATE;
    transaction.seen_version = 0;
    transaction.items = NOTHING_HELD;
    transaction.claimed_regions = 0;
    transaction.device_read_value = NO_PAIR;
    transaction.inode_holes = 0;
    transaction.needs_interval_fallback = false;
}

/// 提交那一刻：串行管道按序发它那几样，事务记为已提交，不再带拷贝与认领的区。
fn finish_commit(model: &Model, shared: &mut SharedState, transaction: &mut TransactionState, shape: TransactionShape) {
    take_resources(shared, shape, model.pipeline_groups, &mut transaction.items);
    transaction.committed = true;
    transaction.program_counter = small(model.steps.len());
    transaction.view = EMPTY_SHARED_STATE;
    transaction.seen_version = 0;
    transaction.claimed_regions = 0;
    transaction.device_read_value = NO_PAIR;
    transaction.needs_interval_fallback = false;
}

/// 在 `region` 里不加同步地取落点（P区：用户数据对与提交内生块槽）；取不到的留着，由 FALLBACK 补。
fn take_from_region(
    model: &Model,
    region: usize,
    shape: TransactionShape,
    view: &mut SharedState,
    held: &mut Holdings,
) {
    let (first_pair, end_pair) = model.region_pair_range(region);
    if model.transaction_groups & GROUP_USER != 0 && shape.user_units > 0 && held.user_pairs[0] == NO_PAIR {
        if let Some(pair) = lowest_free_pair(view.user_taken, first_pair, end_pair) {
            view.user_taken |= 1 << pair;
            held.user_pairs = [pair; MAXIMUM_DEVICES];
        }
    }
    if model.transaction_groups & GROUP_COMMIT_SLOTS != 0 && held.commit_slot_start == NO_SLOT {
        let cursor = view.region_cursor[region];
        let (_, end_slot) = model.region_slot_range(region);
        if cursor + COMMIT_GENERATED_NODES_PER_PUBLISH <= end_slot {
            held.commit_slot_start = cursor;
            view.region_cursor[region] = cursor + COMMIT_GENERATED_NODES_PER_PUBLISH;
        }
    }
}

/// 分区臂在 CLAIM 里预留号区间（`advance` = 推进共享水位；P区同 不推进，每个事务都读到同一个当前值）。
fn reserve_numbers(
    model: &Model,
    shared: &mut SharedState,
    shape: TransactionShape,
    advance: bool,
    transaction: &mut TransactionState,
    groups: u8,
) {
    if groups & GROUP_INODE != 0 && shape.new_inodes > 0 {
        let needed = shape.new_inodes;
        let reserved = match model.inode_reservation {
            InodeReservation::Exact => needed,
            InodeReservation::Fixed(count) => count,
        };
        transaction.items.inode_start = shared.inode_watermark;
        transaction.items.inode_count = reserved.min(needed);
        if advance {
            shared.inode_watermark += reserved;
        }
        transaction.inode_holes = reserved.saturating_sub(needed);
        transaction.needs_interval_fallback = reserved < needed;
    }
    if groups & GROUP_TREE != 0 && !shared.has_file {
        transaction.items.tree_start = shared.tree_watermark;
        if advance {
            shared.tree_watermark += TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION;
        }
    }
    if groups & GROUP_TRANSACTION_NUMBER != 0 {
        transaction.items.transaction_number = shared.highest_transaction + 1;
        if advance {
            shared.highest_transaction += 1;
        }
    }
    if groups & GROUP_JOURNAL != 0 {
        transaction.items.counter = shared.counter + 1;
        if advance {
            shared.counter += 1;
        }
    }
    if groups & GROUP_ROOT_RING != 0 {
        transaction.items.txg = shared.txg + 1;
        if advance {
            shared.txg += 1;
        }
    }
}

/// O读 的校验：只核本事务真读到的（登记 5.2）。
fn read_set_still_valid(model: &Model, shared: &SharedState, transaction: &TransactionState) -> bool {
    let held = &transaction.items;
    let user_pair_still_free = held.user_pairs[0] == NO_PAIR || shared.user_taken & (1 << held.user_pairs[0]) == 0;
    let commit_cursor_unmoved =
        model.transaction_groups & GROUP_COMMIT_SLOTS == 0 || shared.commit_cursor == held.commit_slot_start;
    let inode_start_unmoved = held.inode_count == 0 || shared.inode_watermark == held.inode_start;
    let tree_start_unmoved = held.tree_start == 0 || shared.tree_watermark == held.tree_start;
    let base_version_unmoved = !model.stage.base_version_in_read_set() || shared.version == transaction.seen_version;
    user_pair_still_free && commit_cursor_unmoved && inode_start_unmoved && tree_start_unmoved && base_version_unmoved
}

/// O读 校验通过之后的写回：只加自己那一份，不覆盖别人的。
fn apply_read_set_writes(shared: &mut SharedState, held: &Holdings, groups: u8) {
    if held.user_pairs[0] != NO_PAIR {
        shared.user_taken |= 1 << held.user_pairs[0];
    }
    if groups & GROUP_COMMIT_SLOTS != 0 {
        shared.commit_cursor += COMMIT_GENERATED_NODES_PER_PUBLISH;
    }
    if held.inode_count > 0 {
        shared.inode_watermark += held.inode_count;
    }
    if held.tree_start != 0 {
        shared.tree_watermark += TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION;
    }
    if groups & GROUP_TREE != 0 {
        shared.has_file = true;
    }
}

pub(super) fn apply_step(model: &Model, state: &mut ModelState, transaction_index: usize) -> StepResult {
    let mut transaction = state.transactions[transaction_index];
    let shape = model.shapes[transaction_index];
    let step = model.steps[usize::from(transaction.program_counter)];
    match step {
        StepKind::WholeLocked => {
            take_resources(&mut state.shared, shape, model.transaction_groups, &mut transaction.items);
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::WholeSnapshot => {
            let mut copy = state.shared;
            take_resources(&mut copy, shape, model.transaction_groups, &mut transaction.items);
            transaction.view = copy;
            transaction.program_counter += 1;
        }
        StepKind::WholeWriteBack => {
            copy_group_fields(&mut state.shared, &transaction.view, model.transaction_groups);
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::OutsideSnapshot => {
            let mut copy = state.shared;
            take_resources(
                &mut copy,
                shape,
                LITERAL_LOCK_OUTSIDE_GROUPS & model.transaction_groups,
                &mut transaction.items,
            );
            transaction.view = copy;
            transaction.program_counter += 1;
        }
        StepKind::InsideLocked => {
            take_resources(
                &mut state.shared,
                shape,
                LITERAL_LOCK_INSIDE_GROUPS & model.transaction_groups,
                &mut transaction.items,
            );
            transaction.program_counter += 1;
        }
        StepKind::InsideSnapshot => {
            let inside_groups = LITERAL_LOCK_INSIDE_GROUPS & model.transaction_groups;
            let mut copy = state.shared;
            take_resources(&mut copy, shape, inside_groups, &mut transaction.items);
            copy_group_fields(&mut transaction.view, &copy, inside_groups);
            transaction.program_counter += 1;
        }
        StepKind::InsideWriteBack => {
            copy_group_fields(
                &mut state.shared,
                &transaction.view,
                LITERAL_LOCK_INSIDE_GROUPS & model.transaction_groups,
            );
            transaction.program_counter += 1;
        }
        StepKind::OutsideWriteBack => {
            copy_group_fields(
                &mut state.shared,
                &transaction.view,
                LITERAL_LOCK_OUTSIDE_GROUPS & model.transaction_groups,
            );
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::RegionClaim => {
            let is_normal = model.variant == ArmVariant::Normal;
            let region = if is_normal {
                lowest_free_region(model, &state.shared).expect("RegionClaim 只在有空闲区时使能")
            } else {
                0
            };
            if is_normal {
                state.shared.region_busy |= 1 << region;
            }
            transaction.claimed_regions = 1 << region;
            let mut view = EMPTY_SHARED_STATE;
            view.user_taken = state.shared.user_taken;
            view.region_cursor = state.shared.region_cursor;
            take_from_region(model, region, shape, &mut view, &mut transaction.items);
            reserve_numbers(
                model,
                &mut state.shared,
                shape,
                is_normal,
                &mut transaction,
                model.transaction_groups & !(GROUP_USER | GROUP_COMMIT_SLOTS),
            );
            transaction.view = view;
            transaction.program_counter =
                if transaction.needs_interval_fallback || region_shortage(model, shape, &transaction.items) {
                    1
                } else {
                    2
                };
        }
        StepKind::RegionFallback => {
            if transaction.needs_interval_fallback {
                let missing = shape.new_inodes - transaction.items.inode_count;
                transaction.items.extra_inode_start = state.shared.inode_watermark;
                transaction.items.extra_inode_count = missing;
                state.shared.inode_watermark += missing;
                transaction.needs_interval_fallback = false;
            }
            if region_shortage(model, shape, &transaction.items) {
                let region = lowest_free_region(model, &state.shared).expect("RegionFallback 缺区时只在有空闲区时使能");
                state.shared.region_busy |= 1 << region;
                transaction.claimed_regions |= 1 << region;
                let (first_pair, end_pair) = model.region_pair_range(region);
                let region_mask = pair_range_mask(first_pair, end_pair);
                let mut view = transaction.view;
                view.user_taken = (view.user_taken & !region_mask) | (state.shared.user_taken & region_mask);
                view.region_cursor[region] = state.shared.region_cursor[region];
                take_from_region(model, region, shape, &mut view, &mut transaction.items);
                transaction.view = view;
            }
            transaction.fallbacks += 1;
            transaction.program_counter = 2;
        }
        StepKind::RegionCommit => {
            for region in 0..model.region_count {
                if transaction.claimed_regions & (1 << region) != 0 {
                    let (first_pair, end_pair) = model.region_pair_range(region);
                    let region_mask = pair_range_mask(first_pair, end_pair);
                    state.shared.user_taken =
                        (state.shared.user_taken & !region_mask) | (transaction.view.user_taken & region_mask);
                    state.shared.region_cursor[region] = transaction.view.region_cursor[region];
                }
            }
            state.shared.region_busy &= !transaction.claimed_regions;
            if model.transaction_groups & GROUP_TREE != 0 {
                state.shared.has_file = true;
            }
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::DeviceClaim => {
            reserve_numbers(
                model,
                &mut state.shared,
                shape,
                true,
                &mut transaction,
                model.transaction_groups & !(GROUP_USER | GROUP_COMMIT_SLOTS),
            );
            transaction.program_counter += 1;
        }
        StepKind::DeviceTake(device) => {
            if shape.user_units > 0 {
                let pair = lowest_free_pair(state.shared.device_taken[device], 0, USER_DATA_PAIR_COUNT)
                    .expect("16 个对够 n ≤ 4 个事务用");
                state.shared.device_taken[device] |= 1 << pair;
                transaction.items.user_pairs[device] = pair;
            }
            transaction.program_counter += 1;
        }
        StepKind::DeviceRead(device) => {
            if shape.user_units > 0 {
                transaction.device_read_value = lowest_free_pair(state.shared.device_taken[device], 0, USER_DATA_PAIR_COUNT)
                    .expect("16 个对够 n ≤ 4 个事务用");
            }
            transaction.program_counter += 1;
        }
        StepKind::DeviceWrite(device) => {
            if shape.user_units > 0 {
                let pair = transaction.device_read_value;
                state.shared.device_taken[device] |= 1 << pair;
                transaction.items.user_pairs[device] = pair;
                transaction.device_read_value = NO_PAIR;
            }
            transaction.program_counter += 1;
        }
        StepKind::DeviceCheck => {
            // 各盘答得不一致就拒（今天的实现在 `crates/singlefs-core/src/allocator.rs:159` 拒，`:1377` 的调用点）。
            let picks_agree = transaction.items.user_pairs[..model.device_count]
                .windows(2)
                .all(|picks_of_two_devices| picks_of_two_devices[0] == picks_of_two_devices[1]);
            if picks_agree {
                transaction.program_counter += 1;
            } else if transaction.retries == DEVICE_DISAGREEMENT_RETRY_CAP {
                return StepResult::Truncated;
            } else {
                for device in 0..model.device_count {
                    let pair = transaction.items.user_pairs[device];
                    if pair == NO_PAIR {
                        continue;
                    }
                    let held_by_another_transaction = (0..model.transaction_count).any(|other| {
                        other != transaction_index && state.transactions[other].items.user_pairs[device] == pair
                    });
                    if !held_by_another_transaction {
                        state.shared.device_taken[device] &= !(1 << pair);
                    }
                }
                transaction.items.user_pairs = [NO_PAIR; MAXIMUM_DEVICES];
                transaction.retries += 1;
                transaction.program_counter = FIRST_DEVICE_STEP_INDEX;
            }
        }
        StepKind::DeviceCommit => {
            if model.transaction_groups & GROUP_COMMIT_SLOTS != 0 {
                transaction.items.commit_slot_start = state.shared.commit_cursor;
                state.shared.commit_cursor += COMMIT_GENERATED_NODES_PER_PUBLISH;
            }
            if model.transaction_groups & GROUP_TREE != 0 {
                state.shared.has_file = true;
            }
            finish_commit(model, &mut state.shared, &mut transaction, shape);
        }
        StepKind::OptimisticSnapshot => {
            let snapshot_groups = if model.optimistic_uses_read_set() {
                model.transaction_groups & !OPTIMISTIC_COMMIT_TIME_GROUPS
            } else {
                model.transaction_groups
            };
            let mut copy = state.shared;
            take_resources(&mut copy, shape, snapshot_groups, &mut transaction.items);
            transaction.seen_version = state.shared.version;
            transaction.view = if model.optimistic_uses_read_set() { EMPTY_SHARED_STATE } else { copy };
            transaction.program_counter += 1;
        }
        StepKind::OptimisticCommit => {
            let accepted = if model.variant == ArmVariant::PositiveControl {
                true
            } else if model.optimistic_uses_read_set() {
                read_set_still_valid(model, &state.shared, &transaction)
            } else {
                transaction.seen_version == state.shared.version
            };
            if accepted {
                if model.optimistic_uses_read_set() {
                    apply_read_set_writes(&mut state.shared, &transaction.items, model.transaction_groups);
                    take_resources(
                        &mut state.shared,
                        shape,
                        model.transaction_groups & OPTIMISTIC_COMMIT_TIME_GROUPS,
                        &mut transaction.items,
                    );
                } else {
                    copy_group_fields(&mut state.shared, &transaction.view, model.transaction_groups);
                }
                state.shared.version += 1;
                finish_commit(model, &mut state.shared, &mut transaction, shape);
            } else {
                let retries_after_restart = transaction.retries + 1;
                restart_transaction(&mut transaction);
                transaction.retries = retries_after_restart;
            }
        }
    }
    state.transactions[transaction_index] = transaction;
    StepResult::Applied
}
