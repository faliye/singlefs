//! 池的槽与段的计数维护、事务的开始回滚与提交、记录树脏块的推算。

use super::*;

impl Pool {
    /// 一个全空闲的池：没有 mkfs、没有任何块。单测在它上面直接摆状态。
    pub(super) fn blank(geometry: PoolGeometry, settings: PoolSettings) -> Self {
        let unit_area_slots = geometry.unit_area_slots();
        let segment_count = geometry.segment_count();
        let all_free_segment = SegmentCounters {
            free_slots: CLUSTER_SEGMENT_SLOTS,
            free_pairs: CLUSTER_SEGMENT_SLOTS / 2,
            live_slots: 0,
            pinned_live_slots: 0,
            live_user_units: 0,
            released_slots: 0,
        };
        let leaf_positions = usize_of(DISK_COUNT * (geometry.leaf_of_absolute_slot(geometry.disk_slots - 1) + 1));
        let level_one_positions =
            usize_of(DISK_COUNT * (geometry.level_one_node_of_absolute_slot(geometry.disk_slots - 1) + 1));
        Self {
            geometry,
            settings,
            slot_states: vec![SlotState::Free; usize_of(unit_area_slots)],
            segments: vec![all_free_segment; usize_of(segment_count)],
            is_opened_this_mount: vec![false; usize_of(segment_count)],
            segments_opened_this_mount: Vec::new(),
            user_placement_segments: (0..segment_count).collect(),
            fully_free_segments: (0..segment_count).collect(),
            segments_with_a_free_slot: (0..segment_count).collect(),
            record_empty_segment_count: segment_count,
            compaction_destination_free_pairs: 0,
            free_slot_total: unit_area_slots,
            released_slot_total: 0,
            live_user_unit_total: 0,
            snapshot_only_unit_total: 0,
            free_runs: 1,
            reclaim_queue: VecDeque::new(),
            txg: 0,
            effective_floor_txg: 0,
            pending_floor_raise: None,
            non_empty_root_txgs_in_ring: VecDeque::new(),
            last_non_empty_root_txg: 0,
            open_cursor: None,
            file_tree_node_slots: vec![None; usize_of(geometry.file_tree_nodes_per_publish())],
            leaf_slots: vec![None; leaf_positions],
            level_one_slots: vec![None; level_one_positions],
            root_slot: None,
            units: Vec::new(),
            free_unit_indices: Vec::new(),
            units_to_recycle_after_commit: Vec::new(),
            active_intent: None,
            ended_intents: Vec::new(),
            pending_truth_judgements: Vec::new(),
            undo_log: Vec::new(),
            is_transaction_open: false,
            is_making_filesystem: false,
            dirty_slot_offsets: Vec::new(),
            round_metrics: RoundMetrics::starting_at(segment_count, segment_count, segment_count),
            current_round: 0,
        }
    }
}

// ---- 槽的状态改动：全部经 set_slot_state，计数、runs、集合与回滚日志在这一处一起改 ----
impl Pool {
    pub(super) fn segment_of_offset(offset: u64) -> u64 {
        offset / CLUSTER_SEGMENT_SLOTS
    }

    pub(super) fn slot_state(&self, offset: u64) -> SlotState {
        self.slot_states[usize_of(offset)]
    }

    pub(super) fn is_pair_free(&self, even_offset: u64) -> bool {
        self.slot_state(even_offset).is_free() && self.slot_state(even_offset + 1).is_free()
    }

    pub(super) fn membership_of_segment(&self, segment: u64) -> SegmentMembership {
        membership_of(&self.segments[usize_of(segment)], self.is_opened_this_mount[usize_of(segment)])
    }

    pub(super) fn active_region_segment(&self) -> Option<u64> {
        self.active_intent.map(|intent| intent.region_segment)
    }

    /// 可开聚簇段数：占用口径全空且不是活意图的 R（模型不建隔离与扣住位，登记已登记差异 ⑥）。
    pub(super) fn openable_segment_count(&self) -> u64 {
        let region_is_fully_free = self
            .active_region_segment()
            .is_some_and(|region| self.membership_of_segment(region).is_fully_free);
        self.fully_free_segments.len() as u64 - u64::from(region_is_fully_free)
    }

