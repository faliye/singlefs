//! 产品 GPU 包对核心层 `singlefs_core::background_compute::BackgroundGpuCompute`（连同超 trait `GpuInventory`）的实现
//! （D26（后台整理与放置回收） 已定项 13 第 1、2、4、8 条）：三段各一个方法，接到本包三段已有的两份核上——甲 `checksum_kernels`、
//! 乙 `region_record_kernels`、丙 `live_slot_kernels`——两份核的结果逐条比对、同一条目连续 2 轮不确定的交 CPU 参照判
//! （`two_kernel_reconciliation`）。连续不确定的计数三段各一份，只住在这份实现里：重挂换一份实现就清零。
//! 出错照第 4 条：这一批的判定一条都不交、计数不动，映射成核心层的三种之一，带上出在哪张卡、什么原因（[`core_failure_of_the_checksum_batch`]、
//! [`core_failure_of_the_region_record_batch`]、[`core_failure_of_the_live_slot_batch`]），不换 CPU 接着算。
//! 等 GPU 超过上限（用户 2026-10-26 定「设超时，超时算出错」）在同一张卡上重交 3 次都等超过上限之后（用户 2026-10-28 定）映射成
//! `GpuUnusableAfterResubmissions`，带卡、重交了几次、等的上限，分得清最后那一次是这一批的那一次提交、还是前一次超时的那一次没做完：这是「这张卡用不了」那一声报警。
//! 参与的卡的显存上限连最小的一片都装不下、一张都不剩（用户 2026-10-28 定「连一片都装不下就不用这张卡」）映射成 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`，
//! 带每张被晾下的卡、上限与至少要多少；挂载那一刻与切换生效的那一批开头核心层就经 [`BackgroundGpuCompute::failure_under_the_settings_handed_in`] 读到，
//! 当场报警停着、不拒挂载也不拒切换。
//! 一张参与的卡连这一批最小的一片都装不下映射成 `InputNotSupportedOnTheGpu` 的「一条装不进一段」，带卡、位置与最小的一片的字节数。
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
    GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfTheBatch,
    InputNotSupportedOnTheGpu, RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu,
    SubmissionWaitedForPastTheLimit, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
};
use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};

use crate::adapter_inventory::{BackgroundGpuSelection, DiscreteGpuListRefused};
use crate::background_gpu::{
    open_background_gpu_with_the_memory_leaked_in_this_process, BackgroundGpu,
    BackgroundGpuOpenRefused, BackgroundGpuOverTheCards, BatchNotSplitOverTheGpus,
    CardOfTheBackgroundGpu, ChecksumBatchFailed, GpuFailureCause, LiveSlotBatchFailed,
    RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu,
};
use crate::checksum_kernels::{
    reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
};
use crate::gpus_of_this_host::{
    DiscreteGpuNotOpenedAtTheMount, GpuLeftOutUnderItsMemoryLimit, GpuMemoryLeakedInThisProcess,
    GpusLeakedWhenTheBackgroundGpuIsDropped,
};
use crate::live_slot_kernels::reconciled_live_slots_of_the_segments;
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
    /// [`RegionRecordBatchFailed`] 的三种。
    fn region_record_classes_by_both_kernels(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, RegionRecordEntryKey>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, RegionRecordBatchFailed>;

    /// 丙：`unit_area` 那块盘上每个整段两份核各数一次活槽数（逐条走遍记录那一份在前）。
    ///
    /// # Errors
    /// [`LiveSlotBatchFailed`] 的两种。
    fn live_slots_by_both_kernels(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<(u64, u64)>, LiveSlotBatchFailed>;

    /// 从下一批起照核心层交来的这一组跑：只在它点的卡上、按卡的顺序填满（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；
    /// 挂载时量出的代价不用来分活），照它给的显存上限切段、照它给的等的上限等；不重开设备
    /// （`singlefs_core::background_compute::BackgroundGpuCompute::use_the_settings_from_the_next_batch` 转来的，只在两批之间）。
    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);

    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：分不到交回是哪一种（换设置时量代价出错、参与的卡因显存上限一张都不剩），
    /// 分得到交回 `None`。只读，不交活。
    fn batch_not_split_over_the_gpus_under_the_settings_in_use(
        &self,
    ) -> Option<BatchNotSplitOverTheGpus>;

    /// 上一次交回以来三段各自在卡上是怎么交的（交了几片、重交了几次、显存不足重切了几次、用了哪几张卡），交回之后清零
    /// （`singlefs_core::background_compute::BackgroundGpuCompute::take_the_submissions_of_the_batch` 转来的，一批三段算完之后）。
    /// 不开卡的替身没有这一样，交回什么都没交的那一份。
    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        GpuSubmissionsOfTheBatch::default()
    }
}

