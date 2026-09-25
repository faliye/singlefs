//! 理想模型与实现之间那层胶水（里程碑「第二个事务」增补 3 第 2 件）：把实现交回的东西（`singlefs_core` 的类型）换成模型的观测，
//! 把实现的错误成员映射到模型的拒绝理由。D13（验证路线） 已定项 5 管的是模型本身（`model.rs` 只 `use singlefs_format`）；
//! 拿实现结局与模型比的这一层可以用 core 的类型。
//!
//! 映射只做「这个成员说的是哪条理由」，不做判断，按成员与它的判别字段映射、不看给人看的文字：成员说得出条款理由的映射过去，
//! I/O、盘坏、走读失败、第一版不支持的池形状（小盘写满、各盘落点不一致）这类模型里没有的一律 `Unexplained`（不建崩溃与设备错、
//! 两块等大盘的历史里它们都不该出现）。

use singlefs_core::address::{CheckpointTxg, DataUnitIndexInFile, InstanceGeneration};
use singlefs_core::allocator::PlacementRefusal;
use singlefs_core::block_device::BlockDeviceError;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::mount::{
    InstanceRow, MountError, Mounted, PublishSequenceFailed, RollbackCandidateExclusion,
};
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    PoolVersion, PublishError, TransactionOutput, TransactionUnit, VersionWithoutFilePublishOutput,
};

use crate::model::{
    ModelCheckpointTxg, ModelDeviceIdentity, ModelInstanceGeneration, ModelInstanceRow,
    ModelJournalCounter, ModelRefusalReason, ModelRootKey, ModelUnitRole, ObservedAllocationRecord,
    ObservedEffect, ObservedReadBack, ObservedRefusalReason, ObservedRoot,
};

#[must_use]
pub fn model_instance(instance: InstanceGeneration) -> ModelInstanceGeneration {
    ModelInstanceGeneration(u64::from(instance.0))
}

#[must_use]
pub fn model_txg(checkpoint_txg: CheckpointTxg) -> ModelCheckpointTxg {
    ModelCheckpointTxg(checkpoint_txg.0)
}

#[must_use]
pub fn model_root_key(instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> ModelRootKey {
    ModelRootKey {
        checkpoint_txg: model_txg(checkpoint_txg),
        instance: model_instance(instance),
    }
}

/// 实现的一个单元是模型的哪个角色。分配记录树根之下的节点不对应角色（`None`）：D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
/// 一次发布重写根之下哪几片由这次动了哪些槽定，模型不记落点、不记是哪几片（占槽只取上界）；那几片的分配代、位置与记录归池级 checker 判
/// （I-1.1 位置、I-3.10 每个节点都有记录）。
#[must_use]
pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
    let role = match unit {
        // 模型罩的发布（第一个文件版本 / 覆盖写、写行、暖机空发布）写的文件恒只有一个数据单元：多单元只会从
        // `publish_sequential_write` 那条路径出来，而随机历史与层 0 的固定脚本一次都不调它。
        TransactionUnit::Data(index) => {
            assert_eq!(
                index,
                DataUnitIndexInFile::FIRST,
                "模型今天只罩一个数据单元的文件"
            );
            ModelUnitRole::Data
        }
        // 模型罩的文件恒一个数据单元、只有 inode 1：extent 树上段只有根兼叶，那个单元内嵌在它的条目里（D8（核心索引结构） 已定项 14），
        // 没有下段、上段也没有根之下的节点。
        TransactionUnit::ExtentRoot => ModelUnitRole::ExtentRoot,
        TransactionUnit::ExtentLowerNode(position) => {
            panic!("模型今天只罩一个数据单元的文件（内嵌、没有下段）：{position:?}")
        }
        TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
            panic!("模型今天只罩 inode 1 一个文件（上段只有根兼叶）：{position:?}")
        }
        // 模型罩的三种发布（第一个文件版本 / 覆盖写、写行、暖机空发布）里 inode 树恒只有最左那一片叶容器：
        // 第二片只会从 `publish_new_inodes` 那条路径出来，而随机历史与层 0 的固定脚本一次都不调它
        // （`history.rs` 的操作集合里没有「建 inode」，`ModelPublishKind` 也只有那三种）。
        TransactionUnit::InodeLeafContainer(index) => {
            assert_eq!(
                index,
                InodeLeafContainerIndexInTree::LEFTMOST,
                "模型今天只罩一片叶容器的那几种发布"
            );
            ModelUnitRole::InodeLeaf
        }
        TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
        TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
        TransactionUnit::AllocationTreeNodeBelowTheRoot(_) => return None,
        // 模型罩的池两块盘、15 行记账，映射条目恒 6 条：两棵树恒只有一个节点（根兼叶），根之下的节点只在压小容量的
        // 只供测试的开关下、或记账行装不下一个节点的池（80 块盘起）上出现，随机历史与层 0 的固定脚本都不装那个开关。
        TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
            panic!("模型今天只罩记账树一个节点的池：{position:?}")
        }
        TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
        TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
            panic!("模型今天只罩中央映射树一个节点的池：{position:?}")
        }
        TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
        TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
        // 模型把实例表链当一个角色：各片的落点与分配记录按这次之后的行数现算片数（`model::instance_table_pages_for_rows`）。
        TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
            ModelUnitRole::InstanceTable
        }
    };
    Some(role)
}

