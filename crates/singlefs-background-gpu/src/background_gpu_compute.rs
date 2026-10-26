//! 产品 GPU 包对核心层 `singlefs_core::background_compute::BackgroundGpuCompute`（连同超 trait `GpuInventory`）的实现
//! （D26（后台整理与放置回收） 已定项 13 第 1、2、4、8 条）：三段各一个方法，接到本包三段已有的两份核上——甲 `checksum_kernels`、
//! 乙 `region_record_kernels`、丙 `live_slot_kernels`——两份核的结果逐条比对、同一条目连续 2 轮不确定的交 CPU 参照判
//! （`two_kernel_reconciliation`）。连续不确定的计数三段各一份，只住在这份实现里：重挂换一份实现就清零。
//! 出错照第 4 条：这一批的判定一条都不交、计数不动，映射成核心层的三种之一，带上出在哪张卡、什么原因（[`core_failure_of_the_checksum_batch`]、
//! [`core_failure_of_the_region_record_batch`]、[`core_failure_of_the_live_slot_batch`]），不换 CPU 接着算。
//! 等 GPU 超过上限（用户 2026-10-26 定「设超时，超时算出错」）映射成 `WaitForTheGpuExceededTheLimit`，分得清是这一批的那一次提交、
//! 还是前一次超时的那一次没做完。
//! 核心层交来的设置（挂载那一刻与每次切换生效的那一批开头）转给两份核那一层，从下一批起照它分活、切段、等；三段的连续不确定计数不动。
//!
//! 两份核各算一次的那一层抽成 [`BothKernelsOfTheThreeSegments`]：产品是 `background_gpu::BackgroundGpu`（参与的卡上两份 WGSL 核）；
//! 测试里换上在 CPU 上算的替身，比对、计数、映射这几样不起卡就钉得住。
//! 挂载那一刻的入口 [`open_background_gpu_compute`]：开卡（卡清单、显存上限、等 GPU 的上限，校验同核心层 `admit_background_compute_settings`），
//! 交回核心层挂载与切换收的把手，连同没开成而跳过的卡。

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed,
    BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier,
    GpuDeviceNotOpenedAtTheMount, GpuEntryDecision, GpuFailureOnTheDevice, GpuInventory,
    GpuMemoryInBytes, InputNotSupportedOnTheGpu, RegionRecordEntryKey,
    RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit, UnitChecksumOutcome,
    UnitChecksumToVerifyOnTheGpu,
};
use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};

use crate::adapter_inventory::BackgroundGpuSelection;
use crate::background_gpu::{
    open_background_gpu, BackgroundGpu, BackgroundGpuOpenRefused, ChecksumBatchFailed,
    GpuFailureCause, LiveSlotBatchFailed, RegionRecordBatchFailed,
};
use crate::checksum_kernels::{
    reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
};
use crate::gpus_of_this_host::DiscreteGpuNotOpenedAtTheMount;
use crate::live_slot_kernels::{reconciled_live_slots_of_the_segments, SEGMENT_BOUND_BYTES};
use crate::region_record_kernels::{
    reconciled_region_record_classifications, RegionRecordToClassify,
};
use crate::two_kernel_reconciliation::{ConsecutiveUncertaintyCounts, EntryDecision};