impl<Card: CardOfTheBackgroundGpu> BothKernelsOfTheThreeSegments
    for BackgroundGpuOverTheCards<Card>
{
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
        BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch(self, settings);
    }

    fn batch_not_split_over_the_gpus_under_the_settings_in_use(
        &self,
    ) -> Option<BatchNotSplitOverTheGpus> {
        self.batch_not_split_over_the_gpus()
    }

    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        BackgroundGpuOverTheCards::take_the_submissions_of_the_batch(self)
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

/// 挂载那一刻开后台 GPU 的结局：核心层挂载与切换收的把手，没开成而跳过的卡（给了 `background_gpu_devices` 而没点名的，按总线地址排、各带原因；
/// 没跳过一张是空），与开卡那一组点了而因显存上限连最小的一片都装不下、不用的卡（按参与的次序，各带上限与至少要多少；用户 2026-10-28 定
/// 「连一片都装不下就不用这张卡」）。参与的卡因此一张都不剩时挂载不拒：核心层挂载那一刻把校验过的那一组交给实现、读
/// [`BackgroundGpuCompute::failure_under_the_settings_handed_in`]，读到 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit` 就当场报警停着。
/// 给了 `background_gpu_devices` 时没点名而交不成一张卡、没收进卡清单的独显各带原因（`unnamed_discrete_gpus_left_off_the_list`）；
/// 卸载（丢掉核心层收的把手）之后泄漏了哪几张卡，读 `gpus_leaked_when_dropped`。
#[derive(Debug)]
pub struct OpenedBackgroundGpuCompute {
    pub gpu_compute: BackgroundGpuComputeHandle,
    pub gpus_not_opened_at_the_mount: Vec<DiscreteGpuNotOpenedAtTheMount>,
    pub gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    pub unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
    pub gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
}

