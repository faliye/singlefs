//! 整理的发布：搬迁批、整理的各步、发布前对这一版的核对与免检单元的修复。

use super::*;

/// 整理的一批为什么没发：计划与上一版对不上（调用方按现行那一版重扫再计划，丁 1 交判第 3 点「每批之前对 R 重扫」），或发布路径的错。
/// 计划那几种都在任何读写之前交回，盘上逐字节不变。
#[derive(Debug)]
pub enum RelocationBatchRefused {
    /// 计划里这把映射 key 不是上一版里进映射的单元，也不是留存的映射条目。
    PlannedUnitNotInTheVersion {
        mapping_key: Vec<u8>,
    },
    /// 映射树自己的节点、树表、实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8），整理不搬它们（区域扫描判钉住）。
    PinnedUnitInThePlan {
        mapping_key: Vec<u8>,
        unit: TransactionUnit,
    },
    /// 计划的源落点不是这个单元在上一版映射里的位置，或跨度与它的类不符。
    PlannedSourceIsNotTheMappingPosition {
        mapping_key: Vec<u8>,
        planned_source: Placement,
        mapping_locations: Option<[LocationEntry; 2]>,
    },
    /// 另发的那一批（[`publish_the_relocations_and_the_repair_of_the_exempt_units`] 之前认位置）沿链读实例表读不出、解不开：认不出记下的那几组位置条目是不是实例表片。
    /// 在这一批任何写之前。
    InstanceTableOfTheVersionUnreadable(RecoveryFailure),
    /// 这个单元的字节不在手里：从盘上重建时提示与映射都读不出的数据单元（D19 已定项 5：照抄位置项、不读内容），
    /// 或留存条目指的单元两份都读不出、核不上位置项里的校验和。原样字节写目的地（判决 G3）做不到，不搬。
    UnitBytesNotAvailable {
        mapping_key: Vec<u8>,
    },
    /// 签发时上一版已有一条活整理意图（全池至多一条，判决 G1 e1）：调用方先推完或丢掉那一条。
    CompactionIntentAlreadyLive {
        issued_txg: CheckpointTxg,
    },
    /// 删意图时上一版没有活整理意图。
    NoLiveCompactionIntent,
    /// 意图达成那一支（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`]）在这次发布算好的账上 R 里还有未释放的记录：
    /// 停机谓词「删意图那一刻 R 按记录全空」（判决 G4）不成立，不删。产品路径上瞬时读错走得到（判决 `research/prompts/m4-segment-code-r2-main-verification.md`
    /// 第三节 H2）：计划里的每个单元在这次发布里都离开 R（搬走，或本来就重写），只有释放之前读盘核把一份隔离时那条记录才留在已分配——
    /// 一份副本读与重读都读不出、下一次又读得出，这一遍就核不过而下一遍核得过。另外两处走得到：R 不排他（只供测试的开关关掉排他）、
    /// 这次发布的新落点落进 R；只供测试的 [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制。
    /// 调用方照这一批不删意图再发一次（意图留着，下一批接着搬）。
    RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
        region: CompactionRegion,
    },
    /// 照抄而核 R 那一支（[`CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion`]）在这次发布算好的账上 R 已按记录全空：
    /// 这一批就是最后一批，意图要同一次发布删（判决 m4-segment-code-r1 第三节 F2）。调用方照同一批删意图再发一次。
    RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
        region: CompactionRegion,
    },
    Publish(PublishError),
}

impl From<PublishError> for RelocationBatchRefused {
    fn from(cause: PublishError) -> Self {
        RelocationBatchRefused::Publish(cause)
    }
}

