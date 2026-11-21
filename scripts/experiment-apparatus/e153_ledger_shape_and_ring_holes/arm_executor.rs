//! 一条臂的执行器：核对行、待回收记录、分配前快照与整条臂的发布、回收与读数。

use super::*;

// ---------------------------------------------------------------------
// 每一次发布记账三项快照
// ---------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct AccountingRow {
    pub(super) item1_occupied_slots: u64,
    pub(super) item2_free_slots: u64,
    pub(super) item5_deferred_slots: u64,
}

/// 一次发布之前（该臂的回收步做完之后、这次发布释放与分配之前）的快照：多扣、影子账隔离槽数。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct PreAllocationSnapshot {
    pub(super) over_held_slots: i64, // 「分配器不发的槽」-「有效根∪被抛弃根引用的槽」
    pub(super) isolated_slots: u64,
}

// ---------------------------------------------------------------------
// 臂执行器：账本、物理占用集合、落点、影子账 —— 全部只属于这一条臂。
// ---------------------------------------------------------------------

#[derive(Clone)]
pub(super) struct ArmExecutor {
    pub(super) arm: Arm,
    pub(super) placement: PlacementPolicy,
    pub(super) refs: HashMap<Role, u64>,
    /// 全历史账本：slot -> 记录，只在再分配时被覆写，从不因回收而删除（登记：回收不改记录内容）。
    pub(super) ledger: HashMap<u64, LedgerEntry>,
    /// 「分配器不发的槽」：仍分配 ∪ 已释放未回收(物理) ∪ 影子账隔离 ∪ 基线抬 F 扣住。
    pub(super) occupied: BTreeSet<u64>,
    pub(super) isolated: BTreeSet<u64>,
    withheld: BTreeSet<u64>,
    /// 测量臂：未释放记录跨度和（增量维护）。
    allocated_span_total: u64,
    /// 测量臂：已释放、物理上尚未越过门槛的记录——驱动 occupied / 多扣 / 落点，门槛用「环」（这一刻，
    /// p 还没有自己的账）。
    pending: Vec<PendingRecord>,
    /// 测量臂：独立事件路径的空闲计数，与 pending 同一门槛（物理，驱动多扣）。
    item2_total: u64,
    /// 测量臂：记账行公式专用——与 pending 同源（release 时一起 push），但门槛用「记账环」
    /// （p 自己的账已经换进环之后）。只影响写进根里的第 1/5 项，不影响物理占用（登记第五节
    /// 「发布 p 的记账行」：记账环 是记账用的环，不是回收步物理用的环）。
    formula_pending: Vec<PendingRecord>,
    /// 测量臂：记账第 2 项公式专用，独立事件路径，门槛同 formula_pending（记账环）。
    formula_item2_total: u64,
    /// 基线：占着（含 defer）、空闲、defer 三个物理计数器，直接对应 allocator.rs 三个字段。
    baseline_allocated_slots: u64,
    baseline_free_slots: u64,
    baseline_deferred_slots: u64,
    /// 基线：released 且物理上还没被回收步清掉的槽（回收步只扫描这个集合，不扫整本账）。
    baseline_deferred_set: BTreeSet<u64>,
    /// 落点政策 P-impl 的开放段游标（段起点，段内 bump）。
    open_segment_start: Option<u64>,
    /// 段内下一次扫描的起点：只在段内单调前进，跳过的槽不回填（登记 M0.16「游标 bump」）。
    open_segment_cursor: u64,
    /// 每条仍在环里的根，这条臂自己的账快照（供影子账、回退、违例检查）。
    pub(super) root_accounts: BTreeMap<u64, RootAccount>,
    /// 供回退 / 切换基准恢复用的整臂快照。
    pub(super) full_snapshots: BTreeMap<u64, ArmFullSnapshot>,
    /// 每个回收步实际读的分配记录树棵数（Q9b 的累计器，重建步单独记）。
    pub(super) reads_last_reclaim_step: u64,
    pub(super) reads_last_shadow_recompute: u64,
    /// 违例计数（Q4i 候选根那一半、Q4ii 被抛弃根那一半）。
    pub(super) violations_candidate: u64,
    pub(super) violations_abandoned: u64,
    /// Q10：G12×G28 的候选引用推法（登记第五节「G12」支持者形态：「候选根 r 引用槽 x ⟺ 当前账里 x 的
    /// 记录分配代 ≤ txg(r) ∧（未释放 ∨ txg(r) < 释放代）」，不另读候选根的树）与按 M0.6 走读候选根账
    /// 得到的并集不一致的回收步数。只在 IntervalNarrow 这条臂上累计（G12 与 G28 都齐才有意义）。
    pub(super) interval_candidate_inference_mismatch_count: u64,
}

