//! 把实现的发布、截断、落点、回退与挂载错误换成模型的拒绝理由，连同错误里点名的根环槽、单元副本与上限。

use super::*;

fn explained(reason: ModelRefusalReason) -> ObservedRefusalReason {
    ObservedRefusalReason::Explained(reason)
}

/// 发布的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalReason {
    match error {
        PublishError::PlacementRefused { refusal, .. } => {
            refusal_reason_of_placement_refusal(refusal)
        }
        // 空间准入（D28（挂载期承诺量） 已定项 1 的式子）判这次的普通分配不够：模型的「单元区装不下」就是这一条准入（模型答允许拒绝的区间）。
        PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),
        // 合取 7（全空聚簇段少于 `N₀`，只拦前台写入类发布）：借空间准入不够那一个成员报，模型照单元区墙认。
        PublishError::EmptyClusterSegmentsBelowTheFloor(_) => {
            explained(ModelRefusalReason::UnitAreaWall)
        }
        PublishError::ContentExceedsDataUnit { .. } => {
            explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
        }
        // 第一版同时活着的只读快照至多 10 个（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1）。
        PublishError::SnapshotCreationRefused(
            SnapshotCreationRefusal::MoreSnapshotsThanTheFirstVersionAllows { .. },
        ) => explained(ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows),
        PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
            explained(ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn)
        }
        PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. } => {
            explained(ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile)
        }
        // 释放判定路径的五种：上一版的映射或分配记录与上一版对不上、盘上那条指针的两条位置条目不同槽，健康的历史里不该出现。
        // 分配记录树重写集合迭代不收敛那一条同理：只有强制复用窗口为 0 的只供测试的开关下才可能走到，随机历史不装那个开关。
        // inode 树写入被拒那一条同样：随机历史建 inode 一次一个（`HistoryOperation::CreateInode`）或一次几个、不跨叶容器
        // （`HistoryOperation::CreateInodes`），号从水位起发、恒大于树里的每个号，新记录恒追加在最右那片叶、满 233 条在末尾分裂；叶容器数到一个根装得下的 135 片之前（三万多次建 inode）不拒，
        // 它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
        // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
        // 随机历史里两棵树至多长一两层（记账树在建几百个 inode 之后、映射树在跨三片下段叶的文件上）；它出现就是模型与实现对不上。
        PublishError::MultiLevelCodeTwoTreeRefused { .. }
        | PublishError::InodeTreeWriteRefused(_)
        | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
        | PublishError::ReleaseNotInMapping { .. }
        | PublishError::ReleaseTargetNotAllocated { .. }
        | PublishError::ReleaseTargetAlreadyReleased { .. }
        | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
        | PublishError::ReleaseSpanMismatch { .. }
        | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
        // 映射条目的位置项指池外的盘、或两条指同一块盘（当映射条目损坏）：只有坏镜像、外来镜像上有；
        // 健康的内存盘上都不该出现，模型里没有它们的理由。
        | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
        | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
        // 第一个文件版本读不出那一版的树表、水位离 u64::MAX 不到十二个号（第一个文件版本一次连号发十二棵树的号）：健康的内存盘上都不该出现。
        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
        // 冻结着一次没重发的发布：随机历史里一次发布失败就整段停下、不接着发，健康的内存盘上不该出现。
        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        // 一次发布切出来的记录多于在飞上限（随机历史每次最多写几个数据单元，远在上限之下）、要接的那一版 txg 已是 u64::MAX（坏镜像才有）：
        // 模型里没有它们的理由。
        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        // 墓碑树、谱系树或退役树最右那片的容器号已到顶（C841）：容器号从 1 起逐片加一，健康的内存盘上走不到，坏镜像才有。
        | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        // 写删除中的 inode、写删完的 inode、unlink 不是活记录的号、deleted_inodes 装不下：执行器只在实现报带文件时调覆盖写与 unlink，
        // 队列不空时那两种改动先走空、再拒成带着走空那几步的 `NoFileVersionToChange`（代码轮 U10、U11），不报这几种；
        // 队列至多 inode 1 一个号，造不出这几格。
        | PublishError::WriteToAnInodeBeingDeleted { .. }
        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
        | PublishError::DeletedInodesQueueRefused(_)
        // 建快照时有快照在销毁中、水位加二越过 u64::MAX：会话先把销毁中的快照走完，水位只在坏镜像上到顶，健康的内存盘上不该出现。
        | PublishError::SnapshotCreationRefused(
            SnapshotCreationRefusal::SnapshotBeingDestroyed { .. }
            | SnapshotCreationRefusal::TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees { .. },
        )
        // 删快照点名的快照不在或正在删、deadlist 条目留存的映射条目没有或落点核不过：执行器只点名模型里活着的快照、一次删完，
        // 健康的内存盘上不该出现。
        | PublishError::SnapshotDestructionRefused(_)
        | PublishError::DeadlistEntryNotReleasable { .. }
        | PublishError::BlockDevice(_)
        // 可写挂载取号之前预演读错、读坏那两拒：健康的内存盘上不出读错、读坏，模型里没有它们的理由。
        | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
        | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => {
            ObservedRefusalReason::Unexplained
        }
        PublishError::TruncationRefused(refusal) => refusal_reason_of_truncation_refusal(*refusal),
        // 删完第一个文件之后给建的 inode 写数据（D8（核心索引结构） 已定项 16 射程 K-U5「拦」）。
        PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. } => {
            explained(ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted)
        }
    }
}

