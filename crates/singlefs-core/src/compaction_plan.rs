//! 整理的选段与计划：三层里的后两层，都是纯函数——
//! 不读设备、不发写、不动分配器。`choose_region` 按记录给候选段排序、按扫描分类定资格（没扫过的先让调用方经
//! `crate::region_scan::scan_region` 扫），`plan_batch` 从扫描结果里取下一批要搬的（映射 key, 源落点）。**目的落点不在计划里**：由那一批发布时的分配器按 D3（空间分配） 已定项 8 现取
//!
//! 判据显式（`.claude/rules/fs-design.md` 五条硬要求第 1 条）：第一版只有 greedy 一种（[`RegionChoiceCriterion::Greedy`]），
//! 打分是 [`greedy_score`]；cost-benefit 与轮转留给 E10（老化：结构抗老化 vs 事后整理） 的臂。

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU64, NonZeroUsize};
use std::time::Duration;

use singlefs_format::CLUSTER_SEGMENT_SLOTS;
use singlefs_replay_prefix::compaction_plan_proofs::{
    choose_compaction_segment, plan_compaction_batch, CompactionKernelAllocationClass,
    CompactionKernelPlannedBatch, CompactionKernelRegionChoice, CompactionKernelScannedAllocation,
    CompactionKernelSegment, CompactionKernelSegmentScan,
};

use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use crate::admission::{BytesOnOneDevice, UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED};
use crate::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, UnitAreaStart};
use crate::background_compute::{BackgroundTaskAfterTheComputeError, ClusterSegmentEntryKey};
use crate::logged_operations::{CompactionIntent, CompactionRegion};
use crate::mounted_session::{BackgroundCompactionTrigger, CompactionRunReach};
use crate::region_scan::{AllocationInRegion, RegionScan, RegionScanRefusal, ScannedAllocation};
use crate::root_ring::RootRingSlotsPerRegion;

mod region_choice_and_batch_plan;
mod watermarks;

pub use region_choice_and_batch_plan::*;
pub use watermarks::*;

/// 一批至多搬几个单元从哪来。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionBatchSize {
    /// 这次挂载给的（挂载参数，默认 [`COMPACTION_BATCH_UNITS_BY_DEFAULT`]）。
    OfTheMount(CompactionBatchUnits),
    /// 只供测试。
    ForcedForTests(CompactionBatchUnits),
}

/// 整理什么时候起跑。封闭集合。产品路径照水位（[`CompactionRunForcing::FollowTheWatermarks`]）；另两种只供测试。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRunForcing {
    /// 定时兜底那一次与用户改动发成之后那一次按水位判起不起跑（D26（后台整理与放置回收） 已定项 13 第 5 条，[`CompactionWatermarks`]）；
    /// 后台入口连同不发布的停做的备忘、挂载那一份预算从根环读、破例与回退门都只在这一支判。显式入口
    /// （`mounted_session::MountedSession::run_compaction`）不看水位、不过回退门，照常跑。
    FollowTheWatermarks,
    /// 只供测试：起跑判据不看水位、恒为真——定时兜底查到的是「该跑」，用户改动发成之后记下的也是「该跑了」，由后台入口
    /// （`mounted_session::MountedSession::run_compaction_in_the_background`）去跑。备忘、挂载那一份预算从根环读、破例与回退门这一支都不判：
    /// 用它的用例钉的是一趟怎么推，发布数由用例自己摆。
    ForcedToStartForTests,
    /// 只供测试：两个入口都不起跑，停机原因 [`CompactionRunStop::ForcedToStopForTests`]。
    ForcedToStopForTests,
}

/// 签发之后 R 排不排他（D26（后台整理与放置回收） 已定项 1 第 3 条：`R` 在意图期内不可作分配目的地）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRegionExclusion {
    ExcludedWhileTheIntentIsLive,
    /// 只供测试：活意图的 R 照样当分配目的地。
    SwitchedOffForTests,
}

/// 搬迁的目的落点按哪条政策取。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationDestinationPolicy {
    /// 判决 G2 的 D-a：码 1 走用户数据落点函数（D3（空间分配） 已定项 8 第 1 条），节点与打包容器走开放段 bump（已定项 10 ⑤），都不落在活 R 里。
    UserDataPlacementForDataAndOpenSegmentForTheRest,
    /// 只供测试：每个单元都取单元区里起点最小、整跨度没挡的落点（提交内生块回落那一个函数），仍不落在活 R 里。
    LowestUnblockedSlotForEveryUnitForTests,
}

/// 签发之后强制一次「释放前核出对不上」。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForcedReleaseChecksumMismatchAfterIssue {
    NotForced,
    /// 只供测试：签发之后第一批的第一个源落点当作读盘核两份都核出对不上（每块盘一份都隔离，D19（块指针的结构与宽度预算） 已定项 5），
    /// 整个挂载只强制这一次。
    OnceOnTheFirstSourceOfTheFirstBatchForTests,
}

/// 合取 7 `全空聚簇段数 ≥ N₀` 的 `N₀`（D3（空间分配） 已定项 7 登记表第 7 行）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyClusterSegmentsFloor {
    /// 第一版的值 [`EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION`]（算式等计数模型验过再定，E10（老化：结构抗老化 vs 事后整理））。
    OfTheFirstVersion,
    /// 只供测试。
    ForcedForTests(u64),
}

/// D3（空间分配） 已定项 7 登记表第 7 行：「`N₀` 第一版取 1」。
pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;

impl EmptyClusterSegmentsFloor {
    /// 这一档的 `N₀`。
    #[must_use]
    pub const fn segments(self) -> u64 {
        match self {
            EmptyClusterSegmentsFloor::OfTheFirstVersion => {
                EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION
            }
            EmptyClusterSegmentsFloor::ForcedForTests(segments) => segments,
        }
    }
}

/// 一批搬迁的源落点什么时候释放。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationPublishOrder {
    /// 判决 G3：源落点的释放与映射条目换位置在同一次发布里。
    SourceReleasedInThePublishThatMovesTheMapping,
    /// 只供测试（`.claude/kb/milestone/04-fourth-txn.md` 二·3 的必红世界「源落点在映射发布之前就被释放」）：这一批照样释放源落点、
    /// 写目的地与它的分配记录，映射条目与各处指针却留在源落点上。产品路径一处都不走它。
    SourceReleasedWhileTheMappingStillNamesItForTests,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：一次用户改动发成之后，会话为读到只坏一份的单元另发的那一批搬迁
/// （`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`），在发之前当作被空间准入拒。产品路径恒 [`Self::NotForced`]。
/// 被拒推抬 F 再试那一支在小盘之外的池上造不出「改动放得下、紧接着那一批放不下」，靠它进；
/// 开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationBatchAfterTheChangeRefusedForSpace {
    NotForced,
    /// 接下来这几次发这一批都当作被拒，用完之后照常发。
    ForcedForTheNextAttemptsForTests {
        attempts: NonZeroU64,
    },
    /// 每一次发这一批都当作被拒。
    ForcedOnEveryAttemptForTests,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话另发的那一批搬迁（同 [`RelocationBatchAfterTheChangeRefusedForSpace`]）在发之前
/// 当作被拒、而不是空间不够（计划对不上那一版那一种：`transaction::RelocationBatchRefused::PlannedUnitNotInTheVersion`，点名计划里头一把 key）。
/// 产品路径恒 [`Self::NotForced`]。被拒不是空间不够那一支（不推抬 F、不再试，`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::RefusedNotForSpace`）
/// 在产品路径上要计划在算出来与发之间失效、或发布路径报空间之外的错才进得去，靠它进；开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。
/// 被空间拒的那个开关同时开着时先照那一个拒（它在前面判）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationBatchAfterTheChangeRefusedNotForSpace {
    NotForced,
    /// 每一次发这一批都当作被拒（不是空间不够：不再试，一批只发这一次）。
    ForcedOnEveryAttemptForTests,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话为另发的那一批搬迁被空间拒而推抬 F 时，那一串抬 F 当作自己报了错
/// （`mount::MountError::InstanceTableUnreadable`：`mount::push_one_floor_raise_within_the_admission_budget` 在任何写之前交得出的那一种），一个写都不发。
/// 产品路径恒 [`Self::NotForced`]。抬 F 自己报错那一支（`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::FloorRaiseFailedWhilePushingForSpace`）
/// 在产品路径上要推抬 F 时读盘或落盘出错才进得去，靠它进；要先被空间拒才推抬 F，与 [`RelocationBatchAfterTheChangeRefusedForSpace`] 合着开。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaiseForTheRelocationBatchAfterTheChangeFails {
    NotForced,
    /// 每一次为这一批推抬 F 都当作抬 F 自己报了错。
    ForcedOnEveryRaiseForTests,
}

/// 整理只供测试的开关。
/// 产品路径是 [`CompactionSwitches::PRODUCT`]；只住内存，装在分配器上（`allocator::PoolAllocator::set_compaction_switches`），
/// 平常构造（`PoolAllocator::new`、挂载重建的分配器）一律是产品值；可写挂载续做意图要开关时由调用方交给挂载
/// （`mount::mount_writable_with_compaction_switches`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionSwitches {
    pub batch_size: CompactionBatchSize,
    /// 只供测试：签发时不按判据选、取这一段（资格那五条照判，不合格就停）。`None` 是产品路径。
    pub forced_region: Option<CompactionRegion>,
    pub run_forcing: CompactionRunForcing,
    pub region_exclusion: CompactionRegionExclusion,
    pub destination_policy: RelocationDestinationPolicy,
    pub release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue,
    pub empty_cluster_segments_floor: EmptyClusterSegmentsFloor,
    pub relocation_publish_order: RelocationPublishOrder,
}

impl CompactionSwitches {
    pub const PRODUCT: CompactionSwitches = CompactionSwitches {
        batch_size: CompactionBatchSize::OfTheMount(COMPACTION_BATCH_UNITS_BY_DEFAULT),
        forced_region: None,
        run_forcing: CompactionRunForcing::FollowTheWatermarks,
        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
        destination_policy:
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
        release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::OfTheFirstVersion,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    };
}

