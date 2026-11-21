//! 池的开段与取块、提交内生块的重写、合取判定、回收、发布的开始与结束、各类发布。

use super::*;

// ---- 落点：用户数据、整理目的地、提交内生块的 bump 与回落 ----
impl Pool {
    fn set_open_cursor(&mut self, cursor: Option<OpenSegmentCursor>) {
        let previous = self.open_cursor;
        self.open_cursor = cursor;
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::OpenCursor { previous });
        }
    }

    pub(super) fn open_segment(&mut self, segment: u64) {
        if !self.is_opened_this_mount[usize_of(segment)] {
            self.set_segment_opened_this_mount(segment, true);
            self.segments_opened_this_mount.push(segment);
            if self.is_transaction_open {
                self.undo_log.push(UndoEntry::SegmentOpenedThisMount { segment });
            }
        }
        self.set_open_cursor(Some(OpenSegmentCursor { segment, next_offset_in_segment: 0 }));
    }

    /// 用户数据落点：起点最小的偶数槽、两槽都空闲、不在这次挂载开过的段里、不在活意图的 R 里（登记 M-4）。
    fn lowest_user_data_pair(&self) -> Option<u64> {
        let region = self.active_region_segment();
        self.user_placement_segments
            .iter()
            .copied()
            .filter(|segment| Some(*segment) != region)
            .find_map(|segment| self.lowest_free_pair_in_segment(segment))
    }

    fn lowest_free_pair_in_segment(&self, segment: u64) -> Option<u64> {
        let segment_start = segment * CLUSTER_SEGMENT_SLOTS;
        (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .step_by(2)
            .find(|even_offset| self.is_pair_free(*even_offset))
    }

    /// 整理目的地（登记 M-7）：R 外、目的地四条成立的段里最低的空闲两槽。
    pub(super) fn lowest_compaction_destination_pair(&self) -> Option<u64> {
        let region = self.active_region_segment();
        (0..self.geometry.segment_count())
            .filter(|segment| Some(*segment) != region && self.membership_of_segment(*segment).is_compaction_destination)
            .find_map(|segment| self.lowest_free_pair_in_segment(segment))
    }

    /// R 外合格段的空闲两槽数（停乙签发前判用，登记第五节第 2 行）。
    pub(super) fn compaction_destination_free_pairs_outside_region(&self, region_segment: u64) -> u64 {
        let region_contribution = if self.membership_of_segment(region_segment).is_compaction_destination {
            self.segments[usize_of(region_segment)].free_pairs
        } else {
            0
        };
        self.compaction_destination_free_pairs - region_contribution
    }

    /// 提交内生块一块一槽：从开放段 bump；满了开可开聚簇段里最低的那一段；一段都没有就回落到槽号最小的空闲槽（排除 R），
    /// 回落计数加一（登记 M-4，D3（空间分配） 已定项 8 第 2 条）。
    fn take_commit_generated_block_slot(&mut self) -> Option<u64> {
        let region = self.active_region_segment();
        loop {
            if let Some(cursor) = self.open_cursor {
                if cursor.next_offset_in_segment < CLUSTER_SEGMENT_SLOTS {
                    let offset = cursor.segment * CLUSTER_SEGMENT_SLOTS + cursor.next_offset_in_segment;
                    self.set_open_cursor(Some(OpenSegmentCursor {
                        segment: cursor.segment,
                        next_offset_in_segment: cursor.next_offset_in_segment + 1,
                    }));
                    if self.slot_state(offset).is_free() {
                        return Some(offset);
                    }
                    continue;
                }
            }
            let lowest_openable = self.fully_free_segments.iter().copied().find(|segment| Some(*segment) != region);
            match lowest_openable {
                Some(segment) => self.open_segment(segment),
                None => return self.fallback_commit_generated_block_slot(region),
            }
        }
    }

    fn fallback_commit_generated_block_slot(&mut self, region: Option<u64>) -> Option<u64> {
        let fallback_offset = self
            .segments_with_a_free_slot
            .iter()
            .copied()
            .filter(|segment| Some(*segment) != region)
            .find_map(|segment| {
                let segment_start = segment * CLUSTER_SEGMENT_SLOTS;
                (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS).find(|offset| self.slot_state(*offset).is_free())
            })?;
        self.round_metrics.fallback_blocks += 1;
        if self.is_transaction_open {
            self.undo_log.push(UndoEntry::FallbackBlockCounted);
        }
        Some(fallback_offset)
    }

    /// 重写一块提交内生块：新落点 bump，旧版本在这次发布里释放（被快照钉住的那一版不释放）。
    fn rewrite_commit_generated_block(&mut self, block: BlockIdentity) -> Result<(), PublishRefusal> {
        let new_offset = self.take_commit_generated_block_slot().ok_or(PublishRefusal::NoSlotForCommitGeneratedBlock)?;
        self.allocate_slot(new_offset, SlotOwner::CommitGeneratedBlock { block }, self.publishing_txg());
        if let Some(old_offset) = *self.block_position_mut(block) {
            match self.slot_state(old_offset).live_owner() {
                Some(SlotOwner::SnapshotPinnedFileTreeNode) => {}
                Some(SlotOwner::CommitGeneratedBlock { block: owner_block }) if owner_block == block => {
                    self.release_slot(old_offset);
                }
                other => panic!("块 {block:?} 的旧位置 {old_offset} 上不是它自己：{other:?}"),
            }
        }
        self.set_block_position(block, new_offset);
        Ok(())
    }

    /// 这次发布写出的全部提交内生块：文件树节点与根每次都写，另加点名强制重写的块，再加脏槽弄脏的叶与层级 1 节点；
    /// 写出的块自己的落点与它们换下的旧版本并进脏槽再算，直到不变（登记 M-3 的不动点）。返回 `c_actual`。
    pub(super) fn rewrite_commit_generated_blocks(&mut self, forced_blocks: &[BlockIdentity]) -> Result<u64, PublishRefusal> {
        let mut rewritten: BTreeSet<BlockIdentity> = BTreeSet::new();
        let mut wave: BTreeSet<BlockIdentity> = (0..self.geometry.file_tree_nodes_per_publish())
            .map(|node| BlockIdentity::FileTreeNode { node })
            .chain(forced_blocks.iter().copied())
            .chain(std::iter::once(BlockIdentity::RecordTreeRoot))
            .collect();
        let mut dirty_slots_already_seen = 0;
        let mut completed_waves = 0u64;
        loop {
            let newly_dirty_absolute: Vec<u64> = self.dirty_slot_offsets[dirty_slots_already_seen..]
                .iter()
                .map(|offset| offset + UNIT_AREA_START_SLOT)
                .collect();
            dirty_slots_already_seen = self.dirty_slot_offsets.len();
            wave.extend(record_tree_blocks_dirtied_by(&self.geometry, &newly_dirty_absolute));
            wave.retain(|block| !rewritten.contains(block));
            if wave.is_empty() {
                break;
            }
            for block in std::mem::take(&mut wave) {
                self.rewrite_commit_generated_block(block)?;
                rewritten.insert(block);
            }
            completed_waves += 1;
        }
        assert!(completed_waves >= 1, "每次发布至少写一波（根恒写）");
        Ok(rewritten.len() as u64)
    }
}

