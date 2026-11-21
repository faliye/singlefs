//! 攒批的会话上的后台整理：后台入口一趟的跨度与排空待发，在会话锁外算一批（取输入、读单元、发布算好的那一批、
//! 调用方丢下没回锁的那一批），连同交回的结果与拒绝。

use super::*;

/// [`MountedSessionWithPendingChanges::run_compaction_in_the_background`] 做成了什么：判过起跑之后先排空的那一次（待发空、没起跑时 `None`）与后台入口这一次
/// （整理停着时带着这一次叫起的报警，[`BackgroundCompactionCall::alarm_of_the_compaction_stopped_after_a_compute_error`]）。
#[must_use = "整理因 GPU 出错停着时后台入口每一次都带着报警，要告诉文件系统的用户（用户 2026-10-30 定）"]
#[derive(Debug)]
pub struct BackgroundCompactionAfterThePendingChanges {
    pub pending_changes_published_before_the_run: Option<PublishReportedByTheSession>,
    /// 放锁的三段入口回锁时、发这一批之前排空待发的那一次（锁外那一段期间前台收下的改动；待发空、或这一批不发时 `None`）。锁里推完一趟的入口恒是 `None`。
    pub pending_changes_published_at_the_relock: Option<PublishReportedByTheSession>,
    pub call: BackgroundCompactionCall,
}

/// 放锁的三段入口取输入那一段交回的（[`MountedSessionWithPendingChanges::take_the_input_of_a_background_compaction_batch`]）。封闭集合。
#[must_use = "在锁里就走完的那一次带着整理停着时的报警；交到锁外的那一批要算完、回锁交回，否则这个会话的后台入口一直拒"]
#[derive(Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次调用交回一份，不在热路径上：在锁里走完的那一种带着排空待发的两次报告，装箱只多一次分配，换不来什么"
)]
pub enum BackgroundCompactionInputTaken {
    /// 这一次调用在会话锁里就走完了（没起跑、删除续做走了一步、照判据停、让路、整理停着时删了意图），与
    /// [`MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`] 交回的同形。
    FinishedInTheLock(BackgroundCompactionAfterThePendingChanges),
    /// 一批交到锁外去算：取输入之前排空待发的那一次，与票（自有，不借会话与盘表）。调用方放会话锁，照票算
    /// （[`BackgroundCompactionBatchTicket::compute_out_of_the_lock`]），再回会话锁交
    /// [`MountedSessionWithPendingChanges::publish_the_batch_computed_out_of_the_lock`]。装箱：票带着这一趟的账与整份 R 的单元字节。
    BatchOutOfTheLock {
        pending_changes_published_before_the_run: Option<PublishReportedByTheSession>,
        ticket: Box<BackgroundCompactionBatchTicket>,
    },
}

/// 放锁的三段入口取输入那一段没取。成员按调用方要做的决定分。
#[derive(Debug)]
pub enum BackgroundCompactionInputRefused {
    /// 这个会话已有一批在锁外（取了输入、还没回锁）：第二个后台调用者在取 GPU 实现那把锁与换设置之前就拒，一个盘都不读、不算一次叫起。
    /// 出路：先把在锁外的那一批算完、回锁交回。
    ABatchOfThisSessionIsOutOfTheLock,
    /// 同 [`MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`] 的那两种。
    Refused(BackgroundCompactionAfterThePendingChangesRefused),
}

/// 放锁的三段入口回锁那一段没收下这一批。成员按调用方要做的决定分。
#[derive(Debug)]
pub enum BackgroundCompactionRelockRefused {
    /// 交进来的不是这个会话此刻在锁外的那一批（别的会话的票，或已经回过锁的）：什么都不动，原样交回。
    NotTheBatchOutOfTheLockOfThisSession {
        computed: Box<BackgroundCompactionBatchComputed>,
    },
    /// 同 [`CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount`]：什么都不动、那一批照旧在锁外，原样交回，换挂载核过的那一份盘表再交。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
        computed: Box<BackgroundCompactionBatchComputed>,
    },
    /// 回锁里发这一批之前排空待发没发成（同 [`BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused`]，
    /// 带着这一次叫起的报警）：这一批不发、丢掉，这个会话不再有一批在锁外，这一趟照取输入之前排空被拒那样留着或放回「该跑了」。
    PublishOfThePendingChangesRefused(BackgroundCompactionAfterThePendingChangesRefused),
}