/// 一次整理运行停在哪。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionRunStop {
    /// 只供测试的开关叫停（[`CompactionRunForcing::ForcedToStopForTests`]）。
    ForcedToStopForTests,
    /// 一段都没资格（[`RegionChoice::NoEligibleSegment`]），或强制选的那一段不合格。
    NoEligibleSegment,
    /// 签发前判不过（D26（后台整理与放置回收） 已定项 1 第 2 条）：R 外非全空段的空洞少于 R 里要搬的单元数，不签。
    HolesOutsideTheRegionFewerThanItsUnitsToMove { holes: u64, units_to_move: u64 },
    /// 上一条意图完成了，全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假：不签下一条。
    NoYieldFromTheLastIntent {
        empty_cluster_segments_now: u64,
        empty_cluster_segments_at_issue: u64,
    },
    /// 这次运行的发布预算用完了。
    PublishBudgetSpent { budget: u64 },
    /// 一批被空间拒（准入或落点）：暂停——单独发一次只删意图的发布丢掉意图（R 排他跟着撤），不推抬 F；下一次整理从头签发、重扫。
    /// 只删意图的那一次归删除类，不判式子、落点照判（`transaction::judge_the_publish_before_any_read_or_write`）。
    PausedShortOfSpace,
    /// 签发那次发布被空间拒（准入或落点）：意图没签成、盘上没有它，R 排他跟着撤；不推抬 F。
    IssueRefusedShortOfSpace,
    /// 可写挂载续做（只续做盘上那一条未完成的意图、不签新的，D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」）：
    /// 那一条删了（达成或丢掉），或本来就没有活意图。
    NoLiveIntentLeftToContinue,
    /// R 里一个能搬的都没有、却还有未释放的记录（[`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]）：意图连同 R 排他丢掉。
    NothingMovableButTheRegionIsNotEmpty,
    /// 用户改动之后起跑的那一趟只跑至多一批，跑完了。
    OneBatchAfterTheUserChangeDone,
    /// 后台入口：前台有 fsync 在等，不发下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）。
    /// 全空段低于底额、前台写不进时这一趟的第一次发布不为前台 fsync 让路（前台已写不进，让路换不来什么）。
    YieldedToAForegroundFsync,
    /// 后台入口：调用方在停这条驱动后台入口的线程，不发下一批，低于底额的破例也不发（线程很快 join 得上）。
    YieldedToTheCallerStoppingItsThread,
    /// 后台入口：前台有 fsync 之外的入口在等会话锁（收下改动、单发改动、读、切换后台计算），不发下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）。
    /// 全空段低于底额、前台写不进时这一趟的第一次发布不让；连着让路到了下限（[`CompactionYieldFloor`]）的那一批也不让。
    YieldedToAForegroundEntryWaitingForTheSessionLock,
    /// 放锁的三段入口回锁时（D26 已定项 13 第 5 条）：有活意图那一支比票里那一版与现行那一版起点在 R 里的未释放记录，变了的单元从计划里剔掉之后一个不剩，
    /// 这一批不发；这一趟没做完，下一次调用接着推。`units_pruned` 是剔掉的单元数。
    BatchPrunedToNothingAtTheRelock { units_pruned: u64 },
    /// 放锁的三段入口回锁时：没有活意图那一支选中的段在现行那一版上没了资格（这次挂载开过它、它里面有钉住或隔离的槽、被扫描拒过、
    /// 里面没有未释放的记录了），这一次不签；这一趟没做完，下一批重选。
    ChosenSegmentNoLongerEligibleAtTheRelock,
    /// 放锁的三段入口回锁时：冻结着一次发布（写盘报错、还没重发；取输入之前就冻着的、锁外那一段期间冻结的都算），这一批丢掉、一个写都不发，
    /// 不记成整理失败；这一趟没做完，
    /// 重发成了之后下一次调用接着推。
    BatchDroppedWhileAPublishIsFrozenAtTheRelock,
    /// 显式入口：这个会话有一批在锁外（放锁的三段入口取了输入、还没回锁），这一趟一个写都不发，在取 GPU 实现那把锁与换设置之前。
    NotRunWhileABatchOfThisSessionIsOutOfTheLock,
    /// 后台入口：这一个写带宽窗口里整理写的字节已到写带宽上限，不发下一批（D26 已定项 13 第 6 条，[`CompactionWriteBandwidthLimit`]）。
    WriteBandwidthLimitReached {
        bytes_written_in_the_window: u64,
        limit_in_bytes_per_second: u64,
    },
    /// 这一批算着出了错（D26 已定项 13 第 4 条）：这一批放弃、一个单元都没搬；现行那一版里有活意图就单发一次只删意图的发布丢掉它
    /// （R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），没有就一个写都没发。整理这个后台任务怎样了见 `task_after_the_error`：
    /// 一批里同一个条目键出现两次（`background_compute::BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch`）只作废这一批、任务不停
    /// （[`BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned`]）；别的错停下，或当场解除停下。下一次整理从头签发。
    /// 放锁的三段入口回锁时那一次删意图被回退门压住的，意图留着、会话记下它（[`CompactionOfThisMount::has_an_intent_of_an_abandoned_batch_still_to_drop`]）；之后取下一批的输入之前
    /// 单发的那一次只删意图的发布发成，那一趟同样停在这一条，`task_after_the_error` 是放弃那一批时的。
    BatchAbandonedAfterABackgroundComputeError {
        task_after_the_error: BackgroundTaskAfterTheComputeError,
    },
    /// 整理这个后台任务先前因一批出错停着（D26 已定项 13 第 4 条；挂载那一刻或两批之间实现答交给它的那一组换上了而一张卡都用不上、答开不成而当场停着的同算），
    /// 而这一刻池子每块盘的已分配占它的容量都不超过补跑阈值：
    /// 停在这一刻：现行那一版里有活意图就单发一次只删意图的发布丢掉它（R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），之后一个写都不发；
    /// 没有活意图就从这一刻起一个写都不发（删除续做也不走）。显式入口每一趟开头判这一条；后台入口判出停着而有活意图时起跑一趟只为删它
    /// （`mounted_session::BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`），没有活意图就不起跑。挂着切换之后实现答那一组一张卡都用不上或开不成、
    /// 手里没有活意图时，两个入口都在任何写之前停在这一条。一趟之中在 CPU 上补跑过几批之后、或一趟之中两批之间实现答出那一组的结局才判出的，
    /// 前面那几批照常发过。等占用超过阈值（下一趟照常补跑、从头签发），或一次通过校验的切换请求解除停下；开卡期间出错停着的，实现答那一组换上了也解除。
    BackgroundTaskStoppedAfterAnEarlierComputeError,
    /// GPU 路径上 R 里除了这一轮两份核判得不一样的记录，再没有判定了可搬的（D26 已定项 13 第 8 条「这一批不回收、不搬，留到下一轮重算」）：
    /// 这一批不发，意图照留（不删也不丢），下一批重扫照判。
    OnlyRecordsUncertainThisRoundLeftToMove { records_uncertain_this_round: u64 },
    /// GPU 路径上选中要签的那一段里有这一轮两份核判得不一样的记录（D26 已定项 13 第 8 条）：R 里要搬几个数不准，签发前那一判判不了，
    /// 这一批不签，下一批重扫照判。
    IssueHeldBackByRecordsUncertainThisRound { records_uncertain_this_round: u64 },
    /// 后台入口：这次整理发布要写的根槽与它之后余量（[`root_ring_slots_left_by_a_compaction_publish`]）那几槽里的根一走，根环里不同用户状态就少于 4 个
    /// （D16（发布语义） 已定项 1 的回退候选集；D26 已定项 13 第 5 条的加门）：不发，一个写都没发。全空段低于底额、前台写不进时每趟第一次发布破例不判。
    CompactionPublishWouldLeaveFewerThanFourRollbackStates {
        states_now: u64,
        states_after_the_publish_and_the_margin: u64,
    },
    /// 后台入口：回退门读根环或树表没读通（读不清有几个状态就不发）：一个写都没发，下一次叫起重判。
    RollbackStatesOfTheRingUnreadable,
    /// 后台入口放锁的那一种（`mounted_session::MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`）：
    /// 这一趟这一次调用发成了一次，调用方放锁之后接着调。不是停机，这一趟记在会话上没完。
    OnePublishOfTheTripDone,
    /// 后台入口因删除队列不空起跑的那一趟（D26（后台整理与放置回收） 已定项 13）：队列走空了，这一趟只走队列、一次整理发布都不发。整理下一次调用照常判起跑。
    DeletionQueueWalkedWithoutACompactionPublish,
}