#[must_use]
pub fn model_instance_row(row: &InstanceRow) -> ModelInstanceRow {
    ModelInstanceRow {
        instance: model_instance(row.instance),
        selected_root_txg: model_txg(row.selected_root_txg),
        applied_transaction_high_water: row.applied_transaction_high_water,
        is_rollback: row.is_rollback,
    }
}

/// 带文件的一版：根的身份、jsn、F，和这一版每个单元（`units` 里的角色）在这一版分配记录里的那几条（按槽号找，每块盘一条）。
#[must_use]
pub fn observed_root_of_file_version(output: &TransactionOutput) -> ObservedRoot {
    let unit_allocation_records = output
        .units
        .iter()
        .filter_map(|unit| {
            let role = model_unit_role(unit.identity)?;
            let records = output
                .allocation_records
                .iter()
                .filter(|record| record.slot == unit.slot)
                .map(|record| ObservedAllocationRecord {
                    device: ModelDeviceIdentity(record.device.0),
                    generation: model_txg(record.generation),
                    is_released: record.is_released,
                })
                .collect();
            Some((role, records))
        })
        .collect();
    ObservedRoot {
        key: model_root_key(output.root.instance, output.root.checkpoint_txg),
        journal_counter: ModelJournalCounter(output.record.counter),
        rollback_floor: model_txg(output.root.rollback_floor),
        has_file: true,
        unit_allocation_records,
    }
}

/// 树表 0 条的一版（零单元发布）：没有单元、没有分配记录。
#[must_use]
pub fn observed_root_of_version_without_file(
    output: &VersionWithoutFilePublishOutput,
) -> ObservedRoot {
    ObservedRoot {
        key: model_root_key(output.root.instance, output.root.checkpoint_txg),
        journal_counter: ModelJournalCounter(output.record.counter),
        rollback_floor: model_txg(output.root.rollback_floor),
        has_file: false,
        unit_allocation_records: Vec::new(),
    }
}

#[must_use]
pub fn observed_root_of_pool_version(version: &PoolVersion) -> ObservedRoot {
    match version {
        PoolVersion::WithFile(output) => observed_root_of_file_version(output),
        PoolVersion::WithoutFile(output) => observed_root_of_version_without_file(output),
    }
}

/// 一次挂载做成了什么：取到的号、写的行、写行与暖机那几条根。
#[must_use]
pub fn observed_mount(mounted: &Mounted) -> ObservedEffect {
    ObservedEffect::Mount {
        instance: model_instance(mounted.output.instance),
        rows_written: mounted
            .output
            .rows_written
            .iter()
            .map(model_instance_row)
            .collect(),
        roots: std::iter::once(&mounted.output.row_publish)
            .chain(mounted.output.warm_up_publishes.iter())
            .map(observed_root_of_pool_version)
            .collect(),
    }
}

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
        PublishError::ContentExceedsDataUnit { .. } => {
            explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
        }
        PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
            explained(ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn)
        }
        PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. } => {
            explained(ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile)
        }
        // 释放判定路径的五种：上一版的映射或分配记录与上一版对不上、盘上那条指针的两条位置条目不同槽，健康的历史里不该出现。
        // 分配记录树重写集合迭代不收敛那一条同理：只有强制复用窗口为 0 的只供测试的开关下才可能走到，随机历史不装那个开关。
        // inode 树写入被拒那一条同样：随机历史一次都不调 `publish_new_inodes`，
        // 而它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
        // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
        // 随机历史里两棵树恒只有一个节点；它出现就是模型与实现对不上。
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
        // 第一个文件版本读不出那一版的树表、水位离 u64::MAX 不到八个号：健康的内存盘上都不该出现。
        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
        // 冻结着一次没重发的发布：随机历史里一次发布失败就整段停下、不接着发，健康的内存盘上不该出现。
        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,
    }
}