#[derive(Clone)]
pub(super) struct PendingRecord {
    pub(super) slot: u64,
    pub(super) span: u64,
    pub(super) allocation_generation: u64,
    pub(super) release_generation: u64,
}

/// 两种主谓词的可再分配判定（登记第五节「主谓词 甲-T1 / G12」）：
/// 甲-T1 = 已释放 ∧ 释放代 ≤ 门槛；G12 = 已释放 ∧ 环里没有根 r：txg(r) ∈ [分配代, 释放代) 且 r 属于
/// blocking 集合（候选集 ∪ 被抛弃根，由调用方按记账环或环算好传入）。两条重算路径（物理回收、记账
/// 公式）共用这一处判定，改一处两边同步（M1–M6 的变异锚点）。
pub(super) fn is_record_reclaimable(predicate: Predicate, threshold: u64, blocking: Option<&BTreeSet<u64>>, record: &PendingRecord) -> bool {
    match predicate {
        Predicate::ThresholdFloor => record.release_generation <= threshold,
        Predicate::IntervalBlocking => {
            let blocking = blocking.expect("G12 需要 blocking 集合");
            blocking.range(record.allocation_generation..record.release_generation).next().is_none()
        }
    }
}

#[derive(Clone)]
pub(super) struct ArmFullSnapshot {
    pub(super) refs: HashMap<Role, u64>,
    pub(super) ledger: HashMap<u64, LedgerEntry>,
    pub(super) occupied: BTreeSet<u64>,
    pub(super) isolated: BTreeSet<u64>,
    withheld: BTreeSet<u64>,
    allocated_span_total: u64,
    pending: Vec<PendingRecord>,
    item2_total: u64,
    formula_pending: Vec<PendingRecord>,
    formula_item2_total: u64,
    baseline_allocated_slots: u64,
    baseline_free_slots: u64,
    baseline_deferred_slots: u64,
    baseline_deferred_set: BTreeSet<u64>,
    open_segment_start: Option<u64>,
    open_segment_cursor: u64,
    // root_accounts 不在这份快照里：它按自己的注释（496 行）该一直反映「每条仍在环里的根」，
    // 由 World::prune_evicted_accounts 独立维护生命周期，不随 rebuild_allocator 时光倒流
    // （S1 对拍：把它塞进快照、restore_full 时整份覆盖，会让回退之后
    // recompute_shadow 查不到写行/暖机/C 这些「快照时点还没发生」的 txg 的账，隔离集恒空）。
}