/// 整理的一批（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3，用户定「重构」取乙）：第三个发布入口，额外参数不并进
/// [`PublishPlan`]（照 [`publish_unlink_step`]）。`planned` 里每一项（[`crate::compaction_plan::plan_batch`] 交出的映射 key 与源落点）
/// 原样字节写到目的地（头不盖戳、birth 不变）、映射条目换到目的地、源落点写释放代进 defer、目的地记一条分配代 = 诞生代的已分配记录
/// （D26（后台整理与放置回收） 已定项 5 第 2 条），记账、分配记录树、中央映射树与树表照常重写；journal 点名项 = 搬迁副本 + 新写的节点，
/// 一个事务号。目的落点由这次发布时的分配器现取（判决 G2，不缓存），活 R 由分配器排除。被快照引用的块照样搬（留存的映射条目跟着换位置）。
/// 这次重写本来就换下的角色（记账树、分配记录树里内容变了的节点）不另搬：它们的新落点本来就不在活 R 里。
/// logged_ops 照抄上一版（意图的签发与删在 [`publish_compaction_step`]，这一个入口就是它的「只搬、意图照抄」那一支）。
/// 照删除的三种发布同一张计划：不写文件内容、不建 inode、实例表照抄、不发树号；准入不归删除类，走 `DeletionStepOfThePublish::NotADeletion`
/// 那一臂（`prepare_the_version_publish_relocating_units`）。整理不是释放路径、走常规准入（D26（后台整理与放置回收） 已定项 3 第 2 句；
/// 用户 2026-10-26 定 C906「照条款：整理批走常规准入」）：式子那一判照判，需求是这次重写的角色（这一个入口照抄 logged_ops，重写的只有记账树、
/// 分配记录树、中央映射树与树表这几样固定点），批次最坏占用（计划里每一项在目的地各占它的跨度、每份副本各一份，
/// `admission::worst_case_occupancy_of_a_compaction_batch`）加进这一次读数的已承诺预留；可用压到需求之下就拒成 `SpaceAdmissionRefused`，
/// 在任何写之前，会话照暂停办（单发一次只删意图的发布丢掉意图，下一次整理从头签发）。落点那一判只在重写的角色里有普通分配时判
/// （`admission::publish_has_an_ordinary_allocation`），搬迁副本的落点由取目的地那一步在任何写之前判（取不到交回 `PlacementRefused`）。
/// 合取 7 免（带搬迁的发布不是前台写入，`reach_of_the_empty_cluster_segments_conjunct`）。同一发布路径上（[`publish_compaction_step`]）
/// 签发那一次照判：logged_ops 根重写是普通分配，两判照判。只删意图、不搬单元的那几次（意图达成而 R 已按记录全空、丢意图、整理停下时删意图）
/// 归删除类（D28（挂载期承诺量） 已定项 1 的不判名单）：式子那一判不判；落点那一判照判——logged_ops 删空时这一版没有 `LoggedOperationsRoot`、
/// 重写的只有固定点，没有普通分配、落点那一判也不判，固定点照常取落点；logged_ops 里另有截断意图时它重写 logged_ops 根，落点那一判照判。
///
/// # Errors
/// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种，在任何读写之前；其余同 `publish_version`（`Publish`）。
///
/// # Panics
/// `planned` 是空的：调用方按 `PlannedBatch::Relocate` 才调（至少一项）。
pub fn publish_relocation_batch<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    assert!(
        !planned.is_empty(),
        "一批至少搬一个单元：调用方按 PlannedBatch::Relocate 才调"
    );
    publish_compaction_step(
        pool,
        allocator,
        previous,
        planned,
        CompactionIntentStep::CarryTheIntent,
        instance,
    )
}

/// 整理的一次发布对意图做什么（[`publish_compaction_step`]；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3、G4；
/// D26（后台整理与放置回收） 已定项 2；D8（核心索引结构） 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionIntentStep {
    /// 只搬一批，logged_ops 照抄。
    CarryTheIntent,
    /// 搬一批、logged_ops 照抄，这一批发布之后 R 按记录全空就不发：交回 `RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried`，调用方照同一批
    /// 删意图再发一次（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2，G4「目标态到达与最后一批同发布删」：
    /// 计划里的单元是这次发布本来就重写的节点时，它们不另搬也离开 R）。会话经 `publish_compaction_step_in_the_session` 另核一样：
    /// R 还不空而计划里有这次发布本来就重写、不另搬的单元就不发（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，
    /// `PlannedUnitsRewrittenAnyway`）。
    CarryTheIntentUnlessThisPublishEmptiesTheRegion,
    /// 签发：logged_ops 追加这一条意图（签发 txg = 这次发布的 txg）。上一版不许已有活整理意图（全池至多一条，判决 G1 e1）。
    Issue(CompactionIntent),
    /// 意图达成：删掉上一版那一条活意图。删之前在这次发布算好的账上核 R 按记录全空（停机谓词「删意图那一刻 R 按记录全空」，判决 G4）——
    /// 带着一批时就是最后一批同发布删。
    DeleteOnceTheRegionIsEmptyByTheRecords,
    /// 丢意图（攻方 K-c：R 里有隔离槽；区域扫描拒了 R；R 里一个能搬的都没有）：删掉上一版那一条活意图，不核 R。
    Drop,
}

