//! 把实现的拒绝、错误与恢复结局映射成报告里的成员名，连同现行版本报的空闲字节。

use super::*;

/// 会话现行那一版记账里每块盘「空闲」那一行（[`StepObservation::free_bytes_reported_by_the_current_version`]）。
pub(super) fn free_bytes_reported_by_the_current_version(
    session: Option<&WritableSession>,
) -> Option<Vec<(DeviceIdentity, u64)>> {
    let version = session?.current.version_of_trees()?;
    Some(
        version
            .accounting_entries
            .iter()
            .filter(|entry| entry.statistic == STATISTIC_FREE_BYTES)
            .map(|entry| (entry.device, entry.value))
            .collect(),
    )
}

fn placement_refusal_member(refusal: &PlacementRefusal) -> &'static str {
    match refusal {
        PlacementRefusal::NoFreeSlotOnAnyDevice => "NoFreeSlotOnAnyDevice",
        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. } => {
            "SomeDevicesFullDeviceSetSelectionUndefined"
        }
        PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. } => {
            "UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported"
        }
        PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
            ..
        } => "CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined",
        PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice { .. } => {
            "CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice"
        }
    }
}

/// 改 size 与按偏移写被拒的成员名（统计表按成员分：溢出、点名不是活记录、读—改—写读不回旧单元各拒分得开）。
fn truncation_refusal_member(
    refusal: singlefs_core::transaction::TruncationRefusal,
) -> &'static str {
    match refusal {
        singlefs_core::transaction::TruncationRefusal::InodeIsNotALiveRecord { .. } => {
            "InodeIsNotALiveRecord"
        }
        singlefs_core::transaction::TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {
            ..
        } => "TargetSizeRoundedUpToAWholeDataUnitOverflows",
        singlefs_core::transaction::TruncationRefusal::OldDataUnitToRewriteNotReadBack { .. } => {
            "OldDataUnitToRewriteNotReadBack"
        }
        singlefs_core::transaction::TruncationRefusal::TruncationIntentDoesNotNameTheLiveRecord {
            ..
        } => "TruncationIntentDoesNotNameTheLiveRecord",
        singlefs_core::transaction::TruncationRefusal::NoTruncationIntentToContinue => {
            "NoTruncationIntentToContinue"
        }
    }
}