/// 改 size 与按偏移写被拒说的是哪条理由（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K12；
/// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3）。
#[must_use]
pub fn refusal_reason_of_truncation_refusal(refusal: TruncationRefusal) -> ObservedRefusalReason {
    match refusal {
        TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows { .. } => {
            explained(ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow)
        }
        // 读—改—写读不回旧单元（D4（校验和位置） 已定项 4）：模型有这条理由、健康的内存盘上从不要求它（每个单元都读得回），
        // 实现在健康的历史里拒在这一格，对拍报「模型说该成、实现拒了」，不归「说不出理由」。
        TruncationRefusal::OldDataUnitToRewriteNotReadBack { .. } => {
            explained(ModelRefusalReason::OldDataUnitToRewriteNotReadBack)
        }
        // 执行器只点名树里活着的记录（任意活 inode 都能缩短与变长，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`
        // 第二节 S6）；截断条目只经会话的走空续做，走空只在有一条对得上的条目时调：健康的内存盘上这几种都不该出现，模型里没有它们的理由。
        TruncationRefusal::InodeIsNotALiveRecord { .. }
        | TruncationRefusal::TruncationIntentDoesNotNameTheLiveRecord { .. }
        | TruncationRefusal::NoTruncationIntentToContinue => ObservedRefusalReason::Unexplained,
    }
}

/// 分配器拒落点的原因说的是哪条理由：只有每块盘上都没有合政策的落点（容量不够）是单元区墙。小盘写满、各盘落点不一致是第一版不支持的
/// 池形状，每块盘都答回落而各盘的空槽错开、池里没有一个槽每块盘都空，也不是容量墙；模型的池是两块等大的盘、两盘的空闲图同样地变，
/// 模型里没有它们的理由，出现就对不上（增补 3 第 2 件代码三方
/// 第三节第 2 条：`NoSpaceFor` 装着这四种、一律映射成单元区墙的话，单元区墙的区间一开就被接走）。
#[must_use]
pub fn refusal_reason_of_placement_refusal(refusal: &PlacementRefusal) -> ObservedRefusalReason {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => explained(ModelRefusalReason::UnitAreaWall),
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. }
        | PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. }
        | PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        }
        | PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice { .. } => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 回退目标不在候选集里的那一条说的是哪条理由：按字段一对一映射，不看给人看的文字。
/// 候选集四条（D23（journal 的角色与格式） 已定项 14：根环里、txg ≥ F_生效、按现行那一版的实例表判仍然有效、带文件；D16（发布语义） 已定项 1
/// 「回退候选集」没有「txg 落在一圈窗口里」那一条（RGall 拿掉了它），各映射到自己那一条。
#[must_use]
pub fn refusal_reason_of_rollback_candidate_exclusion(
    exclusion: RollbackCandidateExclusion,
) -> ObservedRefusalReason {
    explained(match exclusion {
        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetNotInRing,
        RollbackCandidateExclusion::BelowEffectiveFloor => {
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor
        }
        RollbackCandidateExclusion::OnAbandonedTimeline => {
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline
        }
        RollbackCandidateExclusion::VersionWithoutFile => {
            ModelRefusalReason::RollbackTargetWithoutFile
        }
    })
}