/// 分配器拒落点的原因说的是哪条理由：只有每块盘上都没有合政策的落点（容量不够）是单元区墙。小盘写满、各盘落点不一致是第一版不支持的
/// 池形状，不是容量墙；模型的池是两块等大的盘、两盘的空闲图同样地变，模型里没有它的理由，出现就对不上（增补 3 第 2 件代码三方第一轮判决
/// 第三节第 2 条：此前 `NoSpaceFor` 装着这四种，一律映射成单元区墙，单元区墙的区间一开就被接走）。
#[must_use]
pub fn refusal_reason_of_placement_refusal(refusal: &PlacementRefusal) -> ObservedRefusalReason {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => explained(ModelRefusalReason::UnitAreaWall),
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. }
        | PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. }
        | PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        } => ObservedRefusalReason::Unexplained,
    }
}

/// 回退目标不在候选集里的那一条说的是哪条理由：按字段一对一映射，不看给人看的文字（增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
/// 「树表 0 条」不在这里：候选集只有这三条（D23（journal 的角色与格式） 已定项 14），目标那一版树表 0 条不是排除项
/// （C493（回退候选集条文与实现说反话） 还清）；环里还留着带文件版本的根时照常回退（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步）。
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
    })
}

/// 零单元发布只会报块设备错：内存盘不报错，模型里没有它的理由。
#[must_use]
pub fn refusal_reason_of_block_device_error(_error: &BlockDeviceError) -> ObservedRefusalReason {
    ObservedRefusalReason::Unexplained
}

/// 挂载、回退、抬 F 的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReason {
    match error {
        MountError::Publish(PublishSequenceFailed { cause, .. })
        | MountError::RaiseFloorSequencePublishFailed(PublishSequenceFailed { cause, .. })
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { cause, .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            refusal_reason_of_publish_error(cause)
        }
        MountError::RollbackTargetNotACandidate { exclusion, .. } => {
            refusal_reason_of_rollback_candidate_exclusion(*exclusion)
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
        // 树表 0 条、而实例表已经不是 mkfs 那一片：零故障走得到（写过行的那一版上再挂载一次），模型照代码今天的读法划进必须拒。
        MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
            explained(ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem)
        }
        // 恢复失败、记录读不出、表解不开、取号失败、坏盘上才有的根、判定与取号之间号变了、有盘不带所选那一版（空盘、停在旧状态）、
        // 交进来的盘少于 w 的下限（随机历史每次都交整池两块盘）：健康的内存盘上都不该出现。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. } => {
            ObservedRefusalReason::Unexplained
        }
    }
}

/// 抬 F 被上限拒时实现报的上限。
#[must_use]
pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckpointTxg> {
    match error {
        MountError::RollbackFloorAboveCeiling { ceiling, .. } => Some(model_txg(*ceiling)),
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RaiseFloorSequencePublishFailed(_)
        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
        | MountError::RollbackTargetNotACandidate { .. }
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. } => None,
    }
}

/// 冷启动读回的结局：所选根、读回的内容或失败的成员。
#[must_use]
pub fn observed_read_back(outcome: &RecoveryOutcome) -> ObservedReadBack {
    match outcome {
        RecoveryOutcome::NoFile { root } => ObservedReadBack::NoFile {
            root: model_root_key(root.0, root.1),
        },
        RecoveryOutcome::FileRead { root, content } => ObservedReadBack::FileRead {
            root: model_root_key(root.0, root.1),
            content: content.clone(),
        },
        RecoveryOutcome::Failed { failure, root } => ObservedReadBack::Failed {
            what: format!("{failure:?}（所选根 {root:?}）"),
        },
    }
}