pub(super) fn publish_error_member(error: &PublishError) -> String {
    let member = match error {
        PublishError::PlacementRefused { refusal, .. } => {
            return format!(
                "PublishError::PlacementRefused({})",
                placement_refusal_member(refusal)
            )
        }
        PublishError::SpaceAdmissionRefused(_) => "SpaceAdmissionRefused",
        PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. } => {
            "AllocationRecordTreeRewriteSetDidNotSettle"
        }
        PublishError::MultiLevelCodeTwoTreeRefused { .. } => "MultiLevelCodeTwoTreeRefused",
        PublishError::ReleaseNotInMapping { .. } => "ReleaseNotInMapping",
        PublishError::ReleaseTargetNotAllocated { .. } => "ReleaseTargetNotAllocated",
        PublishError::ReleaseTargetAlreadyReleased { .. } => "ReleaseTargetAlreadyReleased",
        PublishError::ReleaseTargetLocationsOnDifferentSlots { .. } => {
            "ReleaseTargetLocationsOnDifferentSlots"
        }
        PublishError::ReleaseSpanMismatch { .. } => "ReleaseSpanMismatch",
        PublishError::MappingEntryNarrowerThanItsFieldTable { .. } => {
            "MappingEntryNarrowerThanItsFieldTable"
        }
        PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. } => {
            "MappingEntryLocationOnADeviceOutsideThePool"
        }
        PublishError::MappingEntryLocationsOnTheSameDevice { .. } => {
            "MappingEntryLocationsOnTheSameDevice"
        }
        PublishError::ContentExceedsDataUnit { .. } => "ContentExceedsDataUnit",
        PublishError::InodeTreeWriteRefused(_) => "InodeTreeWriteRefused",
        PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
            "FirstFileVersionDoesNotFollowTheVersionItBuildsOn"
        }
        PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. } => {
            "FirstFileVersionOnAVersionThatAlreadyHasAFile"
        }
        PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. } => {
            "TreeTableOfTheVersionToBuildOnUnreadable"
        }
        PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_) => {
            "TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees"
        }
        PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. } => {
            "PublishFrozenAfterAWriteFailureIsNotResentYet"
        }
        PublishError::JournalRecordsOfThePublishExceedTheLimit { .. } => {
            "JournalRecordsOfThePublishExceedTheLimit"
        }
        PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. } => {
            "NextCheckpointTxgPastTheTopOfItsRange"
        }
        PublishError::NextContainerNumberPastTheTopOfItsRange(_) => {
            "NextContainerNumberPastTheTopOfItsRange"
        }
        PublishError::WriteToAnInodeBeingDeleted { .. } => "WriteToAnInodeBeingDeleted",
        PublishError::WriteToAnInodeThatIsNotInTheTree { .. } => "WriteToAnInodeThatIsNotInTheTree",
        PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. } => {
            "DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion"
        }
        PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. } => {
            "UnlinkOfAnInodeThatIsNotALiveRecord"
        }
        PublishError::DeletedInodesQueueRefused(_) => "DeletedInodesQueueRefused",
        PublishError::SnapshotCreationRefused(_) => "SnapshotCreationRefused",
        PublishError::SnapshotDestructionRefused(_) => "SnapshotDestructionRefused",
        PublishError::DeadlistEntryNotReleasable { .. } => "DeadlistEntryNotReleasable",
        PublishError::EmptyClusterSegmentsBelowTheFloor(_) => "EmptyClusterSegmentsBelowTheFloor",
        PublishError::TruncationRefused(refusal) => {
            return format!(
                "PublishError::TruncationRefused({})",
                truncation_refusal_member(*refusal)
            )
        }
        PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(read) => {
            return format!(
                "PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition({})",
                block_device_error_member(&read.error)
            )
        }
        PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => {
            "ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition"
        }
        PublishError::BlockDevice(cause) => {
            return format!(
                "PublishError::BlockDevice({})",
                block_device_error_member(cause)
            )
        }
    };
    format!("PublishError::{member}")
}

fn block_device_error_member(error: &BlockDeviceError) -> &'static str {
    match error {
        BlockDeviceError::OutOfRange { .. } => "BlockDeviceError::OutOfRange",
        BlockDeviceError::Unaligned { .. } => "BlockDeviceError::Unaligned",
        BlockDeviceError::InputOutput(_) => "BlockDeviceError::InputOutput",
        BlockDeviceError::ProbeFailed { .. } => "BlockDeviceError::ProbeFailed",
        BlockDeviceError::ImageFileAlreadyExists { .. } => {
            "BlockDeviceError::ImageFileAlreadyExists"
        }
        BlockDeviceError::ImageFileMissing { .. } => "BlockDeviceError::ImageFileMissing",
    }
}

fn recovery_failure_member(failure: &RecoveryFailure) -> String {
    match failure {
        RecoveryFailure::NoValidSystemConfiguration { .. } => {
            "RecoveryFailure::NoValidSystemConfiguration".to_string()
        }
        RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized { .. } => {
            "RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized".to_string()
        }
        RecoveryFailure::SystemConfigurationsDisagree => {
            "RecoveryFailure::SystemConfigurationsDisagree".to_string()
        }
        RecoveryFailure::NoDeviceInThePool => "RecoveryFailure::NoDeviceInThePool".to_string(),
        RecoveryFailure::RootRingSlotsPerRegionOutOfRange { out_of_range, .. } => {
            format!(
                "RecoveryFailure::RootRingSlotsPerRegionOutOfRange（S = {}）",
                out_of_range.declared_slots_per_region
            )
        }
        RecoveryFailure::SystemConfigurationValueRefused { value, .. } => {
            format!("RecoveryFailure::SystemConfigurationValueRefused（{value:?}）")
        }
        RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart { .. } => {
            "RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart".to_string()
        }
        RecoveryFailure::NoValidRoot => "RecoveryFailure::NoValidRoot".to_string(),
        RecoveryFailure::UnitUnreadable { .. } => "RecoveryFailure::UnitUnreadable".to_string(),
        RecoveryFailure::UnitMalformed { what } => {
            format!("RecoveryFailure::UnitMalformed（{what}）")
        }
        RecoveryFailure::InvariantViolated { invariant, .. } => {
            format!("RecoveryFailure::InvariantViolated（{invariant}）")
        }
        RecoveryFailure::EntryWidthDiffersFromItsFieldTable { what, .. } => {
            format!("RecoveryFailure::EntryWidthDiffersFromItsFieldTable（{what}）")
        }
        RecoveryFailure::AllocationRecordOutsideThePoolGeometry { what } => {
            format!("RecoveryFailure::AllocationRecordOutsideThePoolGeometry（{what}）")
        }
        RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree => {
            "RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree".to_string()
        }
        RecoveryFailure::MappingMiss { .. } => "RecoveryFailure::MappingMiss".to_string(),
        RecoveryFailure::MappingStillUnreadable { .. } => {
            "RecoveryFailure::MappingStillUnreadable".to_string()
        }
        RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            ..
        } => "RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided"
            .to_string(),
    }
}

