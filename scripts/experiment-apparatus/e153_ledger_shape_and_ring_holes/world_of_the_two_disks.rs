//! 两块盘的世界：臂的次序、发布种类、不变量快照、发布结果、世界的整个推进与三种建池入口。

use e7_index_bench::{i64_from_usize, usize_from_u64};
use super::*;

// ---------------------------------------------------------------------
// 世界：池级状态 + 五条臂，按 Arm 枚举定的固定次序。
// ---------------------------------------------------------------------

pub(super) const ARM_ORDER: [Arm; 5] = [Arm::ThresholdConservative, Arm::ThresholdNarrow, Arm::IntervalConservative, Arm::IntervalNarrow, Arm::Baseline];

#[derive(Clone, Copy)]
pub(super) enum PublishKind {
    Overwrite,
    Empty,
}

/// Q1/Q2/Q3/Q11 在一次发布持久之后（这一刻）的快照——第六节四个量各占一行，不做合取。
#[derive(Clone, Copy, Default)]
pub(super) struct InvariantSnapshot {
    /// Q1：I-3.1（臂内读法）——候选集用 F_生效。
    pub(super) allocated_statistic_mismatch_by_arm_reading: [bool; 5],
    /// Q2：I-3.1（今天 checker 读法）——候选集用最新根自己带的 F（`current_rollback_floor`）。
    pub(super) allocated_statistic_mismatch_by_today_checker_reading: [bool; 5],
    /// Q3：I-5.2——item1 + item2 == 容量。
    pub(super) free_statistic_mismatch: [bool; 5],
    /// Q11：环里最旧有效根的滞后 = 上一次持久根的 txg − 环里最旧有效根的 txg。
    pub(super) oldest_valid_root_lag: u64,
    /// Q12：最新持久根带的 F（`current_rollback_floor`）> F_生效（跨盘取最小）。
    pub(super) floor_only_on_one_disk: bool,
    /// Q7：G27（记账第 1 项 − 第 5 项 == 最新根走读引用，去重求和）判红。
    pub(super) allocated_minus_deferred_reference_mismatch: [bool; 5],
}

#[derive(Clone)]
pub(super) struct PublishOutcome {
    pub(super) txg: u64,
    #[cfg_attr(not(test), expect(dead_code, reason = "只在单测里读"))]
    pub(super) instance: u64,
    pub(super) is_nonempty: bool,
    pub(super) label: &'static str,
    pub(super) pre: [PreAllocationSnapshot; 5],
    pub(super) accounting: [AccountingRow; 5],
    pub(super) invariant_snapshot: InvariantSnapshot,
    /// Q9 轨迹（第四段，见跑前登记第十二节）：这次发布持久之后，每条臂分配记录的条目数
    /// （`arm.ledger.len()`，「曾分配过、尚未被覆盖的落点数」，D3 已定项 7）。
    pub(super) allocation_record_entries: [u64; 5],
    /// Q9b：这次发布自己的回收步扫描的 pending 记录数（`reads_last_reclaim_step`，每次发布都有一次，
    /// 与是否重建无关——登记第五节「回收步的时点」：每次发布释放与分配之前一步）。
    pub(super) reclaim_step_reads: [u64; 5],
    /// Q9b：`reads_last_shadow_recompute` 在这次发布产生之前的取值。只有 `is_rebuild_step` 为真时
    /// 才是「重建步」的读数（挂载 / 切换 / 回退重建 / 抬 F 触发的那一次重算）；模型里普通发布之间不
    /// 触发影子账重算（`recompute_shadow` 只在 `rebuild_allocator` 里调用，`isolate_additional_for_raised_floor`
    /// 只在 `raise_floor` 开头调用一次），所以非重建步这个字段是上一次重建留下的陈值，不单独使用。
    pub(super) shadow_recompute_reads: [u64; 5],
    /// 这次发布是不是紧跟着一次 `rebuild_allocator`（挂载 / 切换 / 回退）或 `raise_floor` 开头那一次
    /// `isolate_additional_for_raised_floor` 之后的第一次发布——Q9b「重建步」的采样点。
    pub(super) is_rebuild_step: bool,
    /// Q6b：这一刻（这次发布释放与分配之前，与 `pre` 同一时点）环里被抛弃根的 item1 代理值
    /// （`abandoned_root_item1_bounds`：(环里 txg 最大的被抛弃根, 全部被抛弃根里的最大值)）。
    pub(super) abandoned_item1_bounds: [(Option<u64>, Option<u64>); 5],
    /// Q6b：这一刻「最近一次回退」的 R_old 的 item1 代理值（回退发生时算好、存进 `World`，一直带到下
    /// 一次回退为止）；这条历史从没回退过时是 `None`。
    pub(super) rollback_target_reference_count: Option<[u64; 5]>,
}

pub(super) struct World {
    pub(super) geometry: GeometryCell,
    pub(super) pool: PoolState,
    pub(super) arms: [ArmExecutor; 5],
    /// Q6b：最近一次回退的 R_old 的 item1 代理值（登记第十二节修订：用 R_old 自己的引用集合大小当
    /// 「它那一刻的记账第 1 项」，回退选中 R_old 那一刻算好、逐臂各存一份，直到下一次回退才更新）。
    last_rollback_target_reference_count: Option<[u64; 5]>,
}

impl World {
    pub(super) fn mkfs(geometry: GeometryCell) -> World {
        let mut arms: Vec<ArmExecutor> = ARM_ORDER.iter().map(|&arm| ArmExecutor::new(arm, geometry.placement)).collect();
        for arm in &mut arms {
            arm.mkfs();
        }
        let mut world =
            World { geometry, pool: PoolState::new(geometry.slots_per_region), arms: arms.try_into().unwrap_or_else(|_| unreachable!()), last_rollback_target_reference_count: None };
        world.snapshot_current_txg(); // 第 0 代根（mkfs）自己不经过任何 publish_* 路径，这里补一次。
        world
    }

    pub(super) fn arm_index(arm: Arm) -> usize {
        ARM_ORDER.iter().position(|candidate| *candidate == arm).expect("Arm 必在 ARM_ORDER 里")
    }

