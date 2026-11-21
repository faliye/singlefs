//! 几条整理崩溃流共用的说法：流的种类、流里每一步的种类与录下来的流，R 取哪一段、整理开关怎么设，与流里几次写的内容。

use super::*;

/// 这几条流强制选的整理区域 R 的起始槽：第一个文件那次发布的提交内生块开在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)），
/// 重挂之后本次挂载不在它上面开段。两块盘同槽，意图按盘 0 点名。
pub const REGION_START_SLOT_OF_THE_CRASH_FLOWS: u64 = 50240;

/// 一批搬几个单元（判决 G6：批大小 1–3；取 2，一批里有不止一个单元，单元段写数 2 ×（提交内生块 + 2）仍在 30 以内）。
pub const COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS: usize = 2;

/// 会话里整理一趟只发一次：「已经过去的发布数」取得比 `3S − 4` 大，发布预算落到下限 1（`compaction_plan::publish_budget_of_a_compaction_run`）。
pub const PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE: u64 = 1_000;

/// 一条意图至多几次整理发布（签发、各批、最后一批）：R 里一个段至多 64 个单元，批大小 2，再加签发；录流时超了就是整理停不下来。
pub const COMPACTION_PUBLISHES_OF_ONE_INTENT_AT_MOST: usize =
    1 + 64 / COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS;

/// 生命周期闭环流「回收」那一段至多几轮（抬 F 到准入上限、建一次 inode）：R 整段回收、会话对它发 discard 之前超了就是回收不下来。
pub const LIFECYCLE_RECLAIM_ROUNDS_AT_MOST: usize = 8;

/// 生命周期闭环流「再分配」那一段至多建几次 inode：R 里有一条未释放的记录之前超了就是分配不到搬出来的空段上。
pub const LIFECYCLE_REALLOCATION_PUBLISHES_AT_MOST: usize = 12;

/// 生命周期闭环流第一次覆盖写（建快照之前）的内容：一个数据单元装得下、与第一个文件与快照下覆盖写那一份都不同。
#[must_use]
pub fn content_overwritten_before_the_snapshot() -> Vec<u8> {
    (0..2000_usize)
        .map(|index| u8::try_from((index * 7 + 3) % 241).expect("小于 256"))
        .collect()
}

/// 意图活着时覆盖写第一个文件的内容：一个数据单元装得下、与流里别的内容都不同。
#[must_use]
pub fn content_overwritten_while_the_intent_is_live() -> Vec<u8> {
    (0..2100_usize)
        .map(|index| u8::try_from((index * 11 + 5) % 239).expect("小于 256"))
        .collect()
}

/// 活意图期间写类改动流在 R 里留几个叶容器。覆盖写第一个文件只重写第一片叶容器、inode 根与 extent 根，R 里只有 A、B、C 留下的单元时
/// （extent 下段节点、extent 根、一片叶容器、inode 根）第一批搬走两个、覆盖写放掉另两个，R 按记录全空，之后是一次只删意图的发布、下一批无从谈起。
/// 取 5：第一批搬走 C 留下的 extent 下段节点与 extent 根，覆盖写放掉第一片叶容器与 inode 根，留下的 4 片够再搬一批、再搬最后一批；R 里连 C 留下的那两个一共 8 个单元，
/// 从签发起每批搬满两个，续做的最后一批也搬单元。
pub const INODE_LEAF_CONTAINERS_LEFT_IN_THE_REGION_BEHIND_THE_WRITE: u64 = 5;

/// 那几个空 inode 的调用方时间比第一个文件晚这么多秒：与这条流里别的写都不同。
pub const SECONDS_FROM_THE_FIRST_WRITE_TO_THE_INODES_LEFT_IN_THE_REGION: u64 = 90;

/// R：盘 0 上段 [`REGION_START_SLOT_OF_THE_CRASH_FLOWS`]。
#[must_use]
pub fn region_of_the_crash_flows() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(REGION_START_SLOT_OF_THE_CRASH_FLOWS),
    )
    .expect("64 的倍数、在六字节槽号以内")
}

/// 这几条流的整理开关：强制选 R、一批 [`COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS`] 个，别的字段取产品值（源落点在映射换位置的那一次发布里释放）。
/// 录流的会话与观察者在崩溃镜像上的可写挂载用同一份。
#[must_use]
pub fn compaction_switches_of_the_crash_flows() -> CompactionSwitches {
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(COMPACTION_BATCH_UNITS_OF_THE_CRASH_FLOWS).expect("至少 1"),
        ),
        forced_region: Some(region_of_the_crash_flows()),
        run_forcing: CompactionRunForcing::FollowTheWatermarks,
        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
        destination_policy:
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
        release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::OfTheFirstVersion,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    }
}

