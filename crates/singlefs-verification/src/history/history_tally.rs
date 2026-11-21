//! 一段历史跑下来的结局与计数：每种操作的计数、整段的结局、交出的步镜像与每一步的观测。

use super::*;

/// 一段历史怎么收尾。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HistoryEnding {
    /// 每一步都跑完，checker 一条违例都没有、没有 panic。
    Completed,
    /// 撞到「已知红」清单里第 `form` 条的形态。
    KnownRed {
        form: usize,
        observation: FailureObservation,
    },
    /// 清单外的失败。
    NewFinding {
        signature: FailureSignature,
        observation: FailureObservation,
    },
}

/// 某一类操作跑了几次、各落在哪种结局。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OperationTally {
    pub applied: u64,
    pub refused: u64,
    pub not_applicable: u64,
}

/// 跑过的历史的计数：证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryTally {
    pub histories: u64,
    pub histories_completed: u64,
    /// 「已知红」清单第几条 → 几段历史以它收尾。
    pub histories_ended_known_red: BTreeMap<usize, u64>,
    pub histories_ended_new_finding: u64,
    pub operations_by_kind: BTreeMap<HistoryOperationKind, OperationTally>,
    /// 发布（第一个文件、覆盖写）的内容长度，按长度种类 → 入口被调了几次。
    pub content_lengths_attempted: BTreeMap<&'static str, u64>,
    /// 发布的内容长度，按长度种类 → 入口返回 Ok 几次（代码三方攻方的 P3：装得下的每一种至少发成一次）。
    pub content_lengths_published: BTreeMap<&'static str, u64>,
    /// 截断（[`HistoryOperation::Truncate`]）的目标，按类（[`TruncationTarget::name`]）→ 入口被调了几次、返回 Ok 几次（判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8「目标按类取」：每一类真的走到了）。
    pub truncation_targets_attempted: BTreeMap<&'static str, u64>,
    pub truncation_targets_published: BTreeMap<&'static str, u64>,
    /// 按偏移写（[`HistoryOperation::WriteAtOffset`]）真落到的格（[`PartialWriteOffsetCell::name`]）→ 入口被调了几次、返回 Ok 几次（判决
    /// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节那五格：每一格真的走到了）。
    pub partial_write_offset_cells_attempted: BTreeMap<&'static str, u64>,
    pub partial_write_offset_cells_published: BTreeMap<&'static str, u64>,
    pub refusals_by_member: BTreeMap<String, u64>,
    pub not_applicable_by_precondition: BTreeMap<&'static str, u64>,
    pub recovery_outcomes: BTreeMap<String, u64>,
    pub most_publishes_in_one_mount: usize,
    /// 一步（起点算一步）里模型写进根环的根最多几条（[`crate::model::IdealModel::roots_written_since_make_filesystem`] 的差）：
    /// 执行器每一步断言它不多于根环槽数（[`crate::model::MODEL_ROOT_RING_SLOTS`]），观察者每步并一次模型根环才不漏版本。
    pub most_roots_written_by_the_model_in_one_step: u64,
    pub most_successful_mounts_in_one_history: usize,
    pub most_mount_attempts_in_one_history: usize,
    pub raises_that_reclaimed: u64,
    pub reclaimed_placements: u64,
    /// 管理员回退复活的用户可见单元（逐盘逐份）合计：复活那一支跑到了没有。
    pub resurrected_user_visible_copies: u64,
    pub records_rewritten_from_released: u64,
    pub records_rewritten_with_changed_span: u64,
    pub released_records_removed: u64,
    /// 已释放的记录被这次新分配或改写的记录罩住起点的次数（第 121 行那一类的路径跑到了；删没删不影响这个数）。
    pub released_records_covered: u64,
    /// 挂载写出的写行与暖机里，比过分配记录的发布次数；挂载之前生效根那棵分配记录树读不出、没比的挂载次数。
    pub mount_publishes_compared: u64,
    pub mounts_with_previous_records_unreadable: u64,
    /// 做成的可写挂载按空间准入怎么判的分（[`MountSpaceAdmissionOutcome::name`]）→ 几次；写行之后推过的那几次一共推了几串抬 F。
    pub mounts_by_space_admission: BTreeMap<&'static str, u64>,
    pub floor_raises_pushed_by_mounts: u64,
    /// 经挂着的会话发成的覆盖写里，发成之前会话推过抬 F 的次数与一共推了几串（发布那一处的推，D16（发布语义） 已定项 1「准入」那一行、C283）。
    pub user_changes_published_after_the_session_raised_the_floor: u64,
    pub floor_raises_pushed_by_the_session_before_a_published_change: u64,
    pub highest_checkpoint_txg: u64,
    /// 一步之后可写会话的分配器里最多有几条分配记录（两块盘合计）：分配记录树按位置寻址之后没有一个节点 812 条那道墙
    /// （D8（核心索引结构） 已定项 14），这个数越过 812 说明历史走过了那道墙。
    pub most_allocation_records_in_one_version: u64,
    pub histories_that_turned_the_root_ring: u64,
    /// 模型比过至少一个不是最后一批的删除批（≥ 2 片下段叶的文件的删除批，代码轮 U13）的段数。
    pub histories_with_a_deletion_batch_keeping_data_units: u64,
    /// 会话里一次用户改动之前先走空、走了至少两步、模型逐步比过的段数（N3「走空多步」）。
    pub histories_with_a_walk_of_several_steps_before_a_user_change: u64,
    /// 会话里一次用户改动之前先走空、走空被空间拒推了抬 F、模型逐串逐步比过的段数（N3「走空被空间拒」）。
    pub histories_with_a_walk_before_a_user_change_refused_for_space: u64,
    /// 挂着时整理（[`HistoryOperation::CompactWhileMounted`]）做成的趟数、其中至少发了一批的趟数、一共发的批数与搬的单元数、
    /// 按停机原因（`compaction_plan::CompactionRunStop::name`）分的趟数。
    pub compaction_runs: u64,
    pub compaction_runs_with_a_batch: u64,
    pub compaction_batches_published: u64,
    pub compaction_units_relocated: u64,
    pub compaction_stops: BTreeMap<&'static str, u64>,
    /// 批间崩溃（[`CompactionRunEnding::CrashesAfterBatchesThenMountsWritable`]）里崩掉那一刻意图还活着、之后可写挂载做成的次数：
    /// 意图推到半路随进程丢了（判决 G7「批间崩溃」真的走到了）。
    pub crashes_between_compaction_batches: u64,
    /// harness 就地跑池级 checker 的次数：那一遍归给定状态引擎两段，harness 一次都不跑（[`PerStepChecker`]），这一格恒为 0，
    /// 用例照它核 harness 没把逐步 checker 偷回来。
    pub checker_runs: u64,
    /// 这一步一个写都没发（录制流一步没多）：镜像逐字节不变，checker 的结论沿用上一次，不重跑。
    pub checker_runs_skipped_because_nothing_was_written: u64,
    /// 经每步镜像那个钩子交出去的镜像张数（[`execute_history_handing_out_every_step_image`]）：起点之后、每一步写过盘之后各一张，
    /// 起点段报错而 mkfs 做完了的那一张也算；跑不跑池级 checker 都交。
    pub step_images_handed_out: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    /// 判成「部分没判」（`singlefs_checker::image::InvariantVerdict::PartiallyJudged`）的次数：不红，也不算判绿。
    pub invariant_partially_judged: BTreeMap<&'static str, u64>,
    /// 理想模型判过的步数（起点那次第一个文件也算一步；前提不满足、入口没调的不算）与各格计数。
    pub model_judged_steps: u64,
    pub model_counts: ModelJudgementCounts,
}