impl ArmExecutor {
    pub(super) fn new(arm: Arm, placement: PlacementPolicy) -> ArmExecutor {
        ArmExecutor {
            arm,
            placement,
            refs: HashMap::new(),
            ledger: HashMap::new(),
            occupied: BTreeSet::new(),
            isolated: BTreeSet::new(),
            withheld: BTreeSet::new(),
            allocated_span_total: 0,
            pending: Vec::new(),
            item2_total: UNIT_AREA_CAPACITY_SLOTS,
            formula_pending: Vec::new(),
            formula_item2_total: UNIT_AREA_CAPACITY_SLOTS,
            baseline_allocated_slots: 0,
            baseline_free_slots: UNIT_AREA_CAPACITY_SLOTS,
            baseline_deferred_slots: 0,
            baseline_deferred_set: BTreeSet::new(),
            open_segment_start: None,
            open_segment_cursor: 0,
            root_accounts: BTreeMap::new(),
            full_snapshots: BTreeMap::new(),
            reads_last_reclaim_step: 0,
            reads_last_shadow_recompute: 0,
            violations_candidate: 0,
            violations_abandoned: 0,
            interval_candidate_inference_mismatch_count: 0,
        }
    }

    pub(super) fn full_snapshot(&self) -> ArmFullSnapshot {
        ArmFullSnapshot {
            refs: self.refs.clone(),
            ledger: self.ledger.clone(),
            occupied: self.occupied.clone(),
            isolated: self.isolated.clone(),
            withheld: self.withheld.clone(),
            allocated_span_total: self.allocated_span_total,
            pending: self.pending.clone(),
            item2_total: self.item2_total,
            formula_pending: self.formula_pending.clone(),
            formula_item2_total: self.formula_item2_total,
            baseline_allocated_slots: self.baseline_allocated_slots,
            baseline_free_slots: self.baseline_free_slots,
            baseline_deferred_slots: self.baseline_deferred_slots,
            baseline_deferred_set: self.baseline_deferred_set.clone(),
            open_segment_start: self.open_segment_start,
            open_segment_cursor: self.open_segment_cursor,
        }
    }

    fn restore_full(&mut self, snapshot: &ArmFullSnapshot) {
        self.refs = snapshot.refs.clone();
        self.ledger = snapshot.ledger.clone();
        self.occupied = snapshot.occupied.clone();
        self.isolated = snapshot.isolated.clone();
        self.withheld = snapshot.withheld.clone();
        self.allocated_span_total = snapshot.allocated_span_total;
        self.pending = snapshot.pending.clone();
        self.item2_total = snapshot.item2_total;
        self.formula_pending = snapshot.formula_pending.clone();
        self.formula_item2_total = snapshot.formula_item2_total;
        self.baseline_allocated_slots = snapshot.baseline_allocated_slots;
        self.baseline_free_slots = snapshot.baseline_free_slots;
        self.baseline_deferred_slots = snapshot.baseline_deferred_slots;
        self.baseline_deferred_set = snapshot.baseline_deferred_set.clone();
        self.open_segment_start = snapshot.open_segment_start;
        self.open_segment_cursor = snapshot.open_segment_cursor;
        // root_accounts 不恢复：见 ArmFullSnapshot 定义处的注释。
    }

    /// mkfs：实例表单元 instance_table_slot @ 起点、树表单元 table_unit_slot = instance_table_slot + 2，第 0 代根引用 {instance_table_slot,table_unit_slot}，分配代 0。
    pub(super) fn mkfs(&mut self) {
        let instance_table_slot = UNIT_AREA_START_SLOT;
        let table_unit_slot = UNIT_AREA_START_SLOT + 2;
        self.ledger.insert(instance_table_slot, LedgerEntry::allocated(Role::InstanceTable.span_in_slots(), 0));
        self.ledger.insert(table_unit_slot, LedgerEntry::allocated(Role::TableUnit.span_in_slots(), 0));
        self.occupied.insert(instance_table_slot);
        self.occupied.insert(instance_table_slot + 1);
        self.occupied.insert(table_unit_slot);
        self.allocated_span_total = Role::InstanceTable.span_in_slots() + Role::TableUnit.span_in_slots();
        self.item2_total = UNIT_AREA_CAPACITY_SLOTS - self.allocated_span_total;
        self.formula_item2_total = self.item2_total;
        self.baseline_allocated_slots = self.allocated_span_total;
        self.baseline_free_slots = self.item2_total;
        self.refs.insert(Role::InstanceTable, instance_table_slot);
        self.refs.insert(Role::TableUnit, table_unit_slot);
        self.root_accounts.insert(0, account_from_refs(&self.refs));
    }

