//! 拒绝原因、一次请求与抬升的记录、挂载的运行与会话、池设置，以及会话的请求流程。

use super::*;

// ── 一次请求的准入循环与四条臂 ──────────────────────────────────────────────────────────

/// 一次请求被拒的五栏拒因（登记第一节「被拒次数」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RejectionReason {
    AdmissionBudgetUsedUp,
    FloorAlreadyAtTheCeiling,
    FloorRaiseSequenceFoundNoPlacement,
    EmptyPublishFoundNoPlacement,
    FloorRaiseReportedItsOwnError,
}

pub(super) const REJECTION_REASON_COUNT: usize = 5;

impl RejectionReason {
    pub(super) fn index(self) -> usize {
        match self {
            Self::AdmissionBudgetUsedUp => 0,
            Self::FloorAlreadyAtTheCeiling => 1,
            Self::FloorRaiseSequenceFoundNoPlacement => 2,
            Self::EmptyPublishFoundNoPlacement => 3,
            Self::FloorRaiseReportedItsOwnError => 4,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::AdmissionBudgetUsedUp => "budget",
            Self::FloorAlreadyAtTheCeiling => "floor_at_ceiling",
            Self::FloorRaiseSequenceFoundNoPlacement => "raise_no_placement",
            Self::EmptyPublishFoundNoPlacement => "empty_no_placement",
            Self::FloorRaiseReportedItsOwnError => "own_error",
        }
    }
}

pub(super) enum StepOutcome {
    Pushed,
    Stopped(RejectionReason),
}

/// 一次抬 F 的读数：F 旧 → F 新，真值上限 C真，越过的真值非空根数，少掉的状态数 L。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RaiseRecord {
    pub(super) old_floor: u64,
    pub(super) new_floor: u64,
    pub(super) truth_ceiling: u64,
    pub(super) crossed_non_empty_roots: u32,
    pub(super) states_lost: usize,
    pub(super) publishes: usize,
}

impl RaiseRecord {
    pub(super) fn floor_is_above_the_truth_ceiling(self) -> bool {
        self.new_floor > self.truth_ceiling
    }
}

/// 一次请求结束时的读数（登记第八节「轨迹」：一步 = 一次请求）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RequestRecord {
    pub(super) accepted: bool,
    pub(super) rejection: Option<RejectionReason>,
    pub(super) was_judged_short: bool,
    pub(super) empty_publishes: u32,
    pub(super) empty_publish_blocks: i64,
    pub(super) publishes_in_the_admission: usize,
    pub(super) crossed_non_empty_roots: u32,
    pub(super) states_lost: usize,
    pub(super) floor_after: u64,
    pub(super) free_above_the_reserve_after: i64,
    pub(super) candidate_states_after: usize,
}

/// 被测挂载的全部读数。
#[derive(Clone, Debug)]
pub(super) struct MountRun {
    pub(super) requests: Vec<RequestRecord>,
    pub(super) raises: Vec<RaiseRecord>,
    pub(super) candidate_states_at_the_start: usize,
    pub(super) cap_active_passes: u32,
    pub(super) cap_blocked_passes: u32,
    pub(super) empty_publish_refusals: u32,
    pub(super) raise_refused_after_an_empty_publish: u32,
    pub(super) empty_publish_bound_exhausted: u32,
    pub(super) lowest_free_blocks_in_the_first_raise: Option<i64>,
    pub(super) trace: Vec<String>,
}

/// 一次请求里累计的东西，请求结束时折进 `RequestRecord`。
#[derive(Clone, Copy, Debug)]
pub(super) struct RequestTally {
    pub(super) pushed_within_the_budget: usize,
    pub(super) empty_publishes: u32,
    pub(super) empty_publish_blocks: i64,
    pub(super) raise_publishes: usize,
    pub(super) pushed_an_empty_publish: bool,
}

pub(super) struct Session {
    pub(super) pool: MountedPool,
    pub(super) arm: Arm,
    pub(super) controls: Controls,
    pub(super) ledger: MountRun,
    pub(super) request_index: usize,
    pub(super) pushed_the_seeded_empty_publish: bool,
    pub(super) tracing: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PoolSettings {
    pub(super) geometry: RingGeometry,
    pub(super) metadata_blocks: i64,
    pub(super) reserve_reading: ReserveReading,
    pub(super) fullness: PoolFullness,
}

impl PoolSettings {
    pub(super) fn reserve_blocks(self) -> i64 {
        self.reserve_reading.reserve_blocks(self.metadata_blocks)
    }