/// 删意图那一支（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`]）落盘之前核「R 按记录全空」怎么判。封闭集合。
/// 产品路径恒 [`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]（[`publish_compaction_step`]）；另一种只供测试
/// （`.claude/rules/fs-design.md`「五条硬要求」第 2 条：每条分支必须能被测试强制进入），经
/// [`publish_compaction_step_with_the_region_check_of_the_intent_deletion`] 交进来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionCheckOfTheIntentDeletion {
    /// 在这次发布算好的账上核 R 按记录全空（停机谓词，判决 G4）。
    JudgedByTheRecords,
    /// 只供测试：算好账之后不看记录，照「R 里还有未释放的记录」拒（`RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`），
    /// 与真核不过走同一段：任何写之前交回、分配器换回进来时那一份。用例拿它强制进入会话「删意图那一支核不过、照这一批不删意图再发」那一支
    /// （`mounted_session::MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`）。
    RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests,
}

/// 整理的一次发布（[`publish_relocation_batch`] 的一般形）：搬 `planned` 里那几个单元（可以一个都没有），同一次发布对意图做 `intent_step`
/// （判决 G1：意图条目写进 logged_ops 树，签发那次发布写、最后一批同发布删；D8（核心索引结构） 已定项 5 三态表：签发之前崩 = 没有意图，
/// 签发之后、最后一批之前崩 = 意图还在、挂载续做，最后一批之后 = 意图没了且目标态达成）。别的同 [`publish_relocation_batch`]。
/// 只删意图、不搬单元的那一次（删意图不带一批、丢意图）取落点时不排除活意图的 R（[`publish_compaction_step_in_the_session`]），
/// 删意图那一核不数它自己在 R 里的新落点；带一批的照旧排除 R。
///
/// 签发那次写下的 `segs0` 就是调用方交进来的那条意图里的：签发那次发布之前的全空聚簇段数（判决
/// `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H3，用户 2026-10-16 定「取签发之前」；会话按上一版的分配记录数，
/// `crate::compaction_plan::empty_cluster_segments_by_the_records`）。签发那次发布自己开了新聚簇段时它比这次写下的记账第 11 项多一，
/// 停机谓词「全局差值」相对它比，这条意图就要多腾出那一段才算有产出。只算一遍账、照交进来的写，不按记账换。
///
/// # Errors
/// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种计划错；签发时上一版已有活意图 ⇒ `CompactionIntentAlreadyLive`；删时、照抄而核 R 时上一版没有活意图 ⇒
/// `NoLiveCompactionIntent`；意图达成那一支在这次发布算好的账上 R 还有未释放的记录 ⇒ `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`；
/// 照抄而核 R 那一支在算好的账上 R 已按记录全空 ⇒ `RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried`。
/// 这几种都在任何写之前交回（后两种在算账之后、落盘之前：分配器换回进来时那一份），盘上逐字节不变；其余同 `publish_version`（`Publish`）。
///
/// # Panics
/// `planned` 是空的而 `intent_step` 是两种照抄之一：这样的发布什么都不做，调用方不许走到这里。
pub fn publish_compaction_step<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    publish_compaction_step_with_the_region_check_of_the_intent_deletion(
        pool,
        allocator,
        previous,
        planned,
        intent_step,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        instance,
    )
}

/// 同 [`publish_compaction_step`]，删意图那一支落盘之前的核照 `region_check_of_the_intent_deletion` 判（别的 `intent_step` 不读它）。
/// 产品路径只经 [`publish_compaction_step`] 进来（[`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]）。
///
/// # Errors
/// 同 [`publish_compaction_step`]；删意图那一支装着只供测试的那一种时，算好账之后照 `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted` 拒。
///
/// # Panics
/// 同 [`publish_compaction_step`]。
pub fn publish_compaction_step_with_the_region_check_of_the_intent_deletion<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    match publish_compaction_step_in_the_session(
        pool,
        allocator,
        previous,
        planned,
        intent_step,
        region_check_of_the_intent_deletion,
        PlannedUnitsRewrittenAnyway::LeftToTheCompactionPublish,
        instance,
    ) {
        Ok(published) => Ok(published),
        Err(CompactionStepRefused::Refused(refused)) => Err(refused),
        Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish { mapping_keys }) => {
            unreachable!(
                "照 PlannedUnitsRewrittenAnyway::LeftToTheCompactionPublish 发，落盘之前不核计划里有没有本来就重写的单元，交不回这一种：{mapping_keys:?}"
            )
        }
    }
}