    fn refresh_segment_membership(&mut self, segment: u64, before: SegmentMembership, free_pairs_before: u64) {
        let after = self.membership_of_segment(segment);
        if before.is_user_placement_candidate != after.is_user_placement_candidate {
            if after.is_user_placement_candidate {
                self.user_placement_segments.insert(segment);
            } else {
                self.user_placement_segments.remove(&segment);
            }
        }
        if before.is_fully_free != after.is_fully_free {
            if after.is_fully_free {
                self.fully_free_segments.insert(segment);
            } else {
                self.fully_free_segments.remove(&segment);
            }
        }
        if before.has_a_free_slot != after.has_a_free_slot {
            if after.has_a_free_slot {
                self.segments_with_a_free_slot.insert(segment);
            } else {
                self.segments_with_a_free_slot.remove(&segment);
            }
        }
        if before.is_record_empty != after.is_record_empty {
            if after.is_record_empty {
                self.record_empty_segment_count += 1;
            } else {
                self.record_empty_segment_count -= 1;
            }
        }
        if before.is_compaction_destination {
            self.compaction_destination_free_pairs -= free_pairs_before;
        }
        if after.is_compaction_destination {
            self.compaction_destination_free_pairs += self.segments[usize_of(segment)].free_pairs;
        }
    }

    fn add_state_contribution(&mut self, offset: u64, state: SlotState, sign_is_positive: bool) {
        let counters = &mut self.segments[usize_of(Self::segment_of_offset(offset))];
        let step = |count: &mut u64| {
            if sign_is_positive {
                *count += 1;
            } else {
                *count -= 1;
            }
        };
        match state {
            SlotState::Free => {
                step(&mut counters.free_slots);
                step(&mut self.free_slot_total);
            }
            SlotState::Live { owner, .. } => {
                step(&mut counters.live_slots);
                if owner.is_pinned() {
                    step(&mut counters.pinned_live_slots);
                }
                if owner.is_user_data() && offset.is_multiple_of(2) {
                    step(&mut counters.live_user_units);
                    step(&mut self.live_user_unit_total);
                }
            }
            SlotState::Released { .. } => {
                step(&mut counters.released_slots);
                step(&mut self.released_slot_total);
            }
        }
    }

    /// 改一个槽的状态。事务开着时记回滚日志；记录被改写（分配、释放）的槽进脏槽集 X。
    pub(super) fn set_slot_state(&mut self, offset: u64, new_state: SlotState) {
        let previous = self.slot_state(offset);
        if previous == new_state {
            return;
        }
        let segment = Self::segment_of_offset(offset);
        let membership_before = self.membership_of_segment(segment);
        let free_pairs_before = self.segments[usize_of(segment)].free_pairs;
        let pair_offset = offset - offset % 2;
        let pair_was_free = self.is_pair_free(pair_offset);
        if is_free_for_runs(previous) != is_free_for_runs(new_state) {
            let left_is_free = offset > 0 && is_free_for_runs(self.slot_state(offset - 1));
            let right_is_free =
                offset + 1 < self.geometry.unit_area_slots() && is_free_for_runs(self.slot_state(offset + 1));
            let free_neighbours = u64::from(left_is_free) + u64::from(right_is_free);
            // runs 的增量（`allocator.rs` 的 mark_allocated / mark_reclaimed 同形）：变空闲时两边都空是并段（−1）、两边都占是新开一段（+1）。
            self.free_runs = if is_free_for_runs(new_state) {
                (self.free_runs + 1).checked_sub(free_neighbours).expect("runs 不为负")
            } else {
                (self.free_runs + free_neighbours).checked_sub(1).expect("占掉一个空闲槽之前至少有一段空闲")
            };
        }
        self.add_state_contribution(offset, previous, false);
        self.add_state_contribution(offset, new_state, true);
        self.slot_states[usize_of(offset)] = new_state;
        let pair_is_free = self.is_pair_free(pair_offset);
        if pair_was_free != pair_is_free {
            let counters = &mut self.segments[usize_of(segment)];
            if pair_is_free {
                counters.free_pairs += 1;
            } else {
                counters.free_pairs -= 1;
            }
        }
        self.refresh_segment_membership(segment, membership_before, free_pairs_before);
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::SlotState { offset, previous });
            let record_is_rewritten = (new_state.is_live() && !previous.is_live())
                || (new_state.is_released() && previous.is_live());
            if record_is_rewritten {
                self.dirty_slot_offsets.push(offset);
            }
        }
    }

    pub(super) fn set_segment_opened_this_mount(&mut self, segment: u64, is_opened: bool) {
        let membership_before = self.membership_of_segment(segment);
        let free_pairs_before = self.segments[usize_of(segment)].free_pairs;
        self.is_opened_this_mount[usize_of(segment)] = is_opened;
        self.refresh_segment_membership(segment, membership_before, free_pairs_before);
    }
}