/// 挂载那一刻开后台 GPU、交回核心层挂载与切换收的把手（D26（后台整理与放置回收） 已定项 13「挂载时由调用方把它的实现递进去」）：
/// 开本机全部独显、逐张量代价，`selection` 定开卡时参与的卡（`BackgroundGpuSelection::of_the_settings` 照挂载那一组取；本包开卡时照它校验一次，
/// 同核心层 `admit_background_compute_settings`）。给了列表而没点名的卡没开成就跳过、在交回的结局里报出来，参与的卡没开成就拒。
/// 核心层挂载时照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）再校验一次，并把校验过的那一组交过来
/// （`use_the_settings_from_the_next_batch`），之后每次切换生效时另交，参与的卡、显存上限、等的上限都照核心层交来的那一组。
/// `memory_leaked_in_this_process` 是调用方在进程里留着的那一份：此前几次挂载泄漏在一张卡上的字节先从它的余量里扣掉，这一次卸载泄漏的记进去。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的四种：设置被拒（GPU 不在、点名的卡不存在、上限个数不对、上限超过显存总量……）、参与的独显交不成清单、
/// 枚举适配器超过等 GPU 的上限、一张参与的卡用不上。显存上限装不下最小的一片的卡不拒（见 [`OpenedBackgroundGpuCompute`]）。
pub fn open_background_gpu_compute(
    selection: &BackgroundGpuSelection,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
) -> Result<OpenedBackgroundGpuCompute, BackgroundGpuOpenRefused> {
    let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
        selection,
        memory_leaked_in_this_process,
    )?;
    let gpus_not_opened_at_the_mount = background_gpu.gpus_not_opened_at_the_mount().to_vec();
    let gpus_left_out_under_their_memory_limit = background_gpu
        .gpus_left_out_under_their_memory_limit()
        .to_vec();
    let unnamed_discrete_gpus_left_off_the_list = background_gpu
        .unnamed_discrete_gpus_left_off_the_list()
        .to_vec();
    let gpus_leaked_when_dropped = background_gpu.gpus_leaked_when_dropped();
    Ok(OpenedBackgroundGpuCompute {
        gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(
            BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu),
        ),
        gpus_not_opened_at_the_mount,
        gpus_left_out_under_their_memory_limit,
        unnamed_discrete_gpus_left_off_the_list,
        gpus_leaked_when_dropped,
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
/// 带上限本身；重交 3 次都等超过上限的（这张卡用不了，用户 2026-10-28 定「都超时才报警」）是 `GpuUnusableAfterResubmissions`，带重交了几次、
/// 最后那一次等的上限与是哪一次提交；别的是 `GpuFailed`，原因一一对过去、原话照带。卡照带。
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
        GpuFailureCause::OutOfMemoryCreatingTheBuffers {
            bytes_held_by_the_chunk,
            reason,
        } => failed_on_the_device(GpuFailureOnTheDevice::OutOfMemoryCreatingTheBuffers {
            bytes_held_by_the_chunk: *bytes_held_by_the_chunk,
            reason: reason.clone(),
        }),
        GpuFailureCause::UnusableAfterResubmissions {
            resubmissions,
            last_cause,
        } => match &**last_cause {
            GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
                BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                    device: device.clone(),
                    resubmissions: *resubmissions,
                    wait_limit: *wait_limit,
                    last_submission_waited_for:
                        SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                }
            }
            GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                wait_limit,
            } => BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                device: device.clone(),
                resubmissions: *resubmissions,
                wait_limit: *wait_limit,
                last_submission_waited_for:
                    SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
            },
            // 只有等超过上限的两种才重交（`crate::waiting_for_the_gpu::outcome_resubmitted_after_timeouts`），别的原因当场交回、不包进这一种；
            // 包进来了也照最后那一次的原因如实报。
            last_cause_other_than_a_wait_past_the_limit @ (GpuFailureCause::DeviceOrPipelineDidNotStart {
                ..
            }
            | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
            | GpuFailureCause::WaitForTheSubmissionFailed { .. }
            | GpuFailureCause::ReadbackMappingFailed { .. }
            | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
            | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
            | GpuFailureCause::UnusableAfterResubmissions { .. }) => {
                core_failure_of_the_gpu(device, last_cause_other_than_a_wait_past_the_limit)
            }
        },
    }
}

/// 参与的卡的显存上限连最小的一片都装不下、一张都不剩，报给核心层：`EveryParticipatingGpuLeftOutUnderItsMemoryLimit`，按参与的次序带每张卡、上限与至少要多少。
#[must_use]
pub fn core_failure_of_every_participating_gpu_left_out(
    gpus_left_out: &[GpuLeftOutUnderItsMemoryLimit],
) -> BackgroundGpuComputeFailed {
    BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
        gpus_left_out: gpus_left_out.to_vec(),
    }
}

/// 这一批参与的卡一张都不剩、其中至少一张是因空闲显存连最小的一片都建不成，报给核心层：`EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`，
/// 因空闲显存不用的按先后、连同这一段因显存上限不用的，原样带过去。
#[must_use]
pub fn core_failure_of_every_participating_gpu_short_of_free_memory(
    gpus_short_of_free_memory: &[GpuShortOfFreeMemoryForTheSmallestPiece],
    gpus_left_out_under_their_memory_limit: &[GpuLeftOutUnderItsMemoryLimit],
) -> BackgroundGpuComputeFailed {
    BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
        gpus_short_of_free_memory: gpus_short_of_free_memory.to_vec(),
        gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit.to_vec(),
    }
}