/// 调用方丢掉锁外算完的那一批、没回锁（[`MountedSessionWithPendingChanges::note_the_batch_computed_out_of_the_lock_dropped_by_the_caller_without_relocking`]）
/// 之后会话怎样了。封闭集合。
#[must_use = "整理停着时这一次叫起带着报警：要告诉文件系统的用户"]
#[derive(Debug)]
pub enum BatchOutOfTheLockDroppedByTheCaller {
    /// 是这个会话此刻在锁外的那一批：会话上不再记有一批在锁外，一个写都没发、发布预算不扣，这一趟照留着；
    /// 整理停着时带着这一次叫起的报警，没停着是 `None`。
    TakenBackWithoutPublishing {
        alarm_of_the_compaction_stopped_after_a_compute_error:
            Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
    },
    /// 交进来的不是这个会话此刻在锁外的那一批（别的会话的票，或已经回过锁的）：什么都不动。
    NotTheBatchOutOfTheLockOfThisSession,
}

/// 锁外算完的那一批（[`BackgroundCompactionBatchTicket::compute_out_of_the_lock`]）：票里那几样，连同算出的提议（或出的错）与这一批在 GPU 上算了什么。
/// 全部自有，`Send + 'static`；交回之前 GPU 实现那把锁已放掉。
pub struct BackgroundCompactionBatchComputed {
    batch: BatchOutOfTheLock,
    trip: TripOfTheBatchOutOfTheLock,
    run: CompactionRunUnderWay,
    computed: CompactionBatchComputed,
}

/// 会话这一层推完一趟里要取一批输入的那一次（[`MountedSession::take_the_background_batch_that_starts`]）：交到锁外的那一批，或这一次调用在锁里就收完了。封闭集合。
pub(super) enum BackgroundBatchTakenOrCallFinished {
    BatchTaken(BackgroundCompactionBatchTicket),
    CallFinished(BackgroundCompactionCall),
}

/// 票与锁外算完的那一批都是 `Send + 'static`（D26（后台整理与放置回收） 已定项 13 第 5 条：锁外那一段可以交给别的线程算）：编不过就是哪一样借了会话或不能跨线程。
const _: fn() = || {
    fn is_send_and_static<Owned: Send + 'static>() {}
    is_send_and_static::<BackgroundCompactionBatchTicket>();
    is_send_and_static::<BackgroundCompactionBatchComputed>();
};

impl fmt::Debug for BackgroundCompactionBatchTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BackgroundCompactionBatchTicket")
            .field("batch", &self.batch)
            .field(
                "txg_of_the_version_of_the_ticket",
                &self.input.version.root.checkpoint_txg,
            )
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for BackgroundCompactionBatchComputed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BackgroundCompactionBatchComputed")
            .field("batch", &self.batch)
            .field(
                "txg_of_the_version_of_the_ticket",
                &self.computed.version.root.checkpoint_txg,
            )
            .field(
                "is_the_computation_failed",
                &self.computed.proposal.is_err(),
            )
            .finish_non_exhaustive()
    }
}

impl BackgroundCompactionBatchTicket {
    /// 锁外那一段：照票算这一批（建映射条目、钉住集与按身份索引，选段打分或对 R 扫描，算出提议），不碰会话。要读盘上一段单元时调
    /// `read_in_the_session_lock`：调用方回会话锁里交 [`MountedSessionWithPendingChanges::read_the_units_for_the_batch_out_of_the_lock`]、交回读到的字节，
    /// 再放锁接着算（每一圈回锁里读，D26（后台整理与放置回收） 已定项 13 第 5 条）。走 GPU 时 GPU 实现那把锁每一段调用借一次、调完就放：
    /// 调 `read_in_the_session_lock` 之前与交回之前都不持着它。算着出的错不在这里报：记在交回的那一份里，回锁时先于一切照它办。
    #[must_use = "算完的那一批要回会话锁交回，否则这个会话的后台入口一直拒"]
    pub fn compute_out_of_the_lock(
        self,
        read_in_the_session_lock: &mut dyn FnMut(
            UnitReadsOfARegionScan,
        ) -> UnitBytesReadBeforeTheRegionScan,
    ) -> BackgroundCompactionBatchComputed {
        let BackgroundCompactionBatchTicket {
            batch,
            trip,
            run,
            input,
        } = self;
        BackgroundCompactionBatchComputed {
            batch,
            trip,
            run,
            computed: compute_the_compaction_batch(*input, read_in_the_session_lock),
        }
    }
}