/// 照抄意图的一批（[`CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion`]）在 R 还不空时，计划里有这次发布本来就重写、
/// 不另搬的单元怎么办（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16 定「规划时补足」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlannedUnitsRewrittenAnyway {
    /// 照发：它们照样离开 R，只是不算搬（公开入口 [`publish_compaction_step`] 一族的行为）。
    LeftToTheCompactionPublish,
    /// 不发，交回 [`CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish`]：会话跳过它们、补上 R 里别的可搬单元再计划，
    /// 照抄意图的一批计划里每个单元都真搬，所以至少搬一个（`mounted_session` 推整理的那一步）。
    RefusedSoThatTheSessionPlansAgain,
}

/// 会话这一侧发整理的一次发布被拒（[`publish_compaction_step_in_the_session`]）：公开入口的那些拒绝，或计划里有这次发布本来就重写的单元。
#[derive(Debug)]
pub(crate) enum CompactionStepRefused {
    Refused(RelocationBatchRefused),
    /// 照抄意图的一批在这次发布算好的账上 R 还不空，而计划里这几把 key 的单元是这次发布本来就重写的节点、不另搬（判决 m4-segment-code-r2
    /// 第三节 H4）。在算账之后、落盘之前交回：分配器换回进来时那一份，盘上逐字节不变。`mapping_keys` 按计划的次序、至少一把。
    PlannedUnitsRewrittenAnywayByThisPublish {
        mapping_keys: Vec<Vec<u8>>,
    },
}

impl From<PublishError> for CompactionStepRefused {
    fn from(cause: PublishError) -> Self {
        CompactionStepRefused::Refused(RelocationBatchRefused::Publish(cause))
    }
}

impl From<RelocationBatchRefused> for CompactionStepRefused {
    fn from(refused: RelocationBatchRefused) -> Self {
        CompactionStepRefused::Refused(refused)
    }
}

/// 同 [`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]，照抄意图的一批计划里有这次发布本来就重写的单元时照
/// `planned_units_rewritten_anyway` 办（别的 `intent_step` 不读它）。会话推整理时经它发，交 [`PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain`]。
///
/// # Errors
/// 同 [`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]（包在 [`CompactionStepRefused::Refused`] 里）；照抄而核 R 那一支
/// 在算好的账上 R 还不空、计划里有本来就重写的单元、又交了 `RefusedSoThatTheSessionPlansAgain` ⇒ `PlannedUnitsRewrittenAnywayByThisPublish`。
///
/// # Panics
/// 同 [`publish_compaction_step`]。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、上一版、计划、对意图做什么、删意图那一核怎么判、计划里有本来就重写的单元怎么办、实例代号"
)]
pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
    planned_units_rewritten_anyway: PlannedUnitsRewrittenAnyway,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, CompactionStepRefused> {
    assert!(
        !(planned.is_empty()
            && matches!(
                intent_step,
                CompactionIntentStep::CarryTheIntent
                    | CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion
            )),
        "整理的一次发布要么搬单元、要么签发或删意图：调用方不发什么都不做的发布"
    );
    let live_intent = live_compaction_intent_of_the_entries(&previous.logged_operations);
    let (logged_operations, check) = match (intent_step, live_intent) {
        (CompactionIntentStep::CarryTheIntent, _) => (
            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
            CompactionStepCheckBeforePersisting::NothingToCheck,
        ),
        (CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion, None) => {
            return Err(RelocationBatchRefused::NoLiveCompactionIntent.into())
        }
        (
            CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
            Some((_issued_txg, intent)),
        ) => (
            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
            CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord {
                region: intent.region,
                planned_units_rewritten_anyway,
            },
        ),
        (CompactionIntentStep::Issue(intent), None) => (
            LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(intent),
            CompactionStepCheckBeforePersisting::NothingToCheck,
        ),
        (CompactionIntentStep::Issue(_), Some((issued_txg, _))) => {
            return Err(RelocationBatchRefused::CompactionIntentAlreadyLive { issued_txg }.into())
        }
        (
            CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords
            | CompactionIntentStep::Drop,
            None,
        ) => return Err(RelocationBatchRefused::NoLiveCompactionIntent.into()),
        (
            CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
            Some((issued_txg, intent)),
        ) => (
            LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg },
            CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
                region: intent.region,
                region_check: region_check_of_the_intent_deletion,
                placements_of_this_publish: if planned.is_empty() {
                    PlacementsOfThisPublishInTheRegion::NotCountedTheRegionIsOpenToThisPublish
                } else {
                    PlacementsOfThisPublishInTheRegion::CountedAsRecordsInTheRegion
                },
            },
        ),
        (CompactionIntentStep::Drop, Some((issued_txg, _))) => (
            LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg },
            CompactionStepCheckBeforePersisting::NothingToCheck,
        ),
    };
    let relocations = relocations_checked_against_the_version(pool, allocator, previous, planned)?;
    // 只删意图、不搬单元的那一次（删意图那一支不带一批、丢意图）取落点时放开 R：它落盘的那一版里已没有意图，排他只挡得住它自己，
    // R 之外取不到落点时意图就删不掉、R 一直排他（D26（后台整理与放置回收） 已定项 2；D28（挂载期承诺量） 已定项 1 的结构判据「只删整理意图、不搬单元」）。
    // 带一批的照旧排除 R：搬出 R 的单元不许落回 R。撤分配器侧的活意图照旧由会话在这一版落盘之后做；发成、被拒都在返回之前放回排他。
    let deletes_only_the_intent = planned.is_empty()
        && match intent_step {
            CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords
            | CompactionIntentStep::Drop => true,
            CompactionIntentStep::CarryTheIntent
            | CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion
            | CompactionIntentStep::Issue(_) => false,
        };
    let exclusion_before_the_publish = if deletes_only_the_intent {
        allocator
            .compaction_mut()
            .open_the_region_to_the_publish_deleting_the_intent()
    } else {
        None
    };
    let published = publish_compaction_step_after_its_check(
        pool,
        allocator,
        previous,
        &relocations,
        logged_operations,
        check,
        instance,
    );
    allocator
        .compaction_mut()
        .put_back_the_region_exclusion(exclusion_before_the_publish);
    published
}