// ---- 发布：回收、准入、根环、F ----
impl Pool {
    /// 合取 7：全空聚簇段数（记录口径）≥ `N₀`（D3（空间分配） 已定项 7 第 7 行）。
    pub(super) fn conjunct_seven_admits(&self) -> bool {
        self.record_empty_segment_count >= self.settings.floor_segments
    }

    pub(super) fn free_slots_outside_region(&self) -> u64 {
        let region_free = self
            .active_region_segment()
            .map_or(0, |region| self.segments[usize_of(region)].free_slots);
        self.free_slot_total - region_free
    }

    fn resident_base_slots(&self) -> u64 {
        self.settings.resident_base_multiplier
            * self.geometry.root_ring_slots_per_region
            * self.geometry.commit_generated_blocks_per_publish
    }

    /// 合取 1 的登记取法：这次发布预演之后，空闲可回收槽（排除 R）≥ 常驻底（登记 M-5，已登记差异 ④）。
    /// 可回收的在这次发布开头已经回收成空闲，所以只数空闲。
    fn conjunct_one_admits(&self) -> bool {
        self.free_slots_outside_region() >= self.resident_base_slots()
    }

    fn oldest_valid_root_txg_at(&self, publish_txg: u64) -> u64 {
        publish_txg.saturating_sub(self.geometry.retention_publishes())
    }