/// [`MountedSessionWithPendingChanges::run_compaction_in_the_background`] 没跑。成员按调用方要做的决定分。
#[derive(Debug)]
pub enum BackgroundCompactionAfterThePendingChangesRefused {
    /// 后台入口自己在任何读写之前拒（同 [`MountedSession::run_compaction_in_the_background`]）。
    CompactionRefused(CompactionRunRefused),
    /// 判过起跑、跑之前排空待发没发成（`refusal` 原样）：这一趟整理没跑，用户改动之后记下的「该跑了」放回去。判过起跑的这一次是一次叫起
    /// （不是入口在任何读写之前就拒的那一种），整理停着时照样数一次叫起、带着这一次的报警（判决 m4-gpugc-code-r2 第三节第 3 条；
    /// 同 [`BackgroundCompactionCall::alarm_of_the_compaction_stopped_after_a_compute_error`]），没停着是 `None`。
    PublishOfThePendingChangesRefused {
        refusal: Box<UserChangeRefused>,
        alarm_of_the_compaction_stopped_after_a_compute_error:
            Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
    },
}

impl MountedSessionWithPendingChanges {
    /// 整理的后台入口（[`MountedSession::run_compaction_in_the_background`]）连同待发：整理是屏障——待发里不并进整理的发布，判过起跑之后、
    /// 那一趟起跑之前先把待发排空（判决第二节「一次发布里攒什么」那一格；整理仍只在后台入口里跑）。没起跑就不排空。
    /// 排空那一次是一次用户改动的发布：`B_gc` 从它重新数、起跑判据为真记「该跑了」（同 [`MountedSession::publish_user_change`]）。
    ///
    /// # Errors
    /// [`BackgroundCompactionAfterThePendingChangesRefused`] 的两种。
    pub fn run_compaction_in_the_background<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<
        BackgroundCompactionAfterThePendingChanges,
        BackgroundCompactionAfterThePendingChangesRefused,
    > {
        self.run_compaction_in_the_background_over_a_span(
            devices,
            pacing,
            caller,
            SpanOfTheBackgroundCall::WholeTrip,
        )
    }

    /// 同 [`Self::run_compaction_in_the_background`]，只差至多一次发布就返回（D26（后台整理与放置回收） 已定项 13 第 5 条：前台与后台经一把锁，每次发布之间放锁）：
    /// 这一趟没完时停因是 [`CompactionRunStop::OnePublishOfTheTripDone`]（或节奏截住），记在会话上，下一次调用不重判起跑、接着推。整理之前的删除续做同样一次调用至多走一步。
    /// 全空段低于底额时为前台 fsync 让路的那一趟不记（下一趟照样破例）；整理停着时删意图的那一趟
    /// （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]）排空待发被拒也不记，下一次调用照停没停重判。
    /// 两次调用之间前台收下的待发在下一次调用先排空。挂着的会话这一层不起线程、不持锁，放锁是调用方的事（运行库 `singlefs_mount_runtime`）。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction_in_the_background`]。
    pub fn run_compaction_in_the_background_for_at_most_one_publish<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<
        BackgroundCompactionAfterThePendingChanges,
        BackgroundCompactionAfterThePendingChangesRefused,
    > {
        self.run_compaction_in_the_background_over_a_span(
            devices,
            pacing,
            caller,
            SpanOfTheBackgroundCall::AtMostOnePublish,
        )
    }