/// 三段各两份核算一次、交回每条两份的结果（不比对、不动计数）的那一层，连同本机的卡清单。产品是 [`BackgroundGpu`]；测试换上在 CPU 上算的替身。
/// 三个方法的约定同 [`BackgroundGpu`] 上对应的那三个：交回的条数与交进去的相同、按交进去的次序；出错时一条都不交。
pub trait BothKernelsOfTheThreeSegments: GpuInventory {
    /// 甲：每个单元两份核各算一次整单元 CRC-32C 与期望值比（查表那一份在前）。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的两种。
    fn unit_checksums_by_both_kernels(
        &mut self,
        units: &[UnitChecksumToVerify<'_, RegionRecordEntryKey>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed>;

    /// 乙：`region` 里每条记录两份核各分一次类（逐条走映射那一份在前）。
    ///
    /// # Errors
    /// [`RegionRecordBatchFailed`] 的四种。
    fn region_record_classes_by_both_kernels(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, RegionRecordEntryKey>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, RegionRecordBatchFailed>;

    /// 丙：`unit_area` 那块盘上每个整段两份核各数一次活槽数（逐条走遍记录那一份在前）。
    ///
    /// # Errors
    /// [`LiveSlotBatchFailed`] 的三种。
    fn live_slots_by_both_kernels(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<(u64, u64)>, LiveSlotBatchFailed>;

    /// 从下一批起照核心层交来的这一组跑：只在它点的卡上、按挂载时量出的代价分活，照它给的显存上限切段、照它给的等的上限等；不重开设备
    /// （`singlefs_core::background_compute::BackgroundGpuCompute::use_the_settings_from_the_next_batch` 转来的，只在两批之间）。
    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);
}

impl BothKernelsOfTheThreeSegments for BackgroundGpu {
    fn unit_checksums_by_both_kernels(
        &mut self,
        units: &[UnitChecksumToVerify<'_, RegionRecordEntryKey>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
        self.both_kernel_outcomes_of_the_batch(units)
    }

    fn region_record_classes_by_both_kernels(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, RegionRecordEntryKey>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, RegionRecordBatchFailed> {
        self.both_kernel_classifications_of_the_region_records(region, records, mapping_entries)
    }

    fn live_slots_by_both_kernels(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<(u64, u64)>, LiveSlotBatchFailed> {
        self.both_kernel_live_slots_of_the_cluster_segments(allocation_records, unit_area, segments)
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        BackgroundGpu::use_the_settings_from_the_next_batch(self, settings);
    }
}

/// 核心层要的那一份实现：两份核那一层，加三段各一份连续不确定的计数（第 8 条，N = 2，只在内存里）。
pub struct BackgroundGpuComputeThroughBothKernels<Kernels> {
    kernels: Kernels,
    unit_checksum_uncertainty: ConsecutiveUncertaintyCounts<RegionRecordEntryKey>,
    region_record_uncertainty: ConsecutiveUncertaintyCounts<RegionRecordEntryKey>,
    live_slot_uncertainty: ConsecutiveUncertaintyCounts<ClusterSegmentEntryKey>,
}

/// 产品那一份：参与的卡上的两份核。
pub type BackgroundGpuComputeOnTheParticipatingGpus =
    BackgroundGpuComputeThroughBothKernels<BackgroundGpu>;

impl<Kernels> BackgroundGpuComputeThroughBothKernels<Kernels> {
    /// 挂载那一刻：三段都还没有一个条目不确定过。
    #[must_use]
    pub fn with_no_entry_uncertain(kernels: Kernels) -> Self {
        Self {
            kernels,
            unit_checksum_uncertainty: ConsecutiveUncertaintyCounts::with_no_entry_uncertain(),
            region_record_uncertainty: ConsecutiveUncertaintyCounts::with_no_entry_uncertain(),
            live_slot_uncertainty: ConsecutiveUncertaintyCounts::with_no_entry_uncertain(),
        }
    }
}

/// 挂载那一刻开后台 GPU 的结局：核心层挂载与切换收的把手，与没开成而跳过的卡（给了 `background_gpu_devices` 而没点名的，按总线地址排、各带原因；
/// 没跳过一张是空）。
#[derive(Debug)]
pub struct OpenedBackgroundGpuCompute {
    pub gpu_compute: BackgroundGpuComputeHandle,
    pub gpus_not_opened_at_the_mount: Vec<DiscreteGpuNotOpenedAtTheMount>,
}

/// 挂载那一刻开后台 GPU、交回核心层挂载与切换收的把手（D26（后台整理与放置回收） 已定项 13「挂载时由调用方把它的实现递进去」）：
/// 开本机全部独显、逐张量代价，`selection` 定开卡时参与的卡（`BackgroundGpuSelection::of_the_settings` 照挂载那一组取；本包开卡时照它校验一次，
/// 同核心层 `admit_background_compute_settings`）。给了列表而没点名的卡没开成就跳过、在交回的结局里报出来，参与的卡没开成就拒。
/// 核心层挂载时照这份实现答的卡清单（本机全部独显，连同没开成的那几张）再校验一次，并把校验过的那一组交过来
/// （`use_the_settings_from_the_next_batch`），之后每次切换生效时另交，参与的卡、显存上限、等的上限都照核心层交来的那一组。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的四种：设置被拒（GPU 不在、点名的卡不存在、上限个数不对、上限超过显存总量……）、独显交不成清单、
/// 枚举适配器超过等 GPU 的上限、一张参与的卡用不上。
pub fn open_background_gpu_compute(
    selection: &BackgroundGpuSelection,
) -> Result<OpenedBackgroundGpuCompute, BackgroundGpuOpenRefused> {
    let background_gpu = open_background_gpu(selection)?;
    let gpus_not_opened_at_the_mount = background_gpu.gpus_not_opened_at_the_mount().to_vec();
    Ok(OpenedBackgroundGpuCompute {
        gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(
            BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu),
        ),
        gpus_not_opened_at_the_mount,
    })
}

/// 本包一条的判定换成核心层的：三种一一对应，结局照 `outcome_for_the_core` 换。
#[must_use]
pub fn core_decision_of_the_entry<KernelOutcome, CoreOutcome>(
    decision: EntryDecision<KernelOutcome>,
    outcome_for_the_core: impl Fn(KernelOutcome) -> CoreOutcome,
) -> GpuEntryDecision<CoreOutcome> {
    match decision {
        EntryDecision::ConfirmedByBothKernels(outcome) => {
            GpuEntryDecision::ConfirmedByBothKernels(outcome_for_the_core(outcome))
        }
        EntryDecision::UncertainThisRound => GpuEntryDecision::UncertainThisRound,
        EntryDecision::JudgedByTheCpuReference(outcome) => {
            GpuEntryDecision::JudgedByTheCpuReference(outcome_for_the_core(outcome))
        }
    }
}

/// 一张卡上的 GPU 出错报给核心层：等超过上限的两种（这一批的那一次、前一次超时的那一次没做完）是 `WaitForTheGpuExceededTheLimit`，
/// 带上限本身；别的是 `GpuFailed`，原因一一对过去、原话照带。卡照带。
#[must_use]
pub fn core_failure_of_the_gpu(
    device: &GpuDeviceIdentifier,
    cause: &GpuFailureCause,
) -> BackgroundGpuComputeFailed {
    let failed_on_the_device =
        |cause_on_the_device: GpuFailureOnTheDevice| BackgroundGpuComputeFailed::GpuFailed {
            device: device.clone(),
            cause: cause_on_the_device,
        };
    match cause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
                device: device.clone(),
                wait_limit: *wait_limit,
                submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
            }
        }
        GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit } => {
            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
                device: device.clone(),
                wait_limit: *wait_limit,
                submission_waited_for:
                    SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
            }
        }
        GpuFailureCause::DeviceOrPipelineDidNotStart { reason } => {
            failed_on_the_device(GpuFailureOnTheDevice::DeviceOrPipelineDidNotStart {
                reason: reason.clone(),
            })
        }
        GpuFailureCause::UncapturedErrorOrDeviceLost { reason } => {
            failed_on_the_device(GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                reason: reason.clone(),
            })
        }
        GpuFailureCause::WaitForTheSubmissionFailed { reason } => {
            failed_on_the_device(GpuFailureOnTheDevice::WaitForTheSubmissionFailed {
                reason: reason.clone(),
            })
        }
        GpuFailureCause::ReadbackMappingFailed { reason } => {
            failed_on_the_device(GpuFailureOnTheDevice::ReadbackMappingFailed {
                reason: reason.clone(),
            })
        }
        GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(unrecognized) => {
            failed_on_the_device(GpuFailureOnTheDevice::KernelLeftAnUnrecognizedVerdictWord {
                entry_position: unrecognized.entry_position,
                verdict_word: unrecognized.verdict_word,
            })
        }
    }
}