    /// 一次普通发布（O 或 E）：五条臂各自回收步 → 释放 → 分配 → 记账；根槽照常持久（写不失败）。
    pub(super) fn publish(&mut self, kind: PublishKind) -> PublishOutcome {
        self.publish_inner(kind, false)
    }

    /// 一次根槽写失败的发布：单元与记录照常持久（分配照做），但根槽保留旧内容（ring 不提交这个 txg）。
    fn publish_with_root_slot_failure(&mut self, kind: PublishKind) -> PublishOutcome {
        self.publish_inner(kind, true)
    }

    fn publish_inner(&mut self, kind: PublishKind, root_slot_write_fails: bool) -> PublishOutcome {
        let next_txg = self.pool.current_txg + 1;
        let roles = ordered_roles(&match kind { PublishKind::Overwrite => overwrite_roles(), PublishKind::Empty => EMPTY_PUBLISH_ROLES.to_vec() });
        let is_nonempty = matches!(kind, PublishKind::Overwrite);
        let rollback_floor = self.pool.current_rollback_floor;
        let instance = self.pool.current_instance;

        // 「多扣」与违例检查用的候选 / 被抛弃账：这一刻（这次发布释放与分配之前）的环，
        // 也就是「环」而不是「记账环」——p 自己还没有账，它这一刻的内容仍是它前一个根的账，
        // 已经由前一个根自己的 txg 代表，不需要再算一次（登记第一节「多扣」：「这一刻」）。
        // 物理回收步（驱动 occupied / 多扣 / 落点）也用这份「环」门槛——它与记账行公式（记账环）不是
        // 同一件事：见 ArmExecutor::drain_formula_pending 的注。
        let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
        let old_ring_threshold = self.pool.reclaim_threshold();
        let old_ring_blocking = self.pool.interval_blocking_generations();

        // 记账环：把 p 的根槽先换成根 p（登记第五节「发布 p 的记账行」），记账三项的公式门槛用这份环。
        // 写失败时不做这一步（ring 保留旧内容）。
        if !root_slot_write_fails {
            self.pool.commit_root(next_txg, instance, rollback_floor, is_nonempty);
        }
        let accounting_ring_threshold = self.pool.reclaim_threshold();
        let accounting_ring_blocking = self.pool.interval_blocking_generations();

        let mut pre = [PreAllocationSnapshot::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            // 物理回收步（测量臂：每次发布之前一步，门槛用「环」；基线：不在这里回收，回收只在
            // 挂载/切换/回退/抬F）。
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                arm.reclaim_measured(old_ring_threshold, blocking);
            }
            let referenced_union: BTreeSet<u64> = {
                let mut set = BTreeSet::new();
                for txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                    if let Some(account) = arm.root_accounts.get(txg) {
                        set.extend(account.expanded_slots());
                    }
                }
                set
            };
            pre[index] = PreAllocationSnapshot { over_held_slots: i64_from_usize(arm.occupied.len()) - i64_from_usize(referenced_union.len()), isolated_slots: arm.isolated.len() as u64 };
        }

        let mut accounting = [AccountingRow::default(); 5];
        let mut violations_candidate = [0u64; 5];
        let mut violations_abandoned = [0u64; 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            for &role in &roles {
                arm.release_role_if_present(role, next_txg);
            }
            let candidate_union: BTreeSet<u64> = candidate_txgs_before.iter().filter_map(|txg| arm.root_accounts.get(txg)).flat_map(|account| account.expanded_slots()).collect();
            let abandoned_union: BTreeSet<u64> = abandoned_txgs_before.iter().filter_map(|txg| arm.root_accounts.get(txg)).flat_map(|account| account.expanded_slots()).collect();
            for &role in &roles {
                let span = role.span_in_slots();
                let start = {
                    arm.allocate_role(role, next_txg);
                    arm.refs[&role]
                };
                for offset in 0..span {
                    let slot = start + offset;
                    if candidate_union.contains(&slot) {
                        violations_candidate[index] += 1;
                    }
                    if abandoned_union.contains(&slot) {
                        violations_abandoned[index] += 1;
                    }
                }
            }
            arm.violations_candidate += violations_candidate[index];
            arm.violations_abandoned += violations_abandoned[index];
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&accounting_ring_blocking);
                arm.drain_formula_pending(accounting_ring_threshold, blocking);
            }
            accounting[index] = arm.accounting_row();
        }

        let invariant_snapshot = if !root_slot_write_fails {
            for arm in &mut self.arms {
                arm.root_accounts.insert(next_txg, account_from_refs(&arm.refs));
            }
            self.prune_evicted_accounts();
            self.snapshot_current_txg();
            self.invariant_snapshot()
        } else {
            // 写失败：根槽保留旧内容（不进环），但这个 txg 已经「用掉」——切换写行发布的 txg 是它 + 1。
            // 环没有变化，Q1/Q2/Q3/Q11 这一刻没有新状态可判（第一节「合法状态」不含写失败那一刻本身）。
            self.pool.current_txg = next_txg;
            InvariantSnapshot::default()
        };

        let label = if root_slot_write_fails {
            "root_slot_write_failure"
        } else {
            match kind {
                PublishKind::Overwrite => "overwrite",
                PublishKind::Empty => "empty",
            }
        };
        let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
        PublishOutcome {
            txg: next_txg,
            instance,
            is_nonempty,
            label,
            pre,
            accounting,
            invariant_snapshot,
            allocation_record_entries,
            reclaim_step_reads,
            shadow_recompute_reads,
            is_rebuild_step: false,
            abandoned_item1_bounds,
            rollback_target_reference_count: self.last_rollback_target_reference_count,
        }
    }

    /// 剪掉不再「环里自证合法」的历史账快照（ring 里已经没有对应 txg 的条目）。
    fn prune_evicted_accounts(&mut self) {
        let live: BTreeSet<u64> = self.pool.live_roots().iter().map(|entry| entry.txg).collect();
        for arm in &mut self.arms {
            arm.root_accounts.retain(|txg, _| live.contains(txg));
            arm.full_snapshots.retain(|txg, _| live.contains(txg));
        }
    }

    /// 这一刻（最新持久根之后、下一次发布之前）的 Q1/Q2/Q3/Q11 快照。要在
    /// `prune_evicted_accounts()` 之后调用——root_accounts 要是这一刻真正live的那份。
    pub(super) fn invariant_snapshot(&self) -> InvariantSnapshot {
        let candidate_by_effective_floor: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let floor_of_newest_root = self.pool.current_rollback_floor;
        let candidate_by_newest_root_floor: Vec<u64> = self
            .pool
            .live_roots()
            .iter()
            .filter(|entry| self.pool.is_valid(entry.instance, entry.txg) && entry.txg >= floor_of_newest_root)
            .map(|entry| entry.txg)
            .collect();
        let mut snapshot = InvariantSnapshot::default();
        for (index, arm) in self.arms.iter().enumerate() {
            let row = arm.accounting_row();
            let union_by_effective_floor = candidate_reference_union_size(arm, &candidate_by_effective_floor);
            let union_by_newest_root_floor = candidate_reference_union_size(arm, &candidate_by_newest_root_floor);
            snapshot.allocated_statistic_mismatch_by_arm_reading[index] = row.item1_occupied_slots != union_by_effective_floor;
            snapshot.allocated_statistic_mismatch_by_today_checker_reading[index] = row.item1_occupied_slots != union_by_newest_root_floor;
            snapshot.free_statistic_mismatch[index] = row.item1_occupied_slots + row.item2_free_slots != UNIT_AREA_CAPACITY_SLOTS;
            // G27（登记第一节）：逐盘 第 1 项 − 第 5 项 == 最新根走读引用（去重求和）。「最新根走读引用」=
            // 这条臂现在的 refs 展开（就是 root_accounts.get(current_txg) 的账，因为 current_txg 这一刻
            // 的账刚由 account_from_refs(&arm.refs) 写入，见 publish_inner/publish_empty_no_roles/
            // publish_write_row_inner 收尾那几行）。
            let newest_reference_size = account_from_refs(&arm.refs).expanded_slots().count() as u64;
            snapshot.allocated_minus_deferred_reference_mismatch[index] = row.item1_occupied_slots.saturating_sub(row.item5_deferred_slots) != newest_reference_size;
        }
        let newest_txg = self.pool.current_txg;
        let oldest_valid = self.pool.oldest_valid_root_txg();
        snapshot.oldest_valid_root_lag = newest_txg.saturating_sub(oldest_valid);
        snapshot.floor_only_on_one_disk = floor_of_newest_root > self.pool.effective_floor();
        snapshot
    }

    /// 给当前 txg 拍一份整臂快照（供回退 / 切换基准恢复）。每个提交点都要调用一次，不能只在
    /// 一批发布跑完之后补一次——那样只会重复拍最后那个 txg，之前的历史帧全部缺失。
    fn snapshot_current_txg(&mut self) {
        let txg = self.pool.current_txg;
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshot();
            arm.full_snapshots.insert(txg, snapshot);
        }
    }

    /// 首次挂载暖机：取号（mkfs 是实例 0，首次挂载取实例 1，不写行，M0.9）、txg 1、2，零单元，
    /// 根照引用 mkfs 的账（M0.4）。取号要发生在两次空发布之前——`crates/singlefs-core/src/mount.rs`
    /// 785-811 的 `establish_instance` 对只做过 mkfs 的池同样先 `acquire_instance` 再暖机；这里若不取号，
    /// `current_instance` 会一直停在 mkfs 的 0，第一次真正的实例切换（回退 / 写失败重发）就会错误地把
    /// 新实例取成 1（应为 2），与登记 M0.9「首次挂载实例 1」不符。
    pub(super) fn first_mount_warmup(&mut self) -> Vec<PublishOutcome> {
        let instance = self.pool.take_next_instance();
        self.pool.current_instance = instance;
        let mut outcomes = Vec::new();
        for _ in 0..2 {
            outcomes.push(self.publish_empty_no_roles());
        }
        outcomes
    }

    /// 「零单元」的空发布：不触碰任何角色，只推进 txg 并把当前 refs 原样写进新根。
    fn publish_empty_no_roles(&mut self) -> PublishOutcome {
        let next_txg = self.pool.current_txg + 1;
        let rollback_floor = self.pool.current_rollback_floor;
        let instance = self.pool.current_instance;
        // 「多扣」与物理回收步用「环」（这一刻，p 还没有账）；记账公式用「记账环」（先换根之后）——
        // 两个不同的环，见 publish_inner 的注。
        let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
        let old_ring_threshold = self.pool.reclaim_threshold();
        let old_ring_blocking = self.pool.interval_blocking_generations();
        self.pool.commit_root(next_txg, instance, rollback_floor, false);
        let accounting_ring_threshold = self.pool.reclaim_threshold();
        let accounting_ring_blocking = self.pool.interval_blocking_generations();
        let mut pre = [PreAllocationSnapshot::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                arm.reclaim_measured(old_ring_threshold, blocking);
            }
            let referenced_union: BTreeSet<u64> = {
                let mut set = BTreeSet::new();
                for txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                    if let Some(account) = arm.root_accounts.get(txg) {
                        set.extend(account.expanded_slots());
                    }
                }
                set
            };
            pre[index] = PreAllocationSnapshot { over_held_slots: i64_from_usize(arm.occupied.len()) - i64_from_usize(referenced_union.len()), isolated_slots: arm.isolated.len() as u64 };
        }
        let mut accounting = [AccountingRow::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&accounting_ring_blocking);
                arm.drain_formula_pending(accounting_ring_threshold, blocking);
            }
            accounting[index] = arm.accounting_row();
        }
        for arm in &mut self.arms {
            arm.root_accounts.insert(next_txg, account_from_refs(&arm.refs));
        }
        self.prune_evicted_accounts();
        self.snapshot_current_txg();
        let invariant_snapshot = self.invariant_snapshot();
        let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
        PublishOutcome {
            txg: next_txg,
            instance,
            is_nonempty: false,
            label: "warmup",
            pre,
            accounting,
            invariant_snapshot,
            allocation_record_entries,
            reclaim_step_reads,
            shadow_recompute_reads,
            is_rebuild_step: false,
            abandoned_item1_bounds,
            rollback_target_reference_count: self.last_rollback_target_reference_count,
        }
    }

    /// 暖机：写行发布之后连推 E，直到本实例的根落到两块盘上（A2：至多 2 次）。
    fn warmup_until_both_devices(&mut self) -> Vec<PublishOutcome> {
        let mut outcomes = Vec::new();
        let mut covered: BTreeSet<Device> = BTreeSet::new();
        covered.insert(root_ring_device(self.pool.current_txg));
        while covered.len() < 2 {
            let outcome = self.publish(PublishKind::Empty);
            covered.insert(root_ring_device(outcome.txg));
            outcomes.push(outcome);
        }
        outcomes
    }

    /// 普通挂载（M0.12，无崩溃、无回退）：进程重开、所选根 = 当前状态——`crates/singlefs-core/src/mount.rs`
    /// 889-957 `mount_writable` 在没有崩溃时 `effective_root == chosen_root == 当前根`，`replay_journal`
    /// 施加的记录前缀为空，`rebuilt_allocator` 按挂载读法门槛（`reclaim_floor(F_生效, 最旧有效根)`）重建，
    /// `establish_instance`（`mount.rs` 785-811）给上一个实例写一行 `(instance, T=恢复后生效根的 txg, W, false)`
    /// （mkfs 实例 0 不写行；本模型 W 恒 0，见 M0.9），取新实例，写行发布，暖机到两块盘。
    /// S1(b)(c) 两段历史的「重开取号」那一步靠它：登记第五节 M0.12 定义过这个发布种类。
    pub(super) fn plain_remount(&mut self) -> Vec<PublishOutcome> {
        let current_txg = self.pool.current_txg;
        let outgoing_instance = self.pool.current_instance;
        let new_instance = self.pool.take_next_instance();

        let candidate_txgs: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let blocking: BTreeSet<u64> = candidate_txgs.iter().chain(abandoned_txgs.iter()).copied().collect();
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshots.get(&current_txg).cloned().expect("当前 txg 必有整臂快照");
            arm.rebuild_allocator(&snapshot, self.pool.effective_floor().max(self.pool.oldest_valid_root_txg()), Some(&blocking), &abandoned_txgs, &candidate_txgs);
        }
        // 上一个实例的行：mkfs（实例 0）不写行（D18 已定项 11，M0.9）；行表按 InstanceTableRecords 同型累加
        // （`mount.rs` 240-245 的 `.any()` 语义：同一实例多条行取并集里最紧的那个上限）。
        if outgoing_instance >= 1 {
            self.pool.instance_table_rows.push((outgoing_instance, current_txg));
        }
        self.pool.current_instance = new_instance;

        let mut outcomes = self.publish_write_row(current_txg + 1, false);
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// 抬 F：目标 = min(每块盘上最新有效根, 第 4 新的非空有效根)；不足 4 个非空有效根取最旧有效根；
    /// 连推带新 F 的空发布直到每块盘都有，期间测量臂门槛用新 F、基线扣住待放行。
    pub(super) fn raise_floor(&mut self) -> Vec<PublishOutcome> {
        let candidates = self.pool.candidate_roots();
        let per_device_newest: HashMap<Device, u64> = {
            let mut map: HashMap<Device, u64> = HashMap::new();
            for entry in self.pool.live_roots() {
                if self.pool.is_valid(entry.instance, entry.txg) {
                    let device = root_ring_device(entry.txg);
                    let slot = map.entry(device).or_insert(entry.txg);
                    if entry.txg > *slot {
                        *slot = entry.txg;
                    }
                }
            }
            map
        };
        let ceiling_per_device = per_device_newest.values().copied().min().unwrap_or(0);
        let mut nonempty: Vec<u64> = candidates.iter().filter(|entry| entry.is_nonempty).map(|entry| entry.txg).collect();
        nonempty.sort_unstable();
        let fourth_newest_nonempty = if nonempty.len() >= 4 { nonempty[nonempty.len() - 4] } else { candidates.iter().map(|entry| entry.txg).min().unwrap_or(0) };
        let target_floor = ceiling_per_device.min(fourth_newest_nonempty).max(self.pool.current_rollback_floor);

        // 抬 F 先按目标 F（还没生效）重算影子账（`mount.rs` 609–628「候选集按新 F 缩小……回收之前先按新 F
        // 重算影子账，不然那个槽回收之后就发得出去」）：只对 Narrow 读法追加隔离——保守读法的重算时点
        // 不含 F 触发（第五节「影子账 保守读法」表「重算时点」一行没有 F 这个条件）；只增不清
        // （隔离位在一次挂载之内只增不清，第三节「隔离位在一次挂载之内只增不清」）。
        let candidate_txgs_at_target_floor: Vec<u64> = self
            .pool
            .live_roots()
            .iter()
            .filter(|entry| self.pool.is_valid(entry.instance, entry.txg) && entry.txg >= target_floor)
            .map(|entry| entry.txg)
            .collect();
        let abandoned_txgs_now: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        for arm in &mut self.arms {
            if arm.shadow_kind() == ShadowLedger::Narrow {
                arm.isolate_additional_for_raised_floor(&abandoned_txgs_now, &candidate_txgs_at_target_floor);
            }
        }

        // 先给每条臂按新 F 扣住可回收的槽（withhold_instead_of_free = true）——只有基线这样做：
        // 测量臂的回收门槛用 F_生效（不是目标 F），F_生效 的公式本身要等新 F 在每块盘都持久才生效，
        // 天然不会提前动用还没生效的目标 F。
        for arm in &mut self.arms {
            if !arm.arm.is_measured() {
                arm.reclaim_baseline(target_floor.max(self.pool.oldest_valid_root_txg()), true);
            }
        }
        let mut outcomes = Vec::new();
        let mut covered: BTreeSet<Device> = BTreeSet::new();
        let mut is_first_raise_floor_publish = true; // Q9b「重建步」采样点：抬 F 开头那次影子账重算只发生一次
        loop {
            let next_txg = self.pool.current_txg + 1;
            let instance = self.pool.current_instance;
            let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
            let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
            let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
            let old_ring_threshold = self.pool.reclaim_threshold();
            let old_ring_blocking = self.pool.interval_blocking_generations();
            self.pool.commit_root(next_txg, instance, target_floor, false);
            let accounting_ring_threshold = self.pool.reclaim_threshold();
            let accounting_ring_blocking = self.pool.interval_blocking_generations();
            let mut pre = [PreAllocationSnapshot::default(); 5];
            for (index, arm) in self.arms.iter_mut().enumerate() {
                if arm.arm.is_measured() {
                    let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                    arm.reclaim_measured(old_ring_threshold, blocking);
                }
                let referenced_union: BTreeSet<u64> = {
                    let mut set = BTreeSet::new();
                    for txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                        if let Some(account) = arm.root_accounts.get(txg) {
                            set.extend(account.expanded_slots());
                        }
                    }
                    set
                };
                pre[index] = PreAllocationSnapshot { over_held_slots: i64_from_usize(arm.occupied.len()) - i64_from_usize(referenced_union.len()), isolated_slots: arm.isolated.len() as u64 };
            }
            for arm in self.arms.iter_mut() {
                for &role in &EMPTY_PUBLISH_ROLES {
                    arm.release_role_if_present(role, next_txg);
                }
                for &role in &ordered_roles(&EMPTY_PUBLISH_ROLES) {
                    arm.allocate_role(role, next_txg);
                }
            }
            let mut accounting = [AccountingRow::default(); 5];
            for (index, arm) in self.arms.iter_mut().enumerate() {
                if arm.arm.is_measured() {
                    let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&accounting_ring_blocking);
                    arm.drain_formula_pending(accounting_ring_threshold, blocking);
                }
                accounting[index] = arm.accounting_row();
            }
            for arm in &mut self.arms {
                arm.root_accounts.insert(next_txg, account_from_refs(&arm.refs));
            }
            self.prune_evicted_accounts();
            self.snapshot_current_txg();
            covered.insert(root_ring_device(next_txg));
            let invariant_snapshot = self.invariant_snapshot();
            let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
            outcomes.push(PublishOutcome {
                txg: next_txg,
                instance,
                is_nonempty: false,
                label: "raise_floor_empty",
                pre,
                accounting,
                invariant_snapshot,
                allocation_record_entries,
                reclaim_step_reads,
                shadow_recompute_reads,
                is_rebuild_step: is_first_raise_floor_publish,
                abandoned_item1_bounds,
                rollback_target_reference_count: self.last_rollback_target_reference_count,
            });
            is_first_raise_floor_publish = false;
            if covered.len() >= 2 && self.pool.effective_floor() >= target_floor {
                break;
            }
        }
        for arm in &mut self.arms {
            arm.release_withheld();
        }
        outcomes
    }

    /// 回退到候选集里的某个 txg（由调用方给定，登记里各族历史写死选哪条）。
    pub(super) fn rollback_to(&mut self, target_txg: u64) -> Vec<PublishOutcome> {
        let target_entry = self.pool.entry_at(target_txg).expect("回退目标必须在环里").clone();
        let target_instance = target_entry.instance;
        let target_table_length = target_entry.instance_table_length_at_publish;
        let abandoned_extended: Vec<u64> = self.pool.abandoned_roots_including_rollback_extension(target_txg, target_instance).iter().map(|entry| entry.txg).collect();
        let new_txg_minus_one = self.pool.live_roots().iter().map(|entry| entry.txg).max().unwrap_or(target_txg);
        let new_instance = self.pool.take_next_instance();

        // 中间实例（在 target 与当前之间、既非 target 也未被上面的扩展抛弃集合覆盖到的实例）：
        // 本实验各族历史没有切换事件夹在回退之间，这个集合恒空，仍按定义算一遍以防将来族用到。
        let mut abandoned_instances: BTreeSet<u64> = BTreeSet::new();
        for entry in self.pool.live_roots() {
            if (entry.txg, entry.instance) > (target_txg, target_instance) && entry.instance != target_instance {
                abandoned_instances.insert(entry.instance);
            }
        }

        // 候选集要按「回退重建那一刻」的实例表算，而回退行此时还没写进 self.pool.instance_table_rows
        // （几行之后才 push）——candidate_roots() 用的是旧表，会把 abandoned_extended 里那些「按旧表
        // 还没被判被抛弃、但一旦回退行写下就会被抛弃」的 txg（包括回退自己抛弃的尾巴）算成候选，
        // 与 mount.rs 的 `isolate_slots_referenced_only_by_abandoned_roots` 用同一个 `is_abandoned`
        // 闭包（含 `extra_abandoned`）过滤候选不同（见跑前登记第十二节）。
        let candidate_txgs: Vec<u64> =
            self.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| !abandoned_extended.contains(txg)).collect();
        let blocking: BTreeSet<u64> = abandoned_extended.iter().chain(candidate_txgs.iter()).copied().collect();
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshots.get(&target_txg).cloned().expect("回退目标必须有整臂快照");
            arm.rebuild_allocator(&snapshot, self.pool.effective_floor().max(self.pool.oldest_valid_root_txg()), Some(&blocking), &abandoned_extended, &candidate_txgs);
        }

        // Q6b：R_old 的 item1 代理值——回退选中 R_old 那一刻算好存住（`abandoned_root_item1_bounds`
        // 同一份逻辑：用 R_old 自己的引用集合大小当它那一刻的 item1）；target_txg 此刻仍在环里，
        // root_accounts 必有它（rebuild_allocator 不碰 root_accounts）。
        let mut rollback_target_reference_counts = [0u64; 5];
        for (index, arm) in self.arms.iter().enumerate() {
            rollback_target_reference_counts[index] = arm.root_accounts.get(&target_txg).expect("回退目标必有账").expanded_slots().count() as u64;
        }
        self.last_rollback_target_reference_count = Some(rollback_target_reference_counts);

        // 在 R_old 指着的那一版实例表上写回退行：先截断回 R_old 发布时的表长（丢掉之后新加的行）。
        self.pool.instance_table_rows.truncate(target_table_length);
        self.pool.instance_table_rows.push((target_instance, target_txg));
        for instance in &abandoned_instances {
            self.pool.instance_table_rows.push((*instance, 0));
        }
        self.pool.current_txg = new_txg_minus_one;
        self.pool.current_instance = new_instance;

        let mut outcomes = self.publish_write_row(new_txg_minus_one + 1, false);
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// 写行发布：txg 由调用方给定（挂载恢复 / 切换 / 回退各自算好）；`redo_overwrite` 为真时
    /// 同一次发布再加 O 的四个文件角色（实例切换里被重发的在飞发布是 O 的情形）。
    fn publish_write_row(&mut self, txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.publish_write_row_inner(txg, redo_overwrite, false)
    }

    /// H3² 用：写行发布自己的根槽也写失败一次（连续两次失败，第五节 H3² 定义）。
    fn publish_write_row_with_root_slot_failure(&mut self, txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.publish_write_row_inner(txg, redo_overwrite, true)
    }

    fn publish_write_row_inner(&mut self, txg: u64, redo_overwrite: bool, root_slot_write_fails: bool) -> Vec<PublishOutcome> {
        assert_eq!(txg, self.pool.current_txg + 1, "写行发布的 txg 必须紧接当前 txg");
        let roles = ordered_roles(&if redo_overwrite { write_row_with_redo_overwrite_roles() } else { write_row_roles() });
        let rollback_floor = self.pool.current_rollback_floor;
        let instance = self.pool.current_instance;
        let candidate_txgs_before: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs_before: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_item1_bounds = abandoned_root_item1_bounds(&self.arms, &abandoned_txgs_before);
        let old_ring_threshold = self.pool.reclaim_threshold();
        let old_ring_blocking = self.pool.interval_blocking_generations();
        if !root_slot_write_fails {
            self.pool.commit_root(txg, instance, rollback_floor, redo_overwrite);
        }
        let accounting_ring_threshold = self.pool.reclaim_threshold();
        let accounting_ring_blocking = self.pool.interval_blocking_generations();
        let mut pre = [PreAllocationSnapshot::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&old_ring_blocking);
                arm.reclaim_measured(old_ring_threshold, blocking);
            }
            let referenced_union: BTreeSet<u64> = {
                let mut set = BTreeSet::new();
                for root_txg in candidate_txgs_before.iter().chain(abandoned_txgs_before.iter()) {
                    if let Some(account) = arm.root_accounts.get(root_txg) {
                        set.extend(account.expanded_slots());
                    }
                }
                set
            };
            pre[index] = PreAllocationSnapshot { over_held_slots: i64_from_usize(arm.occupied.len()) - i64_from_usize(referenced_union.len()), isolated_slots: arm.isolated.len() as u64 };
        }
        for arm in &mut self.arms {
            for &role in &roles {
                arm.release_role_if_present(role, txg);
            }
            for &role in &roles {
                arm.allocate_role(role, txg);
            }
        }
        let mut accounting = [AccountingRow::default(); 5];
        for (index, arm) in self.arms.iter_mut().enumerate() {
            if arm.arm.is_measured() {
                let blocking = matches!(arm.arm.predicate(), Some(Predicate::IntervalBlocking)).then_some(&accounting_ring_blocking);
                arm.drain_formula_pending(accounting_ring_threshold, blocking);
            }
            accounting[index] = arm.accounting_row();
        }
        let invariant_snapshot = if !root_slot_write_fails {
            for arm in &mut self.arms {
                arm.root_accounts.insert(txg, account_from_refs(&arm.refs));
            }
            self.prune_evicted_accounts();
            self.snapshot_current_txg();
            self.invariant_snapshot()
        } else {
            self.pool.current_txg = txg;
            InvariantSnapshot::default()
        };
        let label = if root_slot_write_fails { "write_row_root_slot_write_failure" } else { "write_row" };
        // 写行发布的调用方（`rollback_to` / `switch_rebuild_and_take_instance` / `plain_remount`）都是先
        // `rebuild_allocator` 再调这个函数——写行发布永远是紧跟重建的那一次，Q9b「重建步」采样点。
        let (allocation_record_entries, reclaim_step_reads, shadow_recompute_reads) = snapshot_allocation_record_read_counters(&self.arms);
        vec![PublishOutcome {
            txg,
            instance,
            is_nonempty: redo_overwrite,
            label,
            pre,
            accounting,
            invariant_snapshot,
            allocation_record_entries,
            reclaim_step_reads,
            shadow_recompute_reads,
            is_rebuild_step: true,
            abandoned_item1_bounds,
            rollback_target_reference_count: self.last_rollback_target_reference_count,
        }]
    }

    /// 切换的共有前半段：从 `allocator_basis_txg` 那一版重建分配账（M0.14「分配账退回失败那次所基于
    /// 的根那一版」）、取新实例；被换下的实例的行写 `(outgoing_row_txg)`——正常切换时是它最后成功发布
    /// 的 txg，H3² 的第二次切换时是 0（换下的实例一次都没发布成功，「没发过根的那个实例行 (i, 0, 0)」）。
    fn switch_rebuild_and_take_instance(&mut self, allocator_basis_txg: u64, outgoing_row_txg: u64) {
        let candidate_txgs: Vec<u64> = self.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
        let abandoned_txgs: Vec<u64> = self.pool.abandoned_roots().iter().map(|entry| entry.txg).collect();
        let threshold = self.pool.reclaim_threshold();
        let blocking: BTreeSet<u64> = abandoned_txgs.iter().chain(candidate_txgs.iter()).copied().collect();
        for arm in &mut self.arms {
            let snapshot = arm.full_snapshots.get(&allocator_basis_txg).cloned().expect("切换基准必须有整臂快照");
            arm.rebuild_allocator(&snapshot, threshold, Some(&blocking), &abandoned_txgs, &candidate_txgs);
        }
        let new_instance = self.pool.take_next_instance();
        self.pool.instance_table_rows.push((self.pool.current_instance, outgoing_row_txg));
        self.pool.current_instance = new_instance;
    }

    /// 根槽写失败之后的实例切换：分配账退回失败那次所基于的根那一版（M0.14），取新实例、写行
    /// （redo 失败那次若是 O 就带上 O 的四个文件角色），暖机。
    fn switch_after_write_failure(&mut self, basis_txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.switch_rebuild_and_take_instance(basis_txg, basis_txg);
        // current_txg 保持不变：写行发布的 txg = 失败那次的 txg + 1（M0.10「切换 = 失败那次的 txg + 1」）。
        let mut outcomes = self.publish_write_row(self.pool.current_txg + 1, redo_overwrite);
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// H3²：切换自己的写行发布（重发那个 W）也根槽写失败一次，再切一次（D23（journal 的角色与格式）
    /// 已定项 14「连续两次失败：W 的根槽也写失败，再切一次，txg 再 + 1，没发过根的那个实例行 (i, 0, 0)」）。
    fn switch_after_write_failure_twice(&mut self, basis_txg: u64, redo_overwrite: bool) -> Vec<PublishOutcome> {
        self.switch_rebuild_and_take_instance(basis_txg, basis_txg);
        let mut outcomes = self.publish_write_row_with_root_slot_failure(self.pool.current_txg + 1, redo_overwrite);
        // 第一次切换取的新实例一次都没发布成功：它的行是 (i, 0)，不是 (i, basis_txg)。
        self.switch_rebuild_and_take_instance(basis_txg, 0);
        outcomes.extend(self.publish_write_row(self.pool.current_txg + 1, redo_overwrite));
        outcomes.extend(self.warmup_until_both_devices());
        outcomes
    }

    /// 工作负载段：连续 `length` 次发布，段内第 k 次（k 从 0 起）是 O ⟺ k mod period == 0
    /// （period=1 ⇒ ρ=1，period=4 ⇒ ρ=1/4）。
    pub(super) fn run_workload_segment(&mut self, length: u64, period: u64) -> Vec<PublishOutcome> {
        let mut outcomes = Vec::with_capacity(usize_from_u64(length));
        for step in 0..length {
            let kind = if step % period == 0 { PublishKind::Overwrite } else { PublishKind::Empty };
            outcomes.push(self.publish(kind));
        }
        outcomes
    }

    // -----------------------------------------------------------------
    // 第五节「历史族、几何格与支线」表列出的族，第三段新增 H2/H3φ/H3²/H5早/H5晚；
    // H1/H4/H6 也在这里重写成方法，供第三段统一的 Q 系列扫描调用。
    // -----------------------------------------------------------------

    /// H1：工作负载段 6N 次。
    pub(super) fn history_continuous_overwrite(&mut self) -> Vec<PublishOutcome> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        self.run_workload_segment(6 * ring_capacity, period)
    }

    /// H2：工作负载段 3N 次 → 普通挂载（恢复、W、暖机）→ 工作负载段 3N 次。
    pub(super) fn history_plain_remount_midway(&mut self) -> Vec<PublishOutcome> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity, period);
        outcomes.extend(self.plain_remount());
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        outcomes
    }

    /// H3φ / H3²：工作负载段 3N+φ 次 → 下一次发布（按段内序号定 O/E）根槽写失败一次 → 实例切换
    /// （`twice` 为真时切换自己的写行也失败一次，再切一次，即 H3²，φ 恒 0）→ 工作负载段 3N 次。
    pub(super) fn history_root_slot_write_failure_and_switch(&mut self, phi: u64, twice: bool) -> Vec<PublishOutcome> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity + phi, period);
        let basis_txg = self.pool.current_txg;
        let next_step_index = 3 * ring_capacity + phi;
        let kind = if next_step_index.is_multiple_of(period) { PublishKind::Overwrite } else { PublishKind::Empty };
        let redo_overwrite = matches!(kind, PublishKind::Overwrite);
        outcomes.push(self.publish_with_root_slot_failure(kind));
        if twice {
            outcomes.extend(self.switch_after_write_failure_twice(basis_txg, redo_overwrite));
        } else {
            outcomes.extend(self.switch_after_write_failure(basis_txg, redo_overwrite));
        }
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        outcomes
    }

    /// H4 的前半段（H4d 自己与 H5 共用）：工作负载段 3N 次 → 回退到「txg ≤ 最新根 txg − d 的候选根里
    /// txg 最大的那条」。**不含 H4d 自己的尾段工作负载**——H5早 / H5晚 紧接着要在这条根还活在环里
    /// （`target` / `write_row_txg` 仍是候选）的时候做第二次回退；若把 H4d 自己的尾段 3N 也跑一遍再
    /// 返回，环早已转过好几圈，`target` 与 `write_row_txg` 必然已被轮转挤出候选集——H5早 / H5晚 的
    /// 第二次回退在任何几何格上都找不到候选，两族 24 行 q_family 全部 `q13_no_candidate=true`
    /// （见跑前登记第十二节；`history_single_rollback` 自带尾段 3N，
    /// H5 两族借它当「H4（d=S）」前缀会把这段也借进去）。写死的回退目标不在候选集里（第十一节
    /// V8「无候选」）时返回 `None`。返回值另带 (R_old 的 txg, 回退那次写行发布的 txg)。
    fn history_rollback_prefix(&mut self, rollback_depth: u64) -> Option<(Vec<PublishOutcome>, u64, u64)> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity, period);
        let newest_txg = self.pool.current_txg;
        let ceiling = newest_txg.saturating_sub(rollback_depth);
        let target = self.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg <= ceiling).max()?;
        let rollback_outcomes = self.rollback_to(target);
        let write_row_txg = rollback_outcomes.iter().find(|outcome| outcome.label == "write_row").map(|outcome| outcome.txg).unwrap_or(0);
        outcomes.extend(rollback_outcomes);
        Some((outcomes, target, write_row_txg))
    }

    /// H4d：`history_rollback_prefix` 之后再接工作负载段 3N 次（第五节 H4d 的完整定义）。写死的回退
    /// 目标不在候选集里（第十一节 V8「无候选」）时返回 `None`，调用方单列条数（Q13）。
    pub(super) fn history_single_rollback(&mut self, rollback_depth: u64) -> Option<(Vec<PublishOutcome>, u64, u64)> {
        let ring_capacity = self.geometry.ring_capacity();
        let period = self.geometry.workload_period;
        let (mut outcomes, target, write_row_txg) = self.history_rollback_prefix(rollback_depth)?;
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some((outcomes, target, write_row_txg))
    }

    /// H5早：H4（d=S）的前半段（`history_rollback_prefix`，不含 H4d 自己的尾段 3N）→ 工作负载段 1 次
    /// → 第二次回退到「txg 小于第一次 R_old 的候选根里 txg 最大的那条」→ 工作负载段 3N 次。任一次回退
    /// 的目标不在候选集里时返回 `None`。
    pub(super) fn history_double_rollback_shallower_second(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let (mut outcomes, first_target, _first_write_row_txg) = self.history_rollback_prefix(slots_per_region)?;
        outcomes.extend(self.run_workload_segment(1, period));
        let second_target = self.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg < first_target).max()?;
        outcomes.extend(self.rollback_to(second_target));
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }

    /// H5晚：H4（d=S）的前半段（`history_rollback_prefix`，不含 H4d 自己的尾段 3N）→ 工作负载段 S 次
    /// → 第二次回退到「第一次回退那条 W 的根（新时间线上最早的根）」→ 工作负载段 3N 次。那条根不再是
    /// 候选（已经被抬 F / 轮转挤出）时返回 `None`。
    pub(super) fn history_double_rollback_to_first_rollback_write_row(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let (mut outcomes, _first_target, first_write_row_txg) = self.history_rollback_prefix(slots_per_region)?;
        outcomes.extend(self.run_workload_segment(slots_per_region, period));
        let still_candidate = self.pool.candidate_roots().iter().any(|entry| entry.txg == first_write_row_txg);
        if !still_candidate {
            return None;
        }
        outcomes.extend(self.rollback_to(first_write_row_txg));
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }

    /// H6：工作负载段 3N 次 → 抬 F → 工作负载段 S 次 → 回退到候选集里 txg 最小的那条根 → 工作负载段
    /// 3N 次。抬 F 之后候选集为空（不该发生，环里至少有刚抬完 F 的那条根）时返回 `None`。
    pub(super) fn history_raise_floor_then_rollback(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let mut outcomes = self.run_workload_segment(3 * ring_capacity, period);
        outcomes.extend(self.raise_floor());
        outcomes.extend(self.run_workload_segment(slots_per_region, period));
        let target = self.pool.candidate_roots().iter().map(|entry| entry.txg).min()?;
        outcomes.extend(self.rollback_to(target));
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }

    /// H7：H4（d=N−2）的前半段（`history_rollback_prefix`，不含 H4dNm2 自己的尾段 3N——理由与 H5早 /
    /// H5晚 同：H7 的抬 F 要在回退那条根还活在环里、且回退带来的被抛弃根仍在被隔离的时候发生，
    /// 才测得到「回退 + 抬 F 交错」；若先把 H4dNm2 自己的尾段 3N 跑完，回退那条根早已被轮转挤出，
    /// 抬 F 面对的就是一段与单纯 H1 无异的稳态历史）→ 工作负载段 S 次 → 抬 F → 工作负载段 3N 次。
    /// 回退目标不在候选集里时返回 `None`（抬 F 本身不失败，见 `raise_floor` 的实现）。
    pub(super) fn history_rollback_deep_then_raise_floor(&mut self) -> Option<Vec<PublishOutcome>> {
        let ring_capacity = self.geometry.ring_capacity();
        let slots_per_region = self.geometry.slots_per_region;
        let period = self.geometry.workload_period;
        let rollback_depth = ring_capacity - 2; // 与 H4dNm2 同一个回退深度（登记第五节 H4dNm2 = H4，d = N-2）
        let (mut outcomes, _target, _write_row_txg) = self.history_rollback_prefix(rollback_depth)?;
        outcomes.extend(self.run_workload_segment(slots_per_region, period));
        outcomes.extend(self.raise_floor());
        outcomes.extend(self.run_workload_segment(3 * ring_capacity, period));
        Some(outcomes)
    }
}