/// 崩溃之后的冷启动读回（增补 3 第 3 件）：根取施加 journal 记录前缀之后**实际走的**那条根（`RecoveryReport::effective_root`），
/// 不取 `RecoveryOutcome` 里带的那条。两者的差别只在崩溃状态上看得见：`crates/singlefs-core/src/recovery.rs` 的 `recover` 里
/// `root_key` 取的是 `choose_root`（施加之前所选的根），`walk_to_file` 走的却是 `effective_root`（施加之后的根）——
/// 不建崩溃时记录都在水位之下、两者相等，崩溃状态上记录前缀一施加就不等了，读回的内容属于后者。
/// 层 0 的 oracle（`crash::oracle_violation_for_versions`）判该读出哪一版拿的也是 `effective_root`。
#[must_use]
pub fn observed_read_back_after_a_crash(report: &RecoveryReport) -> ObservedReadBack {
    let Some((instance, checkpoint_txg)) = report.effective_root else {
        return ObservedReadBack::Failed {
            what: format!("没择到根（{:?}）", report.outcome),
        };
    };
    let root = model_root_key(instance, checkpoint_txg);
    match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ObservedReadBack::NoFile { root },
        RecoveryOutcome::FileRead { content, .. } => ObservedReadBack::FileRead {
            root,
            content: content.clone(),
        },
        RecoveryOutcome::Failed { failure, .. } => ObservedReadBack::Failed {
            what: format!("{failure:?}（实际走的根 {root:?}）"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        IdealModel, ModelDeviceIdentity, ModelDisagreementAspect, ModelPoolGeometry,
        ObservedOutcome,
    };
    use singlefs_core::address::{DeviceIdentity, SlotNumber};
    use singlefs_core::allocator::CommitGeneratedDeviceAnswer;
    use singlefs_core::mount::RollbackTarget;
    use singlefs_core::transaction::TransactionUnit;
    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};

    fn every_placement_refusal() -> [PlacementRefusal; 4] {
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
                        CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(SlotNumber(196_638)),
                    ),
                ],
            },
        ]
    }

    /// 单元区墙的区间开着时（单元区只有 640 槽的小盘，第一个文件之后占槽上界 17 × 64 > 640），发布的落点被拒：每块盘上都没有（容量不够）
    /// 映射成单元区墙、模型放行；小盘写满、各盘落点不一致是第一版不支持的池形状，映射成模型没有的理由、判「模型说该成、实现拒了」
    /// （增补 3 第 2 件代码三方第一轮判决第三节第 2 条：此前 `NoSpaceFor` 装着这四种、一律映射成单元区墙，区间一开就被接走）。
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

    /// 回退候选集的三条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
    /// 「目标那一版树表 0 条」**不在**这三条里——它不是候选排除，回退到它照常做（C493（回退候选集条文与实现说反话） 还清）。
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
        ]
        .into_iter()
        .map(|exclusion| MountError::RollbackTargetNotACandidate { target, exclusion })
        .map(|error| refusal_reason_of_mount_error(&error))
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
            ]
        );
    }

    /// 模型模块只用格式常量那一个 crate（D13（验证路线） 已定项 5）：`model.rs` 里注释之外的每一行都不提 `singlefs_core`、`singlefs_checker`，
    /// `use` 只有 `std` 与 `singlefs_format`（测试模块的 `use super::*` 除外）。胶水（这个文件）可以用 core。
    #[test]
    fn the_model_module_uses_only_the_standard_library_and_the_format_constants() {
        let source = include_str!("model.rs");
        let code_lines: Vec<&str> = source
            .lines()
            .map(str::trim_start)
            .filter(|line| !line.starts_with("//"))
            .collect();
        assert!(code_lines.len() > 100, "读到的是 model.rs 本身");
        for line in &code_lines {
            assert!(
                !line.contains("singlefs_core") && !line.contains("singlefs_checker"),
                "模型模块里有一行提到了实现或 checker：{line}"
            );
            if line.starts_with("use ") {
                assert!(
                    line.starts_with("use std::")
                        || line.starts_with("use singlefs_format::")
                        || *line == "use super::*;",
                    "模型模块的 use 只许 std 与 singlefs_format：{line}"
                );
            }
        }
    }
}