    /// 下一次发布开头的回收：释放代 ≤ max(F_生效, 环里最旧有效根) 的槽回到空闲。阈值只增不减，提前到这里做与在那次发布里做
    /// 得到同一个状态（被拒的写入不推 txg，下一次发布的 txg 仍是这一个）。顺带判到了时刻的 `Y1`、`Y2`。
    pub(super) fn reclaim_for_the_next_publish(&mut self) {
        let publish_txg = self.publishing_txg();
        let threshold = reclaim_threshold_at_publish(publish_txg, self.effective_floor_txg, &self.geometry);
        while let Some(&(release_txg, offset)) = self.reclaim_queue.front() {
            if release_txg > threshold {
                break;
            }
            self.reclaim_queue.pop_front();
            assert_eq!(self.slot_state(offset), SlotState::Released { release_txg }, "defer 队列里的槽还是已释放");
            self.set_slot_state(offset, SlotState::Free);
        }
        self.judge_due_truths(publish_txg);
    }

    /// `Y1`：删意图那次发布之后再过 3S − 1 次发布（其间抬过 F 就提前到 F 越过 R 里最后一个释放代那一次），R 按可开聚簇段口径
    /// 是一个可开的段（登记读法第 7 条）；在那次发布的回收之后、分配之前判。`Y2` 同一时刻判。
    fn judge_due_truths(&mut self, publish_txg: u64) {
        let pending = std::mem::take(&mut self.pending_truth_judgements);
        for judgement in pending {
            let is_due = publish_txg > judgement.end_txg
                && (publish_txg >= judgement.end_txg + self.geometry.retention_publishes()
                    || self.effective_floor_txg >= judgement.last_release_txg_in_region);
            if is_due {
                self.record_truth_judgement(judgement);
            } else {
                self.pending_truth_judgements.push(judgement);
            }
        }
    }

    fn record_truth_judgement(&mut self, judgement: PendingTruthJudgement) {
        let region_is_openable = self.membership_of_segment(judgement.region_segment).is_fully_free
            && self.active_region_segment() != Some(judgement.region_segment);
        let openable_now = self.openable_segment_count();
        let ended = &mut self.ended_intents[judgement.ended_intent_index];
        ended.region_became_openable = Some(region_is_openable);
        ended.openable_count_rose = Some(openable_now > judgement.openable_segments_at_issue);
    }

    /// 发布开头：回收、合取 7（只判前台写入类）、开事务。
    pub(super) fn begin_publish(&mut self, kind: PublishKind) -> Result<(), PublishRefusal> {
        self.reclaim_for_the_next_publish();
        if kind.is_judged_by_conjunct_seven() && !self.conjunct_seven_admits() {
            return Err(PublishRefusal::ConjunctSeven);
        }
        self.begin_transaction();
        Ok(())
    }

    /// 发布收尾：写提交内生块、合取 1、提交或回滚；提交之后推 txg、记根、记这一轮的计数。返回 `c_actual`。
    pub(super) fn finish_publish(&mut self, kind: PublishKind, forced_blocks: &[BlockIdentity]) -> Result<u64, PublishRefusal> {
        let commit_generated_blocks = match self.rewrite_commit_generated_blocks(forced_blocks) {
            Ok(blocks) => blocks,
            Err(refusal) => {
                self.roll_back_transaction();
                return Err(refusal);
            }
        };
        if kind.is_judged_by_conjunct_one() && !self.conjunct_one_admits() {
            self.roll_back_transaction();
            return Err(PublishRefusal::ConjunctOne);
        }
        self.commit_transaction();
        self.txg += 1;
        self.record_root(kind);
        if kind == PublishKind::Deletion {
            let metrics = &mut self.round_metrics;
            metrics.deletion_publishes += 1;
            metrics.deletion_commit_generated_block_sum += commit_generated_blocks;
            metrics.deletion_commit_generated_block_peak =
                metrics.deletion_commit_generated_block_peak.max(commit_generated_blocks);
        }
        self.sample_segment_counts();
        Ok(commit_generated_blocks)
    }