    /// 落点：从「不在 occupied 里」的槽里按政策选。P-low：单元区内最低、跨度内每槽都空、跨度 2 起点偶数。
    /// P-impl：数据单元同 P-low 的偶数对规则；提交内生块从开放段 bump（64 槽对齐段，段内无占用才开新段）。
    fn allocate_slot(&mut self, role: Role) -> u64 {
        let span = role.span_in_slots();
        match (self.placement, role) {
            (_, Role::DataUnit) | (PlacementPolicy::Low, _) => self.lowest_free_span(span, span == 2),
            (PlacementPolicy::Impl, _) => self.bump_from_open_segment(span, span == 2),
        }
    }

    pub(super) fn is_free(&self, slot: u64) -> bool {
        !self.occupied.contains(&slot)
    }

    fn lowest_free_span(&mut self, span: u64, start_must_be_even: bool) -> u64 {
        let mut candidate = UNIT_AREA_START_SLOT;
        loop {
            if start_must_be_even && !candidate.is_multiple_of(2) {
                candidate += 1;
                continue;
            }
            let fits = (0..span).all(|offset| self.is_free(candidate + offset));
            if fits {
                return candidate;
            }
            candidate += if start_must_be_even { 2 } else { 1 };
        }
    }

    /// 开放段 = 单元区内最低的、64 槽对齐、段内无任何占用的段；游标只在段内单调前进，被挡（或按
    /// 对齐跳过）的槽从此不再回填——下一次调用从上一次留下的游标位置接着扫，不重新从段首扫
    /// （登记 M0.16「游标 bump」；实测新池新建文件锚点 K1 证明了这一点：槽 50241 被 inode 叶容器的
    /// 偶数对齐跳过之后，inode 根落在 50244 而不是回填 50241）。
    /// 数据单元以外的跨度 2 单元按 32768 字节（= 2 槽）对齐取，即起点必须是偶数（M0.16）。
    fn bump_from_open_segment(&mut self, span: u64, start_must_be_even: bool) -> u64 {
        const SEGMENT_SLOTS: u64 = 64;
        loop {
            if self.open_segment_start.is_none() {
                let mut candidate = UNIT_AREA_START_SLOT;
                loop {
                    let aligned = candidate.div_ceil(SEGMENT_SLOTS) * SEGMENT_SLOTS;
                    if (0..SEGMENT_SLOTS).all(|offset| self.is_free(aligned + offset)) {
                        self.open_segment_start = Some(aligned);
                        self.open_segment_cursor = aligned;
                        break;
                    }
                    candidate = aligned + SEGMENT_SLOTS;
                }
            }
            let segment_start = self.open_segment_start.expect("刚刚赋值过");
            let segment_end = segment_start + SEGMENT_SLOTS;
            while self.open_segment_cursor + span <= segment_end {
                let cursor = self.open_segment_cursor;
                if start_must_be_even && !cursor.is_multiple_of(2) {
                    self.open_segment_cursor += 1;
                    continue;
                }
                if (0..span).all(|offset| self.is_free(cursor + offset)) {
                    self.open_segment_cursor = cursor + span;
                    return cursor;
                }
                self.open_segment_cursor += if start_must_be_even { 2 } else { 1 };
            }
            // 段满：开下一段。
            self.open_segment_start = None;
        }
    }