    pub(super) fn free_blocks_after_the_warm_up(self) -> i64 {
        self.fullness.free_blocks_after_the_warm_up(self.geometry, self.metadata_blocks, self.reserve_blocks())
    }
}

impl Session {
    pub(super) fn merges_all_zero_slots(&self) -> bool {
        self.arm.all_zero_slot == AllZeroSlotDisposition::Merged
    }

    /// 这一臂的准入抬 F 取哪个上限：不并取 C0，其余取并了读坏槽的 C并。
    fn ceiling_of_the_arm(&self, reading: CeilingReading) -> u64 {
        match self.arm.stuck_ceiling {
            StuckCeilingDisposition::DoNotMergeTheBadSlots => reading.unmerged,
            StuckCeilingDisposition::AsToday | StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget => {
                reading.merged
            }
        }
    }

    fn trace_line(&mut self, text: String) {
        if self.tracing {
            self.ledger.trace.push(text);
        }
    }

    /// 封顶起作用与卡住的次数（登记第六节 Q1-6）：不并臂不封顶，不数。
    pub(super) fn note_the_cap(&mut self, reading: CeilingReading) {
        let (txg, floor, free_blocks) = (self.pool.current_txg, self.pool.floor_in_the_current_version, self.pool.space.free_blocks);
        self.trace_line(format!(
            "ceiling_pass request={} txg={txg} floor={floor} c0={} c_merged={} free_blocks={free_blocks}",
            self.request_index, reading.unmerged, reading.merged
        ));
        if self.arm.stuck_ceiling == StuckCeilingDisposition::DoNotMergeTheBadSlots {
            return;
        }
        if reading.merged < reading.unmerged {
            self.ledger.cap_active_passes += 1;
        }
        if reading.merged <= self.pool.floor_in_the_current_version && reading.unmerged > self.pool.floor_in_the_current_version {
            self.ledger.cap_blocked_passes += 1;
        }
    }

    pub(super) fn publish_an_empty_root(&mut self) {
        let (txg, state_number, floor, blocks) =
            (self.pool.current_txg + 1, self.pool.current_state_number, self.pool.floor_in_the_current_version, self.pool.metadata_blocks);
        self.pool.publish_root(txg, state_number, blocks, blocks, floor, true);
    }

    /// 推一次不抬 F 的空发布：带现行 F、txg 加一；空间从空闲块里取落点，不判空间准入式子（登记第五节「推空两条臂的空间从哪出」）。
    fn push_one_empty_publish(&mut self, tally: &mut RequestTally) -> Result<(), RejectionReason> {
        // 推空的空发布只要求空闲 ≥ c（取不到落点就被拒），与抬 F 那一串同一个来源。
        if self.pool.space.free_blocks < self.pool.metadata_blocks {
            self.ledger.empty_publish_refusals += 1;
            return Err(RejectionReason::EmptyPublishFoundNoPlacement);
        }
        self.publish_an_empty_root();
        tally.empty_publishes += 1;
        tally.empty_publish_blocks += self.pool.metadata_blocks;
        tally.pushed_an_empty_publish = true;
        Ok(())
    }