impl CompactionRunStop {
    /// 这一种停在现行那一版不变、开关不变时重判还是同一个结论，又不发布：记进备忘，定时兜底下一次不重判
    /// （[`CompactionOfThisMount::note_the_stop_of_a_run`]）。选段、签发前判、无产出与回退门读的（分配记录、映射、快照钉住的槽、根环）都随发布变，
    /// 内存里的隔离集与不合格段表也只在发布里变。GPU 两份核不一致、被节奏或空间截住、出错的那几种下一次可能翻面，不进备忘。
    #[must_use]
    pub const fn publishes_nothing_until_the_next_publish(self) -> bool {
        match self {
            CompactionRunStop::NoEligibleSegment
            | CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. }
            | CompactionRunStop::NoYieldFromTheLastIntent { .. }
            | CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
                ..
            } => true,
            CompactionRunStop::ForcedToStopForTests
            | CompactionRunStop::PublishBudgetSpent { .. }
            | CompactionRunStop::PausedShortOfSpace
            | CompactionRunStop::IssueRefusedShortOfSpace
            | CompactionRunStop::NoLiveIntentLeftToContinue
            | CompactionRunStop::NothingMovableButTheRegionIsNotEmpty
            | CompactionRunStop::OneBatchAfterTheUserChangeDone
            | CompactionRunStop::YieldedToAForegroundFsync
            | CompactionRunStop::YieldedToTheCallerStoppingItsThread
            | CompactionRunStop::YieldedToAForegroundEntryWaitingForTheSessionLock
            | CompactionRunStop::WriteBandwidthLimitReached { .. }
            | CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. }
            | CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            | CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. }
            | CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. }
            | CompactionRunStop::RollbackStatesOfTheRingUnreadable
            | CompactionRunStop::OnePublishOfTheTripDone
            | CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish
            | CompactionRunStop::BatchPrunedToNothingAtTheRelock { .. }
            | CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock
            | CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock
            | CompactionRunStop::NotRunWhileABatchOfThisSessionIsOutOfTheLock => false,
        }
    }

    /// 运行时报出停在哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            CompactionRunStop::ForcedToStopForTests => "compaction_stopped_by_the_test_only_switch",
            CompactionRunStop::NoEligibleSegment => "compaction_stopped_no_eligible_segment",
            CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. } => {
                "compaction_stopped_holes_outside_the_region_too_few"
            }
            CompactionRunStop::NoYieldFromTheLastIntent { .. } => {
                "compaction_stopped_no_yield_from_the_last_intent"
            }
            CompactionRunStop::PublishBudgetSpent { .. } => {
                "compaction_stopped_publish_budget_spent"
            }
            CompactionRunStop::PausedShortOfSpace => "compaction_paused_short_of_space",
            CompactionRunStop::IssueRefusedShortOfSpace => {
                "compaction_stopped_the_issue_refused_short_of_space"
            }
            CompactionRunStop::NoLiveIntentLeftToContinue => {
                "compaction_stopped_no_live_intent_left_to_continue"
            }
            CompactionRunStop::NothingMovableButTheRegionIsNotEmpty => {
                "compaction_stopped_nothing_movable_in_the_region"
            }
            CompactionRunStop::OneBatchAfterTheUserChangeDone => {
                "compaction_stopped_after_one_batch_following_the_user_change"
            }
            CompactionRunStop::YieldedToAForegroundFsync => {
                "compaction_yielded_to_a_foreground_fsync"
            }
            CompactionRunStop::YieldedToTheCallerStoppingItsThread => {
                "compaction_yielded_to_the_caller_stopping_its_thread"
            }
            CompactionRunStop::YieldedToAForegroundEntryWaitingForTheSessionLock => {
                "compaction_yielded_to_a_foreground_entry_waiting_for_the_session_lock"
            }
            CompactionRunStop::BatchPrunedToNothingAtTheRelock { .. } => {
                "compaction_batch_pruned_to_nothing_at_the_relock"
            }
            CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock => {
                "compaction_chosen_segment_no_longer_eligible_at_the_relock"
            }
            CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock => {
                "compaction_batch_dropped_while_a_publish_is_frozen_at_the_relock"
            }
            CompactionRunStop::NotRunWhileABatchOfThisSessionIsOutOfTheLock => {
                "compaction_not_run_while_a_batch_of_this_session_is_out_of_the_lock"
            }
            CompactionRunStop::WriteBandwidthLimitReached { .. } => {
                "compaction_paced_by_the_write_bandwidth_limit"
            }
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError { .. } => {
                "compaction_batch_abandoned_after_a_background_compute_error"
            }
            CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError => {
                "compaction_stopped_after_an_earlier_background_compute_error"
            }
            CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove { .. } => {
                "compaction_stopped_only_records_uncertain_on_the_gpu_left_to_move"
            }
            CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound { .. } => {
                "compaction_issue_held_back_by_records_uncertain_on_the_gpu"
            }
            CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates {
                ..
            } => "compaction_stopped_the_publish_would_leave_fewer_than_four_rollback_states",
            CompactionRunStop::RollbackStatesOfTheRingUnreadable => {
                "compaction_stopped_the_rollback_states_of_the_ring_are_unreadable"
            }
            CompactionRunStop::OnePublishOfTheTripDone => "compaction_one_publish_of_the_trip_done",
            CompactionRunStop::DeletionQueueWalkedWithoutACompactionPublish => {
                "compaction_deletion_queue_walked_without_a_compaction_publish"
            }
        }
    }
}

/// 一条活意图在这次挂载里的样子：意图本身与它的 R 排没排他（产品值排他；只供测试的开关 [`CompactionRegionExclusion::SwitchedOffForTests`] 不排）。
/// 整理停下（暂停、一批出错停下）删掉意图连同排他，盘上每一版不留「意图在、R 不排他」那一态；只删意图的那一次发布自己取落点时放开 R
/// （[`CompactionSwitchesAndTheExcludedRegion::open_the_region_to_the_publish_deleting_the_intent`]），那一次发布落盘的那一版里已没有意图。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveCompactionIntent {
    pub intent: CompactionIntent,
    pub is_the_region_excluded_from_placement: bool,
}

/// 整理的运行时观测（判决 G8「起跑次数、批数、停机原因、用掉的根槽数」；D26（后台整理与放置回收） 已定项 6 的运行时计数器那两样：
/// 一批整理的搬迁量；runs、全空聚簇段数、回落计数、不可回收字节由分配器与记账给，整理进度不存）。只住内存、只增，丢了不影响判定。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompactionObservations {
    pub runs_started: u64,
    pub intents_issued: u64,
    pub intents_completed: u64,
    pub intents_dropped: u64,
    pub batches_published: u64,
    /// 整理的发布用掉的根槽数：一次发布写一个根槽，所以就是整理发布（签发、搬一批、删意图）的次数。
    pub root_ring_slots_used: u64,
    pub units_relocated: u64,
    /// 最近一批搬了几个单元（D26 已定项 6「一批整理的搬迁量」）。
    pub units_relocated_by_the_last_batch: u64,
    pub last_stop: Option<CompactionRunStop>,
    /// 后台入口为前台在等的 fsync 让路的次数（D26 已定项 13 第 6 条）。
    pub yields_to_a_foreground_fsync: u64,
    /// 后台入口为前台在等会话锁的别的入口让路的次数。
    pub yields_to_a_foreground_entry_waiting_for_the_session_lock: u64,
    /// 放锁的三段入口回锁时从计划里剔掉的单元数（D26 已定项 13 第 5 条：记录变了的单元）。
    pub units_pruned_at_the_relock: u64,
    /// 放锁的三段入口回锁时选中的段没了资格、这一次没签的次数。
    pub issues_given_up_for_a_region_no_longer_eligible_at_the_relock: u64,
    /// 放锁的三段入口回锁时有冻结着的发布、丢掉的批数。
    pub batches_dropped_while_a_publish_is_frozen_at_the_relock: u64,
}

/// 分配器这一侧的整理状态：只供测试的开关与活意图的 R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条「`R` 在意图期内不可作分配目的地」：
/// 分配路径多一条合取，意图住 logged_ops 树、分配器读得到）。活意图的权威在现行那一版的 logged_ops 里（`transaction::TransactionOutput::logged_operations`）；
/// 这里记的是分配器要排除的那一份：会话签发那次发布之前装上（签发那次发布自己的新落点也不进 R）、删意图之后撤掉，可写挂载按挂载那一版的
/// logged_ops 装上（`mount`）。另带两样只有发布路径看得见的：签发之后强制一次「释放前核出对不上」那个开关用过没有，与这次挂载里发布路径
/// 隔离过、还没交给会话的槽（会话把它们并进常驻隔离集，[`CompactionOfThisMount::take_the_quarantines_noted_by_the_publishes`]）。
/// 随分配器只住内存，生命周期一次挂载。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactionSwitchesAndTheExcludedRegion {
    pub switches: CompactionSwitches,
    /// 三条水位线（D26（后台整理与放置回收） 已定项 1，挂载参数，不存盘）。
    pub watermarks: CompactionWatermarks,
    /// 只供测试：不看读数，判起跑判据时当这个量越了线（[`CompactionWatermarkCrossingForcing`]）。
    pub watermark_crossing_forcing: CompactionWatermarkCrossingForcing,
    live_intent: Option<LiveCompactionIntent>,
    has_the_forced_release_checksum_mismatch_been_used: bool,
    quarantines_noted_by_the_publishes: Vec<(SlotNumber, u64)>,
}

impl CompactionSwitchesAndTheExcludedRegion {
    /// 产品路径起步：开关是产品值，没有活意图。
    #[must_use]
    pub fn of_a_new_mount() -> Self {
        Self {
            switches: CompactionSwitches::PRODUCT,
            watermarks: CompactionWatermarks::BY_DEFAULT,
            watermark_crossing_forcing: CompactionWatermarkCrossingForcing::NotForced,
            live_intent: None,
            has_the_forced_release_checksum_mismatch_been_used: false,
            quarantines_noted_by_the_publishes: Vec::new(),
        }
    }

    #[must_use]
    pub fn live_intent(&self) -> Option<LiveCompactionIntent> {
        self.live_intent
    }

    /// 分配器这一侧的签发：记下活意图，R 从此排他（开关关掉 R 排他时不排）。盘上那一条由签发那次发布写进 logged_ops
    /// （`transaction::publish_compaction_step`）；挂载按 logged_ops 里那一条装上。全池活意图至多一条。
    ///
    /// # Panics
    /// 已经有一条活意图：调用方先判过（会话按现行那一版的 logged_ops 判，挂载在新建的分配器上装）。
    pub fn issue(&mut self, intent: CompactionIntent) {
        assert!(
            self.live_intent.is_none(),
            "全池活整理意图至多一条（判决 G1 e1）：签发之前调用方判过没有活意图"
        );
        self.live_intent = Some(LiveCompactionIntent {
            intent,
            is_the_region_excluded_from_placement: match self.switches.region_exclusion {
                CompactionRegionExclusion::ExcludedWhileTheIntentIsLive => true,
                CompactionRegionExclusion::SwitchedOffForTests => false,
            },
        });
    }

    /// 意图删了（达成或丢掉，删它的那次发布已落盘），或签发那次发布没发成：撤掉活意图连同 R 排他，交回撤掉的那一条。
    pub fn withdraw_the_live_intent(&mut self) -> Option<CompactionIntent> {
        self.live_intent.take().map(|live| live.intent)
    }