    /// 释放一个角色（若已存在）：记录改写成已释放，物理上先留在 occupied（回收步再清）。
    pub(super) fn release_role_if_present(&mut self, role: Role, release_generation: u64) {
        if let Some(slot) = self.refs.remove(&role) {
            let span = role.span_in_slots();
            if self.arm.is_measured() {
                self.allocated_span_total -= span;
                let allocation_generation = self.ledger[&slot].allocation_generation;
                self.pending.push(PendingRecord { slot, span, allocation_generation, release_generation });
                self.formula_pending.push(PendingRecord { slot, span, allocation_generation, release_generation });
            } else {
                self.baseline_deferred_slots += span;
                self.baseline_deferred_set.insert(slot);
            }
            if let Some(entry) = self.ledger.get_mut(&slot) {
                entry.released = true;
                entry.release_generation = release_generation;
            }
        }
    }

    pub(super) fn allocate_role(&mut self, role: Role, allocation_generation: u64) {
        let slot = self.allocate_slot(role);
        let span = role.span_in_slots();
        self.ledger.insert(slot, LedgerEntry::allocated(span, allocation_generation));
        for offset in 0..span {
            self.occupied.insert(slot + offset);
        }
        if self.arm.is_measured() {
            self.allocated_span_total += span;
            self.formula_item2_total -= span;
        } else {
            self.baseline_allocated_slots += span;
            self.baseline_free_slots -= span;
        }
        self.item2_total -= span;
        self.refs.insert(role, slot);
    }

    /// 测量臂的回收步：用给定门槛（甲-T1）或给定 blocking 集合（G12）把 pending 里越过门槛的记录
    /// 从 item5 移到 item2，物理上把对应槽从 occupied 里摘掉（除非仍被隔离/扣住）。
    pub(super) fn reclaim_measured(&mut self, threshold: u64, blocking: Option<&BTreeSet<u64>>) {
        let predicate = self.arm.predicate().expect("测量臂才调用");
        let mut reads = 0u64;
        let mut still_pending = Vec::with_capacity(self.pending.len());
        for record in self.pending.drain(..) {
            reads += 1;
            if is_record_reclaimable(predicate, threshold, blocking, &record) {
                self.item2_total += record.span;
                for offset in 0..record.span {
                    let physical_slot = record.slot + offset;
                    if !self.isolated.contains(&physical_slot) && !self.withheld.contains(&physical_slot) {
                        self.occupied.remove(&physical_slot);
                    }
                }
            } else {
                still_pending.push(record);
            }
        }
        self.pending = still_pending;
        self.reads_last_reclaim_step = reads;
    }

    /// 记账行公式专用的重算：用记账环门槛把 formula_pending 里越过门槛的记录移出（登记第五节「发布 p
    /// 的记账行」）。只影响 formula_item2_total / formula_pending，不碰 occupied——它不是物理回收，
    /// 只是「如果現在就按这条根持久之后的环重新判一次，第 1/5 项该是多少」。
    pub(super) fn drain_formula_pending(&mut self, threshold: u64, blocking: Option<&BTreeSet<u64>>) {
        let predicate = self.arm.predicate().expect("测量臂才调用");
        let mut still_pending = Vec::with_capacity(self.formula_pending.len());
        for record in self.formula_pending.drain(..) {
            if is_record_reclaimable(predicate, threshold, blocking, &record) {
                self.formula_item2_total += record.span;
            } else {
                still_pending.push(record);
            }
        }
        self.formula_pending = still_pending;
    }