/// 计划里每一项对着上一版核过、带上搬它要的东西（[`relocation_checked_against_the_version`]）。按留存条目读源读到只坏一份的进这次挂载别的读那一份账
/// （D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：有一项核不过时前面几项读到的也进，读是真发到盘上的。
///
/// # Errors
/// [`relocation_checked_against_the_version`] 的那几种，在任何写之前。
fn relocations_checked_against_the_version<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
) -> Result<Vec<RelocationOfThePublish>, RelocationBatchRefused> {
    let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);
    let devices_counting_unit_reads_with_one_copy_failing =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
    // 计划里每一项按映射 key 查一次上一版：按 key 的索引与批量读者在第一项要查时建一次（七·6，用户 2026-10-21 定并进），
    // 不逐项在 `mapped_units` 上找、逐 key 重解叶。
    let mut previous_read_by_key = None;
    let relocations: Result<Vec<RelocationOfThePublish>, RelocationBatchRefused> = planned
        .iter()
        .map(|planned_relocation| {
            relocation_checked_against_the_version(
                &devices_counting_unit_reads_with_one_copy_failing,
                previous_read_by_key.get_or_insert_with(|| {
                    MappedUnitsOfAVersionReadByKey::of_the_version(previous)
                }),
                planned_relocation,
            )
        })
        .collect();
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        devices_counting_unit_reads_with_one_copy_failing
            .locations_of_the_units_read_with_one_copy_failing(),
    );
    relocations
}

/// 用户改动发成之后另发的那一批（`mounted_session` 的 `relocate_the_units_read_with_one_copy_failing_after_the_change`）：搬 `planned` 那几个单元（可以一个都没有）、
/// 同一次发布换下 `repair` 点名的豁免映射的单元（[`RepairOfTheExemptUnits`]，可以一个都没有），两样不能都空。同一个事务层、同一条发布路径：
/// 意图照抄（照 [`publish_relocation_batch`]，意图的签发与删不归这一批），计划核法、准入、落盘同 [`publish_compaction_step`]，实例表要换就照写行整条链重写。
/// 不新加提交步骤、不改可写挂载的分段（这一批在用户改动之后，不在挂载那一串里）。
///
/// # Errors
/// 同 [`publish_relocation_batch`]。
///
/// # Panics
/// `planned` 与 `repair` 都是空的：调用方两样都空就不发。
pub(crate) fn publish_the_relocations_and_the_repair_of_the_exempt_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    repair: &RepairOfTheExemptUnits,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    assert!(
        !(planned.is_empty() && repair.is_empty()),
        "另发的那一批要么搬单元、要么换豁免映射的单元：调用方两样都空就不发"
    );
    let relocations = relocations_checked_against_the_version(pool, allocator, previous, planned)?;
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 事务号接着这个实例（同整理的每一次发布，[`publish_compaction_step_after_its_check`]）。
    let transaction_of_this_publish = previous.highest_transaction_number_in_this_instance + 1;
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step: DeletionStepOfThePublish::NotADeletion,
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
    };
    publish_version_of_trees_relocating_units_after_a_check(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: transaction_of_this_publish,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: match &repair.instance_table_rewrite {
                Some(rewrite) => InstanceTablePlan::Rewrite(rewrite.clone()),
                None => InstanceTablePlan::Carry(previous.root.instance_table),
            },
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
        previous.tree_identifiers,
        UnmountMarker::NotWrittenByTheUnmountSequence,
        &deletion_and_lineage,
        &relocations,
        &repair.mapping_keys_routed_through_the_nodes,
        &[],
        |_version_after_this_publish| Ok::<(), RelocationBatchRefused>(()),
    )
}