    /// 只删意图、不搬单元的那一次发布（暂停、出错停下、停着时的删意图，R 已按记录全空的删意图；`transaction::publish_compaction_step_in_the_session`）
    /// 取落点之前放开 R：那一次发布落盘之后盘上就没有这条意图，它自己的新落点落进 R 不违 I-5.6（「活意图的 R 内没有分配代 > `c0` 的记录」只管意图活着的那几版）。
    /// 活意图本身不撤（撤的时机照旧在删意图那一版落盘之后，会话撤）；交回放开之前排不排他，没有活意图交回 `None`。发布返回之后（发成或被拒）
    /// 照交回的那一个经 [`Self::put_back_the_region_exclusion`] 放回，别的发布照旧排除 R。
    pub(crate) fn open_the_region_to_the_publish_deleting_the_intent(&mut self) -> Option<bool> {
        self.live_intent
            .as_mut()
            .map(|live| std::mem::replace(&mut live.is_the_region_excluded_from_placement, false))
    }

    /// 放回 [`Self::open_the_region_to_the_publish_deleting_the_intent`] 放开之前的排他。活意图已撤（或从来没有）就什么都不做。
    pub(crate) fn put_back_the_region_exclusion(
        &mut self,
        was_excluded_before_the_publish: Option<bool>,
    ) {
        if let (Some(live), Some(was_excluded)) =
            (self.live_intent.as_mut(), was_excluded_before_the_publish)
        {
            live.is_the_region_excluded_from_placement = was_excluded;
        }
    }

    /// 分配器此刻不许当目的地的 R（活意图且排他时）。
    #[must_use]
    pub fn region_excluded_from_placement(&self) -> Option<CompactionRegion> {
        self.live_intent
            .filter(|live| live.is_the_region_excluded_from_placement)
            .map(|live| live.intent.region)
    }

    /// 发布路径记下一次释放前核出对不上而隔离的落点（跨度里的每一个槽，两盘同槽只记槽号）：等会话取走、并进常驻隔离集。
    /// 发布没成立时分配器换回进来时那一份，这一笔跟着没了。
    pub fn note_quarantined_placement(&mut self, slot: SlotNumber, span: u64) {
        self.quarantines_noted_by_the_publishes.push((slot, span));
    }

    /// 交出发布路径记下、会话还没取走的隔离落点（取走之后清空）。
    pub fn take_the_quarantines_noted_by_the_publishes(&mut self) -> Vec<(SlotNumber, u64)> {
        std::mem::take(&mut self.quarantines_noted_by_the_publishes)
    }

    /// 签发之后强制一次「释放前核出对不上」这一回该不该用上（用上就记下，整个挂载只一次）。
    pub fn take_the_forced_release_checksum_mismatch(&mut self) -> bool {
        match self.switches.release_checksum_mismatch_after_issue {
            ForcedReleaseChecksumMismatchAfterIssue::NotForced => false,
            ForcedReleaseChecksumMismatchAfterIssue::OnceOnTheFirstSourceOfTheFirstBatchForTests => {
                if self.has_the_forced_release_checksum_mismatch_been_used
                    || self.live_intent.is_none()
                {
                    return false;
                }
                self.has_the_forced_release_checksum_mismatch_been_used = true;
                true
            }
        }
    }
}

/// 会话这一侧的整理状态（丁 1 交判第 2 点：不合格段表挂会话；生命周期一次挂载）：这次挂载里被区域扫描拒过的段、这次挂载里释放前核出对不上
/// 而隔离的槽（常驻隔离集）、上一次用户改动之后整理发了几次布（`B_gc`）、观测。住在挂着的会话上
/// （`mounted_session::MountedSession::compaction`），可写挂载续做意图时先建一份、交给会话接着用（`mount::MountOutput::compaction_continuation`）。
/// 整理挪出用户改动的调用线程之后另住四样：起跑判据记下的「该跑了」、上一条完成的意图
/// （签下一条之前那一判跨趟读它）、定时兜底上一次查的钟读数、写带宽这一个窗口里整理写了多少。另记这次挂载里读—改—写读回旧数据单元时只坏一份的次数
/// （会话按挂载累计，`mounted_session::MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`）、可写挂载读盘建上一版时只坏一份的次数
/// （`mounted_session::MountedSession::unit_reads_with_one_copy_failing_while_mounting`），与会话另发那一批搬迁的三个只供测试的开关（被空间拒、
/// 被拒不是空间不够、为它推的抬 F 自己报错）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactionOfThisMount {
    pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
    quarantined_slots: BTreeSet<SlotNumber>,
    /// 上一次用户改动之后整理发了几次布：会话发成一次用户改动就归零。
    pub publishes_since_the_last_user_change: u64,
    pub observations: CompactionObservations,
    /// 最近一次判起跑判据的结论：哪块盘哪个量越了线、读数与线各是多少（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。还没判过是 `None`。
    /// 判了没起跑（[`WatermarkVerdict::NoLineCrossed`]）也记：它答的就是这一次为什么没起跑，所以不进只记起跑之后那几样的 [`CompactionObservations`]。
    pub last_watermark_verdict: Option<WatermarkVerdict>,
    /// 用户改动发成之后起跑判据查出来的结果，后台入口取走（[`CompactionStartJudgement`]）。
    pub start_judgement: CompactionStartJudgement,
    /// 这次挂载里上一条完成的意图（「上一条无产出就不签」读它的 `segs0`）：跨趟留着，不随一趟结束丢掉，
    /// 可写挂载续做做完的那一条也记在这里、交给会话。
    pub last_completed_intent: Option<CompactionIntent>,
    /// 定时兜底上一次查起跑判据时调用方的钟读数；还没查过是 `None`。
    timer_fallback_last_check: Option<CallerMonotonicClockReading>,
    /// 写带宽上限的这一个窗口（[`COMPACTION_WRITE_BANDWIDTH_WINDOW`]）：开了上限的后台入口推第一批时开、过了一个窗口长就换新的。
    write_bandwidth_window: Option<CompactionWriteBandwidthWindow>,
    /// 这次挂载里读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：会话把每次用户改动里各次发布的写入口
    /// 上的数加进来（`transaction::PoolWriter::old_data_unit_reads_back_with_one_copy_failing`）。与常驻隔离集同是这次挂载里见到的坏副本那一类观测，
    /// 同住这一份挂载期状态；只在内存里，新挂载从 0 起（[`Self::of_a_new_mount`]）。
    old_data_unit_reads_back_with_one_copy_failing: u64,
    /// 可写挂载读盘建上一版时按位置条目读一个单元只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：挂载交回之前记一次
    /// （`mount::mount_writable` 那一串）。与上一样分开记：上一样是读—改—写读回旧数据单元，这一样是挂载读的各类单元。只在内存里，新挂载从 0 起。
    unit_reads_with_one_copy_failing_while_mounting: u64,
    /// 可写挂载读盘建上一版时读到只坏一份、到挂载交回那一版仍在中央映射里（或留存条目里）的那几个单元的映射 key（一个单元一把，按读到的先后）：
    /// 会话下一次用户改动发成之后照快照下另发那批同一条路径把它们搬到新槽、两份新写（
    /// `mounted_session::MountedSession::publish_user_change`），交给那一批就清空。只在内存里，新挂载是空的。
    mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec<Vec<u8>>,
    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationBatchAfterTheChangeRefusedForSpace`]）。新挂载是产品值。
    relocation_batch_after_the_change_refused_for_space:
        RelocationBatchAfterTheChangeRefusedForSpace,
    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationBatchAfterTheChangeRefusedNotForSpace`]）。新挂载是产品值。
    relocation_batch_after_the_change_refused_not_for_space:
        RelocationBatchAfterTheChangeRefusedNotForSpace,
    /// 只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationBatchAfterTheChangeFails`]）。新挂载是产品值。
    floor_raise_for_the_relocation_batch_after_the_change_fails:
        FloorRaiseForTheRelocationBatchAfterTheChangeFails,
    /// 上一次不发布的停（[`CompactionRunStop::publishes_nothing_until_the_next_publish`]）记在哪一版、哪一组开关上：这两样都没变，定时兜底不重判。
    stop_that_publishes_nothing_until_the_next_publish:
        Option<StopThatPublishesNothingUntilTheNextPublish>,
    /// 这次挂载那一份预算还没从根环读：挂载不给新预算（D26 已定项 13 第 5 条的加门），头一次判后台起跑时
    /// 把已经过去的发布数取成根环里最新那个用户状态之后的根数；这之前发成用户改动就不用读了。
    publishes_of_the_mount_are_still_to_be_read_from_the_ring: bool,
    /// 放锁的后台入口上一次调用之后还没完的那一趟（[`BackgroundTripUnderWay`]）。
    background_trip_under_way: Option<BackgroundTripUnderWay>,
    /// 后台入口连着为前台让路了几次（一批发成就清零）：让路的下限照它判（[`CompactionYieldFloor`]）。
    yields_in_a_row: u64,
    /// 这一串连着的让路头一次时调用方的钟读数（一批发成就清掉）：让路累计多久照它判。
    reading_when_the_yields_in_a_row_began: Option<CallerMonotonicClockReading>,
    /// 放锁的三段入口交到锁外、还没回锁的那一批（D26 已定项 13 第 5 条「会话上记有一批在锁外」）；没有是 `None`。
    batch_out_of_the_lock: Option<BatchOutOfTheLock>,
    /// 这次挂载里交到锁外的批数（下一批的号）。
    batches_handed_out_of_the_lock: u64,
    /// 回锁里出错那一支删意图被回退门压住的那一条活意图（[`IntentOfAnAbandonedBatchStillToDrop`]）；没有是 `None`。
    intent_of_an_abandoned_batch_still_to_drop: Option<IntentOfAnAbandonedBatchStillToDrop>,
}

