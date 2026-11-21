//! 会话上的整理入口：显式跑一趟（`MountedSession::run_compaction` 一族）与后台入口（`MountedSession::run_compaction_in_the_background`），
//! 起不起跑、一趟的预算、节奏与跨度、跑完的报警，连同入口交回的结果与拒绝（[`CompactionRun`]、[`BackgroundCompactionCall`]）。

use super::*;

/// 一次整理运行（显式入口 [`MountedSession::run_compaction`]，或后台入口 [`MountedSession::run_compaction_in_the_background`] 起跑的那一趟）做成了什么。
/// 带着整理停着时这一次叫起的报警（[`Self::alarm_of_the_compaction_stopped_after_a_compute_error`]）：调用方要处理，整份丢掉编译就警告。
#[must_use = "整理因 GPU 出错停着时这一趟带着报警，要告诉文件系统的用户（用户 2026-10-30 定）"]
#[derive(Debug)]
pub struct CompactionRun {
    /// 这一趟发成的每一版，按先后：先走空的删除续做、再是整理的各批。最后一个就是会话的 `current`。
    pub versions_published: Vec<TransactionOutput>,
    /// 其中删除续做的步数（排在最前面）：其余才是整理的发布。
    pub deletion_steps_published: u64,
    /// 其中整理批的个数：搬了单元的那几次（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2），签发、
    /// 一个没搬的删意图与丢意图不算。
    pub batches_published: u64,
    pub outcome: CompactionRunOutcome,
    /// 这一趟的发布都返回之后发的 discard（显式入口与后台入口在这里发；可写挂载续做那一趟不发）。
    pub discards: DiscardsAfterThePublishes,
    /// 整理每推一次发布（[`advance_compaction_by_one_publish`] 每进一次门：签发、搬一批、删意图，停在判据上的那一次也算）是后台任务的一批，
    /// 按先后一批一项：请求的后端（这一批开头读到的设置）与实际走的路（D26（后台整理与放置回收） 已定项 13 第 1 条；
    /// `.claude/rules/fs-design.md` 五条硬要求第 4 条）。删除续做那几步不是整理的批，不在这里；整理停着、占用不超过补跑阈值而没开成的那一批也不在
    /// （一个都没算，用户 2026-10-28 定案）。
    pub background_compute_of_each_batch: Vec<BackgroundComputeOfOneBatch>,
    /// 这一趟结束时整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条；用户 2026-10-30 定「之后每次都要报警」）：这一次叫起的报警，
    /// 不论这一趟补跑了几批还是照旧停着、此刻 GPU 用不用得上；这一趟里出错停下的也报（叫起次数 1）。没停着是 `None`。
    /// 显式入口与后台入口起跑的那一趟填它（[`MountedSession::run_compaction`]、[`MountedSession::run_compaction_in_the_background`]）；
    /// 可写挂载续做那一趟不是叫起，恒是 `None`。
    pub alarm_of_the_compaction_stopped_after_a_compute_error:
        Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
}

/// 一次整理运行停在哪：照判据停（[`CompactionRunStop`]），或一次发布没发成。
#[derive(Debug)]
pub enum CompactionRunOutcome {
    Stopped(CompactionRunStop),
    Failed(CompactionRunFailure),
}

/// 整理运行里一次发布没发成（一批被空间拒不在这里：那是暂停，[`CompactionRunStop::PausedShortOfSpace`]；暂停或出错停下时只删意图的那一次
/// 没发成在这里，意图留在盘上、R 照旧排他）。已发成的那几版留在盘上、`current` 停在最后那一版。
#[derive(Debug)]
pub enum CompactionRunFailure {
    /// 先走空的删除续做有一步没发成。
    DeletionContinuation(PublishError),
    /// 一批没发成（计划对不上上一版，或发布路径的错）。
    Batch(RelocationBatchRefused),
}

/// 整理运行在任何读写之前就拒了。
#[derive(Debug)]
pub enum CompactionRunRefused {
    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]：交进来的盘表与这次挂载核过的那一份不同。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版树表 0 条：没有可搬的单元。
    NoVersionOfTrees,
}

/// 后台入口一次调用做成了什么（[`MountedSession::run_compaction_in_the_background`]），连同整理停着时这一次叫起的报警
/// （[`Self::alarm_of_the_compaction_stopped_after_a_compute_error`]）：调用方要处理，整份丢掉编译就警告。
#[must_use = "整理因 GPU 出错停着时后台入口每一次都带着报警，要告诉文件系统的用户（用户 2026-10-30 定）"]
#[derive(Debug)]
pub enum BackgroundCompactionCall {
    /// 没起跑，一个写都没发。`alarm_of_the_compaction_stopped_after_a_compute_error` 同 [`CompactionRun::alarm_of_the_compaction_stopped_after_a_compute_error`]：
    /// 整理停着就有（停着而占用不超过补跑阈值没起跑的、超过阈值而这一次没到起跑的都有）。
    NotStarted {
        reason: BackgroundCompactionNotStarted,
        alarm_of_the_compaction_stopped_after_a_compute_error:
            Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
    },
    /// 起跑了一趟：为什么起跑、那一趟停在哪、发了哪几版。装箱：不装箱这一个成员比另一个大两百多字节（clippy `large_enum_variant`）。
    Ran {
        trigger: BackgroundCompactionTrigger,
        run: Box<CompactionRun>,
    },
}