impl HistoryTally {
    /// 把另一份计数并进来。
    pub fn absorb(&mut self, other: &HistoryTally) {
        self.histories += other.histories;
        self.histories_completed += other.histories_completed;
        for (form, count) in &other.histories_ended_known_red {
            *self.histories_ended_known_red.entry(*form).or_insert(0) += count;
        }
        self.histories_ended_new_finding += other.histories_ended_new_finding;
        for (kind, tally) in &other.operations_by_kind {
            let entry = self.operations_by_kind.entry(*kind).or_default();
            entry.applied += tally.applied;
            entry.refused += tally.refused;
            entry.not_applicable += tally.not_applicable;
        }
        for (length, count) in &other.content_lengths_attempted {
            *self.content_lengths_attempted.entry(length).or_insert(0) += count;
        }
        for (length, count) in &other.content_lengths_published {
            *self.content_lengths_published.entry(length).or_insert(0) += count;
        }
        for (target, count) in &other.truncation_targets_attempted {
            *self.truncation_targets_attempted.entry(target).or_insert(0) += count;
        }
        for (target, count) in &other.truncation_targets_published {
            *self.truncation_targets_published.entry(target).or_insert(0) += count;
        }
        for (cell, count) in &other.partial_write_offset_cells_attempted {
            *self
                .partial_write_offset_cells_attempted
                .entry(cell)
                .or_insert(0) += count;
        }
        for (cell, count) in &other.partial_write_offset_cells_published {
            *self
                .partial_write_offset_cells_published
                .entry(cell)
                .or_insert(0) += count;
        }
        for (member, count) in &other.refusals_by_member {
            *self.refusals_by_member.entry(member.clone()).or_insert(0) += count;
        }
        for (precondition, count) in &other.not_applicable_by_precondition {
            *self
                .not_applicable_by_precondition
                .entry(precondition)
                .or_insert(0) += count;
        }
        for (outcome, count) in &other.recovery_outcomes {
            *self.recovery_outcomes.entry(outcome.clone()).or_insert(0) += count;
        }
        self.most_publishes_in_one_mount = self
            .most_publishes_in_one_mount
            .max(other.most_publishes_in_one_mount);
        self.most_roots_written_by_the_model_in_one_step = self
            .most_roots_written_by_the_model_in_one_step
            .max(other.most_roots_written_by_the_model_in_one_step);
        self.most_successful_mounts_in_one_history = self
            .most_successful_mounts_in_one_history
            .max(other.most_successful_mounts_in_one_history);
        self.most_mount_attempts_in_one_history = self
            .most_mount_attempts_in_one_history
            .max(other.most_mount_attempts_in_one_history);
        self.raises_that_reclaimed += other.raises_that_reclaimed;
        self.reclaimed_placements += other.reclaimed_placements;
        self.resurrected_user_visible_copies += other.resurrected_user_visible_copies;
        self.records_rewritten_from_released += other.records_rewritten_from_released;
        self.records_rewritten_with_changed_span += other.records_rewritten_with_changed_span;
        self.released_records_removed += other.released_records_removed;
        self.released_records_covered += other.released_records_covered;
        self.mount_publishes_compared += other.mount_publishes_compared;
        self.mounts_with_previous_records_unreadable +=
            other.mounts_with_previous_records_unreadable;
        for (outcome, count) in &other.mounts_by_space_admission {
            *self.mounts_by_space_admission.entry(outcome).or_insert(0) += count;
        }
        self.floor_raises_pushed_by_mounts += other.floor_raises_pushed_by_mounts;
        self.user_changes_published_after_the_session_raised_the_floor +=
            other.user_changes_published_after_the_session_raised_the_floor;
        self.floor_raises_pushed_by_the_session_before_a_published_change +=
            other.floor_raises_pushed_by_the_session_before_a_published_change;
        self.highest_checkpoint_txg = self
            .highest_checkpoint_txg
            .max(other.highest_checkpoint_txg);
        self.most_allocation_records_in_one_version = self
            .most_allocation_records_in_one_version
            .max(other.most_allocation_records_in_one_version);
        self.histories_that_turned_the_root_ring += other.histories_that_turned_the_root_ring;
        self.histories_with_a_deletion_batch_keeping_data_units +=
            other.histories_with_a_deletion_batch_keeping_data_units;
        self.histories_with_a_walk_of_several_steps_before_a_user_change +=
            other.histories_with_a_walk_of_several_steps_before_a_user_change;
        self.histories_with_a_walk_before_a_user_change_refused_for_space +=
            other.histories_with_a_walk_before_a_user_change_refused_for_space;
        self.compaction_runs += other.compaction_runs;
        self.compaction_runs_with_a_batch += other.compaction_runs_with_a_batch;
        self.compaction_batches_published += other.compaction_batches_published;
        self.compaction_units_relocated += other.compaction_units_relocated;
        for (stop, count) in &other.compaction_stops {
            *self.compaction_stops.entry(stop).or_insert(0) += count;
        }
        self.crashes_between_compaction_batches += other.crashes_between_compaction_batches;
        self.checker_runs += other.checker_runs;
        self.checker_runs_skipped_because_nothing_was_written +=
            other.checker_runs_skipped_because_nothing_was_written;
        self.step_images_handed_out += other.step_images_handed_out;
        for (invariant, count) in &other.invariant_holds {
            *self.invariant_holds.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in &other.invariant_partially_judged {
            *self
                .invariant_partially_judged
                .entry(invariant)
                .or_insert(0) += count;
        }
        self.model_judged_steps += other.model_judged_steps;
        self.model_counts.add(&other.model_counts);
    }

    pub(super) fn note_model_verdict(&mut self, verdict: &ModelVerdict) {
        self.model_judged_steps += 1;
        self.model_counts.add(&verdict.counts);
    }

    fn note_reuse(&mut self, reuse: RecordReuse) {
        self.records_rewritten_from_released += reuse.rewritten_from_released;
        self.records_rewritten_with_changed_span += reuse.rewritten_with_changed_span;
        self.released_records_removed += reuse.released_records_removed;
        self.released_records_covered += reuse.released_records_covered;
    }

    pub(super) fn note_outcome(&mut self, operation: &HistoryOperation, outcome: &StepOutcome) {
        let entry = self.operations_by_kind.entry(operation.kind()).or_default();
        match outcome {
            StepOutcome::Applied(_) => entry.applied += 1,
            StepOutcome::Refused { .. } => entry.refused += 1,
            StepOutcome::NotApplicable(_) => entry.not_applicable += 1,
        }
        match outcome {
            StepOutcome::Applied(AppliedEffect::Published {
                reuse,
                floor_raises_pushed_by_the_session,
                roots_carrying_the_change: _,
            }) => {
                self.note_reuse(*reuse);
                if *floor_raises_pushed_by_the_session > 0 {
                    self.user_changes_published_after_the_session_raised_the_floor += 1;
                    self.floor_raises_pushed_by_the_session_before_a_published_change +=
                        u64::try_from(*floor_raises_pushed_by_the_session)
                            .expect("一次准入推的串数装得进 u64");
                }
            }
            StepOutcome::Applied(AppliedEffect::SnapshotPublished {
                reuse,
                floor_raises_pushed_by_the_session,
                ..
            }) => {
                self.note_reuse(*reuse);
                if *floor_raises_pushed_by_the_session > 0 {
                    self.user_changes_published_after_the_session_raised_the_floor += 1;
                    self.floor_raises_pushed_by_the_session_before_a_published_change +=
                        u64::try_from(*floor_raises_pushed_by_the_session)
                            .expect("一次准入推的串数装得进 u64");
                }
            }
            StepOutcome::Applied(AppliedEffect::SnapshotRead { .. }) => {}
            StepOutcome::Applied(AppliedEffect::Compacted {
                batches_published,
                units_relocated,
                stop,
                is_the_intent_live_when_the_run_stopped,
                crashed_then_mounted,
            }) => {
                self.compaction_runs += 1;
                if *batches_published > 0 {
                    self.compaction_runs_with_a_batch += 1;
                }
                self.compaction_batches_published += batches_published;
                self.compaction_units_relocated += units_relocated;
                *self.compaction_stops.entry(stop).or_insert(0) += 1;
                if *crashed_then_mounted && *is_the_intent_live_when_the_run_stopped {
                    self.crashes_between_compaction_batches += 1;
                }
            }
            StepOutcome::Applied(AppliedEffect::RaisedFloor {
                reclaimed_placements,
                reuse,
                ..
            }) => {
                if *reclaimed_placements > 0 {
                    self.raises_that_reclaimed += 1;
                }
                self.reclaimed_placements += u64::try_from(*reclaimed_placements).expect("落点数");
                self.note_reuse(*reuse);
            }
            StepOutcome::Applied(AppliedEffect::Recovered { outcome, .. }) => {
                *self.recovery_outcomes.entry(outcome.clone()).or_insert(0) += 1;
            }
            StepOutcome::Applied(AppliedEffect::RolledBack {
                resurrected_user_visible_copies,
                ..
            }) => {
                self.resurrected_user_visible_copies +=
                    u64::try_from(*resurrected_user_visible_copies).expect("份数");
            }
            StepOutcome::Applied(AppliedEffect::Mounted {
                allocation_records_compared,
                reuse,
                space_admission,
                ..
            }) => {
                self.note_reuse(*reuse);
                *self
                    .mounts_by_space_admission
                    .entry(space_admission.name())
                    .or_insert(0) += 1;
                let floor_raises = match space_admission {
                    MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises { floor_raises }
                    | MountSpaceAdmissionOutcome::StillShortAfterTheFloorRaises { floor_raises } => {
                        *floor_raises
                    }
                    MountSpaceAdmissionOutcome::AdmittedBeforeAcquisition
                    | MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch => 0,
                };
                self.floor_raises_pushed_by_mounts +=
                    u64::try_from(floor_raises).expect("一次挂载推的串数装得进 u64");
                match allocation_records_compared {
                    MountAllocationComparison::Compared { publishes } => {
                        self.mount_publishes_compared +=
                            u64::try_from(*publishes).expect("一次挂载至多几次发布");
                    }
                    MountAllocationComparison::NoPublishWithFile => {}
                    MountAllocationComparison::PreviousRecordsUnreadable => {
                        self.mounts_with_previous_records_unreadable += 1;
                    }
                }
            }
            StepOutcome::Refused { member } => {
                *self.refusals_by_member.entry(member.clone()).or_insert(0) += 1;
            }
            StepOutcome::NotApplicable(precondition) => {
                *self
                    .not_applicable_by_precondition
                    .entry(precondition.name())
                    .or_insert(0) += 1;
            }
        }
        let content = match operation {
            HistoryOperation::PublishFirstFile(content)
            | HistoryOperation::PublishOverwrite(content) => Some(content),
            HistoryOperation::WriteOfAnInode(choice) => Some(&choice.content),
            // 按偏移写的长度执行时按格取，真落到的格另记（[`HistoryTally::note_partial_write_offset_cell`]）。
            HistoryOperation::WriteAtOffset(_) => None,
            HistoryOperation::PublishWithoutUnits
            | HistoryOperation::CloseAndMountWritable
            | HistoryOperation::RollBackWhileMounted(_)
            | HistoryOperation::RaiseRollbackFloor(_)
            | HistoryOperation::ColdStartRecover
            | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
            | HistoryOperation::Unlink
            | HistoryOperation::CreateInode
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
            | HistoryOperation::CreateSnapshot
            | HistoryOperation::ReadSnapshot(_)
            | HistoryOperation::DestroySnapshot(_)
            | HistoryOperation::CompactWhileMounted(_)
            | HistoryOperation::CreateInodes(_)
            // 截断的目标按类另记（下面）。
            | HistoryOperation::Truncate(_)
            // 那次改动掉电在 journal 记录之前、一次都没发布成：内容长度不记进「进过入口」「发布过」（结局是那之后可写挂载的结局）。
            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
                None
            }
        };
        let entrance_was_called = match outcome {
            StepOutcome::Applied(_) | StepOutcome::Refused { .. } => true,
            StepOutcome::NotApplicable(_) => false,
        };
        if let HistoryOperation::Truncate(choice) = operation {
            if entrance_was_called {
                *self
                    .truncation_targets_attempted
                    .entry(choice.target.name())
                    .or_insert(0) += 1;
            }
            if let StepOutcome::Applied(_) = outcome {
                *self
                    .truncation_targets_published
                    .entry(choice.target.name())
                    .or_insert(0) += 1;
            }
        }
        if let (Some(content), true) = (content, entrance_was_called) {
            *self
                .content_lengths_attempted
                .entry(content.length.name())
                .or_insert(0) += 1;
        }
        if let (Some(content), StepOutcome::Applied(_)) = (content, outcome) {
            *self
                .content_lengths_published
                .entry(content.length.name())
                .or_insert(0) += 1;
        }
    }