/// 一批算着出了错、回锁那一刻单发的删意图被回退门压住的那一条活意图（D26（后台整理与放置回收） 已定项 13 第 4 条「正在算的那一批放弃，意图也删掉」、
/// 第 5 条「门关就不发，意图与 `R` 排他留到门开」）：意图本身，与放弃那一批时这一趟停在的那一条。会话记着它、它仍是现行那一版里的活意图时，
/// 之后任一入口取下一批的输入之前先单发一次只删意图的发布（回退门与破例照停着时那一次只删意图的发布判），不在这条意图下接着搬。只住内存，新挂载没有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IntentOfAnAbandonedBatchStillToDrop {
    pub(crate) intent: CompactionIntent,
    pub(crate) stop_of_the_abandoned_batch: CompactionRunStop,
}

/// 交到锁外的一批的号（这次挂载里第几次交出去）：回锁时认票是不是这个会话此刻在锁外的那一批。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatchOutOfTheLock(u64);

/// 前台在等会话锁的是哪一种入口（D26（后台整理与放置回收） 已定项 13 第 6 条：fsync，或收下改动、单发改动、读、切换后台计算）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForegroundEntryWaitingForTheSessionLock {
    Fsync,
    EntryOtherThanAnFsync,
}

impl ForegroundEntryWaitingForTheSessionLock {
    /// 为它让路时这一趟停在哪一条。
    #[must_use]
    pub const fn stop_of_the_yield(self) -> CompactionRunStop {
        match self {
            ForegroundEntryWaitingForTheSessionLock::Fsync => {
                CompactionRunStop::YieldedToAForegroundFsync
            }
            ForegroundEntryWaitingForTheSessionLock::EntryOtherThanAnFsync => {
                CompactionRunStop::YieldedToAForegroundEntryWaitingForTheSessionLock
            }
        }
    }
}

/// 前台在等时这一批让不让路（[`CompactionOfThisMount::judge_a_yield_under_the_floor`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForegroundYieldUnderTheFloor {
    Yields,
    /// 连着让路到了下限：这一批不让、照发。
    DoesNotYieldForTheFloorIsReached,
}

/// 后台入口为前台让路的下限（D26（后台整理与放置回收） 已定项 13 第 6 条「前台入口不断时有下限」）：连着让路 `yields_in_a_row_at_most` 次，或从这一串让路的头一次起
/// 累计到 `yielding_time_at_most`（读调用方的钟），哪一个先到下一批就不让；一批发成、或取输入问过调用方而没让路，两个都清零。`None` 是那一样不设。
/// 与 [`CompactionPacing`] 并列，
/// 由调用方每次调用交进来；放锁的三段入口收它，锁里推完一趟的入口照 [`Self::NO_FLOOR`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionYieldFloor {
    pub yields_in_a_row_at_most: Option<NonZeroU64>,
    pub yielding_time_at_most: Option<Duration>,
}

impl CompactionYieldFloor {
    /// 不设下限：前台一直在等就一直让。
    pub const NO_FLOOR: CompactionYieldFloor = CompactionYieldFloor {
        yields_in_a_row_at_most: None,
        yielding_time_at_most: None,
    };
}

/// 上一次不发布的停记下的那几样（[`CompactionOfThisMount::note_the_stop_of_a_run`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StopThatPublishesNothingUntilTheNextPublish {
    checkpoint_txg: CheckpointTxg,
    switches: CompactionSwitches,
    stop: CompactionRunStop,
}

/// 放锁的后台入口跨几次调用推的那一趟：一趟只判一次起跑，每次调用至多一次发布。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BackgroundTripUnderWay {
    pub(crate) trigger: BackgroundCompactionTrigger,
    pub(crate) reach: CompactionRunReach,
    /// 这一趟前几次调用已发的批数（用户改动之后那一趟至多一批，跨调用数）。
    pub(crate) batches_published: u64,
    /// 这一趟是全空段低于底额、前台写不进时起跑的，前几次调用还没发过一次整理发布（只走了删除续做、被写带宽截住或排空被拒）：
    /// 下一次调用接着推时，整理的第一次发布照样破例。
    pub(crate) is_the_exception_of_the_trip_still_open: bool,
}

impl CompactionOfThisMount {
    /// 一次挂载起步：各样都空，没有「该跑了」。
    #[must_use]
    pub fn of_a_new_mount() -> Self {
        Self {
            segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan::default(),
            quarantined_slots: BTreeSet::new(),
            publishes_since_the_last_user_change: 0,
            observations: CompactionObservations::default(),
            last_watermark_verdict: None,
            start_judgement: CompactionStartJudgement::NotDue,
            last_completed_intent: None,
            timer_fallback_last_check: None,
            write_bandwidth_window: None,
            old_data_unit_reads_back_with_one_copy_failing: 0,
            unit_reads_with_one_copy_failing_while_mounting: 0,
            mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting: Vec::new(),
            relocation_batch_after_the_change_refused_for_space:
                RelocationBatchAfterTheChangeRefusedForSpace::NotForced,
            relocation_batch_after_the_change_refused_not_for_space:
                RelocationBatchAfterTheChangeRefusedNotForSpace::NotForced,
            floor_raise_for_the_relocation_batch_after_the_change_fails:
                FloorRaiseForTheRelocationBatchAfterTheChangeFails::NotForced,
            stop_that_publishes_nothing_until_the_next_publish: None,
            publishes_of_the_mount_are_still_to_be_read_from_the_ring: true,
            background_trip_under_way: None,
            yields_in_a_row: 0,
            reading_when_the_yields_in_a_row_began: None,
            batch_out_of_the_lock: None,
            batches_handed_out_of_the_lock: 0,
            intent_of_an_abandoned_batch_still_to_drop: None,
        }
    }

    /// 前台有入口在等会话锁时这一批让不让路（D26（后台整理与放置回收） 已定项 13 第 6 条的下限）：连着让路的次数到了 `floor` 的次数、
    /// 或从这一串让路的头一次起到 `now` 到了 `floor` 的时长（调用方的钟往回走时按没过算），这一批就不让；让就记一次（次数加一、头一次记下 `now`、
    /// 照 `waiting` 记进观测）。不让时这里不改计数：调用方照这一次取输入没让路清零（[`Self::break_the_yields_in_a_row_by_a_batch_not_yielding`]）。
    pub(crate) fn judge_a_yield_under_the_floor(
        &mut self,
        floor: CompactionYieldFloor,
        waiting: ForegroundEntryWaitingForTheSessionLock,
        now: CallerMonotonicClockReading,
    ) -> ForegroundYieldUnderTheFloor {
        let is_the_count_reached = floor
            .yields_in_a_row_at_most
            .is_some_and(|at_most| self.yields_in_a_row >= at_most.get());
        let is_the_time_reached = match (
            floor.yielding_time_at_most,
            self.reading_when_the_yields_in_a_row_began,
        ) {
            (Some(at_most), Some(yields_began_at)) => {
                now.0.saturating_sub(yields_began_at.0) >= at_most
            }
            (Some(_), None) | (None, Some(_)) | (None, None) => false,
        };
        if is_the_count_reached || is_the_time_reached {
            return ForegroundYieldUnderTheFloor::DoesNotYieldForTheFloorIsReached;
        }
        self.yields_in_a_row += 1;
        self.reading_when_the_yields_in_a_row_began
            .get_or_insert(now);
        match waiting {
            ForegroundEntryWaitingForTheSessionLock::Fsync => {
                self.observations.yields_to_a_foreground_fsync += 1;
            }
            ForegroundEntryWaitingForTheSessionLock::EntryOtherThanAnFsync => {
                self.observations
                    .yields_to_a_foreground_entry_waiting_for_the_session_lock += 1;
            }
        }
        ForegroundYieldUnderTheFloor::Yields
    }

    /// 一批发成（一次整理发布：签发、搬一批、删意图都算）：连着让路的次数与这一串让路头一次的读数都清掉。
    pub(crate) fn clear_the_yields_in_a_row(&mut self) {
        self.yields_in_a_row = 0;
        self.reading_when_the_yields_in_a_row_began = None;
    }

    /// 后台入口这一次取输入问过调用方而没让路（前台没有入口在等、这一批是破例、或下限到了）：「连着」断了
    /// （D26（后台整理与放置回收） 已定项 13 第 6 条），两个计数照一批发成那样清掉（[`Self::clear_the_yields_in_a_row`]），
    /// 这一批之后不论发没发成，前台再在等时又从头连着数。
    pub(crate) fn break_the_yields_in_a_row_by_a_batch_not_yielding(&mut self) {
        self.clear_the_yields_in_a_row();
    }

    /// 后台入口此刻连着让路了几次。
    #[must_use]
    pub fn yields_in_a_row(&self) -> u64 {
        self.yields_in_a_row
    }

    /// 这个会话此刻有一批在锁外（放锁的三段入口取了输入、还没回锁）：后台入口、取输入、显式入口都在取 GPU 实现那把锁与换设置之前拒。
    #[must_use]
    pub fn has_a_batch_out_of_the_lock(&self) -> bool {
        self.batch_out_of_the_lock.is_some()
    }

    /// 记下交一批到锁外，交回它的号。
    ///
    /// # Panics
    /// 已有一批在锁外：取输入第一步判过（[`Self::has_a_batch_out_of_the_lock`]），走到这里之前没有别的出口交出过一批。
    pub(crate) fn hand_a_batch_out_of_the_lock(&mut self) -> BatchOutOfTheLock {
        assert!(
            self.batch_out_of_the_lock.is_none(),
            "一个会话至多一批在锁外：取输入第一步判过有没有，判过之后到交出这一批之间没有别的出口交出过一批"
        );
        self.batches_handed_out_of_the_lock += 1;
        let batch = BatchOutOfTheLock(self.batches_handed_out_of_the_lock);
        self.batch_out_of_the_lock = Some(batch);
        batch
    }

    /// `batch` 是不是这个会话此刻在锁外的那一批。
    #[must_use]
    pub(crate) fn is_the_batch_out_of_the_lock(&self, batch: BatchOutOfTheLock) -> bool {
        self.batch_out_of_the_lock == Some(batch)
    }