/// 首次挂载暖机 + 新池新建文件（txg 3）：产出五条臂的执行器，供 K1/A6 现算用。
pub(super) fn new_pool_file_creation(cell: GeometryCell) -> World {
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {}
    world.publish(PublishKind::Overwrite);
    world
}

/// Q8 基底 β1（登记第五节「坏镜像」）：H1 主历史最后一次发布之后。
pub(super) fn beta_one_world(cell: GeometryCell) -> World {
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {}
    let _ = world.history_continuous_overwrite();
    world
}

/// Q8 基底 β2：H4（d = N − 2）主历史回退之后第 S 次工作负载发布之后（环里还有被抛弃根）。
/// 与 `history_rollback_deep_then_raise_floor`（H7）共用同一段前缀（`history_rollback_prefix`），
/// 但只跑 S 次工作负载、不抬 F——β2 要的是「刚回退完、正常发布几次」这个状态，不是 H7 那条完整历史。
/// 回退目标不在候选集里时返回 `None`（第十一节 V8「无候选」，H4dNm2 在全部 12 格历史上都有候选，
/// 第四段产物已验证过：`grep -c q13_no_candidate` 命中 0 次）。
pub(super) fn beta_two_world(cell: GeometryCell) -> Option<World> {
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {}
    let rollback_depth = cell.ring_capacity() - 2;
    world.history_rollback_prefix(rollback_depth)?;
    let _ = world.run_workload_segment(cell.slots_per_region, cell.workload_period);
    Some(world)
}

#[cfg(test)]
#[path = "world_of_the_two_disks_tests.rs"]
mod tests;
