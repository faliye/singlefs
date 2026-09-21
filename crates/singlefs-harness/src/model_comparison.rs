//! 理想模型与实现之间那层胶水（里程碑「第二个事务」增补 3 第 2 件）：把实现交回的东西（`singlefs_core` 的类型）换成模型的观测，
//! 把实现的错误成员映射到模型的拒绝理由。D13（验证路线） 已定项 5 管的是模型本身（`model.rs` 只 `use singlefs_format`）；
//! 拿实现结局与模型比的这一层可以用 core 的类型。
//!
//! 映射只做「这个成员说的是哪条理由」，不做判断，按成员与它的判别字段映射、不看给人看的文字：成员说得出条款理由的映射过去，
//! I/O、盘坏、走读失败、第一版不支持的池形状（小盘写满、各盘落点不一致）这类模型里没有的一律 `Unexplained`（不建崩溃与设备错、
//! 两块等大盘的历史里它们都不该出现）。

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::allocator::PlacementRefusal;
use singlefs_core::block_device::BlockDeviceError;
use singlefs_core::mount::{InstanceRow, MountError, Mounted, RollbackCandidateExclusion};
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    PoolVersion, PublishError, TransactionOutput, TransactionUnit, ZeroUnitPublishOutput,
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

#[must_use]
pub fn model_unit_role(unit: TransactionUnit) -> ModelUnitRole {
    match unit {
        TransactionUnit::Data => ModelUnitRole::Data,
        TransactionUnit::ExtentRoot => ModelUnitRole::ExtentRoot,
        TransactionUnit::InodeLeaf => ModelUnitRole::InodeLeaf,
        TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
        TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
        TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
        TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
        TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
        TransactionUnit::InstanceTable => ModelUnitRole::InstanceTable,
    }
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
        .map(|unit| {
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
            (model_unit_role(unit.identity), records)
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
pub fn observed_root_of_version_without_file(output: &ZeroUnitPublishOutput) -> ObservedRoot {
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
        PublishError::AllocationRecordsExceedOneNode { .. } => {
            explained(ModelRefusalReason::AllocationRecordNodeWall)
        }
        PublishError::AccountingEntriesExceedOneNode { .. } => {
            explained(ModelRefusalReason::AccountingNodeWall)
        }
        PublishError::ContentExceedsDataUnit { .. } => {
            explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
        }
        PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp { .. } => {
            explained(ModelRefusalReason::FirstFileNotRightAfterTheWarmUp)
        }
        // 释放判定路径的四种：上一版的映射或分配记录与上一版对不上，健康的历史里不该出现。
        PublishError::ReleaseNotInMapping { .. }
        | PublishError::ReleaseTargetNotAllocated { .. }
        | PublishError::ReleaseTargetAlreadyReleased { .. }
        | PublishError::ReleaseSpanMismatch { .. }
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
/// 「树表 0 条」不在这里：它在候选集里、第一版不支持，是 `MountError` 单独一个成员（第二轮判决第三节第 1 条）。
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
        MountError::Publish(cause)
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            refusal_reason_of_publish_error(cause)
        }
        MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion => {
            explained(ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported)
        }
        MountError::RollbackTargetNotACandidate { exclusion, .. } => {
            refusal_reason_of_rollback_candidate_exclusion(*exclusion)
        }
        // 在候选集里、第一版不支持：映射到模型「第一版不支持」那一类里回退那一条（模型照代码今天的读法划进必须拒）。
        MountError::RollbackToVersionWithoutFileUnsupported(_) => {
            explained(ModelRefusalReason::RollbackToVersionWithoutFileUnsupported)
        }
        MountError::RollbackFloorAboveCeiling { .. } => {
            explained(ModelRefusalReason::FloorAboveCeiling)
        }
        MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. } => {
            explained(ModelRefusalReason::RowsOnVersionWithoutFileUnsupported)
        }
        MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported { .. } => {
            explained(ModelRefusalReason::InstanceTableOnePageWall)
        }
        MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction { .. } => {
            explained(ModelRefusalReason::FormattedPoolMountNotShapedLikeTheFirstTransaction)
        }
        // 恢复失败、记录读不出、表解不开、取号失败、坏盘上才有的根、判定与取号之间号变了：健康的内存盘上都不该出现。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. } => {
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
        | MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion
        | MountError::Acquisition(_)
        | MountError::Publish(_)
        | MountError::RollbackTargetNotACandidate { .. }
        | MountError::RollbackToVersionWithoutFileUnsupported(_)
        | MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. }
        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
        | MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported { .. }
        | MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction { .. } => None,
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

    /// 单元区墙的区间开着时（单元区只有 640 槽的小盘，第一个文件之后占槽上界 13 × 64 > 640），发布的落点被拒：每块盘上都没有（容量不够）
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
                unit: TransactionUnit::Data,
                refusal,
            };
            let observed = ObservedOutcome::Refused {
                member: format!("{error:?}"),
                reason: refusal_reason_of_publish_error(&error),
                publishes_completed: 0,
                wrote_anything: false,
                reported_ceiling: None,
                allocation_records_counted_on_the_image: None,
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

    /// 回退目标被挡下的四种各映射到自己那一条理由、互不相同：候选排除的三条按字段映射（此前「不在候选集里」映射成「低于 F、被抛弃」
    /// 两条之一），树表 0 条是 `MountError` 单独的成员、映射到「第一版不支持」那一类（第二轮判决第三节第 1 条）。
    #[test]
    fn each_rollback_candidate_exclusion_and_the_version_without_file_map_to_their_own_reasons() {
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
        .chain(std::iter::once(
            MountError::RollbackToVersionWithoutFileUnsupported(target),
        ))
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
                ObservedRefusalReason::Explained(
                    ModelRefusalReason::RollbackToVersionWithoutFileUnsupported
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