    /// 两种跨度共用的那一段。这个会话有一批在锁外时不起跑（[`BackgroundCompactionNotStarted::ABatchOfThisSessionIsOutOfTheLock`]），在取 GPU 实现那把锁与换设置之前，
    /// 不排空、不算一次叫起。
    fn run_compaction_in_the_background_over_a_span<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
        span: SpanOfTheBackgroundCall,
    ) -> Result<
        BackgroundCompactionAfterThePendingChanges,
        BackgroundCompactionAfterThePendingChangesRefused,
    > {
        if self.session.compaction.has_a_batch_out_of_the_lock() {
            return Ok(BackgroundCompactionAfterThePendingChanges {
                pending_changes_published_before_the_run: None,
                pending_changes_published_at_the_relock: None,
                call: BackgroundCompactionCall::NotStarted {
                    reason: BackgroundCompactionNotStarted::ABatchOfThisSessionIsOutOfTheLock,
                    alarm_of_the_compaction_stopped_after_a_compute_error: None,
                },
            });
        }
        let now = caller.clock_reading();
        self.note_the_reading_seen(now);
        let (trigger, reach, budget, batches_published_earlier_in_the_trip) = match self
            .session
            .whether_the_background_compaction_starts(devices, pacing, caller)
            .map_err(BackgroundCompactionAfterThePendingChangesRefused::CompactionRefused)?
        {
            BackgroundCompactionStart::NotStarted(reason) => {
                let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
                return Ok(BackgroundCompactionAfterThePendingChanges {
                    pending_changes_published_before_the_run: None,
                    pending_changes_published_at_the_relock: None,
                    call: BackgroundCompactionCall::NotStarted {
                        reason,
                        alarm_of_the_compaction_stopped_after_a_compute_error: alarm,
                    },
                });
            }
            BackgroundCompactionStart::Starts {
                trigger,
                reach,
                budget,
                batches_published_earlier_in_the_trip,
            } => (
                trigger,
                reach,
                budget,
                batches_published_earlier_in_the_trip,
            ),
        };
        let pending_changes_published_before_the_run = match self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction,
        ) {
            Ok(drained) => drained,
            Err(refusal) => {
                return Err(self.refusal_of_the_drain_before_the_background_run(
                    TripOfTheDrainRefused {
                        trigger,
                        reach,
                        budget,
                        span,
                        batches_published_earlier_in_the_trip,
                    },
                    refusal,
                ));
            }
        };
        let call = self.session.run_the_background_compaction_that_starts(
            devices,
            pacing,
            caller,
            trigger,
            reach,
            budget,
            span,
            batches_published_earlier_in_the_trip,
        );
        let has_published = match &call {
            BackgroundCompactionCall::Ran { run, .. } => !run.versions_published.is_empty(),
            BackgroundCompactionCall::NotStarted { .. } => false,
        };
        if has_published {
            self.note_a_publish(now);
        }
        let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
        let call = call.with_the_alarm_at_the_end_of_the_wake(alarm);
        Ok(BackgroundCompactionAfterThePendingChanges {
            pending_changes_published_before_the_run,
            pending_changes_published_at_the_relock: None,
            call,
        })
    }

    /// 判过起跑、跑之前排空待发被拒（放锁的三段入口回锁时排空被拒同走这一段）：这一趟整理没跑，照跨度与触发名把这一趟留着或把「该跑了」放回去；
    /// 判过起跑的这一次是一次叫起，取这一次的报警，交回拒绝。
    fn refusal_of_the_drain_before_the_background_run(
        &mut self,
        trip: TripOfTheDrainRefused,
        refusal: Box<UserChangeRefused>,
    ) -> BackgroundCompactionAfterThePendingChangesRefused {
        let TripOfTheDrainRefused {
            trigger,
            reach,
            budget,
            span,
            batches_published_earlier_in_the_trip,
        } = trip;
        match span {
            // 停着删意图那一趟不记成没完：下一次调用照停没停重判（停着就再判删不删，解除了就照普通整理判起跑），
            // 不顶着这个触发名接着推普通整理。
            SpanOfTheBackgroundCall::AtMostOnePublish
                if trigger
                    == BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}
            SpanOfTheBackgroundCall::AtMostOnePublish => {
                self.session.compaction.keep_the_background_trip_under_way(
                    BackgroundTripUnderWay {
                        trigger,
                        reach,
                        batches_published: batches_published_earlier_in_the_trip,
                        is_the_exception_of_the_trip_still_open: budget
                            == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite,
                    },
                );
            }
            SpanOfTheBackgroundCall::WholeTrip => match trigger {
                BackgroundCompactionTrigger::DueAfterAUserChange => {
                    self.session.compaction.start_judgement =
                        CompactionStartJudgement::DueAfterAUserChange;
                }
                BackgroundCompactionTrigger::TimerFallback
                | BackgroundCompactionTrigger::DeletionQueueNotEmpty
                | BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}
            },
        }
        // 用户 2026-10-30 定每次叫起都报：判过起跑之后排空被拒的这一次也是一次叫起，照数、照报。
        let alarm_of_the_wake_refused_at_the_drain =
            self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
        BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused {
            refusal,
            alarm_of_the_compaction_stopped_after_a_compute_error:
                alarm_of_the_wake_refused_at_the_drain,
        }
    }

    /// 放锁的三段入口之一：会话锁里取一批的输入（D26（后台整理与放置回收） 已定项 13 第 5 条）。这个会话已有一批在锁外就拒，在取 GPU 实现那把锁与换设置之前；
    /// 否则照 [`Self::run_compaction_in_the_background_for_at_most_one_publish`] 判起跑、排空待发、推这一趟（问让路连同它的下限 `yield_floor`、写带宽、回退门），
    /// 推到要算一批就开这一批（读一次后台计算设置、定走法）、把现行那一版的 `Arc` 指针与算要的几样拷进票、有活意图时把 R 里的单元读进票，
    /// 会话上记下有一批在锁外，交回票；在锁里就推完了的交回与那个入口同形的结局。调用方放锁算（[`BackgroundCompactionBatchTicket::compute_out_of_the_lock`]），
    /// 再回锁交 [`Self::publish_the_batch_computed_out_of_the_lock`]。让路只在这里问：回锁那一刻不问。
    ///
    /// # Errors
    /// [`BackgroundCompactionInputRefused`] 的两种。
    pub fn take_the_input_of_a_background_compaction_batch<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        yield_floor: CompactionYieldFloor,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<BackgroundCompactionInputTaken, BackgroundCompactionInputRefused> {
        if self.session.compaction.has_a_batch_out_of_the_lock() {
            return Err(BackgroundCompactionInputRefused::ABatchOfThisSessionIsOutOfTheLock);
        }
        let now = caller.clock_reading();
        self.note_the_reading_seen(now);
        let (trigger, reach, budget, batches_published_earlier_in_the_trip) = match self
            .session
            .whether_the_background_compaction_starts(devices, pacing, caller)
            .map_err(|refused| {
                BackgroundCompactionInputRefused::Refused(
                    BackgroundCompactionAfterThePendingChangesRefused::CompactionRefused(refused),
                )
            })? {
            BackgroundCompactionStart::NotStarted(reason) => {
                let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
                return Ok(BackgroundCompactionInputTaken::FinishedInTheLock(
                    BackgroundCompactionAfterThePendingChanges {
                        pending_changes_published_before_the_run: None,
                        pending_changes_published_at_the_relock: None,
                        call: BackgroundCompactionCall::NotStarted {
                            reason,
                            alarm_of_the_compaction_stopped_after_a_compute_error: alarm,
                        },
                    },
                ));
            }
            BackgroundCompactionStart::Starts {
                trigger,
                reach,
                budget,
                batches_published_earlier_in_the_trip,
            } => (
                trigger,
                reach,
                budget,
                batches_published_earlier_in_the_trip,
            ),
        };
        let pending_changes_published_before_the_run = match self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction,
        ) {
            Ok(drained) => drained,
            Err(refusal) => {
                return Err(BackgroundCompactionInputRefused::Refused(
                    self.refusal_of_the_drain_before_the_background_run(
                        TripOfTheDrainRefused {
                            trigger,
                            reach,
                            budget,
                            span: SpanOfTheBackgroundCall::AtMostOnePublish,
                            batches_published_earlier_in_the_trip,
                        },
                        refusal,
                    ),
                ));
            }
        };
        match self.session.take_the_background_batch_that_starts(
            devices,
            pacing,
            yield_floor,
            caller,
            trigger,
            reach,
            budget,
            batches_published_earlier_in_the_trip,
        ) {
            BackgroundBatchTakenOrCallFinished::BatchTaken(ticket) => {
                Ok(BackgroundCompactionInputTaken::BatchOutOfTheLock {
                    pending_changes_published_before_the_run,
                    ticket: Box::new(ticket),
                })
            }
            BackgroundBatchTakenOrCallFinished::CallFinished(call) => {
                let has_published = match &call {
                    BackgroundCompactionCall::Ran { run, .. } => !run.versions_published.is_empty(),
                    BackgroundCompactionCall::NotStarted { .. } => false,
                };
                if has_published {
                    self.note_a_publish(now);
                }
                let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
                Ok(BackgroundCompactionInputTaken::FinishedInTheLock(
                    BackgroundCompactionAfterThePendingChanges {
                        pending_changes_published_before_the_run,
                        pending_changes_published_at_the_relock: None,
                        call: call.with_the_alarm_at_the_end_of_the_wake(alarm),
                    },
                ))
            }
        }
    }

    /// 放锁的三段入口之二的那一圈读盘：锁外那一段要读盘上一段单元时，调用方回会话锁里调它，交回读到的字节再放锁接着算。只读交进来的盘表，一个写都不发。
    /// 交进来的盘表与挂载核过的那一份不同就一个盘都不读、每个单元都当读不到（那几条记录判成读不出头、不当可搬；回锁时盘表照样要对上才发）。
    #[must_use]
    pub fn read_the_units_for_the_batch_out_of_the_lock<Device: BlockDevice>(
        &self,
        devices: &[(DeviceIdentity, Device)],
        reads: UnitReadsOfARegionScan,
    ) -> UnitBytesReadBeforeTheRegionScan {
        if !self
            .session
            .parameters_and_device_table
            .is_the_device_table_of_the_mount(devices)
        {
            return reads.read_with(|_device, _slot, _unit_bytes| None);
        }
        reads.read_with(|device, slot, unit_bytes| {
            devices.read(device, slot.to_device_offset(), unit_bytes)
        })
    }

    /// 放锁的三段入口之三：回会话锁里发锁外算完的那一批（D26（后台整理与放置回收） 已定项 13 第 5 条）。依次：认是不是这个会话此刻在锁外的那一批、
    /// 核盘表（都在任何读写之前，不对就原样交回）；会话上不再记有一批在锁外；出错先于一切（照锁里出错那一支停下；删意图之前在现行那一版上判破例、
    /// 不成立再过回退门，门关就留着意图、会话记下它，之后任一入口取下一批的输入之前先删）；
    /// 提议本来就是停的照停；回锁时冻结着一次发布的丢掉这一批；要发的先排空待发，再在现行那一版上发（现行那一版变了先重判回退门，重取隔离集，有活意图那一支剔掉记录变了的单元、
    /// 剔完一个不剩就不发，签发那一支重判资格与签发前那一判、意图的量取现行那一版的）。之后照这一趟的账收这一次调用：这一趟没完记在会话上，
    /// 用户改动之后那一趟被截住时把「该跑了」放回去，发 discard，取这一次叫起的报警。让路不在这里问。
    ///
    /// # Errors
    /// [`BackgroundCompactionRelockRefused`] 的三种。
    ///
    /// # Panics
    /// 发成之后再判下一批没停：放锁的入口一次调用至多一次发布，发成之后那一判先停在至多一次发布上。
    pub fn publish_the_batch_computed_out_of_the_lock<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        computed: BackgroundCompactionBatchComputed,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<BackgroundCompactionAfterThePendingChanges, BackgroundCompactionRelockRefused> {
        if !self
            .session
            .compaction
            .is_the_batch_out_of_the_lock(computed.batch)
        {
            return Err(
                BackgroundCompactionRelockRefused::NotTheBatchOutOfTheLockOfThisSession {
                    computed: Box::new(computed),
                },
            );
        }
        if !self
            .session
            .parameters_and_device_table
            .is_the_device_table_of_the_mount(devices)
        {
            return Err(
                BackgroundCompactionRelockRefused::DeviceTableOtherThanTheOneOfTheMount {
                    device_table_of_the_mount: self
                        .session
                        .parameters_and_device_table
                        .device_identities_in_table_order()
                        .to_vec(),
                    device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
                    computed: Box::new(computed),
                },
            );
        }
        let BackgroundCompactionBatchComputed {
            batch,
            trip,
            mut run,
            computed,
        } = computed;
        self.session
            .compaction
            .take_back_the_batch_out_of_the_lock(batch);
        // 这一趟是取输入那一次调用开的；回锁这一次调用里发布被拒时，冻的是不是这一次发的那一次照回锁进门这一刻判（D26（后台整理与放置回收） 已定项 13 第 6 条：
        // 冻结的是用户改动照用户改动记，冻结的是后台批照后台批记）。
        run.note_whether_a_publish_is_frozen_as_the_relock_comes_in(&self.session.allocator);
        let now = caller.clock_reading();
        self.note_the_reading_seen(now);
        let at_the_relock = {
            let MountedSession {
                allocator,
                current,
                instance,
                shadow_ledger: _,
                parameters_and_device_table,
                compaction,
            } = &mut self.session;
            let (parameters, background_compute) = parameters_and_device_table
                .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
            judge_the_computed_compaction_batch_at_the_relock(
                parameters,
                devices.as_mut_slice(),
                allocator,
                compaction,
                current,
                *instance,
                computed,
                background_compute,
                trip.rollback_gate_at_the_relock,
            )
        };
        let (advanced, pending_changes_published_at_the_relock) = match at_the_relock {
            CompactionBatchAtTheRelock::AdvancedInTheLock(advanced) => (*advanced, None),
            CompactionBatchAtTheRelock::ToPublishOnTheCurrentVersion(to_publish) => {
                let drained = match self.publish_the_pending_changes(
                    devices,
                    now,
                    ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction,
                ) {
                    Ok(drained) => drained,
                    // D26（后台整理与放置回收） 已定项 13 第 5 条没写回锁时排空被拒这一格：照取输入之前排空被拒那一支办（这一批丢掉，
                    // 这一趟留着或放回「该跑了」，取报警）。两处排空发的是同一样东西（前台收下、还没发的改动），被拒时这一批都还没往盘上写，
                    // 处置相同，调用方不用分排空落在哪一刻。
                    Err(refusal) => {
                        return Err(
                            BackgroundCompactionRelockRefused::PublishOfThePendingChangesRefused(
                                self.refusal_of_the_drain_before_the_background_run(
                                    TripOfTheDrainRefused {
                                        trigger: trip.trigger,
                                        reach: trip.reach,
                                        budget: trip.budget,
                                        span: SpanOfTheBackgroundCall::AtMostOnePublish,
                                        batches_published_earlier_in_the_trip: trip
                                            .batches_published_earlier_in_the_trip,
                                    },
                                    refusal,
                                ),
                            ),
                        );
                    }
                };
                let MountedSession {
                    allocator,
                    current,
                    instance,
                    shadow_ledger: _,
                    parameters_and_device_table,
                    compaction,
                } = &mut self.session;
                let (parameters, background_compute) = parameters_and_device_table
                    .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
                let advanced = publish_the_computed_compaction_batch_on_the_current_version(
                    parameters,
                    devices.as_mut_slice(),
                    allocator,
                    compaction,
                    current,
                    *instance,
                    *to_publish,
                    background_compute,
                    trip.rollback_gate_at_the_relock,
                );
                (advanced, drained)
            }
        };
        let finished = {
            let MountedSession {
                allocator,
                current,
                instance: _,
                shadow_ledger: _,
                parameters_and_device_table,
                compaction,
            } = &mut self.session;
            let parameters = parameters_and_device_table.parameters_on_disk();
            match note_the_compaction_batch_advanced(
                run,
                advanced,
                allocator,
                compaction,
                current,
                devices.len(),
            ) {
                Err(finished) => *finished,
                Ok(run) => match judge_before_the_next_compaction_batch(
                    run,
                    parameters,
                    devices.as_slice(),
                    allocator,
                    compaction,
                    current,
                    PacingOfTheRun::PacedByTheCaller {
                        write_bandwidth_limit: trip.write_bandwidth_limit,
                        yield_floor: trip.yield_floor,
                        caller,
                        span: SpanOfTheBackgroundCall::AtMostOnePublish,
                        batches_published_earlier_in_the_trip: trip
                            .batches_published_earlier_in_the_trip,
                    },
                ) {
                    Err(finished) => *finished,
                    Ok(_run_going_on_to_another_batch) => unreachable!(
                        "放锁的入口一次调用至多一次发布：这一批发成之后这一趟的发布数是 1，那一判先停在至多一次发布上"
                    ),
                },
            }
        };
        let call = self.session.after_the_background_run(
            devices,
            trip.trigger,
            trip.reach,
            trip.budget,
            SpanOfTheBackgroundCall::AtMostOnePublish,
            trip.batches_published_earlier_in_the_trip,
            trip.compaction_publishes_before_this_call,
            finished,
        );
        let has_published = match &call {
            BackgroundCompactionCall::Ran {
                run: run_of_the_call,
                ..
            } => !run_of_the_call.versions_published.is_empty(),
            BackgroundCompactionCall::NotStarted { .. } => false,
        };
        if has_published {
            self.note_a_publish(now);
        }
        let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();
        Ok(BackgroundCompactionAfterThePendingChanges {
            pending_changes_published_before_the_run: None,
            pending_changes_published_at_the_relock,
            call: call.with_the_alarm_at_the_end_of_the_wake(alarm),
        })
    }

    /// 放锁的三段入口之三的另一条出路：调用方丢掉了锁外算完的那一批、没回锁发它（运行库在停时这样做：GPU 实现放弃那一等交回的错不拿去回锁，
    /// D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交」）。是这个会话此刻在锁外的那一批，就照回锁时排空被拒那一支记账
    /// （[`BackgroundCompactionRelockRefused::PublishOfThePendingChangesRefused`]）：会话上不再记有一批在锁外；一个写都不发、发布预算不扣；
    /// 这一趟留着没完（跨调用发的批数照旧，全空段低于底额时起跑的那一趟破例照旧开着），停着删意图的那一趟不留（下一次调用照停没停重判）；
    /// 判过起跑的这一次是一次叫起，取这一次的报警交回。算着出的错不照它办（调用方没回锁，错是调用方自己在停引起的）。
    /// 不读盘、不写盘、不取 GPU 实现那把锁。不是此刻在锁外的那一批就什么都不动。
    pub fn note_the_batch_computed_out_of_the_lock_dropped_by_the_caller_without_relocking(
        &mut self,
        computed: BackgroundCompactionBatchComputed,
    ) -> BatchOutOfTheLockDroppedByTheCaller {
        let BackgroundCompactionBatchComputed {
            batch,
            trip,
            run: _,
            computed: _,
        } = computed;
        if !self.session.compaction.is_the_batch_out_of_the_lock(batch) {
            return BatchOutOfTheLockDroppedByTheCaller::NotTheBatchOutOfTheLockOfThisSession;
        }
        self.session
            .compaction
            .take_back_the_batch_out_of_the_lock(batch);
        match trip.trigger {
            BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}
            BackgroundCompactionTrigger::DueAfterAUserChange
            | BackgroundCompactionTrigger::TimerFallback
            | BackgroundCompactionTrigger::DeletionQueueNotEmpty => {
                self.session.compaction.keep_the_background_trip_under_way(
                    BackgroundTripUnderWay {
                        trigger: trip.trigger,
                        reach: trip.reach,
                        batches_published: trip.batches_published_earlier_in_the_trip,
                        is_the_exception_of_the_trip_still_open: trip.budget
                            == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite,
                    },
                );
            }
        }
        BatchOutOfTheLockDroppedByTheCaller::TakenBackWithoutPublishing {
            alarm_of_the_compaction_stopped_after_a_compute_error: self
                .session
                .alarm_of_the_compaction_at_the_end_of_a_wake(),
        }
    }
}