    /// 一次按偏移写真落到的格进过入口（调了就算）、做成了没有（返回 Ok）。
    pub fn note_partial_write_offset_cell(
        &mut self,
        offset_cell: PartialWriteOffsetCell,
        outcome: &StepOutcome,
    ) {
        let entrance_was_called = match outcome {
            StepOutcome::Applied(_) | StepOutcome::Refused { .. } => true,
            StepOutcome::NotApplicable(_) => false,
        };
        if entrance_was_called {
            *self
                .partial_write_offset_cells_attempted
                .entry(offset_cell.name())
                .or_insert(0) += 1;
        }
        if let StepOutcome::Applied(_) = outcome {
            *self
                .partial_write_offset_cells_published
                .entry(offset_cell.name())
                .or_insert(0) += 1;
        }
    }

    /// 给人看的一整块：每类操作的结局、见过的 `Err` 成员、冷启动的结局、会话与挂载的最大值、复用、checker 跑了几次与每条不变量判绿几次。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；根环转过一圈的 {} 段；最高 txg {}；一版里最多 {} 条分配记录",
            self.histories,
            self.histories_completed,
            self.histories_ended_known_red,
            self.histories_ended_new_finding,
            self.histories_that_turned_the_root_ring,
            self.highest_checkpoint_txg,
            self.most_allocation_records_in_one_version
        );
        for kind in HistoryOperationKind::ALL {
            let tally = self
                .operations_by_kind
                .get(&kind)
                .copied()
                .unwrap_or_default();
            let _ = writeln!(
                text,
                "  操作 {kind:?}：Ok {}、Err {}、前提不满足没调 {}",
                tally.applied, tally.refused, tally.not_applicable
            );
        }
        for (length, count) in &self.content_lengths_attempted {
            let published = self
                .content_lengths_published
                .get(length)
                .copied()
                .unwrap_or(0);
            let _ = writeln!(
                text,
                "  内容长度 {length}：入口被调 {count} 次、发成 {published} 次"
            );
        }
        for (target, count) in &self.truncation_targets_attempted {
            let published = self
                .truncation_targets_published
                .get(target)
                .copied()
                .unwrap_or(0);
            let _ = writeln!(
                text,
                "  截断目标 {target}：入口被调 {count} 次、发成 {published} 次"
            );
        }
        for (cell, count) in &self.partial_write_offset_cells_attempted {
            let published = self
                .partial_write_offset_cells_published
                .get(cell)
                .copied()
                .unwrap_or(0);
            let _ = writeln!(
                text,
                "  按偏移写落在「{cell}」：入口被调 {count} 次、发成 {published} 次"
            );
        }
        for (member, count) in &self.refusals_by_member {
            let _ = writeln!(text, "  Err 成员 {member}：{count} 次");
        }
        for (precondition, count) in &self.not_applicable_by_precondition {
            let _ = writeln!(text, "  前提不满足（{precondition}）：{count} 次");
        }
        for (outcome, count) in &self.recovery_outcomes {
            let _ = writeln!(text, "  冷启动结局 {outcome}：{count} 次");
        }
        let _ = writeln!(
            text,
            "  一次挂载里最多写出 {} 条根；模型一步里最多写进根环 {} 条根；一段历史里最多挂载成功 {} 次、最多试挂载 {} 次",
            self.most_publishes_in_one_mount,
            self.most_roots_written_by_the_model_in_one_step,
            self.most_successful_mounts_in_one_history,
            self.most_mount_attempts_in_one_history
        );
        let _ = writeln!(
            text,
            "  抬 F 回收到落点 {} 次、共回收 {} 个落点；复用已释放的记录 {} 条（其中跨度变了 {} 条）；已释放的记录被罩住起点 {} 条、被删 {} 条",
            self.raises_that_reclaimed,
            self.reclaimed_placements,
            self.records_rewritten_from_released,
            self.records_rewritten_with_changed_span,
            self.released_records_covered,
            self.released_records_removed
        );
        let _ = writeln!(
            text,
            "  挂载写出的发布比过分配记录 {} 次；挂载之前生效根的分配记录读不出、没比的挂载 {} 次",
            self.mount_publishes_compared, self.mounts_with_previous_records_unreadable
        );
        for (outcome, count) in &self.mounts_by_space_admission {
            let _ = writeln!(text, "  可写挂载的空间准入「{outcome}」：{count} 次");
        }
        let _ = writeln!(
            text,
            "  可写挂载写行之后一共推了 {} 串抬 F",
            self.floor_raises_pushed_by_mounts
        );
        let _ = writeln!(
            text,
            "  经会话的覆盖写：会话先推抬 F 再发成 {} 次、一共推了 {} 串",
            self.user_changes_published_after_the_session_raised_the_floor,
            self.floor_raises_pushed_by_the_session_before_a_published_change
        );
        let _ = writeln!(
            text,
            "  删除：模型比过不是最后一批的删除批（≥ 2 片下段叶）{} 次、在 {} 段里；会话改动之前先走空走了至少两步 {} 次、在 {} 段里；走空被空间拒推了抬 F {} 次、在 {} 段里",
            self.model_counts.deletion_batches_keeping_data_units_matched,
            self.histories_with_a_deletion_batch_keeping_data_units,
            self.model_counts
                .walks_of_several_steps_before_a_user_change_matched,
            self.histories_with_a_walk_of_several_steps_before_a_user_change,
            self.model_counts
                .walks_before_a_user_change_refused_for_space_matched,
            self.histories_with_a_walk_before_a_user_change_refused_for_space
        );
        let _ = writeln!(
            text,
            "  整理：做成 {} 趟、其中发了批的 {} 趟，共 {} 批、搬了 {} 个单元，模型比过 {} 批；模型比过签发 {} 次、随最后一批删意图 {} 次、单独删意图 {} 次、丢意图 {} 次、logged_ops 里的意图 {} 版、搬了用户可见树的根（算非空）{} 次；批间崩溃（崩时意图还活着）{} 次；停机原因 {:?}",
            self.compaction_runs,
            self.compaction_runs_with_a_batch,
            self.compaction_batches_published,
            self.compaction_units_relocated,
            self.model_counts.compaction_batches_matched,
            self.model_counts.compaction_intents_issued_matched,
            self.model_counts
                .compaction_intents_deleted_with_the_last_batch_matched,
            self.model_counts
                .compaction_intents_deleted_without_a_batch_matched,
            self.model_counts.compaction_intents_dropped_matched,
            self.model_counts.logged_compaction_intents_compared,
            self.model_counts
                .compaction_publishes_relocating_a_user_visible_root_matched,
            self.crashes_between_compaction_batches,
            self.compaction_stops
        );
        let _ = writeln!(
            text,
            "  用户改动之后另发的那一批搬迁：模型比过 {} 批",
            self.model_counts
                .relocation_batches_after_a_user_change_matched
        );
        let _ = writeln!(
            text,
            "  截断：模型比过截断意图 {} 次、不是最后一批的批 {} 次、最后一批 {} 次、不缩短的那一次 {} 次",
            self.model_counts.truncation_intents_matched,
            self.model_counts
                .truncation_batches_before_the_final_batch_matched,
            self.model_counts.final_truncation_batches_matched,
            self.model_counts.size_changes_without_shrinking_matched
        );
        let _ = writeln!(
            text,
            "  按偏移写：模型比过写第一个文件 {} 次、写别的文件 {} 次",
            self.model_counts.partial_writes_of_the_first_file_matched,
            self.model_counts
                .partial_writes_of_an_inode_other_than_the_first_matched
        );
        let _ = writeln!(
            text,
            "  checker 跑了 {} 次；一个写都没发、沿用上一次结论的 {} 步；每步镜像交出去 {} 张",
            self.checker_runs,
            self.checker_runs_skipped_because_nothing_was_written,
            self.step_images_handed_out
        );
        let counts = &self.model_counts;
        let _ = writeln!(
            text,
            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次；单元区墙按区间放行 {} 次",
            self.model_judged_steps,
            counts.required_refusals_matched,
            counts.permitted_refusals_taken,
            counts.successes_matched,
            counts.roots_compared,
            counts.allocation_records_compared,
            counts.cold_start_contents_compared,
            counts.ceilings_compared,
            counts.rollbacks_accepted_at_the_effective_floor,
            counts.unit_area_wall_refusals_in_the_interval
        );
        // 每一版比没比实例表与分配代（代码审阅第 12 条）：树表 0 条的一版里「比不了」与「只比重写的角色」只剩一次挂载里第一版就是
        // 零单元发布的那几版（实审 B3c-4 往下带之后）；「比不了」另含带文件的那两格（mkfs 之后第一次重写实例表之前、从盘上重建且多于一片）。
        // 这几个数跑到了多少看这一行。
        let _ = writeln!(
            text,
            "  模型比每一版的实例表：比过 {} 张、输出不带整条链比不了 {} 张；零单元发布只比重写的角色 {} 次；比过文件内容 {} 次",
            counts.instance_tables_compared,
            counts.instance_tables_not_in_the_output,
            counts.rewritten_role_sets_compared,
            counts.file_contents_compared
        );
        for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
            let _ = writeln!(
                text,
                "  {invariant}：判绿 {} 次、不适用 {} 次、部分没判 {} 次",
                self.invariant_holds.get(invariant).copied().unwrap_or(0),
                self.invariant_not_applicable
                    .get(invariant)
                    .copied()
                    .unwrap_or(0),
                self.invariant_partially_judged
                    .get(invariant)
                    .copied()
                    .unwrap_or(0)
            );
        }
        text
    }
}