// ---- 事务：预演与回滚、单元与块的位置 ----
impl Pool {
    /// 这次发布要用的 txg（mkfs 那一版是 txg 0）。
    pub(super) fn publishing_txg(&self) -> u64 {
        if self.is_making_filesystem {
            0
        } else {
            self.txg + 1
        }
    }

    pub(super) fn begin_transaction(&mut self) {
        assert!(!self.is_transaction_open, "事务不嵌套");
        self.is_transaction_open = true;
        self.undo_log.clear();
        self.dirty_slot_offsets.clear();
        self.units_to_recycle_after_commit.clear();
    }

    /// 把这次事务改过的每一样照日志倒着放回去。
    pub(super) fn roll_back_transaction(&mut self) {
        assert!(self.is_transaction_open, "没有开着的事务");
        self.is_transaction_open = false;
        while let Some(entry) = self.undo_log.pop() {
            match entry {
                UndoEntry::SlotState { offset, previous } => self.set_slot_state(offset, previous),
                UndoEntry::ReclaimQueuePushedBack => {
                    self.reclaim_queue.pop_back().expect("事务里推进去的那一条还在队尾");
                }
                UndoEntry::OpenCursor { previous } => self.open_cursor = previous,
                UndoEntry::SegmentOpenedThisMount { segment } => {
                    self.set_segment_opened_this_mount(segment, false);
                    self.segments_opened_this_mount.pop().expect("事务里开的段在名单末尾");
                }
                UndoEntry::BlockPosition { block, previous } => *self.block_position_mut(block) = previous,
                UndoEntry::UnitRecordChanged { unit, previous } => self.restore_unit_record(unit, previous),
                UndoEntry::UnitAppendedToArena => {
                    let record = self.units.pop().expect("事务里追加的单元在末尾");
                    assert!(!record.is_referenced_only_by_snapshots(), "新建的单元不会只被快照引用");
                }
                UndoEntry::UnitTakenFromFreeList { unit, previous } => {
                    assert!(previous.can_be_recycled(), "空闲名单里的记录是可回收的那一版");
                    self.units[usize_of(u64::from(unit))] = previous;
                    self.free_unit_indices.push(unit);
                }
                UndoEntry::FallbackBlockCounted => self.round_metrics.fallback_blocks -= 1,
            }
        }
        self.dirty_slot_offsets.clear();
        self.units_to_recycle_after_commit.clear();
    }

    pub(super) fn block_position_mut(&mut self, block: BlockIdentity) -> &mut Option<u64> {
        let leaf_positions_per_disk = self.leaf_slots.len() as u64 / DISK_COUNT;
        let level_one_positions_per_disk = self.level_one_slots.len() as u64 / DISK_COUNT;
        match block {
            BlockIdentity::FileTreeNode { node } => &mut self.file_tree_node_slots[usize_of(node)],
            BlockIdentity::RecordTreeLeaf { disk, leaf } => {
                &mut self.leaf_slots[usize_of(disk * leaf_positions_per_disk + leaf)]
            }
            BlockIdentity::RecordTreeLevelOneNode { disk, node } => {
                &mut self.level_one_slots[usize_of(disk * level_one_positions_per_disk + node)]
            }
            BlockIdentity::RecordTreeRoot => &mut self.root_slot,
        }
    }