    /// 那一批回锁了：会话上不再记有一批在锁外。不是此刻在锁外的那一批就什么都不做。
    pub(crate) fn take_back_the_batch_out_of_the_lock(&mut self, batch: BatchOutOfTheLock) {
        if self.batch_out_of_the_lock == Some(batch) {
            self.batch_out_of_the_lock = None;
        }
    }

    /// 回锁里出错那一支单发的删意图被回退门压住或没发成：记下这条活意图与放弃那一批时这一趟停在的那一条（[`IntentOfAnAbandonedBatchStillToDrop`]），
    /// 之后取下一批的输入之前再删。已记着一条时换成这一条（现行那一版里至多一条活意图）。
    pub(crate) fn hold_back_the_drop_of_the_intent_of_an_abandoned_batch(
        &mut self,
        intent: CompactionIntent,
        stop_of_the_abandoned_batch: CompactionRunStop,
    ) {
        self.intent_of_an_abandoned_batch_still_to_drop =
            Some(IntentOfAnAbandonedBatchStillToDrop {
                intent,
                stop_of_the_abandoned_batch,
            });
    }

    /// 记着的那一条被回退门压住的删意图；没有是 `None`。
    #[must_use]
    pub(crate) fn intent_of_an_abandoned_batch_still_to_drop(
        &self,
    ) -> Option<IntentOfAnAbandonedBatchStillToDrop> {
        self.intent_of_an_abandoned_batch_still_to_drop
    }

    /// 记着的那一条删掉了，或现行那一版里已不是它：清掉。没记着就什么都不做。
    pub(crate) fn forget_the_intent_of_an_abandoned_batch_still_to_drop(&mut self) {
        self.intent_of_an_abandoned_batch_still_to_drop = None;
    }

    /// 此刻记着一条回锁时被回退门压住、还没删的活意图：放弃那一批时这一趟停在 [`CompactionRunStop::BatchAbandonedAfterABackgroundComputeError`]，
    /// 之后取下一批的输入之前先单发一次只删意图的发布删它。
    #[must_use]
    pub fn has_an_intent_of_an_abandoned_batch_still_to_drop(&self) -> bool {
        self.intent_of_an_abandoned_batch_still_to_drop.is_some()
    }

    /// 一趟停下时记下不发布的停：`current_txg` 是现行那一版，`switches` 是这一趟用的开关。别的结局清掉备忘（下一次照常判）。
    pub(crate) fn note_the_stop_of_a_run(
        &mut self,
        stop: CompactionRunStop,
        current_txg: CheckpointTxg,
        switches: CompactionSwitches,
    ) {
        self.stop_that_publishes_nothing_until_the_next_publish = stop
            .publishes_nothing_until_the_next_publish()
            .then_some(StopThatPublishesNothingUntilTheNextPublish {
                checkpoint_txg: current_txg,
                switches,
                stop,
            });
    }

    /// 备忘记着的停这一刻还算不算数：现行那一版与开关都没变才算，交回那一次的停。
    #[must_use]
    pub(crate) fn stop_still_holding_at(
        &self,
        current_txg: CheckpointTxg,
        switches: CompactionSwitches,
    ) -> Option<CompactionRunStop> {
        self.stop_that_publishes_nothing_until_the_next_publish
            .filter(|noted| noted.checkpoint_txg == current_txg && noted.switches == switches)
            .map(|noted| noted.stop)
    }

    /// 这次挂载那一份预算还要不要从根环读，并记下读过了（读没读通都只读这一次）。
    pub(crate) fn take_the_reading_of_the_publishes_of_the_mount_from_the_ring(&mut self) -> bool {
        std::mem::replace(
            &mut self.publishes_of_the_mount_are_still_to_be_read_from_the_ring,
            false,
        )
    }

    /// 发成一次用户改动：已经过去的发布数从 0 数，根环里最新的用户状态就是这一次，挂载那一份不用再读。
    pub(crate) fn note_a_user_change_published(&mut self) {
        self.publishes_since_the_last_user_change = 0;
        self.publishes_of_the_mount_are_still_to_be_read_from_the_ring = false;
    }

    /// 取走还没完的那一趟（放锁的后台入口接着推它时）。
    pub(crate) fn take_the_background_trip_under_way(&mut self) -> Option<BackgroundTripUnderWay> {
        self.background_trip_under_way.take()
    }

    /// 记下这一趟这次调用之后还没完。
    pub(crate) fn keep_the_background_trip_under_way(&mut self, trip: BackgroundTripUnderWay) {
        self.background_trip_under_way = Some(trip);
    }

    /// 放锁的后台入口有没有一趟没完（运行库按它决定多久之后再醒）。
    #[must_use]
    pub fn has_a_background_trip_under_way(&self) -> bool {
        self.background_trip_under_way.is_some()
    }

    /// 可写挂载读盘建上一版时只坏一份的次数（[`Self::count_unit_reads_with_one_copy_failing_while_mounting`] 记的）。
    #[must_use]
    pub fn unit_reads_with_one_copy_failing_while_mounting(&self) -> u64 {
        self.unit_reads_with_one_copy_failing_while_mounting
    }

    /// 把可写挂载读盘建上一版时只坏一份的次数加进这一份挂载期状态（挂载交回之前调一次）。
    ///
    /// # Panics
    /// 加起来越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
    pub(crate) fn count_unit_reads_with_one_copy_failing_while_mounting(
        &mut self,
        reads_while_mounting: u64,
    ) {
        self.unit_reads_with_one_copy_failing_while_mounting = self
            .unit_reads_with_one_copy_failing_while_mounting
            .checked_add(reads_while_mounting)
            .expect("一次挂载里只坏一份的读次数装得进 u64");
    }