/// 一段历史跑完交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRun {
    pub ending: HistoryEnding,
    /// 每一步操作的结局，按次序；失败在第几步就到第几步为止（panic 的那一步没有结局）。
    pub outcomes: Vec<StepOutcome>,
    pub tally: HistoryTally,
    /// mkfs 占了录制流开头的几步：崩溃注入（增补 3 第 3 件）的基线与崩溃点重放一样取「mkfs 之后」——
    /// mkfs 不是事务，它写到一半的盘面上没有池，恢复报不出根是对的，不该拿事务的 oracle 去判
    /// （崩溃点重放那一路同样从 `mkfs_operation_count` 之后起枚举，见 `crates/singlefs-verification/tests/crash_enumeration_new_pool_file_creation_stream.rs`）。
    pub operations_written_by_make_filesystem: usize,
    /// 这段历史停下时模型根环里的每一条根（`IdealModel::committed_versions`）：故障注入（增补 3 第 4 件）拿它当
    /// 「模型认下来的每一版」判重开走到的那一版——失败那一步的根，模型认了就在里面，没认（判定对不上、模型没往前走）就不在。
    /// 观察者那一路看不到这一份：判出失败的那一步不调观察者，而失败常常出在模型已经认下那一版之后（checker 在那一步判红）。
    /// 内容拷成 `Vec<u8>` 不留 `Rc`：`HistoryRun` 要跨线程送（`run_history_campaign` 的那张表），`Rc` 不是 `Send`。
    pub committed_versions_at_the_end: Vec<(ModelRootKey, Option<Vec<u8>>)>,
    /// 每次掉电（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]）之后录制流里第一步的下标，按次序：
    /// 掉电之前那一截与之后可写挂载另起的实例写的流在这里断开。崩溃注入把它交给记录核对器
    /// （`crate::memory_pool::RecordStreamSeam::AfterAPowerLoss`）：那次改动掉电之前落了盘的单元写没有根，
    /// 不归掉电之后的第一次发布。
    pub earliest_stream_index_after_each_power_loss: Vec<usize>,
    /// 每次截断（[`HistoryOperation::Truncate`]，按步号）交给会话的目标 size：执行时按那一刻的 size 与单元数取（[`TruncationChoice`]），
    /// RefFS 翻译拿它交 `Operation::Truncate(n)`（入口报错、panic 停下的那一步也在里面）。
    pub sizes_requested_by_the_truncations: BTreeMap<usize, u64>,
    /// 每次截断（按步号）点名的 inode 号：RefFS 翻译对第一个文件之外的号写了盘的那一步停下（判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「翻译层对 inode ≠ 1 且写了盘的截断停下」）。
    pub inodes_named_by_the_truncations: BTreeMap<usize, u64>,
    /// 每次按偏移写（[`HistoryOperation::WriteAtOffset`]，按步号）交给会话的号、偏移、长度与真落到的格：RefFS 翻译拿它交按偏移写的那一个
    /// 内容改动（字节由那一步的选择按这个长度现算），对第一个文件之外的号写了盘的那一步停下（入口报错、panic 停下的那一步也在里面）。
    pub partial_writes_requested: BTreeMap<usize, PartialWriteRequested>,
    /// 实现交回的每一份隔离，按交回的先后，跨重挂累积（[`CopyQuarantinedByTheImplementation`]；停下的那一步、panic 的那一步交回的也在里面）。
    pub copies_quarantined_by_the_implementation: Vec<CopyQuarantinedByTheImplementation>,
}