/// 此刻这组设置下一批分不到卡上，报给核心层：换设置时量代价出错照 [`core_failure_of_the_gpu`]，参与的卡一张都不剩照
/// [`core_failure_of_every_participating_gpu_left_out`]。三段交回的与挂载那一刻核心层读到的都照它报。
#[must_use]
pub fn core_failure_of_the_batch_not_split_over_the_gpus(
    not_split: &BatchNotSplitOverTheGpus,
) -> BackgroundGpuComputeFailed {
    match not_split {
        BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
            core_failure_of_the_gpu(device, cause)
        }
        BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
            core_failure_of_every_participating_gpu_left_out(gpus_left_out)
        }
    }
}

/// 一张参与的卡连这一批最小的一片都装不下报给核心层：`InputNotSupportedOnTheGpu` 的「一条装不进一段」，`input_bytes` 是最小的一片合占的字节数
/// （这张卡至少要剩这么多）。
#[must_use]
pub fn core_failure_of_the_smallest_piece_not_held(
    smallest_piece_not_held: &SmallestPieceNotHeldOnTheGpu,
) -> BackgroundGpuComputeFailed {
    BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
        InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
            device: smallest_piece_not_held.device.clone(),
            entry_position: smallest_piece_not_held.entry_position,
            input_bytes: smallest_piece_not_held.smallest_piece_bytes,
        },
    )
}

/// 甲段一批交不成，报给核心层：GPU 出错照 [`core_failure_of_the_gpu`]；连最小的一片都装不下照 [`core_failure_of_the_smallest_piece_not_held`]。
#[must_use]
pub fn core_failure_of_the_checksum_batch(
    failed: &ChecksumBatchFailed,
) -> BackgroundGpuComputeFailed {
    match failed {
        ChecksumBatchFailed::GpuFailed { device, cause } => core_failure_of_the_gpu(device, cause),
        ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held) => {
            core_failure_of_the_smallest_piece_not_held(smallest_piece_not_held)
        }
        ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out) => {
            core_failure_of_every_participating_gpu_left_out(gpus_left_out)
        }
        ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
            gpus_short_of_free_memory,
            gpus_left_out_under_their_memory_limit,
        } => core_failure_of_every_participating_gpu_short_of_free_memory(
            gpus_short_of_free_memory,
            gpus_left_out_under_their_memory_limit,
        ),
    }
}

/// 乙段一批交不成，报给核心层：GPU 出错照 [`core_failure_of_the_gpu`]；映射 key 比核装得下的长是 `InputNotSupportedOnTheGpu` 对应的那一种；
/// 连最小的一片都装不下照 [`core_failure_of_the_smallest_piece_not_held`]。
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
        RegionRecordBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held) => {
            core_failure_of_the_smallest_piece_not_held(smallest_piece_not_held)
        }
        RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out) => {
            core_failure_of_every_participating_gpu_left_out(gpus_left_out)
        }
        RegionRecordBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
            gpus_short_of_free_memory,
            gpus_left_out_under_their_memory_limit,
        } => core_failure_of_every_participating_gpu_short_of_free_memory(
            gpus_short_of_free_memory,
            gpus_left_out_under_their_memory_limit,
        ),
    }
}