impl BackgroundCompactionCall {
    /// 整理停着时这一次叫起的报警：没起跑的在成员里，起跑了的在那一趟里（[`CompactionRun::alarm_of_the_compaction_stopped_after_a_compute_error`]）。
    #[must_use]
    pub fn alarm_of_the_compaction_stopped_after_a_compute_error(
        &self,
    ) -> Option<&AlarmOfABackgroundTaskStoppedAfterAComputeError> {
        match self {
            BackgroundCompactionCall::NotStarted {
                alarm_of_the_compaction_stopped_after_a_compute_error,
                ..
            } => alarm_of_the_compaction_stopped_after_a_compute_error.as_ref(),
            BackgroundCompactionCall::Ran { run, .. } => run
                .alarm_of_the_compaction_stopped_after_a_compute_error
                .as_ref(),
        }
    }

    /// 这一次叫起结束时的报警（`alarm`）装进去：没起跑的装进成员，起跑了的装进那一趟。
    pub(super) fn with_the_alarm_at_the_end_of_the_wake(
        self,
        alarm: Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
    ) -> Self {
        match self {
            BackgroundCompactionCall::NotStarted { reason, .. } => {
                BackgroundCompactionCall::NotStarted {
                    reason,
                    alarm_of_the_compaction_stopped_after_a_compute_error: alarm,
                }
            }
            BackgroundCompactionCall::Ran { trigger, mut run } => {
                run.alarm_of_the_compaction_stopped_after_a_compute_error = alarm;
                BackgroundCompactionCall::Ran { trigger, run }
            }
        }
    }
}

/// 后台入口这一次为什么没起跑。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundCompactionNotStarted {
    /// 整理因一批出错停着（D26（后台整理与放置回收） 已定项 13 第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值（用户 2026-10-28 定案），
    /// 现行那一版里又没有要删的活意图（有而上一次用户改动给的那一份 `B_gc` 扣完、前台写得进的也在这里，意图与 R 排他留到下一次用户改动之后那一次调用）：
    /// 「该跑了」留着，解除之后（或占用超过阈值时）下一次调用再取。
    BackgroundTaskStoppedAfterAComputeError,
    /// 没有「该跑了」，定时兜底关着。
    NothingDueAndTheTimerFallbackOff,
    /// 没有「该跑了」，离定时兜底上一次查还不到一个间隔。
    TimerFallbackIntervalNotElapsed,
    /// 没有「该跑了」，定时兜底这一次查了起跑判据，为假。
    NotDueByTheWatermarksAtTheTimerFallback,
    /// 上一次用户改动给的那一份 `B_gc` 已经扣完（岔路单第 18 行甲：定时兜底与用户改动共用这一份）：等下一次用户改动才再有。
    /// 「该跑了」也取走、不留：下一次用户改动发成之后起跑判据再记。
    PublishBudgetOfTheLastUserChangeSpent,
    /// 水位越线，而上一趟不发布地停在这一条（[`CompactionRunStop::publishes_nothing_until_the_next_publish`]）之后现行那一版没变、开关没变：不重判，
    /// 一个盘都不读。任何一次发布都让 txg 变、这条备忘失效。
    UnchangedSinceAStopThatPublishedNothing { stop: CompactionRunStop },
    /// 这个会话有一批在锁外（放锁的三段入口取了输入、还没回锁，[`MountedSessionWithPendingChanges::take_the_input_of_a_background_compaction_batch`]）：
    /// 第二个后台调用者在取 GPU 实现那把锁与换设置之前就不起跑，不排空待发、不算一次叫起、不取报警。
    ABatchOfThisSessionIsOutOfTheLock,
}

/// 后台入口这一趟为什么起跑。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundCompactionTrigger {
    /// 用户改动发成之后起跑判据记下的「该跑了」（判决 G8 丁：至多一批，预算 `B_gc`）。
    DueAfterAUserChange,
    /// 定时兜底这一次查起跑判据为真（D26 已定项 13 第 5 条）。这一趟没有自己的预算，与用户改动共用上一次用户改动给的那一份 `B_gc`
    /// （岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲）。
    TimerFallback,
    /// 现行那一版的删除队列不空（D26（后台整理与放置回收） 已定项 13）：只走队列、不发整理，不看水位，也不被那一份 `B_gc` 挡（删除续做不占预算）。
    /// 调用方挂载之后不调任何后台入口，队列没走空时前台的写都被拒（[`AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`]），没有别人走它。
    DeletionQueueNotEmpty,
    /// 整理因一批出错停着而不补跑（D26（后台整理与放置回收） 已定项 13 第 4 条），现行那一版里还有活意图（补跑那几批签下的、挂载续做的、停下那一刻没删成的）：
    /// 这一趟开头仍判出停着时只单发一次删意图的发布（同显式入口一趟开头判出停着那一段，[`drop_the_live_intent_on_a_stop`]），停在
    /// [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；删完之后的调用照旧不起跑。预算照后台入口别的起跑给：那一份扣完而前台写得进就不起跑，
    /// 全空段低于底额、前台写不进时破例至少 1 次。
    /// 那一次删意图是整理发布，开关照水位判时发之前过回退门（D26（后台整理与放置回收） 已定项 13 第 5 条）：门关着这一趟停在
    /// [`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]、一个写都不发，意图与 R 排他留到门开之后那一次调用再删。
    /// 破例那一档（前台写不进时给的预算）不过门，门关着照发。
    /// 放锁的入口排空待发被拒时这一趟不记成没完，下一次调用照停没停重判（解除之后照普通整理判起跑与触发）。排空发成、这几笔写把某一块盘推过补跑阈值时，
    /// 这一趟开头判出不再停着，照普通整理推（每一批在 CPU 上补跑），触发名仍是这一个；放锁的入口这时停在
    /// [`CompactionRunStop::OnePublishOfTheTripDone`] 就以这个触发名记成没完的一趟，下一次调用不重判起跑、接着推。
    LiveIntentWhileStoppedAfterAComputeError,
}