    pub(super) fn sample_segment_counts(&mut self) {
        let fully_free = self.fully_free_segments.len() as u64;
        let openable = self.openable_segment_count();
        let metrics = &mut self.round_metrics;
        metrics.lowest_record_empty_segments = metrics.lowest_record_empty_segments.min(self.record_empty_segment_count);
        metrics.lowest_fully_free_segments = metrics.lowest_fully_free_segments.min(fully_free);
        metrics.lowest_openable_segments = metrics.lowest_openable_segments.min(openable);
    }

    /// 根环：每次发布一条根；非空根进非空名单；空发布推进进行中的抬 F，连发够数之后 F 生效（登记 M-6 ①）。
    fn record_root(&mut self, kind: PublishKind) {
        if kind.is_non_empty_root() {
            self.non_empty_root_txgs_in_ring.push_back(self.txg);
            self.last_non_empty_root_txg = self.txg;
        }
        let oldest_valid_root_txg = self.oldest_valid_root_txg_at(self.txg);
        while self.non_empty_root_txgs_in_ring.front().is_some_and(|txg| *txg < oldest_valid_root_txg) {
            self.non_empty_root_txgs_in_ring.pop_front();
        }
        if kind == PublishKind::Empty {
            if let Some((target, empty_publishes_done)) = self.pending_floor_raise {
                let empty_publishes_done = empty_publishes_done + 1;
                if empty_publishes_done == EMPTY_PUBLISHES_FOR_FLOOR_TO_TAKE_EFFECT {
                    self.effective_floor_txg = self.effective_floor_txg.max(target);
                    self.pending_floor_raise = None;
                } else {
                    self.pending_floor_raise = Some((target, empty_publishes_done));
                }
            }
        }
    }
}

// ---- 各类发布 ----
impl Pool {
    /// 前台写入类：写一个 `unit_count` 个单元的新文件。用户数据找不到落点按合取 1 拒（可用不够需求）。
    pub(super) fn publish_new_file(&mut self, unit_count: u64) -> Result<Vec<u32>, PublishRefusal> {
        self.begin_publish(PublishKind::ForegroundWrite)?;
        let txg = self.publishing_txg();
        let mut created_units = Vec::new();
        for _unit_of_the_file in 0..unit_count {
            let Some(even_offset) = self.lowest_user_data_pair() else {
                self.roll_back_transaction();
                return Err(PublishRefusal::ConjunctOne);
            };
            let unit = self.create_unit(even_offset, txg);
            self.allocate_slot(even_offset, SlotOwner::UserDataUnit { unit }, txg);
            self.allocate_slot(even_offset + 1, SlotOwner::UserDataUnit { unit }, txg);
            created_units.push(unit);
        }
        self.finish_publish(PublishKind::ForegroundWrite, &[])?;
        self.round_metrics.foreground_writes += 1;
        Ok(created_units)
    }

    pub(super) fn release_unit_slots(&mut self, unit: u32) {
        let first_slot_offset = self.units[usize_of(u64::from(unit))].first_slot_offset;
        self.release_slot(first_slot_offset);
        self.release_slot(first_slot_offset + 1);
    }

    /// 删除类：头一侧删掉这几个单元；没有快照钉着的当场释放，钉着的进 deadlist、不进 defer（D3（空间分配） 已定项 7）。
    pub(super) fn publish_head_deletion(&mut self, units: &[u32]) -> Result<u64, PublishRefusal> {
        self.begin_publish(PublishKind::Deletion)?;
        for unit in units.iter().copied() {
            let mut record = self.units[usize_of(u64::from(unit))];
            assert!(record.is_alive_in_head, "删的是头一侧还活着的单元");
            record.is_alive_in_head = false;
            if record.snapshot_pins == 0 {
                self.release_unit_slots(unit);
                record.is_released = true;
            }
            self.set_unit_record(unit, record);
        }
        self.finish_publish(PublishKind::Deletion, &[])
    }