/// 整理的一次发布落盘之前在算好的账上核什么（[`publish_compaction_step`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompactionStepCheckBeforePersisting {
    NothingToCheck,
    /// 删意图：R 按记录全空（停机谓词，判决 G4），照 `region_check` 判（产品路径恒照记录）；这次发布自己在 R 里的新落点算不算照 `placements_of_this_publish`。
    RegionEmptyByTheRecords {
        region: CompactionRegion,
        region_check: RegionCheckOfTheIntentDeletion,
        placements_of_this_publish: PlacementsOfThisPublishInTheRegion,
    },
    /// 照抄意图，依次核：R 还有一条未释放的记录（判决 m4-segment-code-r1 第三节 F2：这一批腾空了 R 就同一次发布删意图）；
    /// 再照 `planned_units_rewritten_anyway` 核计划里每个单元都真搬了（判决 m4-segment-code-r2 第三节 H4）。
    RegionKeepsAnUnreleasedRecord {
        region: CompactionRegion,
        planned_units_rewritten_anyway: PlannedUnitsRewrittenAnyway,
    },
}

/// [`publish_compaction_step`] 核过计划之后那一半：按 `logged_operations` 算这一版、落盘之前照 `check` 核、核过才落盘。
fn publish_compaction_step_after_its_check<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    relocations: &[RelocationOfThePublish],
    logged_operations: LoggedOperationsStepOfThePublish,
    check: CompactionStepCheckBeforePersisting,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, CompactionStepRefused> {
    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
    // 整理的每一次发布（搬一批、签发、删意图）都是一个事务，事务号接着这个实例（同建快照、删除的意图那一次：不写数据单元也取下一个号）。
    let transaction_of_this_publish = previous.highest_transaction_number_in_this_instance + 1;
    let deletion_and_lineage = DeletionAndLineageOfThePublish {
        step: DeletionStepOfThePublish::NotADeletion,
        container_tree_capacities: pool.container_tree_capacities(),
        lineage: LineageStepOfThePublish::NoLineageRecord,
        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
        logged_operations,
    };
    publish_version_of_trees_relocating_units_after_a_check(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: transaction_of_this_publish,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
        previous.tree_identifiers,
        UnmountMarker::NotWrittenByTheUnmountSequence,
        &deletion_and_lineage,
        relocations,
        &[],
        &[],
        |version_after_this_publish| {
            compaction_step_check_on_the_version_after_this_publish(
                check,
                relocations,
                version_after_this_publish,
            )
        },
    )
}

/// 删意图那一核（[`CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords`]）数不数这次发布自己在 R 里的新落点。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlacementsOfThisPublishInTheRegion {
    /// 带一批的删意图：取落点照旧排除 R，这次发布在 R 里有一条分配代是它的记录也是 R 不空（只供测试的开关关掉排他时走得到）。
    CountedAsRecordsInTheRegion,
    /// 不带一批的删意图：取落点放开了 R（[`publish_compaction_step_in_the_session`]），分配代是这次发布的记录是它落盘那一版自己的节点，
    /// 那一版里已没有意图，不算 R 不空；上一版就在 R 里的记录照算。
    NotCountedTheRegionIsOpenToThisPublish,
}