/// 这一份录的几条流。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionCrashFlow {
    /// A → B → C → 重挂 → 签发 → 一批 → 最后一批。
    CompactionRelocation,
    /// A → S → W → 重挂 → 签发 → 几批 → 最后一批（R 里有只被快照引用的单元）。
    CompactionRelocatingABlockSharedWithASnapshot,
    /// A → B → C → 建空 inode 填满几个叶容器 → 重挂 → 签发 → 一批 → 意图活着时覆盖写第一个文件（放掉 R 里这次发布重写的落点）→ 一批 → 最后一批。
    CompactionWithAWriteWhileTheIntentIsLive,
    /// A → B → C → 重挂 → 签发 → 一批 →（压准入余量）最后一批被空间拒、暂停删意图 →（解开余量）重新签发 → 最后一批。
    CompactionPausedShortOfSpaceDroppingTheIntent,
    /// A → B → C → 重挂 → 签发 → 一批 →（注入一次算着出错）那一批放弃、删意图、整理停下 →（切换解除停下）重新签发 → 最后一批。
    CompactionStoppedAfterAComputeErrorDroppingTheIntent,
    /// A → B → C → 重挂 → 签发 → 一批 →（R 之外的空槽全隔离）最后一批取不到落点、暂停删意图，删意图那一次的新落点落在 R 里 →（解开隔离）。
    CompactionPausedWithOnlyTheRegionFreeDroppingTheIntentInTheRegion,
    /// A → 覆盖写 → S → 再覆盖写 → 删文件 → 重挂 → 整理搬共享块 → 删快照 → 回收 → 在搬出来的空段上再分配。
    LifecycleClosure,
}

/// 流里一步是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionFlowStepKind {
    /// 多记录流的 B、C（实例 1）。
    SequentialWrite,
    /// 生命周期闭环流建快照之前那一次覆盖写（实例 1）。
    OverwriteBeforeTheSnapshot,
    /// 快照流那几步（实例 1，发布照 `common_snapshot_prefix`）。
    Snapshot(SnapshotFlowStepKind),
    /// 重开镜像、可写挂载（取号、写行、暖机）。
    MountWritable,
    /// 整理签发：一次只把意图写进 logged_ops 树的发布。
    CompactionIssue,
    /// 整理的一批（意图还在）。
    CompactionBatch,
    /// 整理的最后一批：同一次发布删意图。
    CompactionFinalBatch,
    /// 会话里删快照一次调用：意图那次发布与全部批。
    DestroySnapshotThroughTheSession,
    /// 抬 F 到准入上限那一串（先写系统配置、再推带新 F 的空发布）。
    RaiseFloorToTheAdmissionCeiling,
    /// 建空 inode。生命周期闭环流与搬快照单元流在会话里建一个（文件删光之后的一版只许建 inode）；活意图期间写类改动流在重挂之前的实例 1 里建，把叶容器留在 R 里。
    NewInodesThroughTheSession,
    /// 整理意图活着的时候会话里覆盖写第一个文件（写类用户改动）。
    OverwriteWhileTheIntentIsLive,
    /// 整理停下（一批被空间拒的暂停、一批算着出错放弃）：单独一次只删意图的发布（不搬单元，R 里还有未释放的记录）。
    CompactionIntentDroppedOnAStop,
}