/// 丙段一批交不成，报给核心层：GPU 出错照 [`core_failure_of_the_gpu`]；连最小的一片都装不下照 [`core_failure_of_the_smallest_piece_not_held`]。
#[must_use]
pub fn core_failure_of_the_live_slot_batch(
    failed: &LiveSlotBatchFailed,
) -> BackgroundGpuComputeFailed {
    match failed {
        LiveSlotBatchFailed::GpuFailed { device, cause } => core_failure_of_the_gpu(device, cause),
        LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held) => {
            core_failure_of_the_smallest_piece_not_held(smallest_piece_not_held)
        }
        LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out) => {
            core_failure_of_every_participating_gpu_left_out(gpus_left_out)
        }
        LiveSlotBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
            gpus_short_of_free_memory,
            gpus_left_out_under_their_memory_limit,
        } => core_failure_of_every_participating_gpu_short_of_free_memory(
            gpus_short_of_free_memory,
            gpus_left_out_under_their_memory_limit,
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

    /// 问两份核那一层此刻这组设置下一批分不分得到卡上，分不到照 [`core_failure_of_the_batch_not_split_over_the_gpus`] 报。
    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        self.kernels
            .batch_not_split_over_the_gpus_under_the_settings_in_use()
            .map(|not_split| core_failure_of_the_batch_not_split_over_the_gpus(&not_split))
    }

    /// 转给两份核那一层（[`BothKernelsOfTheThreeSegments::take_the_submissions_of_the_batch`]）。
    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        self.kernels.take_the_submissions_of_the_batch()
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
            (
                GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk: 4096,
                    reason: "假的：显存不足".to_string(),
                },
                failed_on_the_card(GpuFailureOnTheDevice::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk: 4096,
                    reason: "假的：显存不足".to_string(),
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

    /// 连最小的一片都装不下（一张卡都没交活）与映射 key 太长报给核心层时带着是哪一种、哪张卡、哪一条与字节数。
    #[test]
    fn inputs_the_gpu_cannot_take_are_reported_with_their_card_and_entry() {
        let smallest_piece_not_held = SmallestPieceNotHeldOnTheGpu {
            device: card(),
            entry_position: 3,
            memory_limit: Some(GpuMemoryInBytes(50)),
            smallest_piece_bytes: 104,
        };
        let reported = BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                device: card(),
                entry_position: 3,
                input_bytes: 104,
            },
        );
        assert_eq!(
            core_failure_of_the_checksum_batch(
                &ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(
                    smallest_piece_not_held.clone()
                )
            ),
            reported,
            "甲段"
        );
        assert_eq!(
            core_failure_of_the_region_record_batch(
                &RegionRecordBatchFailed::GpuCannotHoldTheSmallestPiece(
                    smallest_piece_not_held.clone()
                )
            ),
            reported,
            "乙段"
        );
        assert_eq!(
            core_failure_of_the_live_slot_batch(
                &LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held)
            ),
            reported,
            "丙段"
        );
        assert_eq!(
            core_failure_of_the_region_record_batch(
                &RegionRecordBatchFailed::MappingKeyLongerThanTheKernelsHold(
                    MappingKeyLongerThanTheKernelsHold {
                        entry_position: 0,
                        key_bytes: 29,
                    }
                )
            ),
            BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold {
                    entry_position: 0,
                    key_bytes: 29,
                }
            ),
            "乙段映射 key 太长"
        );
    }

    /// 用户 2026-10-28 定「都超时才报警」：重交 3 次都等超过上限的那一次报给核心层是「这张卡用不了」那一个成员，带卡、重交了几次、
    /// 最后那一次等的上限与是哪一次提交（这一批的那一次，或重交之前等的前一次）；三段都这样报。
    #[test]
    fn the_gpu_unusable_after_its_resubmissions_is_reported_with_its_resubmissions_and_wait_limit()
    {
        let wait_limit = Duration::from_secs(10);
        for (last_cause, last_submission_waited_for) in [
            (
                GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit },
                SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
            ),
            (
                GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit,
                },
                SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
            ),
        ] {
            let unusable = GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 3,
                last_cause: Box::new(last_cause),
            };
            let expected = BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                device: card(),
                resubmissions: 3,
                wait_limit,
                last_submission_waited_for,
            };
            assert_eq!(
                core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
                    device: card(),
                    cause: unusable.clone(),
                }),
                expected,
                "甲段"
            );
            assert_eq!(
                core_failure_of_the_region_record_batch(&RegionRecordBatchFailed::GpuFailed {
                    device: card(),
                    cause: unusable.clone(),
                }),
                expected,
                "乙段"
            );
            assert_eq!(
                core_failure_of_the_live_slot_batch(&LiveSlotBatchFailed::GpuFailed {
                    device: card(),
                    cause: unusable,
                }),
                expected,
                "丙段"
            );
        }
    }

    /// 用户 2026-10-28 定「连一片都装不下就不用这张卡」「不能因为装不下就拒绝」：参与的卡一张都不剩时，三段交回的与挂载那一刻实现答的
    /// （`failure_under_the_settings_handed_in` 照它报）都报成核心层「参与的卡一张都不剩」那一个成员，按参与的次序带每张卡、上限与至少要多少；
    /// 换设置时量代价出错照那张卡的 GPU 出错报。
    #[test]
    fn every_participating_gpu_left_out_is_reported_with_each_card_its_limit_and_the_smallest_piece(
    ) {
        let gpus_left_out = vec![
            GpuLeftOutUnderItsMemoryLimit {
                device: card(),
                memory_limit: GpuMemoryInBytes(50),
                smallest_piece_bytes: 976,
                bytes_already_held_on_the_device: 0,
            },
            GpuLeftOutUnderItsMemoryLimit {
                device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec()),
                memory_limit: GpuMemoryInBytes(900),
                smallest_piece_bytes: 976,
                bytes_already_held_on_the_device: 600,
            },
        ];
        let expected =
            BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
                gpus_left_out: gpus_left_out.clone(),
            };
        assert_eq!(
            core_failure_of_the_checksum_batch(
                &ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                    gpus_left_out.clone()
                )
            ),
            expected,
            "甲段"
        );
        assert_eq!(
            core_failure_of_the_region_record_batch(
                &RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                    gpus_left_out.clone()
                )
            ),
            expected,
            "乙段"
        );
        assert_eq!(
            core_failure_of_the_live_slot_batch(
                &LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                    gpus_left_out.clone()
                )
            ),
            expected,
            "丙段"
        );
        assert_eq!(
            core_failure_of_the_batch_not_split_over_the_gpus(
                &BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)
            ),
            expected,
            "挂载那一刻实现答的"
        );
        let cause = GpuFailureCause::DeviceOrPipelineDidNotStart {
            reason: "假的：量代价出错".to_string(),
        };
        assert_eq!(
            core_failure_of_the_batch_not_split_over_the_gpus(
                &BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                    device: card(),
                    cause: cause.clone(),
                }
            ),
            core_failure_of_the_gpu(&card(), &cause),
            "换设置时量代价出错照那张卡的 GPU 出错报"
        );
    }

    /// 用户 2026-10-28「连一片都装不下就不用这张卡」用于空闲显存：三段报的「这一批参与的卡一张都不剩、其中有因空闲显存不用的」报给核心层是
    /// `EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`，因空闲显存不用的（按先后）与因显存上限不用的原样带过去；核心层读得出点名的卡
    /// （因空闲显存不用的在前）。
    #[test]
    fn every_participating_gpu_short_of_free_memory_is_reported_with_each_card_short_and_those_left_out_under_their_limit(
    ) {
        let gpus_short_of_free_memory = vec![GpuShortOfFreeMemoryForTheSmallestPiece {
            device: card(),
            bytes_of_the_last_chunk_not_created: 64,
            smallest_piece_bytes: 64,
        }];
        let gpus_left_out_under_their_memory_limit = vec![GpuLeftOutUnderItsMemoryLimit {
            device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec()),
            memory_limit: GpuMemoryInBytes(50),
            smallest_piece_bytes: 64,
            bytes_already_held_on_the_device: 0,
        }];
        let expected =
            BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                gpus_short_of_free_memory: gpus_short_of_free_memory.clone(),
                gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit
                    .clone(),
            };
        assert_eq!(
            core_failure_of_the_checksum_batch(
                &ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                    gpus_short_of_free_memory: gpus_short_of_free_memory.clone(),
                    gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit
                        .clone(),
                }
            ),
            expected,
            "甲段"
        );
        assert_eq!(
            core_failure_of_the_region_record_batch(
                &RegionRecordBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                    gpus_short_of_free_memory: gpus_short_of_free_memory.clone(),
                    gpus_left_out_under_their_memory_limit: gpus_left_out_under_their_memory_limit
                        .clone(),
                }
            ),
            expected,
            "乙段"
        );
        assert_eq!(
            core_failure_of_the_live_slot_batch(
                &LiveSlotBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                    gpus_short_of_free_memory,
                    gpus_left_out_under_their_memory_limit,
                }
            ),
            expected,
            "丙段"
        );
        assert_eq!(
            expected.gpu_devices_named(),
            vec![
                card(),
                GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec())
            ],
            "因空闲显存不用的在前、因显存上限不用的在后"
        );
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