/// 管理员回退（挂着时的一次向前发布）的错误成员说的是哪条理由：候选集那四条按字段映射；回退那次发布自己的错照发布路径映射；
/// 账读不出、账对不上、单元读不出、冻结着没重发、实例表或系统配置读不出、两版树的号不同：健康的内存盘上都不该出现。
#[must_use]
pub fn refusal_reason_of_rollback_error(error: &RollbackError) -> ObservedRefusalReason {
    match error {
        RollbackError::TargetNotACandidate { exclusion, .. } => {
            refusal_reason_of_rollback_candidate_exclusion(*exclusion)
        }
        RollbackError::Publish(failed) => refusal_reason_of_publish_error(&failed.cause),
        RollbackError::CurrentVersionUnitNotReleasable { cause } => {
            refusal_reason_of_publish_error(cause)
        }
        // 第一版有快照就拒回退（判决 S8、岔路 3）：模型按它记着的活快照判同一条。
        RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { .. } => {
            explained(ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses)
        }
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | RollbackError::Recovery(_)
        // 调用方的参数或盘表与盘上不一致（随机历史每次都交整池两块盘、用建池的那一份参数）、现行那一版 txg 已是 u64::MAX（坏镜像才有）。
        | RollbackError::CallerInputsDisagreeWithTheDisk(_)
        | RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | RollbackError::CurrentInstanceTableMalformed
        | RollbackError::TargetVersionUnreadable { .. }
        | RollbackError::CurrentAccountUnreadable { .. }
        | RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        }
        | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
        | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
        // 回退目标引用的单元（复活集里的、隔离的）在一块盘上的那一份读不出或对不上：健康的内存盘上不该出现；故障注入里说谎的设备吞了落在
        // 那一份上的写才有（`research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4），那一格不在这里映射成理由，
        // 由故障注入按实现点名的那一份认（`unit_copy_unreadable_or_mismatched_of_rollback_error`，`fault_injection::left_by_the_lying_device`）。
        | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }
        // 判候选集读根环或算 F_生效 重读一次仍读坏（实审 A3b Q7）：读路径上注入故障、或同一个进程里一次根槽写被说谎的设备吞掉
        // （那一槽这个进程知道住着根、盘上却不是它写的那一份）才有，与 `MountError::Recovery(_)` 同一处置。
        | RollbackError::CandidateJudgementStillUnreadableAfterOneReread(_) => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 挂载、抬 F 的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReason {
    match error {
        MountError::Publish(failed) | MountError::RaiseFloorSequencePublishFailed(failed) => {
            refusal_reason_of_publish_error(&failed.cause)
        }
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { cause, .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            refusal_reason_of_publish_error(cause)
        }
        // 可写挂载写行与暖机之后推抬 F、抬 F 自己报错（实审 A1b Q5）：理由就是抬 F 那个错的理由。
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            refusal_reason_of_mount_error(&failed.cause)
        }
        MountError::RollbackFloorAboveCeiling { .. } => {
            explained(ModelRefusalReason::FloorAboveCeiling)
        }
        // 取号之后那一串自己的落点在取号之前就取不到：说的是分配器那一条原因（每块盘上都没有 = 单元区墙）。
        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
            refusal, ..
        } => refusal_reason_of_placement_refusal(refusal),
        // 取号之前空间准入不够（实例切换的预留拿不到）：同发布那一条，是模型的单元区墙。
        MountError::SpaceAdmissionRefusedBeforeAcquisition { .. } => {
            explained(ModelRefusalReason::UnitAreaWall)
        }
        // 读阶段判出系统配置见证过比所选那一版新的发布、重读一次仍判真（C554 乙）：崩溃恢复抛弃根那一步把最新那条根读成全 0 就走到，
        // 模型那一步答必须拒（`IdealModel::answer_mount_writable_with_the_newest_root_unreadable`）；每块盘最新那一槽系统配置读回全 0 的
        // 可写挂载照乙入判不出（C971（多处读坏时可写挂载写出倒挂的新实例根））把判据 N-配置 按真处置也走到，模型那一步同样答必须拒
        // （`IdealModel::answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots`）。
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
                    explained(
                        ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
                    )
                }
                // 重建分配器时最新那条根的实例表重读仍读不出（代码审阅第 22 条）：读阶段判完、判据为假之后才读它，崩溃恢复抛弃根
                // 那一步在读阶段就拒了走不到；只在读路径上注入故障、正好落在那一片上才有，模型没有这一条。
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                // 挂着时抬 F 算 F 生效值读最新那条根的实例表、重算影子账读根环里这个进程知道住着根的槽，重读一次仍读坏（实审 A3b Q4）：只在读路径上注入故障才有，模型没有这一条。
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
                // 没施加的发布的账读不出、读得出的记录不齐、映射也现算不成（C_MAPPING）：随机历史不藏 journal 记录，模型没有这一条。
                | StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { .. }
                // 水位之上一次缺记录、读得出的记录点名的单元又验不过的发布（F1 乙）：随机历史不藏 journal 记录，模型没有这一条。
                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
                    ObservedRefusalReason::Unexplained
                }
            }
        }
        // 恢复失败、记录读不出、表解不开、取号失败、坏盘上才有的根、判定与取号之间号变了、有盘不带所选那一版（空盘、停在旧状态）、
        // 交进来的盘少于 w 的下限（随机历史每次都交整池两块盘）、抬 F 先写系统配置那一步的块设备错、要抬到的 F 低于盘上的生效值
        // （同一个进程里上一次先写系统配置只写进一部分盘才有）、算抬 F 上限时一个知道住着根的根环槽读坏又重读仍坏（读路径上注入故障才有）：
        // 健康的内存盘上都不该出现。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        // 树表 0 条的那一版上一个单元的指针是实现写不出的样子（两条位置条目不同槽，或没有分配记录树而实例表、树表不是 mkfs 那一片）：
        // 手造的坏镜像才有，随机历史里造不出来，模型没有这一条；实现要是在随机历史里拒在这一格，对拍报「模型说该成、实现拒了」。
        | MountError::VersionWithoutFileUnitPointerOfABadImage { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        // 盘表里同一个身份交了几次、调用方参数与盘上系统配置不一致、有盘落后于现行那一版又缺它的单元或它的根：随机历史每次都交整池两块盘
        // （没换过盘）、用建池的那一份参数，走不到。
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        // 盘上的实例代号或 txg 已到顶：坏镜像才有。
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        // 重建分配器读根环有一块盘没读出有效根、整环重读一遍，两遍逐槽并起来整个环仍一条不被抛弃的自证有效根都没有（Q3 另起的拒因；
        // R1 按槽并两遍之后，第一遍读出过一条的不拒）：读阶段择得到根，只在读路径上注入故障才有，模型没有这一条。
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread
        // 读阶段判完之后（判据 N-配置 为假）逐盘核或取号那一刻系统配置又读不出（C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里、
        // 映射成模型那一条理由）：两遍读之间的瞬时读错或一块盘两槽都坏才有，模型没有这一条，不映射成「见证过比所选那一版新的发布」。
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 实现的根环槽 (区域, 槽) 换成模型的同一个槽。
#[must_use]
pub fn model_ring_position(ring_slot: RootRingSlot) -> ModelRingPosition {
    ModelRingPosition {
        region: ring_slot.region,
        slot_in_region: ring_slot.slot,
    }
}

/// 算抬 F 的上限时一个知道住着根的根环槽读坏、重读仍坏（`MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）
/// 时实现点名的那个槽；别的成员 None。
#[must_use]
pub fn root_ring_slot_still_bad_after_one_reread_of_mount_error(
    error: &MountError,
) -> Option<ModelRingPosition> {
    match error {
        MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot, ..
        } => Some(model_ring_position(*ring_slot)),
        // 可写挂载推的抬 F 报的错装在里面（实审 A1b Q5）：点名的槽照抬 F 那个错取。
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            root_ring_slot_still_bad_after_one_reread_of_mount_error(&failed.cause)
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::RollbackFloorAboveCeiling { .. }
        | MountError::VersionWithoutFileUnitPointerOfABadImage { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
    }
}

/// 挂着时回退判候选集读根环，一个知道住着根的根环槽读坏、重读仍坏
/// （`RollbackError::CandidateJudgementStillUnreadableAfterOneReread` 装着 `RootRingSlotKnownToHoldARoot`）时实现点名的那个槽；别的成员 None。
#[must_use]
pub fn root_ring_slot_still_bad_after_one_reread_of_rollback_error(
    error: &RollbackError,
) -> Option<ModelRingPosition> {
    match error {
        RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                    ring_slot, ..
                } => Some(model_ring_position(ring_slot)),
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. }
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { .. }
                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => None,
            }
        }
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | RollbackError::Recovery(_)
        | RollbackError::CallerInputsDisagreeWithTheDisk(_)
        | RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | RollbackError::CurrentInstanceTableMalformed
        | RollbackError::TargetNotACandidate { .. }
        | RollbackError::TargetVersionUnreadable { .. }
        | RollbackError::CurrentAccountUnreadable { .. }
        | RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        }
        | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
        | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
        | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }
        | RollbackError::CurrentVersionUnitNotReleasable { .. }
        | RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { .. }
        | RollbackError::Publish(_) => None,
    }
}