pub(super) fn mount_error_member(error: &MountError) -> String {
    let member = match error {
        MountError::Recovery(failure) => {
            return format!("MountError::Recovery({})", recovery_failure_member(failure))
        }
        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
            "NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread"
        }
        MountError::FileVersionWithoutAnyJournalRecord => "FileVersionWithoutAnyJournalRecord",
        MountError::InstanceTableMalformed => "InstanceTableMalformed",
        MountError::InstanceTableUnreadable => "InstanceTableUnreadable",
        MountError::Acquisition(acquisition) => {
            return format!(
                "MountError::Acquisition({})",
                block_device_error_member(&acquisition.cause)
            )
        }
        MountError::Publish(failed) => {
            return format!(
                "MountError::Publish({})",
                publish_error_member(&failed.cause)
            )
        }
        MountError::RaiseFloorSequencePublishFailed(failed) => {
            return format!(
                "MountError::RaiseFloorSequencePublishFailed(publishes_persisted = {}, {})",
                failed.writes_of_persisted_publishes.len(),
                publish_error_member(&failed.cause)
            )
        }
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
            refused_publish_in_the_sequence,
            publishes_in_the_sequence,
            cause,
        } => {
            return format!(
                "MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite(publish {refused_publish_in_the_sequence} of {publishes_in_the_sequence}, {})",
                publish_error_member(cause)
            )
        }
        MountError::RollbackFloorAboveCeiling { .. } => "RollbackFloorAboveCeiling",
        MountError::VersionWithoutFileUnitPointerOfABadImage { .. } => {
            "VersionWithoutFileUnitPointerOfABadImage"
        }
        MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { failure, .. } => {
            return format!(
                "MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable({})",
                recovery_failure_member(failure)
            )
        }
        MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot,
            first_reading,
            reread,
        } => {
            return format!(
                "MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread(区域 {} 槽 {}，{first_reading:?} 之后重读 {reread:?})",
                ring_slot.region, ring_slot.slot
            )
        }
        MountError::InstanceGenerationChangedBeforeAcquisition { .. } => {
            "InstanceGenerationChangedBeforeAcquisition"
        }
        MountError::SpaceAdmissionRefusedBeforeAcquisition { .. } => {
            "SpaceAdmissionRefusedBeforeAcquisition"
        }
        MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. } => {
            return format!(
                "MountError::RowPublishAdmissionRefusedBeforeAcquisition({})",
                publish_error_member(cause)
            )
        }
        MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            return format!(
                "MountError::WarmUpAdmissionRefusedBeforeAcquisition({})",
                publish_error_member(cause)
            )
        }
        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
            refusal, ..
        } => {
            return format!(
                "MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided({})",
                placement_refusal_member(refusal)
            )
        }
        MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. } => {
            "WritableMountRefusedByDevicesWithoutTheSelectedVersion"
        }
        MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. } => {
            "WritableDeviceCountBelowTheStripeWidthLowerBound"
        }
        MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed) => {
            return format!(
                "MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(devices_carrying_the_raised_floor = {}, {})",
                failed.devices_carrying_the_raised_floor.len(),
                block_device_error_member(&failed.cause)
            )
        }
        MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. } => {
            "RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided"
        }
        MountError::DeviceIdentitiesHandedInMoreThanOnce { .. } => {
            "DeviceIdentitiesHandedInMoreThanOnce"
        }
        MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. } => {
            "CallerParametersDisagreeWithTheSelectedSystemConfiguration"
        }
        MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. } => {
            "DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot"
        }
        MountError::SequenceNumberPastTheTopOfItsRange(_) => "SequenceNumberPastTheTopOfItsRange",
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            return format!(
                "MountError::FloorRaiseFailedAfterTheMountsPublishes({})",
                mount_error_member(&failed.cause)
            )
        }
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
            return format!(
                "MountError::NewerStateStillUnreadableAfterOneReread({})",
                still_unreadable_after_one_reread_member(still_unreadable)
            )
        }
        MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable) => {
            return format!(
                "MountError::SystemConfigurationUnreadableAfterTheReadStage({})",
                match unreadable.unreadable {
                    SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                        ..
                    } => "CheckingEachDeviceBeforeTheAcquisition",
                    SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
                        ..
                    } => "AcquisitionReadsAnUndecidableWitness",
                    SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
                        ..
                    } => "AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot",
                }
            )
        }
    };
    format!("MountError::{member}")
}