/// 整理的一次发布落盘之前那一核（[`CompactionStepCheckBeforePersisting`]）：在这次发布算好的这一版上判，过了交 `Ok`，不过交给调用方的拒绝。
/// 只读这一版与计划，不读盘：判定与随后落盘的读的是同一份算好的账。
fn compaction_step_check_on_the_version_after_this_publish(
    check: CompactionStepCheckBeforePersisting,
    relocations: &[RelocationOfThePublish],
    version_after_this_publish: &TransactionOutput,
) -> Result<(), CompactionStepRefused> {
    match check {
        CompactionStepCheckBeforePersisting::NothingToCheck => Ok(()),
        CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
            region,
            region_check,
            placements_of_this_publish,
        } => {
            let is_the_region_empty_by_the_records = match region_check {
                RegionCheckOfTheIntentDeletion::JudgedByTheRecords => {
                    match placements_of_this_publish {
                        PlacementsOfThisPublishInTheRegion::CountedAsRecordsInTheRegion => {
                            region_is_empty_by_the_records(
                                &version_after_this_publish.allocation_records,
                                region,
                            )
                        }
                        PlacementsOfThisPublishInTheRegion::NotCountedTheRegionIsOpenToThisPublish => {
                            let txg_of_this_publish = version_after_this_publish.root.checkpoint_txg;
                            let records_from_before_this_publish: Vec<AllocationRecord> =
                                version_after_this_publish
                                    .allocation_records
                                    .iter()
                                    .filter(|record| {
                                        record.is_released
                                            || record.generation != txg_of_this_publish
                                    })
                                    .copied()
                                    .collect();
                            region_is_empty_by_the_records(
                                &records_from_before_this_publish,
                                region,
                            )
                        }
                    }
                }
                RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests => {
                    false
                }
            };
            if is_the_region_empty_by_the_records {
                Ok(())
            } else {
                Err(
                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                        region,
                    }
                    .into(),
                )
            }
        }
        CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord {
            region,
            planned_units_rewritten_anyway,
        } => {
            if region_is_empty_by_the_records(
                &version_after_this_publish.allocation_records,
                region,
            ) {
                return Err(
                    RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
                        region,
                    }
                    .into(),
                );
            }
            match planned_units_rewritten_anyway {
                PlannedUnitsRewrittenAnyway::LeftToTheCompactionPublish => return Ok(()),
                PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain => {}
            }
            // 真搬了的（不是这次本来就重写的那几个，`settle_the_allocation_record_tree` 的 `relocations_in_effect`）就是这一版记下的搬迁。
            let relocated_keys: BTreeSet<&[u8]> = version_after_this_publish
                .units_relocated_by_this_publish
                .iter()
                .map(|relocated| relocated.mapping_key.as_slice())
                .collect();
            let rewritten_anyway: Vec<Vec<u8>> = relocations
                .iter()
                .filter(|relocation| !relocated_keys.contains(relocation.mapping_key.as_slice()))
                .map(|relocation| relocation.mapping_key.clone())
                .collect();
            if rewritten_anyway.is_empty() {
                Ok(())
            } else {
                Err(
                    CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {
                        mapping_keys: rewritten_anyway,
                    },
                )
            }
        }
    }
}

/// 整理发批核计划时按映射 key 查上一版的两样（[`relocation_checked_against_the_version`]）：`mapped_units` 按 key 的索引
/// （同一把 key 取最先那一项，同在表上按 key `iter().find`），与中央映射的批量读者（答案同逐 key 查 [`central_mapping_locations_in_the_version`]）。
/// 计划里的单元是这一版的角色时，它的字节按角色从批量读者手里那份单元索引取（不另建一份）；不是这一版进映射的单元时，按 key 在留存的映射条目里找，
/// 那个索引在第一次要找时才建（七·6 代码轮第三轮判决第三节第 1 条，同一把 key 取最先那一项，同在表上 `iter().find`）。
struct MappedUnitsOfAVersionReadByKey<'version> {
    version: &'version TransactionOutput,
    mapped_units_by_key:
        FirstEntryOfEachIdentity<'version, &'version [u8], (TransactionUnit, Vec<u8>)>,
    central_mapping: CentralMappingOfAVersionReadLeafByLeaf<'version>,
    kept_mapping_entries_by_key_built_at_the_first_lookup:
        Option<KeptMappingEntriesOfAVersionByKey<'version>>,
}

/// 一版留存的映射条目按映射 key 的索引（[`TransactionOutput::mapping_entries_kept_for_the_snapshots_by_key`]）。
type KeptMappingEntriesOfAVersionByKey<'version> =
    FirstEntryOfEachIdentity<'version, &'version [u8], (Vec<u8>, [LocationEntry; 2])>;

impl<'version> MappedUnitsOfAVersionReadByKey<'version> {
    pub(super) fn of_the_version(version: &'version TransactionOutput) -> Self {
        Self {
            version,
            mapped_units_by_key: version.mapped_units_by_key(),
            central_mapping: CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version),
            kept_mapping_entries_by_key_built_at_the_first_lookup: None,
        }
    }

    /// 这一版留存的映射条目里 key 是 `mapping_key` 的最先那一项（同在 `mapping_entries_kept_for_the_snapshots` 上按 key `iter().find`）。
    fn kept_mapping_entry_of_key(
        &mut self,
        mapping_key: &[u8],
    ) -> Option<&'version (Vec<u8>, [LocationEntry; 2])> {
        let version = self.version;
        self.kept_mapping_entries_by_key_built_at_the_first_lookup
            .get_or_insert_with(|| version.mapping_entries_kept_for_the_snapshots_by_key())
            .first_entry_of(&mapping_key)
    }
}

