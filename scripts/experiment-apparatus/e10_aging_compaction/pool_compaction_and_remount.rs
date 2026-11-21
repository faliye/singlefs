//! 池的整理：选段、整理入口、一批整理的发布、意图的结束，以及新建与重新挂载和计数核对。

use super::*;

// ---- 整理（自建 GC，登记 M-7）：选段、签发、一批、收、两种停机、甲与丁 ----
impl Pool {
    /// greedy 选段（`compaction_plan.rs` 的 `choose_region`）：资格是不在这次挂载开过的段里、不是活意图的 R、不含钉住槽、
    /// 至少一个未释放记录；活槽最少的先，同分取起点最小。
    pub(super) fn choose_region_greedy(&self) -> Option<u64> {
        let region = self.active_region_segment();
        let mut chosen: Option<(u64, u64)> = None;
        for segment in 0..self.geometry.segment_count() {
            let counters = &self.segments[usize_of(segment)];
            let opened_this_mount = self.is_opened_this_mount[usize_of(segment)];
            let is_eligible = !opened_this_mount && counters.pinned_live_slots == 0 && counters.live_slots > 0;
            if !is_eligible || Some(segment) == region {
                continue;
            }
            let live_slots = counters.live_slots;
            let is_better = chosen.is_none_or(|(best_live_slots, _best_segment)| live_slots < best_live_slots);
            if is_better {
                chosen = Some((live_slots, segment));
            }
        }
        chosen.map(|(_live_slots, segment)| segment)
    }

    /// 停乙的签发前判：R 外合格段的空闲两槽数 ≥ R 里可搬的用户数据单元数（D26（后台整理与放置回收） 已定项 1 第 2 条）。
    pub(super) fn pre_issue_check_passes(&self, region_segment: u64) -> bool {
        self.compaction_destination_free_pairs_outside_region(region_segment)
            >= self.segments[usize_of(region_segment)].live_user_units
    }

    /// 三个起跑水位任一越线（登记 M-7 的登记取法）：全空段数（记录口径）< `N₀` + 2；runs ≥ L ÷ 4；不可回收槽 ≥ 单元区的 10%。
    fn any_start_watermark_crossed(&self) -> bool {
        let empty_segments_are_low = self.record_empty_segment_count < self.settings.floor_segments + 2;
        let runs_are_high = self.free_runs >= self.live_user_unit_total / 4;
        let unrecoverable_slots = self.released_slot_total + 2 * self.snapshot_only_unit_total;
        let unrecoverable_is_high = unrecoverable_slots >= self.geometry.unit_area_slots() / 10;
        empty_segments_are_low || runs_are_high || unrecoverable_is_high
    }

    /// R 里分配代 ≤ c0 的未释放记录：(总数, 其中钉住的)。
    fn region_live_records_at_or_before(&self, region_segment: u64, allocation_txg_at_issue: u64) -> (u64, u64) {
        let segment_start = region_segment * CLUSTER_SEGMENT_SLOTS;
        let mut live_records = 0;
        let mut pinned_records = 0;
        for offset in segment_start..segment_start + CLUSTER_SEGMENT_SLOTS {
            if let SlotState::Live { owner, allocation_txg } = self.slot_state(offset) {
                if allocation_txg <= allocation_txg_at_issue {
                    live_records += 1;
                    if owner.is_pinned() {
                        pinned_records += 1;
                    }
                }
            }
        }
        (live_records, pinned_records)
    }

    /// 一次调用整理入口甲：预算 `B_gc`；返回签了几条意图。
    pub(super) fn call_compaction_entry(&mut self) -> u64 {
        let publishes_since_last_non_empty_root = self.txg - self.last_non_empty_root_txg;
        let budget = compaction_call_publish_budget(self.geometry.root_ring_slots_per_region, publishes_since_last_non_empty_root);
        self.run_compaction(budget)
    }

    /// 丁：前台写入类或删除类发布成功之后，起跑水位有一个越线就跑至多一批（登记 M-7；每次当一次预算为 1 的调用）。
    pub(super) fn run_compaction_after_publish(&mut self) {
        if self.settings.is_compaction_enabled && self.any_start_watermark_crossed() {
            self.run_compaction(1);
        }
    }