fn still_unreadable_after_one_reread_member(
    still_unreadable: &StillUnreadableAfterOneReread,
) -> &'static str {
    match still_unreadable {
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
            "PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion"
        }
        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. } => {
            "InstanceTableOfTheNewestRootForTheShadowLedger"
        }
        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. } => {
            "InstanceTableOfTheNewestRootForTheEffectiveFloor"
        }
        StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. } => {
            "RootRingSlotKnownToHoldARoot"
        }
        StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => "AccountOfAnAbandonedRoot",
        StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
            ..
        } => "AccountOfAnUnappliedPublishWithIncompleteReadableRecords",
        StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
            "PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify"
        }
    }
}

pub(super) fn rollback_error_member(error: &RollbackError) -> String {
    let member = match error {
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. } => {
            "PublishFrozenAfterAWriteFailureIsNotResentYet".to_string()
        }
        RollbackError::Recovery(failure) => {
            format!("Recovery({})", recovery_failure_member(failure))
        }
        RollbackError::CurrentInstanceTableMalformed => "CurrentInstanceTableMalformed".to_string(),
        RollbackError::TargetNotACandidate { exclusion, .. } => {
            format!("TargetNotACandidate({exclusion:?})")
        }
        RollbackError::TargetVersionUnreadable { failure, .. } => {
            format!("TargetVersionUnreadable({})", recovery_failure_member(failure))
        }
        RollbackError::CurrentAccountUnreadable { failure } => {
            format!("CurrentAccountUnreadable({})", recovery_failure_member(failure))
        }
        RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        } => "TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided".to_string(),
        RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. } => {
            "UserVisibleUnitWithoutItsRecordInTheCurrentAccount".to_string()
        }
        RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. } => {
            "UserVisibleUnitStillAllocatedUnderAnotherGeneration".to_string()
        }
        RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. } => {
            "ResurrectedUnitCopyUnreadableOrMismatched".to_string()
        }
        RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. } => {
            "IsolatedUnitCopyUnreadableOrMismatched".to_string()
        }
        RollbackError::CurrentVersionUnitNotReleasable { cause } => {
            format!("CurrentVersionUnitNotReleasable({})", publish_error_member(cause))
        }
        RollbackError::Publish(failed) => format!("Publish({})", publish_error_member(&failed.cause)),
        RollbackError::CallerInputsDisagreeWithTheDisk(_) => {
            "CallerInputsDisagreeWithTheDisk".to_string()
        }
        RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. } => {
            "NextCheckpointTxgPastTheTopOfItsRange".to_string()
        }
        RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
            version,
            ..
        } => format!("SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses({version:?})"),
        RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_unreadable) => {
            format!(
                "CandidateJudgementStillUnreadableAfterOneReread({})",
                still_unreadable_after_one_reread_member(still_unreadable)
            )
        }
    };
    format!("RollbackError::{member}")
}

pub(super) fn recovery_outcome_member(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
        RecoveryOutcome::FileRead { .. } => "FileRead".to_string(),
        RecoveryOutcome::Failed { failure, .. } => {
            format!("Failed({})", recovery_failure_member(failure))
        }
    }
}