    /// 抬 F 那一串（登记第五节「模型·算上限那一遍」与冻结树 `mount.rs:3066`）：第二遍算上限、回收门槛里的环里最旧有效根、
    /// 回收扣到生效、整串预演、真发、放开。`enforce_the_ceiling` 为假时不判「新 F 不高于第二遍上限」（阳性对照把目标抬过上限）。
    fn raise_the_floor_to(&mut self, new_floor: u64, enforce_the_ceiling: bool) -> Result<usize, RejectionReason> {
        let second_reading = self.pool.ceiling_pass(self.merges_all_zero_slots()).map_err(|_| RejectionReason::FloorRaiseReportedItsOwnError)?;
        if enforce_the_ceiling && new_floor > self.ceiling_of_the_arm(second_reading) {
            return Err(RejectionReason::FloorRaiseReportedItsOwnError);
        }
        let oldest_valid_root = self.pool.oldest_valid_root_for_raising_the_floor();
        let planned_txgs = txgs_of_the_publishes_carrying_the_floor_to_every_device(self.pool.current_txg);
        let old_floor = self.pool.floor_in_the_current_version;
        let (truth_ceiling, crossed_non_empty_roots) = truth_ceiling_and_crossed_roots(&self.pool.truth_ring, old_floor, new_floor);
        let candidates_before = candidate_states_at_the_floor(&self.pool.truth_ring, old_floor);
        let candidates_after = candidate_states_at_the_floor(&self.pool.truth_ring, new_floor);
        let states_lost = candidates_before.min(NEWEST_DISTINCT_STATES_THE_CEILING_KEEPS).saturating_sub(candidates_after);
        let space_before_the_raise = self.pool.space.clone();
        self.pool.space.reclaim_up_to(reclaim_floor(new_floor, oldest_valid_root), ReclaimedReuse::HeldUntilTheFloorTakesEffect);
        let mut rehearsal = self.pool.clone();
        for planned_txg in &planned_txgs {
            if rehearsal.space.free_blocks < rehearsal.metadata_blocks {
                self.pool.space = space_before_the_raise;
                return Err(RejectionReason::FloorRaiseSequenceFoundNoPlacement);
            }
            let (state_number, blocks) = (rehearsal.current_state_number, rehearsal.metadata_blocks);
            rehearsal.publish_root(*planned_txg, state_number, blocks, blocks, new_floor, false);
        }
        let mut lowest_free_blocks = self.pool.space.free_blocks;
        for planned_txg in &planned_txgs {
            let (state_number, blocks) = (self.pool.current_state_number, self.pool.metadata_blocks);
            self.pool.publish_root(*planned_txg, state_number, blocks, blocks, new_floor, true);
            lowest_free_blocks = lowest_free_blocks.min(self.pool.space.free_blocks);
        }
        self.pool.space.release_the_holds();
        self.pool.floor_in_the_current_version = new_floor;
        self.pool.clock.first_floor_raise_done = true;
        if self.ledger.raises.is_empty() {
            self.ledger.lowest_free_blocks_in_the_first_raise = Some(lowest_free_blocks);
        }
        self.ledger.raises.push(RaiseRecord {
            old_floor,
            new_floor,
            truth_ceiling,
            crossed_non_empty_roots,
            states_lost,
            publishes: planned_txgs.len(),
        });
        Ok(planned_txgs.len())
    }

    /// 准入里再推一串抬 F（冻结树 `mount.rs:2958` `push_one_floor_raise_within_the_admission_budget`）：先核预算（已推 + 这一串次数 + 1 ≤ B），
    /// 再算上限，不高于现行 F 就交回 `FloorAlreadyAtTheCeiling`，高于就抬到它。`reading_already_taken` 是推空臂在分流之前已经算过的第一遍。
    pub(super) fn push_one_floor_raise_within_the_budget(
        &mut self,
        tally: &mut RequestTally,
        reading_already_taken: Option<CeilingReading>,
    ) -> StepOutcome {
        let planned_publishes = txgs_of_the_publishes_carrying_the_floor_to_every_device(self.pool.current_txg).len();
        if tally.pushed_within_the_budget + planned_publishes + 1 > self.controls.publishes_per_admission_at_most {
            return StepOutcome::Stopped(RejectionReason::AdmissionBudgetUsedUp);
        }
        let reading = match reading_already_taken {
            Some(reading) => reading,
            None => match self.pool.ceiling_pass(self.merges_all_zero_slots()) {
                Ok(reading) => {
                    self.note_the_cap(reading);
                    reading
                }
                Err(_) => return StepOutcome::Stopped(RejectionReason::FloorRaiseReportedItsOwnError),
            },
        };
        let ceiling = self.ceiling_of_the_arm(reading);
        if ceiling <= self.pool.floor_in_the_current_version {
            return StepOutcome::Stopped(RejectionReason::FloorAlreadyAtTheCeiling);
        }
        let first_raise_of_the_mount = self.ledger.raises.is_empty();
        let (new_floor, enforce_the_ceiling) = if first_raise_of_the_mount && self.controls.first_raise_target_is_the_current_txg {
            (self.pool.current_txg, false)
        } else if let (true, Some(extra)) = (first_raise_of_the_mount, self.controls.first_raise_target_above_the_ceiling_by) {
            (ceiling + extra, false)
        } else {
            (ceiling, true)
        };
        let old_floor = self.pool.floor_in_the_current_version;
        match self.raise_the_floor_to(new_floor, enforce_the_ceiling) {
            Ok(publishes) => {
                tally.pushed_within_the_budget += publishes;
                tally.raise_publishes += publishes;
                self.trace_line(format!("raise {old_floor}->{new_floor} publishes={publishes} ceiling={ceiling}"));
                StepOutcome::Pushed
            }
            Err(RejectionReason::FloorRaiseSequenceFoundNoPlacement) => {
                if tally.pushed_an_empty_publish {
                    self.ledger.raise_refused_after_an_empty_publish += 1;
                }
                StepOutcome::Stopped(RejectionReason::FloorRaiseSequenceFoundNoPlacement)
            }
            Err(reason) => StepOutcome::Stopped(reason),
        }
    }