/// 计划里的一项对着上一版核过、带上搬它要的东西（[`publish_relocation_batch`]）：它是这一版进映射的单元（取它的角色与内存里的字节），
/// 或是留存的映射条目（字节从盘上按映射位置读、核位置项里的校验和，两份取先核得上的那一份）；源落点要就是映射位置、跨度合它的类。
/// 按 key 找角色、取映射条目、找留存的映射条目、取角色的字节都走 `previous_read_by_key`（它就是上一版的索引与批量读者）。
///
/// # Errors
/// [`RelocationBatchRefused`] 的四种计划错。
fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    previous_read_by_key: &mut MappedUnitsOfAVersionReadByKey<'_>,
    planned: &PlannedRelocation,
) -> Result<RelocationOfThePublish, RelocationBatchRefused> {
    let mapping_key = planned.mapping_key.clone();
    let holder_and_locations = match previous_read_by_key
        .mapped_units_by_key
        .first_entry_of(&mapping_key.as_slice())
    {
        Some((role, _)) => {
            let locations = match previous_read_by_key
                .central_mapping
                .locations_of_key(&mapping_key)
            {
                MappingLookup::Found(locations) => Some(locations),
                MappingLookup::NodeMalformedOrNoEntryWithThisKey
                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
            };
            Some((
                RelocatedUnitHolder::RoleOfThisVersion(*role),
                *role,
                locations,
            ))
        }
        None => previous_read_by_key
            .kept_mapping_entry_of_key(&mapping_key)
            .and_then(|(_, locations)| {
                user_visible_role_standing_for_a_unit_class(
                    mapping_key.first().copied().unwrap_or(0),
                )
                .map(|role| {
                    (
                        RelocatedUnitHolder::KeptForTheSnapshots,
                        role,
                        Some(*locations),
                    )
                })
            }),
    };
    let Some((holder, role_for_the_checks, mapping_locations)) = holder_and_locations else {
        return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });
    };
    match role_for_the_checks {
        TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
            return Err(RelocationBatchRefused::PinnedUnitInThePlan {
                mapping_key,
                unit: role_for_the_checks,
            })
        }
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
        | TransactionUnit::DeadlistTree
        | TransactionUnit::DeletedInodesRoot
        | TransactionUnit::LoggedOperationsRoot
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_) => {}
    }
    let Some(source_locations) = mapping_locations.filter(|locations| {
        locations
            .iter()
            .all(|location| location.slot == planned.source.slot)
            && planned.source.span == role_for_the_checks.span_slots()
    }) else {
        return Err(
            RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {
                mapping_key,
                planned_source: planned.source,
                mapping_locations,
            },
        );
    };
    let bytes = match holder {
        RelocatedUnitHolder::RoleOfThisVersion(role) => previous_read_by_key
            .central_mapping
            .unit_of_role(role)
            .bytes
            .to_vec(),
        RelocatedUnitHolder::KeptForTheSnapshots => {
            let unit_bytes = usize::try_from(planned.source.span * SLOT_BYTES)
                .expect("一个单元两槽以内，字节数装得进 usize");
            let mut some_copy_before_this_one_failed = false;
            source_locations
                .iter()
                .find_map(|location| {
                    let copy = reader
                        .read(
                            location.device,
                            location.slot.to_device_offset(),
                            unit_bytes,
                        )
                        .filter(|copy| crc32_castagnoli(copy) == location.unit_checksum);
                    match (&copy, some_copy_before_this_one_failed) {
                        (Some(_), true) => reader
                            .note_that_a_unit_was_read_with_one_copy_failing(&source_locations),
                        (Some(_), false) => {}
                        (None, _) => some_copy_before_this_one_failed = true,
                    }
                    copy
                })
                .unwrap_or_default()
        }
    };
    if bytes.is_empty() {
        return Err(RelocationBatchRefused::UnitBytesNotAvailable { mapping_key });
    }
    let Some(birth_txg) = birth_txg_of_a_mapping_key(&mapping_key) else {
        return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });
    };
    Ok(RelocationOfThePublish {
        mapping_key,
        holder,
        role_for_the_checks,
        source_locations,
        source: planned.source,
        bytes,
        birth_txg,
    })
}