/// 甲段一批交不成，报给核心层：GPU 出错照 [`core_failure_of_the_gpu`]；单个单元装不进一段（第一版不支持，一张卡都没交活）是
/// `InputNotSupportedOnTheGpu` 的「一条装不进一段」，带卡、位置与单元字节数。
#[must_use]
pub fn core_failure_of_the_checksum_batch(
    failed: &ChecksumBatchFailed,
) -> BackgroundGpuComputeFailed {
    match failed {
        ChecksumBatchFailed::GpuFailed { device, cause } => core_failure_of_the_gpu(device, cause),
        ChecksumBatchFailed::UnitLargerThanAChunkOnTheGpu {
            device,
            entry_position,
            unit_bytes,
        } => BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                device: device.clone(),
                entry_position: *entry_position,
                input_bytes: *unit_bytes,
            },
        ),
    }
}

/// 乙段一批交不成，报给核心层：GPU 出错照 [`core_failure_of_the_gpu`]；映射 key 比核装得下的长、映射装不进一个绑定、单条记录装不进一段
/// （第一版不支持，一张卡都没交活）是 `InputNotSupportedOnTheGpu` 对应的那一种。
#[must_use]
pub fn core_failure_of_the_region_record_batch(
    failed: &RegionRecordBatchFailed,
) -> BackgroundGpuComputeFailed {
    match failed {
        RegionRecordBatchFailed::GpuFailed { device, cause } => {
            core_failure_of_the_gpu(device, cause)
        }
        RegionRecordBatchFailed::MappingKeyLongerThanTheKernelsHold(longer) => {
            BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold {
                    entry_position: longer.entry_position,
                    key_bytes: longer.key_bytes,
                },
            )
        }
        RegionRecordBatchFailed::MappingTablesLargerThanABindingOnTheGpu {
            device,
            largest_buffer_bytes,
        } => BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::TablesLargerThanABindingOnTheDevice {
                device: device.clone(),
                largest_buffer_bytes: *largest_buffer_bytes,
            },
        ),
        RegionRecordBatchFailed::RecordLargerThanAChunkOnTheGpu {
            device,
            entry_position,
            unit_bytes,
        } => BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                device: device.clone(),
                entry_position: *entry_position,
                input_bytes: *unit_bytes,
            },
        ),
    }
}