/// 后台入口这一次起不起跑（[`MountedSession::run_compaction_in_the_background`] 开头那几判的结论）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BackgroundCompactionStart {
    NotStarted(BackgroundCompactionNotStarted),
    /// 起跑：为什么起跑、这一趟推到哪、预算按哪一条算；放锁的后台入口接着推没完的一趟时，这一趟前几次调用已发了几批。
    Starts {
        trigger: BackgroundCompactionTrigger,
        reach: CompactionRunReach,
        budget: CompactionRunBudget,
        batches_published_earlier_in_the_trip: u64,
    },
}

/// 一趟整理的发布预算按哪一条算。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompactionRunBudget {
    /// 判决 G8 `B_gc`：上一次用户改动之后已经过去的发布数之外还剩的（[`publish_budget_of_a_compaction_run`]，至少 1）。
    /// 显式入口与可写挂载续做照它算。
    SinceTheLastUserChange,
    /// 后台入口的一趟（用户改动之后那一趟与定时兜底那几趟）：上一次用户改动给的那一份 `B_gc` 还剩的，没有至少 1，扣完是 0
    /// （[`publishes_left_of_the_budget_given_by_the_last_user_change`]；岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲）。
    LeftOfTheBudgetGivenByTheLastUserChange,
    /// 同上，只差全空段低于底额、前台写不进时破例（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：这一趟至少发 1 次，第一次发布不判回退门，也不为前台 fsync 让路。
    /// 前台等整理腾段，整理不能再等前台的用户改动给预算。
    AtLeastOnePublishWhileTheForegroundCannotWrite,
}

/// 后台入口一次调用推多少。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpanOfTheBackgroundCall {
    /// 一次调用把一趟推到停（[`MountedSession::run_compaction_in_the_background`]）。
    WholeTrip,
    /// 至多一次发布就返回（[`MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`]）：
    /// 调用方放锁之后接着调，这一趟记在会话上，不重判起跑。整理之前的删除续做也一次调用至多走一步。一趟之后的 discard 切不开。
    AtMostOnePublish,
}

/// 一趟整理每一批之前看不看调用方（D26（后台整理与放置回收） 已定项 13 第 6 条）。封闭集合：显式入口与可写挂载续做不看，后台入口看。
#[derive(Clone, Copy)]
pub(super) enum PacingOfTheRun<'caller> {
    NotPaced,
    PacedByTheCaller {
        write_bandwidth_limit: CompactionWriteBandwidthLimit,
        /// 前台在等会话锁时让路的下限（D26（后台整理与放置回收） 已定项 13 第 6 条）：今天锁里推完一趟的入口不设下限。
        yield_floor: CompactionYieldFloor,
        caller: &'caller dyn CallerOfTheBackgroundCompaction,
        span: SpanOfTheBackgroundCall,
        batches_published_earlier_in_the_trip: u64,
    },
}

/// 这一趟整理的写带宽上限：后台入口照调用方给的那一个，显式入口与可写挂载续做不限。
pub(super) fn write_bandwidth_limit_of_the_pacing(
    pacing: PacingOfTheRun<'_>,
) -> CompactionWriteBandwidthLimit {
    match pacing {
        PacingOfTheRun::NotPaced => CompactionWriteBandwidthLimit::Unlimited,
        PacingOfTheRun::PacedByTheCaller {
            write_bandwidth_limit,
            ..
        } => write_bandwidth_limit,
    }
}