    /// 可写挂载读盘建上一版时读到只坏一份、还没交给另发那一批的那几个单元的映射 key（[`Self::keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting`]
    /// 记的，会话下一次用户改动发成之后取走）。
    #[must_use]
    pub fn mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
        &self,
    ) -> &[Vec<u8>] {
        &self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
    }

    /// 记下读到只坏一份的那几个单元的映射 key（挂载交回之前调一次，接在已记的后面、同一把只记一次）。
    pub(crate) fn keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
        &mut self,
        mapping_keys: Vec<Vec<u8>>,
    ) {
        for mapping_key in mapping_keys {
            if !self
                .mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
                .contains(&mapping_key)
            {
                self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting
                    .push(mapping_key);
            }
        }
    }

    /// 取走记着的那几把 key（交给另发那一批），这一份挂载期状态里清空：另发那一批不管搬没搬成都只试这一次，被拒的结局随那次改动报出。
    pub(crate) fn take_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
        &mut self,
    ) -> Vec<Vec<u8>> {
        std::mem::take(
            &mut self.mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting,
        )
    }

    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationBatchAfterTheChangeRefusedForSpace`]）。产品路径一处都不调它。
    pub fn force_the_relocation_batch_after_the_change_refused_for_space_for_tests(
        &mut self,
        forcing: RelocationBatchAfterTheChangeRefusedForSpace,
    ) {
        self.relocation_batch_after_the_change_refused_for_space = forcing;
    }

    /// 会话另发那一批的这一次该不该当作被空间准入拒（开关用掉一次就记下：「接下来这几次」少一次，用完回到产品值）。
    pub(crate) fn take_a_forced_space_refusal_of_the_relocation_batch_after_the_change(
        &mut self,
    ) -> bool {
        match self.relocation_batch_after_the_change_refused_for_space {
            RelocationBatchAfterTheChangeRefusedForSpace::NotForced => false,
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedOnEveryAttemptForTests => true,
            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                attempts,
            } => {
                self.relocation_batch_after_the_change_refused_for_space =
                    match NonZeroU64::new(attempts.get() - 1) {
                        Some(attempts_left) => {
                            RelocationBatchAfterTheChangeRefusedForSpace::ForcedForTheNextAttemptsForTests {
                                attempts: attempts_left,
                            }
                        }
                        None => RelocationBatchAfterTheChangeRefusedForSpace::NotForced,
                    };
                true
            }
        }
    }

    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationBatchAfterTheChangeRefusedNotForSpace`]）。
    /// 产品路径一处都不调它。
    pub fn force_the_relocation_batch_after_the_change_refused_not_for_space_for_tests(
        &mut self,
        forcing: RelocationBatchAfterTheChangeRefusedNotForSpace,
    ) {
        self.relocation_batch_after_the_change_refused_not_for_space = forcing;
    }

    /// 会话另发那一批的这一次该不该当作被拒、不是空间不够。
    #[must_use]
    pub(crate) fn is_the_relocation_batch_after_the_change_forced_refused_not_for_space(
        &self,
    ) -> bool {
        match self.relocation_batch_after_the_change_refused_not_for_space {
            RelocationBatchAfterTheChangeRefusedNotForSpace::NotForced => false,
            RelocationBatchAfterTheChangeRefusedNotForSpace::ForcedOnEveryAttemptForTests => true,
        }
    }

    /// 装上只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationBatchAfterTheChangeFails`]）。产品路径一处都不调它。
    pub fn force_the_floor_raise_for_the_relocation_batch_after_the_change_to_fail_for_tests(
        &mut self,
        forcing: FloorRaiseForTheRelocationBatchAfterTheChangeFails,
    ) {
        self.floor_raise_for_the_relocation_batch_after_the_change_fails = forcing;
    }

    /// 为另发那一批推的这一串抬 F 该不该当作自己报了错。
    #[must_use]
    pub(crate) fn is_the_floor_raise_for_the_relocation_batch_after_the_change_forced_to_fail(
        &self,
    ) -> bool {
        match self.floor_raise_for_the_relocation_batch_after_the_change_fails {
            FloorRaiseForTheRelocationBatchAfterTheChangeFails::NotForced => false,
            FloorRaiseForTheRelocationBatchAfterTheChangeFails::ForcedOnEveryRaiseForTests => true,
        }
    }

    /// 这次挂载里读—改—写读回旧数据单元时只坏一份的次数（各次发布之和）。
    #[must_use]
    pub fn old_data_unit_reads_back_with_one_copy_failing(&self) -> u64 {
        self.old_data_unit_reads_back_with_one_copy_failing
    }

    /// 把一次用户改动里各次发布读回时只坏一份的次数加进这次挂载的累计。
    ///
    /// # Panics
    /// 加起来越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
    pub(crate) fn count_old_data_unit_reads_back_with_one_copy_failing(
        &mut self,
        reads_of_the_change: u64,
    ) {
        self.old_data_unit_reads_back_with_one_copy_failing = self
            .old_data_unit_reads_back_with_one_copy_failing
            .checked_add(reads_of_the_change)
            .expect("一次挂载里只坏一份的读回次数装得进 u64");
    }

    /// 定时兜底这一次该不该再查一次起跑判据（D26（后台整理与放置回收） 已定项 13 第 5 条「另有定时兜底，每隔一段时间再查一次」）：
    /// 关着就不查；开着而还没查过、或离上一次查过了一个间隔，就该查，并把 `now` 记成上一次查的读数。调用方的钟往回走时按没过间隔算。
    pub(crate) fn check_the_timer_fallback(
        &mut self,
        timer_fallback: CompactionTimerFallback,
        now: CallerMonotonicClockReading,
    ) -> TimerFallbackCheck {
        let interval = match timer_fallback {
            CompactionTimerFallback::Off => return TimerFallbackCheck::Off,
            CompactionTimerFallback::EveryMilliseconds(interval_in_milliseconds) => {
                Duration::from_millis(interval_in_milliseconds.get())
            }
        };
        let is_due = match self.timer_fallback_last_check {
            None => true,
            Some(last_check) => now.0.saturating_sub(last_check.0) >= interval,
        };
        if is_due {
            self.timer_fallback_last_check = Some(now);
            TimerFallbackCheck::Due
        } else {
            TimerFallbackCheck::IntervalNotElapsed
        }
    }

    /// 写带宽上限放不放下一批（D26（后台整理与放置回收） 已定项 13 第 6 条）：不限就放；限着就看这一个窗口——没有窗口、或离窗口起点过了
    /// [`COMPACTION_WRITE_BANDWIDTH_WINDOW`]，在 `now` 开一个新窗口；窗口里已写的字节到了上限就不放。所以一个窗口里整理至多写上限加一批的字节。
    ///
    /// # Errors
    /// [`CompactionRunStop::WriteBandwidthLimitReached`]：这一个窗口里已写的字节不少于上限。
    pub(crate) fn admit_the_next_batch_under_the_write_bandwidth_limit(
        &mut self,
        write_bandwidth_limit: CompactionWriteBandwidthLimit,
        now: CallerMonotonicClockReading,
    ) -> Result<(), CompactionRunStop> {
        let limit_in_bytes_per_second = match write_bandwidth_limit {
            CompactionWriteBandwidthLimit::Unlimited => return Ok(()),
            CompactionWriteBandwidthLimit::BytesPerSecond(limit) => limit.get(),
        };
        let window = match self.write_bandwidth_window {
            Some(window)
                if now.0.saturating_sub(window.started_at.0)
                    < COMPACTION_WRITE_BANDWIDTH_WINDOW =>
            {
                window
            }
            Some(_) | None => CompactionWriteBandwidthWindow {
                started_at: now,
                bytes_written: 0,
            },
        };
        self.write_bandwidth_window = Some(window);
        if window.bytes_written >= limit_in_bytes_per_second {
            Err(CompactionRunStop::WriteBandwidthLimitReached {
                bytes_written_in_the_window: window.bytes_written,
                limit_in_bytes_per_second,
            })
        } else {
            Ok(())
        }
    }

    /// 后台入口发成一次整理发布之后记下它写的字节（限着写带宽时才记；窗口在放这一批时已开好）。
    pub(crate) fn note_the_bytes_written_by_a_compaction_publish(
        &mut self,
        write_bandwidth_limit: CompactionWriteBandwidthLimit,
        bytes_written: u64,
    ) {
        match write_bandwidth_limit {
            CompactionWriteBandwidthLimit::Unlimited => {}
            CompactionWriteBandwidthLimit::BytesPerSecond(_) => {
                let window = self
                    .write_bandwidth_window
                    .as_mut()
                    .expect("限着写带宽时放这一批之前开了窗口（admit_the_next_batch_under_the_write_bandwidth_limit）");
                window.bytes_written = window.bytes_written.saturating_add(bytes_written);
            }
        }
    }

    /// 这次挂载里释放前核出对不上而隔离的槽（常驻隔离集）。
    #[must_use]
    pub fn quarantined_slots(&self) -> &BTreeSet<SlotNumber> {
        &self.quarantined_slots
    }

    /// 把发布路径记在分配器上、还没取走的隔离落点并进常驻隔离集（跨度里的每一个槽）。
    pub fn take_the_quarantines_noted_by_the_publishes(
        &mut self,
        on_the_allocator: &mut CompactionSwitchesAndTheExcludedRegion,
    ) {
        for (slot, span) in on_the_allocator.take_the_quarantines_noted_by_the_publishes() {
            for covered in slot.0..slot.0 + span {
                self.quarantined_slots.insert(SlotNumber(covered));
            }
        }
    }
}

/// 一块盘上按记录口径的全空聚簇段数（D3（空间分配） 已定项 10 ① 与 D26（后台整理与放置回收） 已定项 1 第 1 条收口的那一句：
/// 段内 64 槽都没有未释放分配记录的段数，C372）。只数单元区里的整段。整理（签发时的 `segs0`、一条意图完成之后「无产出就停」）
/// 从这一版的分配记录现扫它；记账第 11 项与合取 7 读分配器按同一个定义增量维护的那一份（`allocator::DeviceFreeMap::empty_segments`）：
/// 定义相同，算法不同（准入路径上不许扫，整理在后台可以扫）。
#[must_use]
pub fn empty_cluster_segments_by_the_records(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> u64 {
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let unit_area_start = unit_area.start.slot().0;
    let mut segments_with_an_unreleased_record: BTreeSet<u64> = BTreeSet::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
            if let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) {
                let segment = offset_in_unit_area / CLUSTER_SEGMENT_SLOTS;
                if segment < full_segments {
                    segments_with_an_unreleased_record.insert(segment);
                }
            }
        }
    }
    full_segments - u64::try_from(segments_with_an_unreleased_record.len()).expect("段数")
}

/// 签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条）的左边：R 那块盘上、R 之外、「非全空」的段里没挡的槽数。
/// 「非全空」按记录口径；只装提交内生块的段不算非全空——段里至少一个槽罩在 `slots_holding_user_data`（码 1 数据单元的落点，
/// 两盘同槽只比槽号）里才算。`is_unblocked(槽)` 说那一槽此刻能不能当目的地（已分配、隔离、抬 F 扣住三种位一个都没置）。
#[must_use]
pub fn holes_outside_the_region_in_segments_holding_user_data<IsUnblocked>(
    allocation_records: &[AllocationRecord],
    slots_holding_user_data: &BTreeSet<SlotNumber>,
    unit_area: UnitAreaOfOneDevice,
    region: CompactionRegion,
    is_unblocked: IsUnblocked,
) -> u64
where
    IsUnblocked: Fn(SlotNumber) -> bool,
{
    let unit_area_start = unit_area.start.slot().0;
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let mut segments_holding_user_data: BTreeSet<u64> = BTreeSet::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        let holds_user_data = (record.slot.0..record.slot.0 + u64::from(record.span_slots))
            .any(|slot| slots_holding_user_data.contains(&SlotNumber(slot)));
        if !holds_user_data {
            continue;
        }
        if let Some(offset_in_unit_area) = record.slot.0.checked_sub(unit_area_start) {
            let segment = offset_in_unit_area / CLUSTER_SEGMENT_SLOTS;
            if segment < full_segments {
                segments_holding_user_data.insert(segment);
            }
        }
    }
    segments_holding_user_data
        .into_iter()
        .map(|segment| unit_area_start + segment * CLUSTER_SEGMENT_SLOTS)
        .filter(|segment_start| !region.contains_slot(SlotNumber(*segment_start)))
        .map(|segment_start| {
            u64::try_from(
                (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
                    .filter(|slot| is_unblocked(SlotNumber(*slot)))
                    .count(),
            )
            .expect("一段 64 槽")
        })
        .sum()
}

/// 单元区里没有一条未释放记录罩到的整段，各交一份空的扫描结果（[`crate::region_scan::scan_region`] 对这样的段交回的就是它：
/// 没有起点在段里的记录，也没有跨进段里的记录）。选段之前带上它们，[`choose_region`] 就不为每个全空段各要一次「先扫」——
/// 否则一次签发要扫单元区的每一个全空段、每扫一段再选一遍。只读记录，不读设备。
#[must_use]
pub fn region_scans_of_the_segments_without_an_unreleased_record(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<RegionScan> {
    let unit_area_start = unit_area.start.slot().0;
    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
    let mut segments_touched_by_an_unreleased_record: BTreeSet<u64> = BTreeSet::new();
    for record in allocation_records
        .iter()
        .filter(|record| record.device == unit_area.device && !record.is_released)
    {
        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
            if let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) {
                segments_touched_by_an_unreleased_record
                    .insert(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS);
            }
        }
    }
    (0..full_segments)
        .filter(|segment| !segments_touched_by_an_unreleased_record.contains(segment))
        .map(|segment| RegionScan {
            region: CompactionRegion::of_one_cluster_segment(
                unit_area.device,
                SlotNumber(unit_area_start + segment * CLUSTER_SEGMENT_SLOTS),
            )
            .expect("单元区起点在聚簇段边界上（UnitAreaStart 的不变量），整段的起点都是 64 的倍数"),
            allocations: Vec::new(),
        })
        .collect()
}

/// 签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条）：空洞数 ≥ R 里要搬的单元数才签。
#[must_use]
pub const fn holes_admit_the_issue(holes: u64, units_to_move: u64) -> bool {
    holes >= units_to_move
}