    /// 基线的回收：只在挂载 / 切换 / 回退重建 / 抬 F 时调用；门槛为 max(floor, oldest_valid)。
    /// 只扫描「已释放、物理上还没被回收步清掉」的集合，不扫整本账。
    /// 扣住期间账目上的第 2 项（空闲）照样涨：真实实现
    /// `allocator.rs` 250–269 的 `mark_reclaimed` 与 611–626 的 `reclaim_released_up_to` 显示
    /// `free_slots += span` 与 `hold_until_floor_takes_effect(...)` 是两件独立的事：`free_slots`
    /// 无条件在回收那一刻就涨，「不许发出去」完全靠 `held_until_floor_takes_effect` 这个独立位图
    /// （`is_free` 另外查它，239 行），不影响 `free_slots`/`allocated_slots` 这两个记账计数器。
    /// 不这样记会让抬 F 扣住期间 I-5.2（空闲统计对得上）里的 item1+item2==容量 判红，被冒烟测试
    /// `all_history_families_run_to_completion_on_every_geometry_cell` 的 H2 抓到（S=4，txg=39）。
    pub(super) fn reclaim_baseline(&mut self, threshold: u64, withhold_instead_of_free: bool) {
        let mut reads = 0u64;
        let mut still_deferred: BTreeSet<u64> = BTreeSet::new();
        for slot in std::mem::take(&mut self.baseline_deferred_set) {
            reads += 1;
            let entry = self.ledger[&slot];
            if entry.release_generation <= threshold {
                let span = entry.span_in_slots;
                self.baseline_allocated_slots -= span;
                self.baseline_deferred_slots -= span;
                self.baseline_free_slots += span;
                if withhold_instead_of_free {
                    for offset in 0..span {
                        self.withheld.insert(slot + offset);
                    }
                } else {
                    for offset in 0..span {
                        let physical_slot = slot + offset;
                        if !self.isolated.contains(&physical_slot) && !self.withheld.contains(&physical_slot) {
                            self.occupied.remove(&physical_slot);
                        }
                    }
                }
            } else {
                still_deferred.insert(slot);
            }
        }
        self.baseline_deferred_set = still_deferred;
        self.reads_last_reclaim_step = reads;
    }

    /// 抬 F 放开时，把 withheld 里门槛已经 <= 新 F_生效 的槽放行（`mount.rs` 抬 F 收尾那一步）。
    pub(super) fn release_withheld(&mut self) {
        for slot in std::mem::take(&mut self.withheld) {
            if !self.isolated.contains(&slot) {
                self.occupied.remove(&slot);
            }
        }
    }

    pub(super) fn accounting_row(&self) -> AccountingRow {
        if self.arm.is_measured() {
            let item5 = self.formula_pending.iter().map(|record| record.span).sum::<u64>();
            AccountingRow { item1_occupied_slots: self.allocated_span_total + item5, item2_free_slots: self.formula_item2_total, item5_deferred_slots: item5 }
        } else {
            AccountingRow { item1_occupied_slots: self.baseline_allocated_slots, item2_free_slots: self.baseline_free_slots, item5_deferred_slots: self.baseline_deferred_slots }
        }
    }

    pub(super) fn shadow_kind(&self) -> ShadowLedger {
        match self.arm {
            Arm::ThresholdConservative | Arm::IntervalConservative => ShadowLedger::Conservative,
            Arm::ThresholdNarrow | Arm::IntervalNarrow | Arm::Baseline => ShadowLedger::Narrow,
        }
    }