/// 管理员回退拒在「回退目标引用的一个单元在一块盘上的那一份读不出或整单元 CRC-32C 对不上」上时实现点名的那一份
/// （复活集里的 `ResurrectedUnitCopyUnreadableOrMismatched` 与隔离单元的 `IsolatedUnitCopyUnreadableOrMismatched` 同一形，
/// `research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4）；别的成员 None。
#[must_use]
pub fn unit_copy_unreadable_or_mismatched_of_rollback_error(
    error: &RollbackError,
) -> Option<ModelUnitCopyLocation> {
    match error {
        RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { unit, device, slot }
        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { unit, device, slot } => {
            Some(ModelUnitCopyLocation {
                device: ModelDeviceIdentity(device.0),
                span_in_slots: u32::try_from(unit.span_slots())
                    .expect("一个单元两槽以内，装得进 u32"),
                first_slot: slot.0,
            })
        }
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | RollbackError::Recovery(_)
        | RollbackError::CallerInputsDisagreeWithTheDisk(_)
        | RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | RollbackError::CurrentInstanceTableMalformed
        | RollbackError::TargetNotACandidate { .. }
        | RollbackError::TargetVersionUnreadable { .. }
        | RollbackError::CurrentAccountUnreadable { .. }
        | RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        }
        | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
        | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
        | RollbackError::CurrentVersionUnitNotReleasable { .. }
        | RollbackError::CandidateJudgementStillUnreadableAfterOneReread(_)
        | RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { .. }
        | RollbackError::Publish(_) => None,
    }
}