/// 丙段一批交不成，报给核心层：GPU 出错照 [`core_failure_of_the_gpu`]；记录表装不进一个绑定、单个段连同记录表装不进一段
/// （第一版不支持，一张卡都没交活）是 `InputNotSupportedOnTheGpu` 对应的那一种（一个段自己的输入是它的段界）。
#[must_use]
pub fn core_failure_of_the_live_slot_batch(
    failed: &LiveSlotBatchFailed,
) -> BackgroundGpuComputeFailed {
    match failed {
        LiveSlotBatchFailed::GpuFailed { device, cause } => core_failure_of_the_gpu(device, cause),
        LiveSlotBatchFailed::RecordTablesLargerThanABindingOnTheGpu {
            device,
            largest_buffer_bytes,
        } => BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::TablesLargerThanABindingOnTheDevice {
                device: device.clone(),
                largest_buffer_bytes: *largest_buffer_bytes,
            },
        ),
        LiveSlotBatchFailed::SegmentLargerThanAChunkOnTheGpu {
            device,
            entry_position,
        } => BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                device: device.clone(),
                entry_position: *entry_position,
                input_bytes: SEGMENT_BOUND_BYTES,
            },
        ),
    }
}

fn unit_checksum_outcome_for_the_core(outcome: ChecksumKernelOutcome) -> UnitChecksumOutcome {
    UnitChecksumOutcome {
        computed_crc32c: outcome.computed_crc32c,
        matches_the_expected_crc32c: outcome.matches_the_expected_crc32c,
    }
}

impl<Kernels: GpuInventory> GpuInventory for BackgroundGpuComputeThroughBothKernels<Kernels> {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.kernels.devices_with_total_memory()
    }

    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        self.kernels.devices_not_opened_at_the_mount()
    }
}