impl MountedSession {
    /// 整理的显式入口（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G8 甲）：先把删除续做走空，再一次一次推整理意图——
    /// 没有活意图就选段、签发前判、发一次把意图写进 logged_ops 的发布（判决 G1）；有就对 R 重扫（不复用上一批的扫描，丁 1 交判第 3 点）、
    /// R 里有隔离槽就发一次删意图的发布丢掉它（攻方 K-c）、计划一批、发这一批，最后一批同一次发布删意图（D8（核心索引结构） 已定项 5）；
    /// 全局差值为假就不签下一条。这一趟至多发 `B_gc` 次整理发布（[`publish_budget_of_a_compaction_run`]，签发、搬一批、删意图各算一次）；
    /// 一次整理发布被空间拒就暂停：发一次只删意图的发布丢掉意图、连同 R 排他一起撤，不推抬 F，下一趟从头签发。发布都返回之后发 discard（判决 G5）。
    ///
    /// # Errors
    /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前。
    pub fn run_compaction<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        self.run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region(
            devices,
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        )
    }

    /// 同 [`Self::run_compaction`]，只差一处：一批照抄意图核出腾空了 R、照同一批删意图再发时，删意图那一支落盘之前的核照 `region_check` 判
    /// （判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2）。别处删意图（最后一批、R 已空的只删意图）照记录核，不读它。
    /// 产品路径只经 [`Self::run_compaction`] 进来（[`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]）；只供测试的
    /// [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制进入「删意图那一支核不过、照这一批不删意图再发」
    /// 那一支（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：发出去的那一版 R 按记录全空、意图还在，下一次发布照「R 已按记录全空」只删意图。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction`]。
    pub fn run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        region_check: RegionCheckOfTheIntentDeletion,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        self.run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch(
            devices,
            region_check,
            InTheMiddleOfEachBackgroundBatch::NothingHappens,
        )
    }

    /// 同 [`Self::run_compaction`]，只差一处：整理的每一批读完后台计算设置、开算之前调用例给的钩子（只供测试，
    /// `.claude/rules/fs-design.md`「五条硬要求」第 2 条）。钩子拿到这次挂载的后台计算状态，在这一刻请求切换，强制进入
    /// 「一批跑到一半时请求切换：这一批照旧、下一批才换」那一格（D26（后台整理与放置回收） 已定项 13 第 1 条）。产品路径只经
    /// [`Self::run_compaction`] 进来，钩子那一处什么都不做。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction`]。
    pub fn run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        hook_in_the_middle_of_each_batch: &mut dyn FnMut(&mut BackgroundComputeOfThisMount),
    ) -> Result<CompactionRun, CompactionRunRefused> {
        self.run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch(
            devices,
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
            InTheMiddleOfEachBackgroundBatch::CallTheTestOnlyHook(hook_in_the_middle_of_each_batch),
        )
    }

    /// 显式入口的三种起法共用的那一段：这个会话有一批在锁外（放锁的三段入口取了输入、还没回锁）就一个写都不发、停在
    /// [`CompactionRunStop::NotRunWhileABatchOfThisSessionIsOutOfTheLock`]，在取 GPU 实现那把锁与换设置之前；盘表核过、现行那一版有树，就推一趟、发 discard。
    fn run_compaction_with_the_region_check_and_a_hook_in_the_middle_of_each_batch<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        region_check: RegionCheckOfTheIntentDeletion,
        mut in_the_middle_of_each_batch: InTheMiddleOfEachBackgroundBatch<'_>,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction,
        } = self;
        if compaction.has_a_batch_out_of_the_lock() {
            return Ok(CompactionRun {
                versions_published: Vec::new(),
                deletion_steps_published: 0,
                batches_published: 0,
                outcome: CompactionRunOutcome::Stopped(
                    CompactionRunStop::NotRunWhileABatchOfThisSessionIsOutOfTheLock,
                ),
                discards: DiscardsAfterThePublishes::default(),
                background_compute_of_each_batch: Vec::new(),
                alarm_of_the_compaction_stopped_after_a_compute_error: None,
            });
        }
        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        match &*current {
            PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
            PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
        }
        let (parameters, background_compute) = parameters_and_device_table
            .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
        // 过了在任何读写之前的那两判，这一趟就是一次叫起：这一趟里一批出错停下的，这一趟的报警不数它（用户 2026-11-01 定）。
        background_compute.begin_a_wake();
        let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(
            parameters,
            devices.as_mut_slice(),
            allocator,
            compaction,
            current,
            *instance,
            CompactionRunReach::UntilAStopOrTheBudget,
            CompactionRunBudget::SinceTheLastUserChange,
            PacingOfTheRun::NotPaced,
            region_check,
            background_compute,
            &mut in_the_middle_of_each_batch,
        );
        run.discards = discard_the_segments_reusable_as_a_whole(
            devices.as_mut_slice(),
            allocator,
            parameters,
            current.root(),
        );
        // 用户 2026-10-30 定：整理停着期间显式入口每一趟都报警（这一趟里出错停下的也报，叫起次数报 0）。
        run.alarm_of_the_compaction_stopped_after_a_compute_error =
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
        Ok(run)
    }

    /// 只供测试（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：此刻在现行那一版上判一次回退门（整理每次发布之前那一判，[`stop_of_the_rollback_gate`]），
    /// 门关时交回停在哪一条，门开是 `None`。只读根环与每条有效根的树表，分配器与盘一样都不动。用例拿它造「取输入时门开、回锁时门关」那一格。
    pub fn stop_of_the_rollback_gate_now_for_tests<Device: BlockDevice>(
        &self,
        devices: &[(DeviceIdentity, Device)],
    ) -> Option<CompactionRunStop> {
        stop_of_the_rollback_gate(
            self.parameters_and_device_table.parameters_on_disk(),
            devices,
            &self.allocator,
            &self.current,
        )
    }

    /// 整理停着时这一次叫起的报警（[`BackgroundComputeOfThisMount::alarm_at_the_end_of_a_wake`]），后台入口每一次判过起跑、跑完之后取。
    pub(super) fn alarm_of_the_compaction_at_the_end_of_a_wake(
        &mut self,
    ) -> Option<AlarmOfABackgroundTaskStoppedAfterAComputeError> {
        self.parameters_and_device_table
            .background_compute_of_this_mount_mut()
            .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
    }

    /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条：GC 挪出用户改动的调用线程，三样保证照旧）：调用方在它自己的线程上
    /// 一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）。每次调用先看起不起跑，依次：
    /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值：现行那一版里有活意图（补跑那几批签下的、挂载续做的、
    ///    停下那一刻没删成的）而上一次用户改动给的那一份 `B_gc` 还剩（或全空段低于底额、前台写不进）⇒ 起跑一趟只单发一次删意图的发布
    ///    （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]，同显式入口一趟开头判出停着那一段），停在
    ///    [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；那一次删意图也是整理发布，开关照水位判时发之前同样过回退门（第 5 条），
    ///    门关着就停在 [`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]、一个写都不发，意图与 R 排他留到门开；
    ///    没有活意图、或那一份扣完而前台写得进 ⇒ 不起跑，「该跑了」留着，
    ///    一个写都不发（删完意图之后的调用都走这一支）。挂着切到一张卡都用不上的那一组、手里没有活意图时，这一次就在任何写之前报停着；
    ///    超过阈值就照下面几条照常判起不起跑，起跑之后每一批在 CPU 上补跑（第 4 条）；
    /// 2. 用户改动之后起跑判据记下了「该跑了」⇒ 取走，照用户改动之后那一趟推（判决 G8 丁：至多一批）；
    /// 3. 没有 ⇒ 看定时兜底（`pacing.timer_fallback`）：关着、或离上一次查还不到一个间隔就不起跑；到了就再查一次起跑判据，为真就推一趟到停；
    /// 4. 用户改动之后那一趟与定时兜底那一趟都从上一次用户改动给的那一份 `B_gc` 里扣（岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，
    ///    用户 2026-10-26 定「与用户改动共用 `B_gc`」；[`publishes_left_of_the_budget_given_by_the_last_user_change`]）：那一份扣完就不起跑
    ///    （[`BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent`]），等下一次用户改动才再有。闲置时整理发布因此合计不过
    ///    一份 `B_gc`，不随定时兜底查了几次涨。全空段低于底额、前台写不进时破例：那一份扣完也起跑、这一趟至少发 1 次
    ///    （[`CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite`]）；
    /// 5. 开关照水位判（[`CompactionRunForcing::FollowTheWatermarks`]）时另有三样：头一次判起跑把挂载那一份预算从根环读（挂载不给新预算；排在第 1 条那一判之前，
    ///    停着时起跑的删意图那一趟同样吃这一份）；上一趟不发布地停下之后
    ///    现行那一版与开关都没变就不重判（[`BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing`]；全空段低于底额时停在回退门上的那一次不算）；每一次整理发布之前判回退门
    ///    （[`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]）。
    ///
    /// 整理没停着而现行那一版的删除队列不空：排在第 2、3、4 条之前起跑（[`BackgroundCompactionTrigger::DeletionQueueNotEmpty`]），只走队列、不发整理，不看水位、不被预算挡，「该跑了」与定时兜底都不动；
    /// 走空之后下一次调用照上面几条判。整理停着时队列也不走（第 1 条：至多单发一次删意图，队列照抄）。
    ///
    /// 起跑之后每一批之前问调用方：前台有 fsync 在等就不发下一批（[`CompactionRunStop::YieldedToAForegroundFsync`]；全空段低于底额时这一趟的第一次发布除外），这一个写带宽窗口里
    /// 整理写的字节到了 `pacing.write_bandwidth_limit` 也不发（[`CompactionRunStop::WriteBandwidthLimitReached`]）；用户改动之后起跑的那一趟
    /// 停在这两样上时「该跑了」放回去，下一次调用接着推（不发下一批是缓发，不是做完）。签下一条意图那一判
    /// 读会话上跨趟留着的上一条完成的意图（`compaction_plan::CompactionOfThisMount::last_completed_intent`）。发布都返回之后发 discard（判决 G5）。
    ///
    /// # Errors
    /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前（「该跑了」留着）。
    pub fn run_compaction_in_the_background<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<BackgroundCompactionCall, CompactionRunRefused> {
        // 有一批在锁外：第二个后台调用者在取 GPU 实现那把锁与换设置之前就不起跑，这一次不算叫起（不取报警，那一批的叫起还没结束）。
        if self.compaction.has_a_batch_out_of_the_lock() {
            return Ok(BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::ABatchOfThisSessionIsOutOfTheLock,
                alarm_of_the_compaction_stopped_after_a_compute_error: None,
            });
        }
        let call = match self.whether_the_background_compaction_starts(devices, pacing, caller)? {
            BackgroundCompactionStart::NotStarted(reason) => BackgroundCompactionCall::NotStarted {
                reason,
                alarm_of_the_compaction_stopped_after_a_compute_error: None,
            },
            BackgroundCompactionStart::Starts {
                trigger,
                reach,
                budget,
                batches_published_earlier_in_the_trip,
            } => self.run_the_background_compaction_that_starts(
                devices,
                pacing,
                caller,
                trigger,
                reach,
                budget,
                SpanOfTheBackgroundCall::WholeTrip,
                batches_published_earlier_in_the_trip,
            ),
        };
        // 用户 2026-10-30 定：整理停着期间后台入口每一次判起跑都报警，起没起跑、补没补跑都报。
        let alarm = self.alarm_of_the_compaction_at_the_end_of_a_wake();
        Ok(call.with_the_alarm_at_the_end_of_the_wake(alarm))
    }

    /// [`Self::run_compaction_in_the_background`] 开头那几判：盘表、有没有树、整理停没停、有没有「该跑了」、定时兜底、上一次用户改动给的那一份
    /// `B_gc` 还剩不剩。取走「该跑了」与定时兜底记下这一次查的读数都在这里做，一个盘都不读、一个写都不发。攒批的会话
    /// （[`MountedSessionWithPendingChanges::run_compaction_in_the_background`]）在这一判说起跑之后、跑之前先排空待发。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction_in_the_background`]。
    pub(super) fn whether_the_background_compaction_starts<Device: BlockDevice>(
        &mut self,
        devices: &[(DeviceIdentity, Device)],
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
    ) -> Result<BackgroundCompactionStart, CompactionRunRefused> {
        let MountedSession {
            allocator,
            current,
            instance: _,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction,
        } = self;
        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        match &*current {
            PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
            PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
        }
        // 过了在任何读写之前的那两判，这一次判起跑就是一次叫起（结束时取报警）：之后起跑的那一趟里一批出错停下的，这一次的报警不数它（用户 2026-11-01 定）。
        parameters_and_device_table
            .background_compute_of_this_mount_mut()
            .begin_a_wake();
        // 挂着切到一张卡都用不上的那一组：在判停没停之前换上它（两次调用之间不在一批之内），与挂载那一刻一样当场停着，不取走「该跑了」。
        parameters_and_device_table
            .background_compute_of_this_mount_mut()
            .put_the_switch_requested_from_the_next_batch_in_effect();
        let parameters = parameters_and_device_table.parameters_on_disk();
        let background_compute = parameters_and_device_table.background_compute_of_this_mount();
        let is_judged_as_the_product_does = matches!(
            allocator.compaction().switches.run_forcing,
            CompactionRunForcing::FollowTheWatermarks
        );
        // 挂载不给新预算（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：这次挂载头一次判后台起跑时，已经过去的发布数取根环里最新那个用户状态之后的根数
        // （写行、暖机、上一次挂载的整理都在里面）。读不通当那一份已扣完。排在停没停那一判之前：停着而有活意图时起跑的那一趟同样吃这一份。
        if is_judged_as_the_product_does
            && compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()
        {
            let publishes_read_from_the_ring = allocator
                .reading_and_counting_the_other_unit_reads_with_one_copy_failing(
                    &PoolReaderOfOneReadPass::reading_for_the_first_time(devices),
                    |reader| rollback_states_of_the_ring(reader, parameters, current.root()),
                )
                .map_or(
                    publish_budget_of_a_compaction_run(
                        parameters.geometry.root_ring_slots_per_region.count(),
                        0,
                    ),
                    |states| states.publishes_after_the_newest_state,
                );
            compaction.publishes_since_the_last_user_change = compaction
                .publishes_since_the_last_user_change
                .max(publishes_read_from_the_ring);
        }
        if compaction_stays_stopped(allocator, background_compute) {
            return Ok(start_while_the_compaction_stays_stopped(
                allocator,
                current,
                parameters.geometry.root_ring_slots_per_region.count(),
                compaction.publishes_since_the_last_user_change,
            ));
        }
        let foreground_cannot_write = foreground_cannot_write_under_the_switches(allocator);
        // 放锁的后台入口上一次调用之后还没完的那一趟接着推：一趟只判一次起跑。整理停着的那一判在上面：停着时这一趟留着，解除之后接着推。
        // 那一份预算扣完就不再推整理，只有两样例外：删除续做不占预算，队列没走空照走；这一趟的破例还没用掉。
        // 只走队列的那一趟，队列已被前台走空时没什么可推，这一趟到此为止，下面照常判。
        if let Some(trip) = compaction
            .take_the_background_trip_under_way()
            .filter(|trip| {
                trip.reach != CompactionRunReach::OnlyTheDeletionQueue
                    || !deletion_queue_of_the_version_is_empty(current)
            })
        {
            let is_the_exception_still_open =
                trip.is_the_exception_of_the_trip_still_open && foreground_cannot_write;
            if publishes_left_of_the_budget_given_by_the_last_user_change(
                parameters.geometry.root_ring_slots_per_region.count(),
                compaction.publishes_since_the_last_user_change,
            ) == 0
                && deletion_queue_of_the_version_is_empty(current)
                && !is_the_exception_still_open
            {
                return Ok(BackgroundCompactionStart::NotStarted(
                    BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,
                ));
            }
            return Ok(BackgroundCompactionStart::Starts {
                trigger: trip.trigger,
                reach: trip.reach,
                budget: if is_the_exception_still_open {
                    CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
                } else {
                    CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange
                },
                batches_published_earlier_in_the_trip: trip.batches_published,
            });
        }
        // 队列不空就起跑，只走队列：「该跑了」与定时兜底都不动，队列走空之后照常判。
        if !deletion_queue_of_the_version_is_empty(current) {
            return Ok(BackgroundCompactionStart::Starts {
                trigger: BackgroundCompactionTrigger::DeletionQueueNotEmpty,
                reach: CompactionRunReach::OnlyTheDeletionQueue,
                budget: CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange,
                batches_published_earlier_in_the_trip: 0,
            });
        }
        let (trigger, reach) = match compaction.start_judgement {
            CompactionStartJudgement::DueAfterAUserChange => {
                compaction.start_judgement = CompactionStartJudgement::NotDue;
                (
                    BackgroundCompactionTrigger::DueAfterAUserChange,
                    CompactionRunReach::AtMostOneBatchAfterAUserChange,
                )
            }
            CompactionStartJudgement::NotDue => {
                match compaction
                    .check_the_timer_fallback(pacing.timer_fallback, caller.clock_reading())
                {
                    TimerFallbackCheck::Off => {
                        return Ok(BackgroundCompactionStart::NotStarted(
                            BackgroundCompactionNotStarted::NothingDueAndTheTimerFallbackOff,
                        ))
                    }
                    TimerFallbackCheck::IntervalNotElapsed => {
                        return Ok(BackgroundCompactionStart::NotStarted(
                            BackgroundCompactionNotStarted::TimerFallbackIntervalNotElapsed,
                        ))
                    }
                    TimerFallbackCheck::Due => {}
                }
                let watermark_verdict = compaction_watermark_verdict_of_the_allocator(allocator);
                compaction.last_watermark_verdict = Some(watermark_verdict);
                if !watermark_verdict.starts() {
                    return Ok(BackgroundCompactionStart::NotStarted(
                        BackgroundCompactionNotStarted::NotDueByTheWatermarksAtTheTimerFallback,
                    ));
                }
                if is_judged_as_the_product_does {
                    // 全空段低于底额、前台写不进时每趟第一次发布不判门：门关着的备忘挡不住这一趟。
                    if let Some(stop) = compaction
                        .stop_still_holding_at(
                            current.root().checkpoint_txg,
                            allocator.compaction().switches,
                        )
                        .filter(|stop| {
                            !(foreground_cannot_write
                                && matches!(
                                    stop,
                                    CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
                                ))
                        })
                    {
                        return Ok(BackgroundCompactionStart::NotStarted(
                            BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing {
                                stop,
                            },
                        ));
                    }
                }
                (
                    BackgroundCompactionTrigger::TimerFallback,
                    CompactionRunReach::UntilAStopOrTheBudget,
                )
            }
        };
        // 两种起跑共用上一次用户改动给的那一份 `B_gc`：扣完就不起跑，等下一次用户改动。全空段低于底额、前台写不进时破例。
        if publishes_left_of_the_budget_given_by_the_last_user_change(
            parameters.geometry.root_ring_slots_per_region.count(),
            compaction.publishes_since_the_last_user_change,
        ) == 0
            && !foreground_cannot_write
        {
            return Ok(BackgroundCompactionStart::NotStarted(
                BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent,
            ));
        }
        Ok(BackgroundCompactionStart::Starts {
            trigger,
            reach,
            budget: if foreground_cannot_write {
                CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
            } else {
                CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange
            },
            batches_published_earlier_in_the_trip: 0,
        })
    }

    /// [`Self::run_compaction_in_the_background`] 判过起跑之后那一段：推一趟（每一批之前问调用方），再照停在哪收这一次调用（[`Self::after_the_background_run`]）。
    /// `span` 是至多一次发布时，这一趟没完（发了一次、或被节奏截住）就记在会话上，下一次调用接着推。今天的调用方让路不设下限（[`CompactionYieldFloor::NO_FLOOR`]）。
    #[allow(
        clippy::too_many_arguments,
        reason = "参数各是一样东西：盘表、节奏、调用方、为什么起跑、推到哪、预算按哪一条算、这次调用推多少、这一趟前几次调用发的批数"
    )]
    pub(super) fn run_the_background_compaction_that_starts<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        caller: &dyn CallerOfTheBackgroundCompaction,
        trigger: BackgroundCompactionTrigger,
        reach: CompactionRunReach,
        budget: CompactionRunBudget,
        span: SpanOfTheBackgroundCall,
        batches_published_earlier_in_the_trip: u64,
    ) -> BackgroundCompactionCall {
        let compaction_publishes_before_this_call =
            self.compaction.publishes_since_the_last_user_change;
        let run = {
            let MountedSession {
                allocator,
                current,
                instance,
                shadow_ledger: _,
                parameters_and_device_table,
                compaction,
            } = &mut *self;
            let (parameters, background_compute) = parameters_and_device_table
                .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
            run_compaction_on_with_the_region_check_of_the_intent_deletion(
                parameters,
                devices.as_mut_slice(),
                allocator,
                compaction,
                current,
                *instance,
                reach,
                budget,
                PacingOfTheRun::PacedByTheCaller {
                    write_bandwidth_limit: pacing.write_bandwidth_limit,
                    yield_floor: CompactionYieldFloor::NO_FLOOR,
                    caller,
                    span,
                    batches_published_earlier_in_the_trip,
                },
                RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
                background_compute,
                &mut InTheMiddleOfEachBackgroundBatch::NothingHappens,
            )
        };
        self.after_the_background_run(
            devices,
            trigger,
            reach,
            budget,
            span,
            batches_published_earlier_in_the_trip,
            compaction_publishes_before_this_call,
            run,
        )
    }

    /// 后台入口的一趟停下之后收这一次调用（锁里推完的、放锁的三段入口回锁之后推完的共用）：节奏截住时把「该跑了」放回去、
    /// 至多一次发布的那一种这一趟没完就记在会话上，发 discard。哪些停算没做完（[`stop_holds_back_the_trip_without_ending_it`]）、
    /// 哪些是给前台让路（[`stop_yields_to_the_foreground`]）各是一张穷举的表。
    #[allow(
        clippy::too_many_arguments,
        reason = "参数各是一样东西：盘表、为什么起跑、推到哪、预算按哪一条算、这次调用推多少、这一趟前几次调用发的批数、这次调用之前整理发了几次、推完的那一趟"
    )]
    pub(super) fn after_the_background_run<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        trigger: BackgroundCompactionTrigger,
        reach: CompactionRunReach,
        budget: CompactionRunBudget,
        span: SpanOfTheBackgroundCall,
        batches_published_earlier_in_the_trip: u64,
        compaction_publishes_before_this_call: u64,
        mut run: CompactionRun,
    ) -> BackgroundCompactionCall {
        let MountedSession {
            allocator,
            current,
            instance: _,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction,
        } = self;
        let parameters = parameters_and_device_table.parameters_on_disk();
        // 节奏让下一批缓发（前台有入口在等会话锁、写带宽到了上限）、回锁时这一批没发成（剔完一个不剩、选中那一段没了资格、有冻结着的发布）不是做完：
        // 用户改动之后那一趟被它截住时「该跑了」放回去，下一次调用接着推。
        let is_held_back_by_the_pacing = match run.outcome {
            CompactionRunOutcome::Stopped(stop) => stop_holds_back_the_trip_without_ending_it(stop),
            CompactionRunOutcome::Failed(_) => false,
        };
        let is_done_with_this_call_only = matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::OnePublishOfTheTripDone)
        );
        // 全空段低于底额时为前台让路的这一趟不留着：留着的话下一次调用接着让路，前台一直有入口在等就再也轮不到下一趟的破例。
        let is_yielding_while_the_foreground_cannot_write = match run.outcome {
            CompactionRunOutcome::Stopped(stop) => stop_yields_to_the_foreground(stop),
            CompactionRunOutcome::Failed(_) => false,
        }
            && foreground_cannot_write_under_the_switches(allocator);
        match span {
            SpanOfTheBackgroundCall::AtMostOnePublish
                if (is_held_back_by_the_pacing
                    && !is_yielding_while_the_foreground_cannot_write)
                    || is_done_with_this_call_only =>
            {
                compaction.keep_the_background_trip_under_way(BackgroundTripUnderWay {
                    trigger,
                    reach,
                    batches_published: batches_published_earlier_in_the_trip
                        + run.batches_published,
                    is_the_exception_of_the_trip_still_open: budget
                        == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite
                        && compaction.publishes_since_the_last_user_change
                            == compaction_publishes_before_this_call,
                });
            }
            SpanOfTheBackgroundCall::AtMostOnePublish | SpanOfTheBackgroundCall::WholeTrip => {
                match trigger {
                    BackgroundCompactionTrigger::DueAfterAUserChange
                        if is_held_back_by_the_pacing =>
                    {
                        compaction.start_judgement = CompactionStartJudgement::DueAfterAUserChange;
                    }
                    BackgroundCompactionTrigger::DueAfterAUserChange
                    | BackgroundCompactionTrigger::TimerFallback
                    | BackgroundCompactionTrigger::DeletionQueueNotEmpty
                    | BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}
                }
            }
        }
        run.discards = discard_the_segments_reusable_as_a_whole(
            devices.as_mut_slice(),
            allocator,
            parameters,
            current.root(),
        );
        BackgroundCompactionCall::Ran {
            trigger,
            run: Box::new(run),
        }
    }

    /// 放锁的三段入口判过起跑、排空待发之后那一段（[`MountedSessionWithPendingChanges::take_the_input_of_a_background_compaction_batch`]）：
    /// 照至多一次发布推这一趟，推到要算一批就取这一批的输入、交回票（会话上记下有一批在锁外）；在锁里就推完了（删除续做走了一步、照判据停、让路、
    /// 整理停着时删了意图）就照 [`Self::after_the_background_run`] 收这一次调用。
    #[allow(
        clippy::too_many_arguments,
        reason = "参数各是一样东西：盘表、节奏、让路的下限、调用方、为什么起跑、推到哪、预算按哪一条算、这一趟前几次调用发的批数"
    )]
    pub(super) fn take_the_background_batch_that_starts<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        pacing: CompactionPacing,
        yield_floor: CompactionYieldFloor,
        caller: &dyn CallerOfTheBackgroundCompaction,
        trigger: BackgroundCompactionTrigger,
        reach: CompactionRunReach,
        budget: CompactionRunBudget,
        batches_published_earlier_in_the_trip: u64,
    ) -> BackgroundBatchTakenOrCallFinished {
        let compaction_publishes_before_this_call =
            self.compaction.publishes_since_the_last_user_change;
        let run = {
            let MountedSession {
                allocator,
                current,
                instance,
                shadow_ledger: _,
                parameters_and_device_table,
                compaction,
            } = &mut *self;
            let (parameters, background_compute) = parameters_and_device_table
                .parameters_on_disk_and_the_background_compute_of_this_mount_mut();
            match take_the_next_compaction_batch_or_finish_the_run(
                parameters,
                devices.as_mut_slice(),
                allocator,
                compaction,
                current,
                *instance,
                reach,
                budget,
                PacingOfTheRun::PacedByTheCaller {
                    write_bandwidth_limit: pacing.write_bandwidth_limit,
                    yield_floor,
                    caller,
                    span: SpanOfTheBackgroundCall::AtMostOnePublish,
                    batches_published_earlier_in_the_trip,
                },
                background_compute,
            ) {
                CompactionBatchTakenOrRunFinished::RunFinished(run) => *run,
                CompactionBatchTakenOrRunFinished::BatchTaken { run, input } => {
                    let rollback_gate_at_the_relock = rollback_gate_at_the_relock_of(&run);
                    return BackgroundBatchTakenOrCallFinished::BatchTaken(
                        BackgroundCompactionBatchTicket {
                            batch: compaction.hand_a_batch_out_of_the_lock(),
                            trip: TripOfTheBatchOutOfTheLock {
                                trigger,
                                reach,
                                budget,
                                batches_published_earlier_in_the_trip,
                                compaction_publishes_before_this_call,
                                write_bandwidth_limit: pacing.write_bandwidth_limit,
                                yield_floor,
                                rollback_gate_at_the_relock,
                            },
                            run,
                            input,
                        },
                    );
                }
            }
        };
        BackgroundBatchTakenOrCallFinished::CallFinished(self.after_the_background_run(
            devices,
            trigger,
            reach,
            budget,
            SpanOfTheBackgroundCall::AtMostOnePublish,
            batches_published_earlier_in_the_trip,
            compaction_publishes_before_this_call,
            run,
        ))
    }
}