/// 每一步之后交给观察者的东西：第几步、这一步是什么、结局、此刻的整份镜像、跟到这一步的理想模型。
/// 只在这一步没判出失败时调（判出失败的那一步直接停下，观察者看不到），所以模型与镜像都是判过的状态。
pub struct StepObservation<'run> {
    pub position: StepPosition,
    pub operation: Option<&'run HistoryOperation>,
    pub outcome: Option<&'run StepOutcome>,
    pub image: &'run MemoryPool,
    /// 崩溃注入（增补 3 第 3 件）每一步之后从它取一次 `committed_versions`，攒成「模型提交过的每一版」。
    pub model: &'run IdealModel,
    /// 这个进程的会话现行那一版记账里每块盘「空闲」那一行（D5（快照 / 空间记账机制） 已定项 8，`df` 报的就是它）；没有会话、
    /// 现行那一版树表 0 条时是 `None`。判「整理不把做得成的改动变成空间不够，除非 `df` 也跟着少报」（判决 G7）的用例拿它比。
    pub free_bytes_reported_by_the_current_version: Option<Vec<(DeviceIdentity, u64)>>,
    /// 这个进程的会话现行那一版（树表有条目的那种；没有会话、现行那一版树表 0 条时是 `None`）：判「随一版带着、提交时增量维护的数」
    /// 与逐片现数的那一份逐版相等的用例拿它（`tests/inode_record_counts_kept_with_each_version.rs` 里随机历史那两条）。
    pub current_version_of_trees: Option<&'run TransactionOutput>,
    /// 这一步是截断时交给会话的目标 size（[`HistoryRun::sizes_requested_by_the_truncations`] 里这一步那一项）；别的步、前提不满足没调的是 None。
    pub size_requested_by_the_truncation: Option<u64>,
    /// 这一步是截断时点名的 inode 号（[`HistoryRun::inodes_named_by_the_truncations`] 里这一步那一项）；别的步、前提不满足没调的是 None。
    pub inode_named_by_the_truncation: Option<u64>,
    /// 这一步是按偏移写时交给会话的那一次（[`HistoryRun::partial_writes_requested`] 里这一步那一项）；别的步、前提不满足没调的是 None。
    pub partial_write_requested: Option<PartialWriteRequested>,
}