/// 抬 F 被上限拒时实现报的上限。
#[must_use]
pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckpointTxg> {
    match error {
        MountError::RollbackFloorAboveCeiling { ceiling, .. } => Some(model_txg(*ceiling)),
        // 可写挂载推的抬 F 报的错装在里面（实审 A1b Q5）：上限照抬 F 那个错取。
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            reported_ceiling_of_mount_error(&failed.cause)
        }
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::InstanceTableUnreadable
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::VersionWithoutFileUnitPointerOfABadImage { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_)
        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        IdealModel, ModelDeviceIdentity, ModelDisagreementAspect, ModelPoolGeometry,
        ObservedOutcome,
    };
    use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber};
    use singlefs_core::allocator::CommitGeneratedDeviceAnswer;
    use singlefs_core::mount::RollbackTarget;
    use singlefs_core::transaction::{OldDataUnitReadBackFailure, TransactionUnit};
    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};

    fn every_placement_refusal() -> [PlacementRefusal; 5] {
        [
            PlacementRefusal::NoFreeSlotOnAnyDevice,
            PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined {
                full_devices: vec![DeviceIdentity(1)],
            },
            PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                slot_per_device: vec![
                    (DeviceIdentity(0), SlotNumber(50_184)),
                    (DeviceIdentity(1), SlotNumber(50_182)),
                ],
            },
            PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
                answer_per_device: vec![
                    (
                        DeviceIdentity(0),
                        CommitGeneratedDeviceAnswer::OpenEmptySegment(SlotNumber(196_672)),
                    ),
                    (
                        DeviceIdentity(1),
                        CommitGeneratedDeviceAnswer::FallBack(SlotNumber(196_638)),
                    ),
                ],
            },
            PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice {
                fallback_slot_per_device: vec![
                    (DeviceIdentity(0), SlotNumber(56_850)),
                    (DeviceIdentity(1), SlotNumber(56_851)),
                ],
            },
        ]
    }

    /// 单元区墙的区间开着时（单元区只有 640 槽的小盘，第一个文件之后占槽上界 17 × 64 > 640），发布的落点被拒：每块盘上都没有（容量不够）
    /// 映射成单元区墙、模型放行；小盘写满、各盘落点不一致、回落取不到每块盘都空的落点，映射成模型没有的理由、判「模型说该成、实现拒了」
    /// （增补 3 第 2 件代码三方第 2 条：`NoSpaceFor` 装着这四种、一律映射成单元区墙的话，区间一开就被接走）。
    #[test]
    fn only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall() {
        let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
            devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
            device_size_in_bytes: (UNIT_AREA_START_SLOT + 640) * SLOT_BYTES,
        });
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let answer = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        assert!(
            answer.required_refusals.is_empty(),
            "条款不要求拒：{answer:?}"
        );
        for refusal in every_placement_refusal() {
            let is_capacity = refusal == PlacementRefusal::NoFreeSlotOnAnyDevice;
            let error = PublishError::PlacementRefused {
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                refusal,
            };
            let observed = ObservedOutcome::Refused {
                member: format!("{error:?}"),
                reason: refusal_reason_of_publish_error(&error),
                publishes_completed: 0,
                wrote_anything: false,
                reported_ceiling: None,
                reported_root_ring_slot_still_bad_after_one_reread: None,
            };
            let judged = model
                .clone()
                .judge_and_advance(&answer, &observed)
                .map(|_| ())
                .map_err(|disagreement| disagreement.aspect);
            if is_capacity {
                assert_eq!(judged, Ok(()), "容量不够是单元区墙：{error:?}");
            } else {
                assert_eq!(
                    judged,
                    Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess),
                    "不是容量墙：{error:?}"
                );
            }
        }
    }

    /// 可写挂载写行与暖机之后推的抬 F 报错时，挂载交回的成员装着抬 F 的错（实审 A1b Q5）：理由、报的上限与点名的根环槽都照里面那个错取。
    #[test]
    fn floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside() {
        use singlefs_core::mount::FloorRaiseFailedAfterTheMountsPublishes;
        use singlefs_core::recovery::BadRootRingSlotReading;
        let wrapped = |cause: MountError| {
            MountError::FloorRaiseFailedAfterTheMountsPublishes(Box::new(
                FloorRaiseFailedAfterTheMountsPublishes {
                    cause,
                    writes_of_persisted_publishes: Vec::new(),
                    floor_raises: Vec::new(),
                },
            ))
        };
        let above_the_ceiling = || MountError::RollbackFloorAboveCeiling {
            requested: CheckpointTxg(9),
            ceiling: CheckpointTxg(7),
        };
        assert_eq!(
            refusal_reason_of_mount_error(&wrapped(above_the_ceiling())),
            ObservedRefusalReason::Explained(ModelRefusalReason::FloorAboveCeiling)
        );
        assert_eq!(
            reported_ceiling_of_mount_error(&wrapped(above_the_ceiling())),
            Some(model_txg(CheckpointTxg(7)))
        );
        let ring_slot = RootRingSlot { region: 1, slot: 2 };
        let still_bad = MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot,
            first_reading: BadRootRingSlotReading::Unreadable,
            reread: BadRootRingSlotReading::Unreadable,
        };
        assert_eq!(
            root_ring_slot_still_bad_after_one_reread_of_mount_error(&wrapped(still_bad)),
            Some(model_ring_position(ring_slot))
        );
    }

    /// 挂着时回退判候选集读根环、一个知道住着根的槽重读仍坏：回退交回的成员点名的槽原样交给模型比对（故障注入按它认被吞的根槽写）；
    /// 同一个成员里装的另外三样、回退别的成员都不点名槽。
    #[test]
    fn rollback_refused_on_a_root_ring_slot_known_to_hold_a_root_names_that_slot_and_no_other_refusal_names_one(
    ) {
        use singlefs_core::mount::{
            NewerPublishWitness, SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread,
            WitnessedCounterComparison,
        };
        use singlefs_core::recovery::BadRootRingSlotReading;
        let candidate_judgement = |still_unreadable: StillUnreadableAfterOneReread| {
            RollbackError::CandidateJudgementStillUnreadableAfterOneReread(Box::new(
                still_unreadable,
            ))
        };
        let root_ring_slot_still_bad = candidate_judgement(
            StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                ring_slot: RootRingSlot { region: 0, slot: 6 },
                first_reading: BadRootRingSlotReading::NotSelfVerified,
                reread: BadRootRingSlotReading::NotSelfVerified,
            },
        );
        assert_eq!(
            root_ring_slot_still_bad_after_one_reread_of_rollback_error(&root_ring_slot_still_bad),
            Some(ModelRingPosition {
                region: 0,
                slot_in_region: 6
            }),
            "回退点名的槽要原样交给模型比对"
        );
        let newest_root = RollbackTarget {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(18),
        };
        let against_the_witness = SelectedVersionAgainstTheWitness {
            selected_version: newest_root,
            witness: NewerPublishWitness {
                witnessed_journal_counter: 5,
                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                    selected_version_last_record_counter: 4,
                },
            },
        };
        let refusals_naming_no_slot = [
            candidate_judgement(
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                    first_read: against_the_witness,
                    reread: against_the_witness,
                },
            ),
            candidate_judgement(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                    root_chosen_by_the_read_stage: newest_root,
                },
            ),
            candidate_judgement(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor {
                    newest_root_on_the_reread: Some(newest_root),
                },
            ),
            RollbackError::CurrentInstanceTableMalformed,
            RollbackError::TargetNotACandidate {
                target: newest_root,
                exclusion: RollbackCandidateExclusion::NotInRing,
            },
        ];
        for error in &refusals_naming_no_slot {
            assert_eq!(
                root_ring_slot_still_bad_after_one_reread_of_rollback_error(error),
                None,
                "不是「知道住着根的根环槽重读仍坏」的回退拒绝不点名槽：{error:?}"
            );
        }
    }

    /// 管理员回退拒在「回退目标引用的一个单元在一块盘上的那一份读不出或对不上」上（`research/prompts/code-backlog-b2-code-r1-main-verification.md`
    /// 「更正」一节 A4）：复活集里的与隔离单元的两个成员照同一个判据交出点名的那一份（哪块盘、从哪个槽起、几个槽）；
    /// 别的回退拒绝一份都不交，同样点名了盘与槽的「账里没有它的记录」也不交（说的不是那一份读不出）。
    #[test]
    fn both_unit_copy_refusals_of_a_rollback_name_the_copy_and_no_other_refusal_does() {
        let unit = TransactionUnit::Data(DataUnitIndexInFile::FIRST);
        let named_copy = ModelUnitCopyLocation {
            device: ModelDeviceIdentity(1),
            span_in_slots: 2,
            first_slot: 51_000,
        };
        for error in [
            RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {
                unit,
                device: DeviceIdentity(1),
                slot: SlotNumber(51_000),
            },
            RollbackError::IsolatedUnitCopyUnreadableOrMismatched {
                unit,
                device: DeviceIdentity(1),
                slot: SlotNumber(51_000),
            },
        ] {
            assert_eq!(
                unit_copy_unreadable_or_mismatched_of_rollback_error(&error),
                Some(named_copy),
                "数据单元占两槽：{error:?}"
            );
        }
        for error in [
            RollbackError::CurrentInstanceTableMalformed,
            RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
                unit,
                device: DeviceIdentity(1),
                slot: SlotNumber(51_000),
            },
        ] {
            assert_eq!(
                unit_copy_unreadable_or_mismatched_of_rollback_error(&error),
                None,
                "不是「那一份读不出或对不上」的回退拒绝不交那一份：{error:?}"
            );
        }
    }

    /// C554 乙那一拒（`MountError::NewerStateStillUnreadableAfterOneReread`）按它说的是哪一样分：系统配置见证过比所选那一版新的发布、
    /// 重读一次仍判真，映射成模型那一条理由（崩溃恢复抛弃根那一步模型答必须拒的就是它）；重建分配器时最新那条根的实例表重读仍读不出，
    /// 模型没有这一条，照 Unexplained——两样不许混成一条。
    #[test]
    fn only_the_witnessed_newer_publish_still_unreadable_after_one_reread_maps_to_the_models_reason(
    ) {
        use singlefs_core::mount::{
            NewerPublishWitness, SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread,
            WitnessedCounterComparison,
        };
        let against_the_witness = SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            witness: NewerPublishWitness {
                witnessed_journal_counter: 5,
                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                    selected_version_last_record_counter: 4,
                },
            },
        };
        let witnessed_newer_publish = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
            StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                first_read: against_the_witness,
                reread: against_the_witness,
            },
        ));
        let instance_table_of_the_newest_root =
            MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                    root_chosen_by_the_read_stage: against_the_witness.selected_version,
                },
            ));
        assert_eq!(
            (
                refusal_reason_of_mount_error(&witnessed_newer_publish),
                refusal_reason_of_mount_error(&instance_table_of_the_newest_root)
            ),
            (
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
                ),
                ObservedRefusalReason::Unexplained
            )
        );
    }

    /// C664（挂载拒绝合法复算把两种拒因报成一个）：读阶段判完之后逐盘核或取号那一刻系统配置又读不出
    /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，三处各一个）映射成 Unexplained，不映射成读阶段判据 N-配置 那一条模型理由——
    /// 两种拒因分开报之后，模型那一侧也不许再把它们混成一条。
    /// 判别力：这一成员映射成「见证过比所选那一版新的发布、重读仍判真」（`crates/mutations.tsv` 一行）时这一条红。
    #[test]
    fn system_configuration_unreadable_after_the_read_stage_does_not_map_to_the_witnessed_newer_publish(
    ) {
        use singlefs_core::mount::{
            DeviceNotRewrittenByTheAcquisition, NewerPublishWitness, RewriteGateNotMet,
            SelectedVersionAgainstTheWitness, SystemConfigurationUnreadableAfterTheReadStage,
            SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
        };
        let read_stage = SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            witness: NewerPublishWitness {
                witnessed_journal_counter: 4,
                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                    selected_version_last_record_counter: 4,
                },
            },
        };
        for unreadable in [
            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device: DeviceIdentity(1),
                not_rewritten_because:
                    DeviceNotRewrittenByTheAcquisition::SystemConfigurationSlotReportsAReadError,
            },
            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device: DeviceIdentity(1),
                not_rewritten_because: DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet(
                    RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice,
                ),
            },
            SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
                first_device_reporting_a_read_error: DeviceIdentity(0),
            },
            SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
                device: DeviceIdentity(1),
            },
        ] {
            let refusal = MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
                SystemConfigurationUnreadableAfterTheReadStage {
                    read_stage,
                    unreadable,
                },
            ));
            assert_eq!(
                refusal_reason_of_mount_error(&refusal),
                ObservedRefusalReason::Unexplained,
                "读阶段之后系统配置又读不出（{unreadable:?}）模型没有这一条理由，不并进判据 N-配置 那一条"
            );
        }
    }

    /// 回退候选集的四条排除各映射到自己那一条理由、互不相同；
    /// 「目标那一版树表 0 条」是第四条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。原来的「落在一圈窗口外」那一条
    /// RGall 拿掉（D16（发布语义） 已定项 1「回退候选集」）。
    #[test]
    fn each_rollback_candidate_exclusion_maps_to_its_own_reason() {
        let target = RollbackTarget {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(7),
        };
        let mapped: Vec<ObservedRefusalReason> = [
            RollbackCandidateExclusion::NotInRing,
            RollbackCandidateExclusion::BelowEffectiveFloor,
            RollbackCandidateExclusion::OnAbandonedTimeline,
            RollbackCandidateExclusion::VersionWithoutFile,
        ]
        .into_iter()
        .map(|exclusion| RollbackError::TargetNotACandidate { target, exclusion })
        .map(|error| refusal_reason_of_rollback_error(&error))
        .collect();
        assert_eq!(
            mapped,
            vec![
                ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetNotInRing),
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::RollbackTargetBelowEffectiveFloor
                ),
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::RollbackTargetOnAbandonedTimeline
                ),
                ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetWithoutFile),
            ]
        );
    }

    /// 改 size 被拒（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K12）：N 按整单元取整溢出映射到模型自己那一条理由；
    /// 别的几种（健康的内存盘上走不到）是说不清的理由。第一个文件之外的 inode 改 size 不再有拒（判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S6 删了那个成员）。
    /// 读—改—写读不回要重写的旧单元（部分写与截断、变长重写尾单元同一个成员，判决 `research/prompts/m4-partial-write-r1-main-verification.md`
    /// 第二节 R3）：两样读回失败都映射到模型「读不回旧单元」那一条理由，不归「说不出理由」。判别力：映射改回「说不出理由」时这一条红
    /// （`crates/mutations.tsv` 一行）。
    #[test]
    fn an_old_data_unit_not_read_back_maps_to_the_models_reason_for_it_and_not_to_an_unexplained_one(
    ) {
        let mapped: Vec<ObservedRefusalReason> = [
            OldDataUnitReadBackFailure::UnreadableAtTheHintsAndAtTheMappedLocations,
            OldDataUnitReadBackFailure::UnreadableAtTheHintsAndMissingFromTheMapping,
            OldDataUnitReadBackFailure::DisagreesWithTheLookupPath {
                detail: "单元头解不开",
            },
        ]
        .into_iter()
        .map(|read_back| {
            refusal_reason_of_publish_error(&PublishError::TruncationRefused(
                TruncationRefusal::OldDataUnitToRewriteNotReadBack {
                    inode: InodeNumber(7),
                    unit: DataUnitIndexInFile(2),
                    read_back,
                },
            ))
        })
        .collect();
        assert_eq!(
            mapped,
            vec![
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::OldDataUnitToRewriteNotReadBack
                );
                3
            ],
            "三种读回失败都是「读不回旧单元」那一条理由"
        );
    }

    #[test]
    fn each_truncation_refusal_the_model_has_a_reason_for_maps_to_its_own_reason() {
        let mapped: Vec<ObservedRefusalReason> = [
            TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {
                target_size_in_bytes: u64::MAX,
            },
            TruncationRefusal::InodeIsNotALiveRecord { inode: 1 },
            TruncationRefusal::NoTruncationIntentToContinue,
        ]
        .into_iter()
        .map(|refusal| refusal_reason_of_publish_error(&PublishError::TruncationRefused(refusal)))
        .collect();
        assert_eq!(
            mapped,
            vec![
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow
                ),
                ObservedRefusalReason::Unexplained,
                ObservedRefusalReason::Unexplained,
            ]
        );
    }
}