    pub(super) fn set_block_position(&mut self, block: BlockIdentity, offset: u64) {
        let slot = self.block_position_mut(block);
        let previous = *slot;
        *slot = Some(offset);
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::BlockPosition { block, previous });
        }
    }

    /// 改一个单元的记录，连带「只被快照引用」的计数；事务开着时记回滚日志。
    pub(super) fn set_unit_record(&mut self, unit: u32, new_record: UnitRecord) {
        let previous = self.units[usize_of(u64::from(unit))];
        self.restore_unit_record(unit, new_record);
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::UnitRecordChanged { unit, previous });
        }
        if new_record.can_be_recycled() {
            self.units_to_recycle_after_commit.push(unit);
        }
    }

    fn restore_unit_record(&mut self, unit: u32, record: UnitRecord) {
        let slot = &mut self.units[usize_of(u64::from(unit))];
        let was_snapshot_only = slot.is_referenced_only_by_snapshots();
        *slot = record;
        let is_snapshot_only = record.is_referenced_only_by_snapshots();
        if was_snapshot_only && !is_snapshot_only {
            self.snapshot_only_unit_total -= 1;
        } else if !was_snapshot_only && is_snapshot_only {
            self.snapshot_only_unit_total += 1;
        }
    }

    /// 新建一个单元的记录（落点由调用方随后分配）。
    pub(super) fn create_unit(&mut self, first_slot_offset: u64, birth_txg: u64) -> u32 {
        assert!(self.is_transaction_open, "单元只在发布的事务里新建");
        let record = UnitRecord {
            first_slot_offset,
            birth_txg,
            is_alive_in_head: true,
            snapshot_pins: 0,
            is_released: false,
        };
        if let Some(unit) = self.free_unit_indices.pop() {
            let previous = self.units[usize_of(u64::from(unit))];
            assert!(previous.can_be_recycled(), "空闲名单里只有可回收的单元");
            self.units[usize_of(u64::from(unit))] = record;
            self.undo_log.push(UndoEntry::UnitTakenFromFreeList { unit, previous });
            unit
        } else {
            self.units.push(record);
            self.undo_log.push(UndoEntry::UnitAppendedToArena);
            u32::try_from(self.units.len() - 1).expect("单元数装得进 u32")
        }
    }

    /// 释放一个已分配的槽：记录改成「已释放 + 释放代 = 这次的 txg」，进 defer 队列。
    pub(super) fn release_slot(&mut self, offset: u64) {
        assert!(self.slot_state(offset).is_live(), "只释放已分配的槽");
        let release_txg = self.publishing_txg();
        self.set_slot_state(offset, SlotState::Released { release_txg });
        self.reclaim_queue.push_back((release_txg, offset));
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::ReclaimQueuePushedBack);
        }
    }

    pub(super) fn allocate_slot(&mut self, offset: u64, owner: SlotOwner, allocation_txg: u64) {
        assert!(self.slot_state(offset).is_free(), "只分配空闲的槽");
        self.set_slot_state(offset, SlotState::Live { owner, allocation_txg });
    }

    pub(super) fn commit_transaction(&mut self) {
        assert!(self.is_transaction_open, "没有开着的事务");
        self.is_transaction_open = false;
        self.undo_log.clear();
        self.dirty_slot_offsets.clear();
        let mut recycled = std::mem::take(&mut self.units_to_recycle_after_commit);
        recycled.sort_unstable();
        recycled.dedup();
        for unit in recycled {
            if self.units[usize_of(u64::from(unit))].can_be_recycled() {
                self.free_unit_indices.push(unit);
            }
        }
    }
}

/// 一组脏槽（绝对槽号）弄脏的分配记录树的叶与层级 1 节点，两块盘各一份（登记 M-3）。
pub(super) fn record_tree_blocks_dirtied_by(geometry: &PoolGeometry, absolute_slots: &[u64]) -> BTreeSet<BlockIdentity> {
    let mut blocks = BTreeSet::new();
    for absolute_slot in absolute_slots.iter().copied() {
        for disk in 0..DISK_COUNT {
            blocks.insert(BlockIdentity::RecordTreeLeaf { disk, leaf: geometry.leaf_of_absolute_slot(absolute_slot) });
            if geometry.has_level_one_nodes_below_the_root() {
                blocks.insert(BlockIdentity::RecordTreeLevelOneNode {
                    disk,
                    node: geometry.level_one_node_of_absolute_slot(absolute_slot),
                });
            }
        }
    }
    blocks
}

/// 只按一组脏槽算一次发布写出的提交内生块数、不做不动点那一步：`(c − b) + 2 × |叶(X)| + 2 × |层级 1(X)| + 1`（登记 M-3，第七节乙 2）。
/// 正式跑走的是 `rewrite_commit_generated_blocks` 的不动点，它与这里共用 `record_tree_blocks_dirtied_by`；这个函数只给单测钉乙 2。
#[cfg_attr(not(test), expect(dead_code, reason = "只在单测里钉乙 2 的绝对值；正式跑经不动点调同一个 record_tree_blocks_dirtied_by"))]
pub(super) fn commit_generated_block_count_for_dirty_slots(geometry: &PoolGeometry, absolute_slots: &[u64]) -> u64 {
    geometry.file_tree_nodes_per_publish() + record_tree_blocks_dirtied_by(geometry, absolute_slots).len() as u64 + 1
}