    /// 删除类：销毁快照的一批（登记 M-8）：放掉这几个单元上这个快照的钉；没人钉、头一侧也删了的释放；这个快照钉住的文件树节点一起释放。
    pub(super) fn publish_snapshot_destruction_batch(
        &mut self,
        units: &[u32],
        pinned_file_tree_offsets: &[u64],
    ) -> Result<u64, PublishRefusal> {
        self.begin_publish(PublishKind::Deletion)?;
        for unit in units.iter().copied() {
            let mut record = self.units[usize_of(u64::from(unit))];
            assert!(record.snapshot_pins > 0, "销毁的快照钉着它");
            record.snapshot_pins -= 1;
            if record.snapshot_pins == 0 && !record.is_alive_in_head {
                self.release_unit_slots(unit);
                record.is_released = true;
            }
            self.set_unit_record(unit, record);
        }
        for offset in pinned_file_tree_offsets.iter().copied() {
            assert_eq!(
                self.slot_state(offset).live_owner(),
                Some(SlotOwner::SnapshotPinnedFileTreeNode),
                "快照钉住的文件树节点还在"
            );
            self.release_slot(offset);
        }
        self.finish_publish(PublishKind::Deletion, &[])
    }

    pub(super) fn publish_empty(&mut self) -> Result<u64, PublishRefusal> {
        self.begin_publish(PublishKind::Empty)?;
        let blocks = self.finish_publish(PublishKind::Empty, &[])?;
        self.round_metrics.empty_publishes += 1;
        Ok(blocks)
    }

    /// 建快照（登记 M-8）：钉住那一刻全部活的用户数据单元与这次发布写下的那 `c − b` 个文件树节点（不钉分配记录树）。
    pub(super) fn publish_snapshot_creation(&mut self) -> Result<(Vec<u32>, Vec<u64>), PublishRefusal> {
        self.begin_publish(PublishKind::SnapshotCreation)?;
        let alive_units: Vec<u32> = (0..self.units.len())
            .filter(|unit| self.units[*unit].is_alive_in_head)
            .map(|unit| u32::try_from(unit).expect("单元数装得进 u32"))
            .collect();
        for unit in alive_units.iter().copied() {
            let mut record = self.units[usize_of(u64::from(unit))];
            record.snapshot_pins += 1;
            self.set_unit_record(unit, record);
        }
        self.finish_publish(PublishKind::SnapshotCreation, &[])?;
        let pinned_file_tree_offsets: Vec<u64> = self.file_tree_node_slots.iter().flatten().copied().collect();
        for offset in pinned_file_tree_offsets.iter().copied() {
            let SlotState::Live { allocation_txg, .. } = self.slot_state(offset) else {
                panic!("文件树节点的现行版本是已分配的");
            };
            self.set_slot_state(offset, SlotState::Live { owner: SlotOwner::SnapshotPinnedFileTreeNode, allocation_txg });
        }
        Ok((alive_units, pinned_file_tree_offsets))
    }

    /// 抬 F（登记 M-6 ①）：目标是环里第 4 新的非空根；F 已经不低于它就跳过；否则连发 2 次空发布，第 2 次之后生效。
    /// 空发布要的槽找不到一个记「删除发不出去」那一栏的空发布一列（登记 M-5）。
    pub(super) fn raise_floor(&mut self) {
        let oldest_valid_root_txg = self.oldest_valid_root_txg_at(self.txg);
        let target = floor_raise_target(&self.non_empty_root_txgs_in_ring, oldest_valid_root_txg);
        if self.effective_floor_txg >= target {
            return;
        }
        self.pending_floor_raise = Some((target, 0));
        for _empty_publish in 0..EMPTY_PUBLISHES_FOR_FLOOR_TO_TAKE_EFFECT {
            if self.publish_empty().is_err() {
                self.round_metrics.empty_publish_failures += 1;
                return;
            }
        }
    }
}

/// 一批整理发布之后意图怎样了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BatchOutcome {
    /// 一个都没搬成、R 里还有可搬的（没有目的地）：没发布，意图留着（暂停）。
    NothingMoved,
    /// 发了一批；意图结束了的话带它在 `ended_intents` 里的下标。
    Published { ended_intent_index: Option<usize> },
}