    /// 推空B：卡住时推一次不抬 F 的空发布，算进这次准入的已推次数；放不下这一次（已推 + 1 + 1 > B）或取不到落点就拒。
    fn push_one_empty_publish_within_the_budget(&mut self, tally: &mut RequestTally) -> StepOutcome {
        if tally.pushed_within_the_budget + 1 + 1 > self.controls.publishes_per_admission_at_most {
            return StepOutcome::Stopped(RejectionReason::AdmissionBudgetUsedUp);
        }
        match self.push_one_empty_publish(tally) {
            Ok(()) => {
                tally.pushed_within_the_budget += 1;
                self.trace_line(format!("empty push (within budget) txg={}", self.pool.current_txg));
                StepOutcome::Pushed
            }
            Err(reason) => StepOutcome::Stopped(reason),
        }
    }

    /// 推空W：推到卡住上限的那几个槽都被写掉为止，推空那几次不受 B 限、不算进已推次数；为了收得住，一次准入推空至多 3S 次。
    fn push_empty_publishes_until_the_ceiling_is_unstuck(&mut self, tally: &mut RequestTally) -> StepOutcome {
        let bound = usize::try_from(self.pool.geometry.ring_length_in_txgs()).expect("环长");
        let mut pushed_here = 0;
        loop {
            let reading = match self.pool.ceiling_pass(self.merges_all_zero_slots()) {
                Ok(reading) => reading,
                Err(_) => return StepOutcome::Stopped(RejectionReason::FloorRaiseReportedItsOwnError),
            };
            let floor = self.pool.floor_in_the_current_version;
            if !(reading.merged <= floor && reading.unmerged > floor) {
                return StepOutcome::Pushed;
            }
            if pushed_here >= bound {
                self.ledger.empty_publish_bound_exhausted += 1;
                return StepOutcome::Stopped(RejectionReason::AdmissionBudgetUsedUp);
            }
            if let Err(reason) = self.push_one_empty_publish(tally) {
                return StepOutcome::Stopped(reason);
            }
            pushed_here += 1;
            self.trace_line(format!("empty push (beyond budget) txg={}", self.pool.current_txg));
        }
    }

    /// 准入不够时按臂处置一步（登记第五节「Q1 四条臂」）。
    fn push_for_space(&mut self, tally: &mut RequestTally) -> StepOutcome {
        match self.arm.stuck_ceiling {
            StuckCeilingDisposition::AsToday | StuckCeilingDisposition::DoNotMergeTheBadSlots => self.push_one_floor_raise_within_the_budget(tally, None),
            StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget => {
                let reading = match self.pool.ceiling_pass(self.merges_all_zero_slots()) {
                    Ok(reading) => reading,
                    Err(_) => return StepOutcome::Stopped(RejectionReason::FloorRaiseReportedItsOwnError),
                };
                self.note_the_cap(reading);
                let floor = self.pool.floor_in_the_current_version;
                if reading.merged > floor {
                    return self.push_one_floor_raise_within_the_budget(tally, Some(reading));
                }
                if reading.unmerged > floor {
                    return match self.arm.stuck_ceiling {
                        StuckCeilingDisposition::PushEmptyBeyondBudget => self.push_empty_publishes_until_the_ceiling_is_unstuck(tally),
                        StuckCeilingDisposition::PushEmptyWithinBudget => self.push_one_empty_publish_within_the_budget(tally),
                        StuckCeilingDisposition::AsToday | StuckCeilingDisposition::DoNotMergeTheBadSlots => {
                            unreachable!("只有推空两臂走到这里")
                        }
                    };
                }
                StepOutcome::Stopped(RejectionReason::FloorAlreadyAtTheCeiling)
            }
        }
    }