    /// 重算隔离集（整个替换）：保守 = ∪被抛弃根账；G28/基线 = ∪被抛弃根账 − ∪候选根账。
    /// 摘掉不再隔离的槽时，若它已经越过测量臂的门槛（已经不在 pending 里、也不在 occupied 之外的
    /// 已回收记录里）就从 occupied 摘掉；仍未达门槛的保持在 occupied（因为它本来就该被回收步扣着）。
    fn recompute_shadow(&mut self, abandoned_txgs: &[u64], candidate_txgs: &[u64]) {
        check_interval_candidate_inference_against_walk(self, candidate_txgs);
        let mut abandoned_union: BTreeSet<u64> = BTreeSet::new();
        for txg in abandoned_txgs {
            if let Some(account) = self.root_accounts.get(txg) {
                abandoned_union.extend(account.expanded_slots());
            }
        }
        let new_isolated: BTreeSet<u64> = match self.shadow_kind() {
            ShadowLedger::Conservative => abandoned_union,
            ShadowLedger::Narrow => {
                let mut candidate_union: BTreeSet<u64> = BTreeSet::new();
                for txg in candidate_txgs {
                    if let Some(account) = self.root_accounts.get(txg) {
                        candidate_union.extend(account.expanded_slots());
                    }
                }
                abandoned_union.difference(&candidate_union).copied().collect()
            }
        };
        let reads = abandoned_txgs.len() as u64 + if self.shadow_kind() == ShadowLedger::Narrow { candidate_txgs.len() as u64 } else { 0 };
        // 放开不再隔离的槽：若它不是「已分配未释放」「已释放未越过门槛」，才真正摘出 occupied。
        // 跨度 2 的记录只用起点做键，物理槽可能是起点或起点 + 1，先找到这个槽所属记录的起点。
        for slot in self.isolated.difference(&new_isolated).copied().collect::<Vec<u64>>() {
            let record_start = if self.ledger.contains_key(&slot) {
                Some(slot)
            } else if slot > 0 && self.ledger.get(&(slot - 1)).is_some_and(|entry| entry.span_in_slots == 2) {
                Some(slot - 1)
            } else {
                None
            };
            let still_held_for_other_reason = record_start.is_some_and(|start| {
                let entry = self.ledger[&start];
                !entry.released || (self.arm.is_measured() && self.pending.iter().any(|record| record.slot == start)) || (!self.arm.is_measured() && self.baseline_deferred_set.contains(&start))
            }) || self.withheld.contains(&slot);
            if !still_held_for_other_reason {
                self.occupied.remove(&slot);
            }
        }
        for slot in &new_isolated {
            self.occupied.insert(*slot);
        }
        self.isolated = new_isolated;
        self.reads_last_shadow_recompute = reads;
    }

    /// 抬 F 专用的追加式隔离重算（`mount.rs` 609–628「候选集按新 F 缩小，只被被抛弃根引用的槽会变多，
    /// 回收之前先按新 F 重算影子账，不然那个槽回收之后就发得出去」）：只增不减——
    /// `isolate_slots_referenced_only_by_abandoned_roots` 只调用 `isolate_abandoned`，不撤销已经
    /// 隔离的槽（隔离位在一次挂载之内只增不清，第三节）。豁免集 = 当前账里还分配着的槽 ∪ 传入的候选账
    /// （候选按目标 F 过滤，调用方算好传入，与 mount.rs 611-628 同：floor 用还没生效的新 F，不是 F_生效）。
    pub(super) fn isolate_additional_for_raised_floor(&mut self, abandoned_txgs: &[u64], candidate_txgs: &[u64]) {
        check_interval_candidate_inference_against_walk(self, candidate_txgs);
        let mut exempt: BTreeSet<u64> = BTreeSet::new();
        for (&slot, entry) in &self.ledger {
            if !entry.released {
                for offset in 0..entry.span_in_slots {
                    exempt.insert(slot + offset);
                }
            }
        }
        for txg in candidate_txgs {
            if let Some(account) = self.root_accounts.get(txg) {
                exempt.extend(account.expanded_slots());
            }
        }
        let mut abandoned_union: BTreeSet<u64> = BTreeSet::new();
        for txg in abandoned_txgs {
            if let Some(account) = self.root_accounts.get(txg) {
                abandoned_union.extend(account.expanded_slots());
            }
        }
        let newly_isolated: Vec<u64> = abandoned_union.difference(&exempt).copied().collect();
        self.reads_last_shadow_recompute = abandoned_txgs.len() as u64 + candidate_txgs.len() as u64;
        for slot in newly_isolated {
            self.isolated.insert(slot);
            self.occupied.insert(slot);
        }
    }