impl<Kernels: BothKernelsOfTheThreeSegments> BackgroundGpuCompute
    for BackgroundGpuComputeThroughBothKernels<Kernels>
{
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        let units: Vec<UnitChecksumToVerify<'_, RegionRecordEntryKey>> = units
            .iter()
            .map(|unit| UnitChecksumToVerify {
                entry_key: unit.entry_key,
                unit_bytes: unit.unit_bytes,
                expected_crc32c: unit.expected_crc32c,
            })
            .collect();
        let both_kernel_outcomes = self.kernels.unit_checksums_by_both_kernels(&units);
        let reconciled = reconciled_unit_checksums(
            &units,
            both_kernel_outcomes,
            &mut self.unit_checksum_uncertainty,
        )
        .map_err(|failed| core_failure_of_the_checksum_batch(&failed))?;
        Ok(reconciled
            .decisions
            .into_iter()
            .map(|decision| {
                core_decision_of_the_entry(decision, unit_checksum_outcome_for_the_core)
            })
            .collect())
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        let records: Vec<RegionRecordToClassify<'_, RegionRecordEntryKey>> = records
            .iter()
            .map(|to_classify| RegionRecordToClassify {
                entry_key: to_classify.entry_key,
                record: to_classify.record,
                unit_bytes: to_classify.unit_bytes,
            })
            .collect();
        let both_kernel_classifications =
            self.kernels
                .region_record_classes_by_both_kernels(region, &records, mapping_entries);
        let reconciled = reconciled_region_record_classifications(
            region,
            &records,
            mapping_entries,
            both_kernel_classifications,
            &mut self.region_record_uncertainty,
        )
        .map_err(|failed| core_failure_of_the_region_record_batch(&failed))?;
        Ok(reconciled
            .decisions
            .into_iter()
            .map(|decision| core_decision_of_the_entry(decision, |class| class))
            .collect())
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        let both_kernel_live_slots =
            self.kernels
                .live_slots_by_both_kernels(allocation_records, unit_area, segments);
        let reconciled = reconciled_live_slots_of_the_segments(
            allocation_records,
            unit_area,
            segments,
            both_kernel_live_slots,
            &mut self.live_slot_uncertainty,
        )
        .map_err(|failed| core_failure_of_the_live_slot_batch(&failed))?;
        Ok(reconciled
            .decisions
            .into_iter()
            .map(|decision| core_decision_of_the_entry(decision, |live_slots| live_slots))
            .collect())
    }

    /// 转给两份核那一层；三段的连续不确定计数不动（第 8 条：只在重挂、换一份实现时清零）。
    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.kernels.use_the_settings_from_the_next_batch(settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use crate::checksum_kernels::UnrecognizedVerdictWord;
    use crate::region_record_kernels::MappingKeyLongerThanTheKernelsHold;

    fn card() -> GpuDeviceIdentifier {
        GpuDeviceIdentifier::of_opaque_bytes(b"0000:01:00.0".to_vec())
    }

    /// 本包报得出的每一种 GPU 出错，与它报给核心层该是哪一个：卡照带，等超过上限的两种分得清是哪一次提交，别的原因一一对过去、原话照带。
    fn every_gpu_failure_cause_and_its_core_failure(
    ) -> Vec<(GpuFailureCause, BackgroundGpuComputeFailed)> {
        let wait_limit = Duration::from_secs(10);
        let failed_on_the_card =
            |cause: GpuFailureOnTheDevice| BackgroundGpuComputeFailed::GpuFailed {
                device: card(),
                cause,
            };
        vec![
            (
                GpuFailureCause::DeviceOrPipelineDidNotStart {
                    reason: "假的：起不了设备".to_string(),
                },
                failed_on_the_card(GpuFailureOnTheDevice::DeviceOrPipelineDidNotStart {
                    reason: "假的：起不了设备".to_string(),
                }),
            ),
            (
                GpuFailureCause::UncapturedErrorOrDeviceLost {
                    reason: "假的：设备丢了".to_string(),
                },
                failed_on_the_card(GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                    reason: "假的：设备丢了".to_string(),
                }),
            ),
            (
                GpuFailureCause::WaitForTheSubmissionFailed {
                    reason: "假的：等不成".to_string(),
                },
                failed_on_the_card(GpuFailureOnTheDevice::WaitForTheSubmissionFailed {
                    reason: "假的：等不成".to_string(),
                }),
            ),
            (
                GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit },
                BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
                    device: card(),
                    wait_limit,
                    submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                },
            ),
            (
                GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit,
                },
                BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
                    device: card(),
                    wait_limit,
                    submission_waited_for:
                        SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
                },
            ),
            (
                GpuFailureCause::ReadbackMappingFailed {
                    reason: "假的：映射不成".to_string(),
                },
                failed_on_the_card(GpuFailureOnTheDevice::ReadbackMappingFailed {
                    reason: "假的：映射不成".to_string(),
                }),
            ),
            (
                GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(UnrecognizedVerdictWord {
                    entry_position: 3,
                    verdict_word: 7,
                }),
                failed_on_the_card(GpuFailureOnTheDevice::KernelLeftAnUnrecognizedVerdictWord {
                    entry_position: 3,
                    verdict_word: 7,
                }),
            ),
        ]
    }

    /// 第 4 条「停下并报错」：三段的每一种 GPU 出错报给核心层时都带着出在哪张卡、什么原因，等超过上限的两种报成等超过上限、分得清是哪一次提交。
    #[test]
    fn every_gpu_failure_is_reported_to_the_core_with_its_card_and_cause() {
        for (cause, expected) in every_gpu_failure_cause_and_its_core_failure() {
            assert_eq!(
                core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
                    device: card(),
                    cause: cause.clone(),
                }),
                expected,
                "甲段：{cause:?}"
            );
            assert_eq!(
                core_failure_of_the_region_record_batch(&RegionRecordBatchFailed::GpuFailed {
                    device: card(),
                    cause: cause.clone(),
                }),
                expected,
                "乙段：{cause:?}"
            );
            assert_eq!(
                core_failure_of_the_live_slot_batch(&LiveSlotBatchFailed::GpuFailed {
                    device: card(),
                    cause: cause.clone(),
                }),
                expected,
                "丙段：{cause:?}"
            );
        }
    }

    /// GPU 那一侧第一版不支持的输入（一张卡都没交活）报给核心层时带着是哪一种、哪张卡、哪一条与它的字节数。
    #[test]
    fn inputs_the_first_version_does_not_support_on_the_gpu_are_reported_with_their_card_and_entry()
    {
        assert_eq!(
            core_failure_of_the_checksum_batch(
                &ChecksumBatchFailed::UnitLargerThanAChunkOnTheGpu {
                    device: card(),
                    entry_position: 3,
                    unit_bytes: 1 << 20,
                }
            ),
            BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                    device: card(),
                    entry_position: 3,
                    input_bytes: 1 << 20,
                }
            ),
            "甲段单个单元装不进一段"
        );
        for (failed, expected) in [
            (
                RegionRecordBatchFailed::MappingKeyLongerThanTheKernelsHold(
                    MappingKeyLongerThanTheKernelsHold {
                        entry_position: 0,
                        key_bytes: 29,
                    },
                ),
                InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold {
                    entry_position: 0,
                    key_bytes: 29,
                },
            ),
            (
                RegionRecordBatchFailed::MappingTablesLargerThanABindingOnTheGpu {
                    device: card(),
                    largest_buffer_bytes: 1 << 32,
                },
                InputNotSupportedOnTheGpu::TablesLargerThanABindingOnTheDevice {
                    device: card(),
                    largest_buffer_bytes: 1 << 32,
                },
            ),
            (
                RegionRecordBatchFailed::RecordLargerThanAChunkOnTheGpu {
                    device: card(),
                    entry_position: 1,
                    unit_bytes: 32 << 10,
                },
                InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                    device: card(),
                    entry_position: 1,
                    input_bytes: 32 << 10,
                },
            ),
        ] {
            assert_eq!(
                core_failure_of_the_region_record_batch(&failed),
                BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(expected),
                "乙段：{failed:?}"
            );
        }
        for (failed, expected) in [
            (
                LiveSlotBatchFailed::RecordTablesLargerThanABindingOnTheGpu {
                    device: card(),
                    largest_buffer_bytes: 1 << 32,
                },
                InputNotSupportedOnTheGpu::TablesLargerThanABindingOnTheDevice {
                    device: card(),
                    largest_buffer_bytes: 1 << 32,
                },
            ),
            (
                LiveSlotBatchFailed::SegmentLargerThanAChunkOnTheGpu {
                    device: card(),
                    entry_position: 2,
                },
                InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                    device: card(),
                    entry_position: 2,
                    input_bytes: SEGMENT_BOUND_BYTES,
                },
            ),
        ] {
            assert_eq!(
                core_failure_of_the_live_slot_batch(&failed),
                BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(expected),
                "丙段：{failed:?}"
            );
        }
    }

    #[test]
    fn each_entry_decision_maps_to_the_core_decision_of_the_same_kind() {
        assert_eq!(
            core_decision_of_the_entry(EntryDecision::ConfirmedByBothKernels(5_u32), u64::from),
            GpuEntryDecision::ConfirmedByBothKernels(5_u64)
        );
        assert_eq!(
            core_decision_of_the_entry(EntryDecision::<u32>::UncertainThisRound, u64::from),
            GpuEntryDecision::<u64>::UncertainThisRound
        );
        assert_eq!(
            core_decision_of_the_entry(EntryDecision::JudgedByTheCpuReference(9_u32), u64::from),
            GpuEntryDecision::JudgedByTheCpuReference(9_u64)
        );
    }
}