/// 停机谓词（D26（后台整理与放置回收） 已定项 1 第 2 条，判决 G4）：删意图那一刻 R 按记录全空——R 那块盘上起点在 R 里的未释放记录一条都没有。
#[must_use]
pub fn region_is_empty_by_the_records(
    allocation_records: &[AllocationRecord],
    region: CompactionRegion,
) -> bool {
    !allocation_records.iter().any(|record| {
        record.device == region.device() && !record.is_released && region.contains_slot(record.slot)
    })
}

/// 一条意图完成之后签不签下一条：全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为真才签。
#[must_use]
pub const fn the_completed_intent_yielded(
    empty_cluster_segments_now: u64,
    empty_cluster_segments_at_issue: u64,
) -> bool {
    empty_cluster_segments_now > empty_cluster_segments_at_issue
}

/// 签发之后 R 里出现了隔离槽没有（一个槽既钉住又在隔离集里，区域扫描判「钉住」照旧，这一判另外直接查隔离集，
/// 两者同时成立也算有）。
#[must_use]
pub fn region_holds_a_quarantined_slot(
    region: CompactionRegion,
    quarantined_slots: &BTreeSet<SlotNumber>,
) -> bool {
    quarantined_slots
        .range(region.start_slot()..region.end_slot())
        .next()
        .is_some()
}

/// 用户改动发成之后起跑判据查出的结果。整理挪出了用户改动的调用线程（
/// D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条「不许进前台路径」）：发布路径只查、一批都不发，查出「该跑了」记在会话上，
/// 由后台入口（`mounted_session::MountedSession::run_compaction_in_the_background`）取走去跑。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionStartJudgement {
    NotDue,
    /// 一次用户改动发成之后起跑判据为真，后台入口还没来取；几次改动之间没来取的并成这一个。
    DueAfterAUserChange,
}

/// 调用方单调钟的一次读数（从调用方自己定的起点算）：核心层不起线程、不读钟，后台入口的定时兜底与写带宽上限只读调用方交进来的读数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallerMonotonicClockReading(pub Duration);

/// 定时兜底（D26（后台整理与放置回收） 已定项 13 第 5 条「另有定时兜底，每隔一段时间再查一次，间隔是挂载参数」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionTimerFallback {
    Off,
    /// 每隔这么多毫秒查一次起跑判据。
    EveryMilliseconds(NonZeroU64),
}

/// 定时兜底间隔的默认：关。核心层不起线程，调用方不给间隔就没有定时兜底；挂载就起的周期线程那一层给它的间隔是周期的一半
/// （`singlefs_mount_runtime`，周期默认 30 秒）。
pub const COMPACTION_TIMER_FALLBACK_BY_DEFAULT: CompactionTimerFallback =
    CompactionTimerFallback::Off;

/// 定时兜底这一次查不查（[`CompactionOfThisMount::check_the_timer_fallback`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerFallbackCheck {
    Off,
    IntervalNotElapsed,
    Due,
}

/// 整理占盘写带宽的上限（D26（后台整理与放置回收） 已定项 13 第 6 条，挂载参数）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionWriteBandwidthLimit {
    Unlimited,
    BytesPerSecond(NonZeroU64),
}

/// 写带宽上限的默认：不限。默认值待 C412（整理的挂钟代价没量过） 与
/// E10（老化：结构抗老化 vs 事后整理）。
pub const COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT: CompactionWriteBandwidthLimit =
    CompactionWriteBandwidthLimit::Unlimited;

/// 写带宽上限按字节每秒给，一个窗口就是一秒（[`CompactionOfThisMount::admit_the_next_batch_under_the_write_bandwidth_limit`]）。
pub const COMPACTION_WRITE_BANDWIDTH_WINDOW: Duration = Duration::from_secs(1);

/// 写带宽的一个窗口：从哪一刻起、窗口里整理已写了多少字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CompactionWriteBandwidthWindow {
    started_at: CallerMonotonicClockReading,
    bytes_written: u64,
}

/// 后台入口的节奏参数（D26（后台整理与放置回收） 已定项 13 第 5、6 条；每次调用由调用方交进来，挂载参数那一层有了之后从那里取）。
/// 定时兜底那几趟没有自己的发布预算：与用户改动共用同一份 `B_gc`（[`publishes_left_of_the_budget_given_by_the_last_user_change`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionPacing {
    pub timer_fallback: CompactionTimerFallback,
    pub write_bandwidth_limit: CompactionWriteBandwidthLimit,
}

impl CompactionPacing {
    /// 两样都取默认（[`COMPACTION_TIMER_FALLBACK_BY_DEFAULT`]、[`COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT`]）。
    pub const BY_DEFAULT: CompactionPacing = CompactionPacing {
        timer_fallback: COMPACTION_TIMER_FALLBACK_BY_DEFAULT,
        write_bandwidth_limit: COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
    };
}

/// 这一批之前调用方的回答（D26（后台整理与放置回收） 已定项 13 第 6 条「前台有 fsync 在等时不发下一批」）：发不发下一批，要不要让路。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerOfTheCallerBeforeTheNextBatch {
    NoForegroundFsyncWaiting,
    ForegroundFsyncIsWaiting,
    /// 前台有 fsync 之外的入口在等会话锁（收下改动、单发改动、读、切换后台计算；D26（后台整理与放置回收） 已定项 13 第 6 条）：不发下一批，
    /// 全空段低于底额、前台写不进时的破例批与连着让路到了下限（[`CompactionYieldFloor`]）的那一批照发。只在取一批的输入之前问，回锁那一刻不问。
    ForegroundEntryOtherThanAnFsyncIsWaitingForTheSessionLock,
    /// 前台不一定有 fsync 在等，是调用方在停这条驱动后台入口的线程：照样不发下一批，低于底额的破例也不发（线程很快 join 得上）。
    CallerIsStoppingItsThread,
}

/// 后台入口每推一批之前问调用方的两样：此刻的钟、这一批之前它的回答。核心层不起线程，调用方在自己的线程上驱动后台入口，
/// 钟与前台的 fsync 队列都在调用方手里；测试里由一份假的实现。
pub trait CallerOfTheBackgroundCompaction {
    fn clock_reading(&self) -> CallerMonotonicClockReading;
    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch;
}

/// 回退门给根环留的余量（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：一次整理发布之后，再发这么多次发布，根环里仍有 4 个不同用户状态。
/// 取每区根槽数 S（根环 3S 槽的三分之一）：最新的 4 个状态之后门放 `3S − 4 − S = 2S − 4` 次发布，S = 5 时是 6 次，
/// 产品批大小 16 下一条整段意图（至多 `1 + ⌈64 ÷ 16⌉ = 5` 次发布）走得完；S = 4 时是 4 次，走不完就停在门上报出来。
/// 下一次可写挂载自己的发布不经门：写行加暖机至多 3 次，续做一条意图至多 5 次，合起来至多 8 次。S = 8 时一次挂载落在余量里；S 更小时一次挂载就可能用完余量。
/// 连崩几次用完这几槽之后，挂载自己的发布照样挤掉用户状态。
/// 这笔算术不算 `accept` 被空间拒时当场推的抬 F 那几串（`mounted_session::MountedSessionWithPendingChanges::accept`）：它们不经门，推一串也用掉这几槽。
#[must_use]
pub fn root_ring_slots_left_by_a_compaction_publish(
    root_ring_slots_per_region: RootRingSlotsPerRegion,
) -> u64 {
    root_ring_slots_per_region.count()
}

/// 一次整理运行的发布预算：
/// `3S − 4 − 已经过去的发布数`，至少 1。`S` 是每区根槽数。
#[must_use]
pub fn publish_budget_of_a_compaction_run(
    root_ring_slots_per_region: u64,
    publishes_already_passed: u64,
) -> u64 {
    (3 * root_ring_slots_per_region)
        .saturating_sub(4)
        .saturating_sub(publishes_already_passed)
        .max(1)
}

/// 后台入口的一趟还能发几次整理发布（与用户改动共用
/// `B_gc`）：一次用户改动给出一份 `B_gc`（[`publish_budget_of_a_compaction_run`] 在已经过去的发布数为 0 时的值，至少 1），之后用户改动那一趟
/// 与定时兜底那几趟都从这同一份里扣；扣完是 0，等下一次用户改动（会话发成一次用户改动就把已经过去的发布数归零）才再有。
/// 不像显式入口那样每一趟至少 1：那一条加上定时兜底，闲置时整理发布就没有上限（200 拍连着 213 次，线是 `3S − 4`）。
#[must_use]
pub fn publishes_left_of_the_budget_given_by_the_last_user_change(
    root_ring_slots_per_region: u64,
    publishes_since_the_last_user_change: u64,
) -> u64 {
    publish_budget_of_a_compaction_run(root_ring_slots_per_region, 0)
        .saturating_sub(publishes_since_the_last_user_change)
}

/// 整理的两样挂载参数（都不存盘）：批大小与三条水位线（[`crate::mount::mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionMountParameters {
    pub batch_units: CompactionBatchUnits,
    pub watermarks: CompactionWatermarks,
}

impl CompactionMountParameters {
    /// 两样都取产品默认。
    pub const BY_DEFAULT: CompactionMountParameters = CompactionMountParameters {
        batch_units: COMPACTION_BATCH_UNITS_BY_DEFAULT,
        watermarks: CompactionWatermarks::BY_DEFAULT,
    };
}

/// 全空段低于底额、前台写入类发布已写不进（合取 7，`N₀` 是开关给的那一档）：任一块盘低于就算。后台入口的破例（每趟至少发 1 次）判它，代价随盘数。
#[must_use]
pub fn foreground_writes_are_below_the_empty_cluster_segments_floor(
    allocator: &PoolAllocator,
) -> bool {
    let floor = allocator
        .compaction()
        .switches
        .empty_cluster_segments_floor
        .segments();
    allocator
        .devices
        .iter()
        .any(|device_map| device_map.empty_segments() < floor)
}