    pub(super) fn run_compaction(&mut self, publish_budget: u64) -> u64 {
        let mut publishes_used = 0;
        let mut intents_signed = 0;
        while publishes_used < publish_budget {
            let new_region = if self.active_intent.is_some() {
                None
            } else {
                let Some(region) = self.choose_region_greedy() else {
                    break;
                };
                let must_check_holes = self.settings.stop_arm == StopArm::LocalRegionPredicate;
                if must_check_holes && !self.pre_issue_check_passes(region) {
                    break;
                }
                Some(region)
            };
            let outcome = match self.publish_compaction_batch(new_region) {
                Ok(outcome) => outcome,
                Err(_refusal) => break,
            };
            let BatchOutcome::Published { ended_intent_index } = outcome else {
                break;
            };
            publishes_used += 1;
            intents_signed += u64::from(new_region.is_some());
            if let Some(index) = ended_intent_index {
                let global_difference_is_positive = self.ended_intents[index].global_difference_is_positive;
                let must_stop = match self.settings.stop_arm {
                    StopArm::LocalRegionPredicate => !global_difference_is_positive,
                    StopArm::GlobalDifference => global_difference_is_positive,
                };
                if must_stop {
                    break;
                }
            }
        }
        intents_signed
    }

    /// 一批（登记 M-7）：签发（`new_region` 给了的话，随这一批一起发）；按槽升序搬 R 里至多 K 个可搬单元到目的地，
    /// R 里的提交内生块随这一批重写；之后 R 里没有分配代 ≤ c0 的未释放记录就同一次发布里删意图，只剩钉住的就丢意图。
    pub(super) fn publish_compaction_batch(&mut self, new_region: Option<u64>) -> Result<BatchOutcome, PublishRefusal> {
        if let Some(region_segment) = new_region {
            self.active_intent = Some(CompactionIntent {
                region_segment,
                allocation_txg_at_issue: self.txg,
                record_empty_segments_at_issue: 0,
                openable_segments_at_issue: 0,
            });
        }
        let intent = self.active_intent.expect("一批属于一条活意图");
        let region = intent.region_segment;
        if let Err(refusal) = self.begin_publish(PublishKind::CompactionBatch) {
            self.forget_intent_issued_by_a_batch_that_did_not_publish(new_region);
            return Err(refusal);
        }
        let segment_start = region * CLUSTER_SEGMENT_SLOTS;
        let user_units_in_region: Vec<u32> = (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .step_by(2)
            .filter_map(|even_offset| match self.slot_state(even_offset).live_owner() {
                Some(SlotOwner::UserDataUnit { unit }) => Some(unit),
                Some(SlotOwner::InstanceTable | SlotOwner::SnapshotPinnedFileTreeNode | SlotOwner::CommitGeneratedBlock { .. })
                | None => None,
            })
            .collect();
        let mut units_moved = 0;
        for unit in user_units_in_region.iter().copied().take(COMPACTION_BATCH_UNITS) {
            let Some(destination) = self.lowest_compaction_destination_pair() else {
                break;
            };
            let mut record = self.units[usize_of(u64::from(unit))];
            // 目的地那条分配记录的分配代写单元的诞生代（D26（后台整理与放置回收） 已定项 5 第 2 条）。
            self.allocate_slot(destination, SlotOwner::UserDataUnit { unit }, record.birth_txg);
            self.allocate_slot(destination + 1, SlotOwner::UserDataUnit { unit }, record.birth_txg);
            self.release_unit_slots(unit);
            record.first_slot_offset = destination;
            self.set_unit_record(unit, record);
            units_moved += 1;
        }
        let forced_blocks: Vec<BlockIdentity> = (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .filter_map(|offset| match self.slot_state(offset).live_owner() {
                Some(SlotOwner::CommitGeneratedBlock { block }) => Some(block),
                Some(SlotOwner::InstanceTable | SlotOwner::SnapshotPinnedFileTreeNode | SlotOwner::UserDataUnit { .. })
                | None => None,
            })
            .collect();
        if units_moved == 0 && forced_blocks.is_empty() {
            let (live_records, pinned_records) =
                self.region_live_records_at_or_before(region, intent.allocation_txg_at_issue);
            let region_can_end = live_records == pinned_records;
            if !region_can_end {
                self.roll_back_transaction();
                self.forget_intent_issued_by_a_batch_that_did_not_publish(new_region);
                return Ok(BatchOutcome::NothingMoved);
            }
        }
        if let Err(refusal) = self.finish_publish(PublishKind::CompactionBatch, &forced_blocks) {
            self.forget_intent_issued_by_a_batch_that_did_not_publish(new_region);
            return Err(refusal);
        }
        self.round_metrics.compaction_batches += 1;
        self.round_metrics.units_moved += units_moved;
        if new_region.is_some() {
            let record_empty_segments_at_issue = self.record_empty_segment_count;
            let openable_segments_at_issue = self.openable_segment_count();
            let issued = self.active_intent.as_mut().expect("刚签发的意图");
            issued.record_empty_segments_at_issue = record_empty_segments_at_issue;
            issued.openable_segments_at_issue = openable_segments_at_issue;
        }
        let intent_after_the_batch = self.active_intent.expect("一批之后意图还在");
        let (live_records, pinned_records) = self.region_live_records_at_or_before(region, intent_after_the_batch.allocation_txg_at_issue);
        let ended_intent_index = if live_records == 0 {
            Some(self.end_intent(intent_after_the_batch, true))
        } else if live_records == pinned_records {
            Some(self.end_intent(intent_after_the_batch, false))
        } else {
            None
        };
        Ok(BatchOutcome::Published { ended_intent_index })
    }

    fn forget_intent_issued_by_a_batch_that_did_not_publish(&mut self, new_region: Option<u64>) {
        if new_region.is_some() {
            self.active_intent = None;
        }
    }

    /// 删意图（或丢意图），记 `P_R`、`P_G`，排上 `Y1`、`Y2` 的判定（登记读法第 7 条）。
    fn end_intent(&mut self, intent: CompactionIntent, is_completed: bool) -> usize {
        let region = intent.region_segment;
        self.active_intent = None;
        let region_is_record_empty = self.membership_of_segment(region).is_record_empty;
        let global_difference = i128::from(self.record_empty_segment_count) - i128::from(intent.record_empty_segments_at_issue);
        let global_difference_is_positive = global_difference >= 1;
        let segment_start = region * CLUSTER_SEGMENT_SLOTS;
        let last_release_txg_in_region = (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
            .filter_map(|offset| match self.slot_state(offset) {
                SlotState::Released { release_txg } => Some(release_txg),
                SlotState::Free | SlotState::Live { .. } => None,
            })
            .max()
            .unwrap_or(0);
        self.ended_intents.push(EndedIntent {
            round: self.current_round,
            is_completed,
            region_is_record_empty,
            global_difference_is_positive,
            region_became_openable: None,
            openable_count_rose: None,
        });
        let ended_intent_index = self.ended_intents.len() - 1;
        self.pending_truth_judgements.push(PendingTruthJudgement {
            ended_intent_index,
            region_segment: region,
            end_txg: self.txg,
            last_release_txg_in_region,
            openable_segments_at_issue: intent.openable_segments_at_issue,
        });
        if is_completed {
            self.round_metrics.intents_completed += 1;
        } else {
            self.round_metrics.intents_discarded += 1;
        }
        ended_intent_index
    }
}

// ---- mkfs、重挂与守恒自检 ----
impl Pool {
    /// mkfs（txg 0）：实例表两槽钉住在单元区起点；第一次发布的提交内生块从最低的可开聚簇段 bump；之后挂载一次。
    pub(super) fn made_by_make_filesystem(geometry: PoolGeometry, settings: PoolSettings) -> Self {
        let mut pool = Self::blank(geometry, settings);
        pool.begin_transaction();
        pool.allocate_slot(0, SlotOwner::InstanceTable, 0);
        pool.allocate_slot(1, SlotOwner::InstanceTable, 0);
        pool.is_making_filesystem = true;
        pool.rewrite_commit_generated_blocks(&[]).expect("空池上 mkfs 那一版的块放得下");
        pool.is_making_filesystem = false;
        pool.commit_transaction();
        assert!(pool.reclaim_queue.is_empty(), "mkfs 那一版没有旧版本可释放");
        pool.remount();
        pool
    }

    /// 重挂：「这次挂载开过的段」清空、bump 游标作废，下一块提交内生块开新段（登记 M-4）。
    pub(super) fn remount(&mut self) {
        assert!(!self.is_transaction_open, "重挂不在事务里");
        let opened = std::mem::take(&mut self.segments_opened_this_mount);
        for segment in opened {
            self.set_segment_opened_this_mount(segment, false);
        }
        self.open_cursor = None;
    }

    /// 全扫重算，与增量计数逐项比（作废条款 V4）；不等就是装置错，当场停。
    pub(super) fn verify_incremental_counters_against_a_full_scan(&self) {
        let mut segments = vec![SegmentCounters::default(); usize_of(self.geometry.segment_count())];
        let mut free_runs = 0;
        let mut previous_is_free = false;
        let mut live_user_units = 0;
        for offset in 0..self.geometry.unit_area_slots() {
            let state = self.slot_state(offset);
            let counters = &mut segments[usize_of(Self::segment_of_offset(offset))];
            match state {
                SlotState::Free => counters.free_slots += 1,
                SlotState::Live { owner, .. } => {
                    counters.live_slots += 1;
                    if owner.is_pinned() {
                        counters.pinned_live_slots += 1;
                    }
                    if owner.is_user_data() && offset % 2 == 0 {
                        counters.live_user_units += 1;
                        live_user_units += 1;
                    }
                }
                SlotState::Released { .. } => counters.released_slots += 1,
            }
            if offset % 2 == 1 && self.is_pair_free(offset - 1) {
                counters.free_pairs += 1;
            }
            if is_free_for_runs(state) && !previous_is_free {
                free_runs += 1;
            }
            previous_is_free = is_free_for_runs(state);
        }
        assert_eq!(segments, self.segments, "逐段计数与全扫重算对得上（V4）");
        assert_eq!(free_runs, self.free_runs, "runs 增量与全扫重算对得上（V4）");
        assert_eq!(live_user_units, self.live_user_unit_total, "活的用户数据单元数对得上（V4）");
        let free_total: u64 = segments.iter().map(|counters| counters.free_slots).sum();
        let live_total: u64 = segments.iter().map(|counters| counters.live_slots).sum();
        let released_total: u64 = segments.iter().map(|counters| counters.released_slots).sum();
        assert_eq!(free_total + live_total + released_total, self.geometry.unit_area_slots(), "三态守恒（V4）");
        assert_eq!((free_total, released_total), (self.free_slot_total, self.released_slot_total), "全池计数（V4）");
        let record_empty = segments.iter().filter(|counters| counters.live_slots == 0).count() as u64;
        assert_eq!(record_empty, self.record_empty_segment_count, "记录口径全空段数（V4）");
        let fully_free = segments.iter().filter(|counters| counters.free_slots == CLUSTER_SEGMENT_SLOTS).count() as u64;
        assert_eq!(fully_free, self.fully_free_segments.len() as u64, "占用口径全空段数（V4）");
        let destination_pairs: u64 = (0..self.geometry.segment_count())
            .filter(|segment| self.membership_of_segment(*segment).is_compaction_destination)
            .map(|segment| segments[usize_of(segment)].free_pairs)
            .sum();
        assert_eq!(destination_pairs, self.compaction_destination_free_pairs, "整理目的地的空闲两槽数（V4）");
        let user_placement: BTreeSet<u64> = (0..self.geometry.segment_count())
            .filter(|segment| self.membership_of_segment(*segment).is_user_placement_candidate)
            .collect();
        assert_eq!(user_placement, self.user_placement_segments, "用户数据候选段集合（V4）");
        let with_a_free_slot: BTreeSet<u64> = (0..self.geometry.segment_count())
            .filter(|segment| segments[usize_of(*segment)].free_slots > 0)
            .collect();
        assert_eq!(with_a_free_slot, self.segments_with_a_free_slot, "有空闲槽的段集合（V4）");
        let snapshot_only = self.units.iter().filter(|record| record.is_referenced_only_by_snapshots()).count() as u64;
        assert_eq!(snapshot_only, self.snapshot_only_unit_total, "只被快照引用的单元数（V4）");
    }
}