impl CompactionFlowStepKind {
    /// 入口的名字：崩溃放量那一路按它设点（`<流程>.<周期>.<步>` 的流程那一段；设点在 `common_crash_points`）。
    /// 快照那几步与快照流同名（节点共享），B、C 与多记录流同名（`common_crash_points::sequential_write`）。
    #[must_use]
    pub const fn entry_name(self) -> &'static str {
        match self {
            Self::SequentialWrite => "sequential_write",
            Self::OverwriteBeforeTheSnapshot => "publish_overwrite",
            Self::Snapshot(kind) => kind.entry_name(),
            Self::MountWritable => "mount_writable",
            Self::CompactionIssue => "publish_compaction_issue",
            Self::CompactionBatch => "publish_compaction_batch",
            Self::CompactionFinalBatch => "publish_compaction_final_batch",
            Self::DestroySnapshotThroughTheSession => "destroy_snapshot_through_the_session",
            Self::RaiseFloorToTheAdmissionCeiling => {
                "raise_rollback_floor_to_the_admission_ceiling"
            }
            Self::NewInodesThroughTheSession => "publish_new_inodes_through_the_session",
            Self::OverwriteWhileTheIntentIsLive => "publish_overwrite_while_the_intent_is_live",
            Self::CompactionIntentDroppedOnAStop => "publish_compaction_intent_drop_on_a_stop",
        }
    }

    /// 计数的名字里这一种那一段（观察者按恢复落到哪一种的那一版分别计数）。
    #[must_use]
    pub const fn counter_name(self) -> &'static str {
        match self {
            Self::SequentialWrite => "sequential_write",
            Self::OverwriteBeforeTheSnapshot => "overwrite_before_the_snapshot",
            Self::Snapshot(kind) => kind.counter_name(),
            Self::MountWritable => "mount_writable",
            Self::CompactionIssue => "compaction_issue",
            Self::CompactionBatch => "compaction_batch",
            Self::CompactionFinalBatch => "compaction_final_batch",
            Self::DestroySnapshotThroughTheSession => "destroy_snapshot_through_the_session",
            Self::RaiseFloorToTheAdmissionCeiling => "raise_floor_to_the_admission_ceiling",
            Self::NewInodesThroughTheSession => "new_inodes_through_the_session",
            Self::OverwriteWhileTheIntentIsLive => "overwrite_while_the_intent_is_live",
            Self::CompactionIntentDroppedOnAStop => "compaction_intent_dropped_on_a_stop",
        }
    }
}

/// 流里一步：哪一种、它发出的那段录制流（绝对下标的半开区间）、它发成的每一版（按先后；一步一次发布的只有一版）、它之后会话发的 discard（盘、段起点）。
#[derive(Clone, Debug)]
pub struct CompactionFlowStep {
    pub kind: CompactionFlowStepKind,
    pub operations: Range<usize>,
    pub versions: Vec<TransactionOutput>,
    pub discards: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 录好的一条流：池（录制流在 `pool.stream` 上）、A 的输出、A 之后的每一步、建过的快照（生命周期闭环流与搬共享块流有）。
pub struct RecordedCompactionFlow {
    pub flow: CompactionCrashFlow,
    pub pool: BuiltPool,
    pub first_file: TransactionOutput,
    pub steps: Vec<CompactionFlowStep>,
    pub snapshot: Option<TreeIdentifier>,
}

impl RecordedCompactionFlow {
    /// 某一种的每一步，按先后。
    #[must_use]
    pub fn steps_of(&self, kind: CompactionFlowStepKind) -> Vec<&CompactionFlowStep> {
        self.steps.iter().filter(|step| step.kind == kind).collect()
    }

    /// A 之后每一步的入口名与它发出的那段录制流（崩溃放量照它设点，`common_crash_points::crash_points_of_a_flow_after_the_first_file`）。
    #[must_use]
    pub fn crash_point_entries_of_the_steps(&self) -> Vec<(&'static str, Range<usize>)> {
        self.steps
            .iter()
            .map(|step| (step.kind.entry_name(), step.operations.clone()))
            .collect()
    }

    /// 整理那几次发布搬走的单元：映射 key 的类标签、源落点、目的落点（盘 0 那一条位置项的槽），按先后。
    #[must_use]
    pub fn relocated_units(&self) -> Vec<RelocatedUnitOfTheFlow> {
        self.steps
            .iter()
            .flat_map(|step| step.versions.iter())
            .flat_map(|version| {
                version
                    .units_relocated_by_this_publish
                    .iter()
                    .map(|relocated| RelocatedUnitOfTheFlow {
                        unit_class: relocated.mapping_key[0],
                        is_kept_only_for_the_snapshots: matches!(
                            relocated.holder,
                            singlefs_core::transaction::RelocatedUnitHolder::KeptForTheSnapshots
                        ),
                        source: relocated
                            .source
                            .iter()
                            .map(|location| (location.device, location.slot))
                            .collect(),
                        destination: relocated
                            .destination
                            .iter()
                            .map(|location| (location.device, location.slot))
                            .collect(),
                    })
            })
            .collect()
    }
}

/// 整理搬走的一个单元（录流的形状核与必红世界的说明用）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelocatedUnitOfTheFlow {
    pub unit_class: u8,
    pub is_kept_only_for_the_snapshots: bool,
    pub source: Vec<(DeviceIdentity, SlotNumber)>,
    pub destination: Vec<(DeviceIdentity, SlotNumber)>,
}