    /// 一次用户覆盖写请求的准入循环（冻结树 `mounted_session.rs:423`）：发一次，被拒是空间不够就按臂推一步再判，推满或抬不动就交回。
    /// 循环：每一轮要么发成、要么推一步进下一轮、要么拒；推空B / 照今天 / 不并至多 B − 1 次推，推空W 的推空受 3S 的界；轮数有界。
    pub(super) fn run_one_request(&mut self) -> RequestRecord {
        let blocks_for_the_overwrite = DATA_BLOCKS_PER_OVERWRITE + self.pool.metadata_blocks;
        let raises_before = self.ledger.raises.len();
        let mut tally = RequestTally {
            pushed_within_the_budget: 0,
            empty_publishes: 0,
            empty_publish_blocks: 0,
            raise_publishes: 0,
            pushed_an_empty_publish: false,
        };
        let judged_short_throughout = self.controls.first_request_is_judged_short_throughout && self.request_index == 0;
        let mut was_judged_short = false;
        let mut rejection: Option<RejectionReason> = None;
        let mut accepted = false;
        let mut rounds = 0;
        while rounds < 1000 {
            rounds += 1;
            let is_enough = !judged_short_throughout && self.pool.free_blocks_above_the_reserve() >= blocks_for_the_overwrite;
            if is_enough {
                let (txg, state_number, floor) =
                    (self.pool.current_txg + 1, self.pool.current_state_number + 1, self.pool.floor_in_the_current_version);
                self.pool.publish_root(txg, state_number, blocks_for_the_overwrite, blocks_for_the_overwrite, floor, true);
                accepted = true;
                break;
            }
            was_judged_short = true;
            match self.push_for_space(&mut tally) {
                StepOutcome::Pushed => {}
                StepOutcome::Stopped(reason) => {
                    if reason == RejectionReason::FloorAlreadyAtTheCeiling
                        && self.controls.push_one_empty_publish_on_the_first_floor_already_at_the_ceiling
                        && !self.pushed_the_seeded_empty_publish
                    {
                        self.pushed_the_seeded_empty_publish = true;
                        self.publish_an_empty_root();
                        tally.empty_publishes += 1;
                        tally.empty_publish_blocks += self.pool.metadata_blocks;
                        tally.pushed_an_empty_publish = true;
                        continue;
                    }
                    rejection = Some(reason);
                    break;
                }
            }
        }
        assert!(accepted || rejection.is_some(), "准入循环在 1000 轮之内收住");
        let raises_in_this_request = &self.ledger.raises[raises_before..];
        let record = RequestRecord {
            accepted,
            rejection,
            was_judged_short,
            empty_publishes: tally.empty_publishes,
            empty_publish_blocks: tally.empty_publish_blocks,
            publishes_in_the_admission: usize::try_from(tally.empty_publishes).expect("次数") + tally.raise_publishes + usize::from(accepted),
            crossed_non_empty_roots: raises_in_this_request.iter().map(|raise| raise.crossed_non_empty_roots).sum(),
            states_lost: raises_in_this_request.iter().map(|raise| raise.states_lost).sum(),
            floor_after: self.pool.floor_in_the_current_version,
            free_above_the_reserve_after: self.pool.free_blocks_above_the_reserve(),
            candidate_states_after: candidate_states_at_the_floor(&self.pool.truth_ring, self.pool.floor_in_the_current_version),
        };
        self.trace_line(format!(
            "request {} accepted={} rejection={} txg={} floor={} free_above_reserve={}",
            self.request_index,
            record.accepted,
            record.rejection.map_or("none", RejectionReason::label),
            self.pool.current_txg,
            record.floor_after,
            record.free_above_the_reserve_after
        ));
        self.request_index += 1;
        record
    }
}