    /// 一次挂载 / 切换 / 回退重建之后的整体重建：从给定的历史账（一份 ArmFullSnapshot 的账本部分）
    /// 重新载入分配账（M0.13「分配账 = R_old 那一版」；M0.14「分配账退回失败那次所基于的根那一版」），
    /// 但落点游标 / occupied 物理集合按重建规则重算：先按 threshold 立刻回收，再重算影子账。
    pub(super) fn rebuild_allocator(&mut self, base: &ArmFullSnapshot, threshold: u64, blocking: Option<&BTreeSet<u64>>, abandoned_txgs: &[u64], candidate_txgs: &[u64]) {
        self.restore_full(base);
        self.open_segment_start = None;
        self.open_segment_cursor = 0;
        // occupied 需要从账本重新推导：仍分配 ∪（已释放但未达门槛）。
        self.occupied.clear();
        self.pending.clear();
        self.item2_total = UNIT_AREA_CAPACITY_SLOTS;
        self.baseline_allocated_slots = 0;
        self.baseline_free_slots = UNIT_AREA_CAPACITY_SLOTS;
        self.baseline_deferred_slots = 0;
        self.baseline_deferred_set.clear();
        self.withheld.clear();
        for (&slot, entry) in self.ledger.clone().iter() {
            if !entry.released {
                for offset in 0..entry.span_in_slots {
                    self.occupied.insert(slot + offset);
                }
                self.item2_total -= entry.span_in_slots;
                if self.arm.is_measured() {
                    // allocated_span_total 已经由 restore_full 带回（它是账本的派生量，这里重新推导以防不一致）。
                } else {
                    self.baseline_allocated_slots += entry.span_in_slots;
                    self.baseline_free_slots -= entry.span_in_slots;
                }
            } else {
                let reclaimable = if self.arm.is_measured() {
                    match self.arm.predicate().expect("测量臂") {
                        Predicate::ThresholdFloor => entry.release_generation <= threshold,
                        Predicate::IntervalBlocking => !blocking.expect("G12 需要 blocking").range(entry.allocation_generation..entry.release_generation).next().is_some(),
                    }
                } else {
                    entry.release_generation <= threshold
                };
                if reclaimable {
                    // item2_total 已初始化为容量，「已释放且可再分配」的槽不参与任何扣减，自然计入空闲。
                } else {
                    for offset in 0..entry.span_in_slots {
                        self.occupied.insert(slot + offset);
                    }
                    self.item2_total -= entry.span_in_slots;
                    if self.arm.is_measured() {
                        self.pending.push(PendingRecord { slot, span: entry.span_in_slots, allocation_generation: entry.allocation_generation, release_generation: entry.release_generation });
                    } else {
                        self.baseline_allocated_slots += entry.span_in_slots;
                        self.baseline_deferred_slots += entry.span_in_slots;
                        self.baseline_deferred_set.insert(slot);
                        // 这一支要扣这一行：
                        // 「已释放但还没达到门槛」的槽仍然占着（进了 item1/deferred），baseline_free_slots
                        // 不跟着扣就会让 item1+item2 比容量多出「这一支累计的槽数」（冒烟测试
                        // all_history_families_run_to_completion_on_every_geometry_cell 的 H2 抓到，
                        // S=4 时多 110，与 `self.item2_total -= entry.span_in_slots;` 那一行是同一件事的
                        // 两份账，baseline 这份要同样扣）。
                        self.baseline_free_slots -= entry.span_in_slots;
                    }
                }
            }
        }
        if self.arm.is_measured() {
            self.allocated_span_total = self.ledger.values().filter(|entry| !entry.released).map(|entry| entry.span_in_slots).sum();
            // 重建那一刻，记账公式与物理回收共用同一个门槛（还没有「记账环 比环多一代」这回事——
            // 那个差异是接下来正常发布之后才会长出来的），所以两份 pending/item2 先置成一样。
            self.formula_pending = self.pending.clone();
            self.formula_item2_total = self.item2_total;
        }
        self.recompute_shadow(abandoned_txgs, candidate_txgs);
    }
}

#[cfg(test)]
#[path = "arm_executor_tests.rs"]
mod tests;
