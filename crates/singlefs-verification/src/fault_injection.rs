//! 故障注入（里程碑「覆盖写、释放、回退与复用」增补 3 第 4 件）：一个通用的设备包装，按种子让任意一次写、读、刷盘返回 `BlockDeviceError`，
//! 或让一次读返回改坏的字节。被测的性质有两条：注入之后 `singlefs-core` 那一侧**返回错误而不是 panic**，
//! 出错之后**重开要恢复到模型允许的版本**。
//!
//! [`FaultInjectingBlockDevice`] 本身不定包在录制器的哪一边，录制流记的是什么由搭栈的一方定。两条要紧的栈次序相反：
//! * 这一件（历史里的盘，`crate::history::HistoryDevice`）：注入在录制器**外面**（调用方与录制器之间）。报错的写、被吞掉的写与
//!   被吞掉的屏障都不进录制流，录制流因此恒等于真正落到内存盘上的那一串——注入之后的镜像可以从录制流重建
//!   （`crate::memory_pool::MemoryPool::apply`），不必把执行器里的两块盘再交出来一份。
//! * 门禁 qemu-device-streams 的装置（`src/bin/new_pool_file_creation_on_device.rs` 的 `CountedDevice`）：录制器在**外面**、注入在里面。录制流记的是
//!   程序发给这块盘的每一次调用，被吞掉的屏障也在里面；真设备收到了什么另看这一层的计数（[`FaultDeviceCounts`] 的
//!   `barriers_forwarded` / `barriers_swallowed`）——那条栈上录制流不等于落盘的那一串，不能拿它按上一条那样重建镜像。
//!
//! 一段历史上怎么注入：先跑一遍不注入的（测量跑），记下每一步跑完时读 / 写 / 刷盘各调了多少次、模型到那一步提交过哪些版本；
//! 再按种子在起点之后的调用里挑几个注入点，每个注入点重跑一遍同一段历史（同一个种子逐位复现同一串数，重跑到注入那一刻为止逐字节相同）。
//! 重跑之后判三样：
//! 1. 整段历史里一个 panic 都没有（`singlefs-core` 的断言被盘上内容或设备错走到了，就是缺口，增补 3 第 6 件那一类）；
//! 2. 注入那一步的结局是「入口返回 Err」或「冷启动读回报错」这两种形态之一，不是别的；
//! 3. 重开（拿录制流重建镜像、跑 `recovery::recover`）走到的那一版在模型允许的集合里，读回的内容与模型记的逐字节相同
//!    （`crate::model::crash_recovery_disagreement`，与崩溃注入同一条判据、同一份实现）。
//!
//! 允许的集合 = 注入那一步之前模型提交过的每一版 ∪ 不注入时那一步会提交的那几版。后一半是刻意放行的：
//! 「发布在最后一步失败、根已 FUA 落盘，重开之后那一版看得见」正是增补 2 收口表第 40 行（C381）在争的那一格，
//! 三方三轮判完、改法未定，这里不替它定，只把落在那一格的次数记成一笔（`FaultInjectionTally::reopened_into_the_version_the_faulted_step_was_writing`）。
//! 第 40 行那一格的判别力由写死的那条用例钉（`tests/fault_injection_campaign.rs`）。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::{
    ViolationSite, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MKFS_INSTANCE_GENERATION;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, recover, JournalPolicy, PoolReader,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_format::{ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION_MAXIMUM, SLOT_BYTES};

use crate::crash_injection::seed_slices;
use crate::heavy_threads::{
    default_heavy_threads_of_the_cap_value, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use crate::history::{
    classify_failure, execute_history_stopping_at_the_given_pool_checker_violations,
    execute_history_with_faults, generate_history_with_weights, newest_ring_root_and_slot_count,
    raised_floor_lands_only_on_abandoned_roots, AppliedEffect, ColdStartReadBack,
    CopyQuarantinedByTheImplementation, FailureObservation, FailureSignature, GeneratedHistory,
    GenerationWeights, HarnessJudgement, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistoryRun, HistorySeed, PerStepChecker,
    SeededRandomSource, StartingPointStep, StepOutcome, StepPosition,
};
use crate::memory_pool::{
    newest_persisted_root, root_identity_written_by, writes_and_segments_with_stream_indexes,
    MemoryPool, RecordCheck,
};
use crate::model::{
    crash_recovery_disagreement, ModelDeviceIdentity, ModelDisagreementAspect, ModelInstanceRow,
    ModelReportedUnreadablePlace, ModelRingPosition, ModelRollbackFloorCeilingsWithoutTheRoots,
    ModelRootKey, ModelUnitCopyLocation, ObservedReadBack, ObservedSnapshotReadBack,
};
use crate::model_comparison::{
    model_ring_position, model_root_key, observed_read_back_after_a_crash,
};
use crate::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use crate::reference_filesystem_translation::{
    JointVerdictOfTheReferenceFileSystemAndTheModel, ReferenceFileSystemTranslation,
    ReferenceFileSystemVote, WhatTheReopenMustRecover,
};
use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordedOperation, RecordedOperationKind, SharedStream};

mod fault_injection_campaign;
mod findings_and_tally;
mod injection_into_a_history;
mod one_fault_up_to_the_pool_checker;
mod what_a_lying_device_leaves;

pub use fault_injection_campaign::{
    run_fault_injection_campaign, run_fault_injection_campaign_handing_out_the_images,
    FaultInjectionCampaign, FaultInjectionReport, FaultInjectionWorkerThreads,
};
pub use findings_and_tally::{
    known_red_form_only_after_a_fault, DrawnFault, FactsAfterTheFault, FaultFinding,
    FaultInjectionTally, FaultOutcome, FaultedSegment, FaultedSegmentKind, InjectionPoint,
    KnownRedAtAFault, KnownRedFormAtAFault, KnownRedFormOnlyAfterAFault, ReopenedVersion,
};
use injection_into_a_history::{
    acquired_instance_left_after_a_failed_mount,
    raised_floor_of_the_newest_root_lands_only_on_abandoned_roots, the_fault_surfaced_as_an_error,
    FaultInjectionAccumulator, StepMark,
};
pub use injection_into_a_history::{
    inject_faults_into_history, inject_faults_into_history_handing_out_the_images,
    inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine,
    inject_one_fault_into_the_segment, inject_one_fault_into_the_segment_handing_out_the_images,
    AcquiredInstanceLeft, AcquiredInstanceLeftAfterAFailedMount, HistoryFaultInjection,
    PoolCheckerVerdictsAfterAFault,
};
use one_fault_up_to_the_pool_checker::{
    merge_one_fault_with_the_pool_checker_verdicts, run_one_fault_up_to_the_pool_checker,
    settle_one_fault_with_the_pool_checker_verdicts,
};
pub use what_a_lying_device_leaves::{
    classify_a_failure_after_a_fault,
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault,
    invariants_still_violated_on_the_comparison_image, left_by_the_lying_device,
    stream_with_the_swallowed_writes_landed, unit_start_slots_quarantined_over_a_swallowed_write,
    FailureAfterAFault, ImagesAfterAFault, PoolCheckerVerdictsOfTheImagesAfterAFault,
    RecordedStreamOfAnImage, StepImageInTheRecordedStream, SwallowedWriteByteRange,
    ViolationsNamingTheSites, WhatTheSwallowedWritesExplain,
};

/// 故障注入的工作线程数从这个环境变量取（十进制正整数）；没设就取每台默认重活线程数
/// （[`crate::heavy_threads::default_heavy_threads`]：`SINGLEFS_THREAD_CAP`，再没设取登记值）。
pub const FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_FAULT_INJECTION_WORKER_THREADS";

/// 调用块设备的三种动作里，注入点认得的那三种（`probe_physical_block_size` 与 `size_in_bytes` 不碰盘，不注入）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FaultCallKind {
    Read,
    Write,
    Barrier,
}

impl FaultCallKind {
    pub const ALL: [FaultCallKind; 3] = [
        FaultCallKind::Read,
        FaultCallKind::Write,
        FaultCallKind::Barrier,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            FaultCallKind::Read => "read",
            FaultCallKind::Write => "write",
            FaultCallKind::Barrier => "barrier",
        }
    }
}

/// 一次读回来的字节里翻掉哪一位：第几个字节、字节里第几位。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlippedBit {
    /// 这次读的缓冲区里第几个字节（比缓冲区长时按缓冲区长度取模，读多短都翻得到一位）。
    pub byte_index: u64,
    /// 那个字节里第几位（0..8）。
    pub bit_index: u32,
}

impl FlippedBit {
    /// 缓冲区里真正被翻的那个字节的下标与掩码。
    #[must_use]
    fn within(self, buffer_length: usize) -> Option<(usize, u8)> {
        if buffer_length == 0 {
            return None;
        }
        let length = u64::try_from(buffer_length).expect("读的长度装得进 u64");
        let index = usize::try_from(self.byte_index % length).expect("取模之后装得回 usize");
        Some((index, 1u8 << (self.bit_index % 8)))
    }
}

/// 注入什么。每个成员自带它作用在哪一种调用上，写不出「让一次屏障返回改坏的字节」这种非法组合
/// （`code-discipline.md`「类型：让非法状态写不出来」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InjectedFault {
    /// 这一次写报块设备错，一个字节都不落盘。
    WriteFails,
    /// 这一次写报成功、其实一个字节都没落盘（写进了设备缓存又掉电的那一形）。
    WriteIsSwallowed,
    /// 这一次读报块设备错，缓冲区不动。
    ReadFails,
    /// 这一次读报成功，读回的字节里翻掉一位。
    ReadReturnsCorruptedBytes { flipped_bit: FlippedBit },
    /// 这一次刷盘报块设备错。
    BarrierFails,
    /// 这一次刷盘报成功、其实没发给设备（漏一道屏障；虚机档拿它量「少一道屏障多出哪些崩溃状态」）。
    BarrierIsSwallowed,
}

impl InjectedFault {
    #[must_use]
    pub const fn call_kind(self) -> FaultCallKind {
        match self {
            InjectedFault::WriteFails | InjectedFault::WriteIsSwallowed => FaultCallKind::Write,
            InjectedFault::ReadFails | InjectedFault::ReadReturnsCorruptedBytes { .. } => {
                FaultCallKind::Read
            }
            InjectedFault::BarrierFails | InjectedFault::BarrierIsSwallowed => {
                FaultCallKind::Barrier
            }
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            InjectedFault::WriteFails => "write_fails",
            InjectedFault::WriteIsSwallowed => "write_is_swallowed",
            InjectedFault::ReadFails => "read_fails",
            InjectedFault::ReadReturnsCorruptedBytes { .. } => "read_returns_corrupted_bytes",
            InjectedFault::BarrierFails => "barrier_fails",
            InjectedFault::BarrierIsSwallowed => "barrier_is_swallowed",
        }
    }

    /// 这一种注入是不是「设备说谎」：报了成功，其实什么也没做。
    ///
    /// 说谎的设备与报错的设备判定档不同。报错时实现拿到的是 `Err`，它**有**办法把错交回去，所以
    /// 「返回错误而不是 panic」「重开恢复到模型允许的版本」两条都照判。说谎时实现拿到的是 `Ok`，
    /// 那一份内容没落盘而它无从知道：盘上因此留下的不一致（根指着一份从没写下去的单元）不是实现的缺口，
    /// 它是设备丢的。这一格记进 [`FaultInjectionTally::faults_where_a_lying_device_left_the_image_inconsistent`]
    /// 并按签名列进报告，不记成新发现。**仍旧照判的两条**：注入之后一次 panic 都不许有；
    /// 重开不许悄悄读回一版模型从没提交过的内容（只放行「重开走读失败」——单元头里的校验和是旧那一份的，
    /// 读出来发现对不上、报错正是要的结果）。
    #[must_use]
    pub const fn the_device_lies(self) -> bool {
        match self {
            InjectedFault::WriteIsSwallowed | InjectedFault::BarrierIsSwallowed => true,
            InjectedFault::WriteFails
            | InjectedFault::ReadFails
            | InjectedFault::ReadReturnsCorruptedBytes { .. }
            | InjectedFault::BarrierFails => false,
        }
    }
}

/// 注入落在哪块盘上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultDeviceSelector {
    EveryDevice,
    OnlyDevice(DeviceIdentity),
}

/// 按 (区域, 槽) 点名的一组根环槽：R = 3 个区域（`singlefs_format::ROOT_RING_REGIONS`）×
/// 每区域至多 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM` 个槽 = 48 位，一个槽占一位，装得进 `u64`，
/// 因此这个集合是 `Copy` 的、摆得进注入计划里。
///
/// ⚠️ **位号按区间的上界排，不按这个池的 S 排**：S 是系统配置字段、每个池自己一个值
/// （C506（每区槽数 S 写成编译期常量，条文说它住系统配置）），位号跟着它走的话，同一个 (区域, 槽)
/// 在 S = 8 的池与 S = 16 的池里会落到不同的位，一个注入计划换个池就点到别的槽上。
/// 这个位图只管「点了哪几个 (区域, 槽)」；这个池的环有多长是调用方的事
/// （[`NamedRootRingSlots::every_slot_in_the_ring`] 按传进来的 S 枚举）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct NamedRootRingSlots {
    named: u64,
}

impl NamedRootRingSlots {
    /// 一个都不点名。
    pub const NONE: NamedRootRingSlots = NamedRootRingSlots { named: 0 };

    /// 这个槽在位图里的位号；区域号或槽号越界时 panic（区域数是格式常量、槽号上界是格式承诺的区间上界，
    /// 越界是调用点的 bug）。
    #[must_use]
    fn bit_of(slot: RootRingSlot) -> u32 {
        assert!(
            slot.region < ROOT_RING_REGIONS,
            "区域号 {} 越界（根环 {ROOT_RING_REGIONS} 个区域）",
            slot.region
        );
        assert!(
            slot.slot < ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            "槽号 {} 越界（每区域至多 {ROOT_RING_SLOTS_PER_REGION_MAXIMUM} 个槽）",
            slot.slot
        );
        u32::try_from(slot.region * ROOT_RING_SLOTS_PER_REGION_MAXIMUM + slot.slot)
            .expect("位号小于 48")
    }

    /// 点名这几个槽。
    #[must_use]
    pub fn naming(slots: &[RootRingSlot]) -> Self {
        slots
            .iter()
            .fold(NamedRootRingSlots::NONE, |named, slot| named.with(*slot))
    }

    /// 再点名一个槽。
    #[must_use]
    pub fn with(self, slot: RootRingSlot) -> Self {
        NamedRootRingSlots {
            named: self.named | (1u64 << Self::bit_of(slot)),
        }
    }

    /// 这个池的根环里每个槽都点名（「一条根都读不出」那一格）：R × S 个，S 由调用方从它那个池的
    /// 系统配置里取，不按 mkfs 那一档算。
    #[must_use]
    pub fn every_slot_in_the_ring(slots_per_region: RootRingSlotsPerRegion) -> Self {
        let mut named = NamedRootRingSlots::NONE;
        for region in 0..ROOT_RING_REGIONS {
            for slot in 0..slots_per_region.count() {
                named = named.with(RootRingSlot { region, slot });
            }
        }
        named
    }

    #[must_use]
    pub fn contains(self, slot: RootRingSlot) -> bool {
        self.named & (1u64 << Self::bit_of(slot)) != 0
    }

    /// 点名了几个槽。
    #[must_use]
    pub fn count(self) -> u32 {
        self.named.count_ones()
    }

    /// 点名的槽，按 (区域, 槽) 升序。枚举到区间的上界为止、不到这个池的 S 为止：
    /// 交出来的只有真被点名的那几位，枚举范围宽一点不会多出一个槽，而按 S 枚举就要求这里也知道 S。
    #[must_use]
    pub fn named_slots(self) -> Vec<RootRingSlot> {
        let mut named_slots = Vec::new();
        for region in 0..ROOT_RING_REGIONS {
            for slot in 0..ROOT_RING_SLOTS_PER_REGION_MAXIMUM {
                let candidate = RootRingSlot { region, slot };
                if self.contains(candidate) {
                    named_slots.push(candidate);
                }
            }
        }
        named_slots
    }

    /// 报告里的名字。
    #[must_use]
    pub fn render(self) -> String {
        if self.named == 0 {
            return "没点名".to_string();
        }
        self.named_slots()
            .iter()
            .map(|slot| format!("(区域 {}, 槽 {})", slot.region, slot.slot))
            .collect::<Vec<String>>()
            .join("、")
    }
}

/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：按 (区域, 槽) 点名根槽，点名的槽上**每一次**读都失败。
///
/// 与同一份模块里按种子的随机注入（`FaultSchedule::the_nth_call_across_the_pool`）分工不同，两样都要：
/// 那一路挑的是「整池第 n 次读」、一段历史只注一次，问的是「任意一次读坏掉，实现会不会 panic」；
/// 这一路点名的是**落点**，而且**持续**——第一版没有根环槽的重定位（C335（根槽持续读不出时实例表只增不减）），
/// 「一个根槽持续读不出时它永远读不通」这条论证要的正是「每一次都失败」，一次瞬时错顶不上它。
///
/// 区域 r 的根槽住哪块盘由系统配置的 `region_devices` 定（`recovery::visit_valid_roots` 只在那块盘上读区域 r），
/// 所以点名一个 (区域, 槽) 就点定了一个 (盘, 盘内偏移)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRingSlotTarget {
    pub named_slots: NamedRootRingSlots,
    /// 三个区域各住哪块盘（系统配置 `immutable.region_devices` 原样）。
    pub region_devices: [DeviceIdentity; 3],
    /// 固定结构槽距：区域起点 + 槽号 × 槽距就是这个槽的盘内偏移（`singlefs_core::root_ring::slot_offset`）。
    pub fixed_structure_slot_spacing: u32,
}

impl RootRingSlotTarget {
    /// 这一次读 / 写落在点名的哪个槽里；不落在任何点名的槽里就 None。
    /// 判的是整槽区间 `[槽起点, 槽起点 + 槽距)`，不是「偏移恰好等于槽起点」：根槽读的是整块（宽 `physical_block_size`）、
    /// 写的是整槽，落在槽里的任何偏移都算这个槽。
    #[must_use]
    pub fn slot_covering(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
    ) -> Option<RootRingSlot> {
        let spacing = u64::from(self.fixed_structure_slot_spacing);
        self.named_slots.named_slots().into_iter().find(|slot| {
            let region_index = usize::try_from(slot.region).expect("区域号小于 3");
            let start = slot_offset(*slot, self.fixed_structure_slot_spacing).0;
            self.region_devices[region_index] == device
                && offset.0 >= start
                && offset.0 < start + spacing
        })
    }

    /// 报告里的名字。
    #[must_use]
    pub fn render(&self) -> String {
        self.named_slots.render()
    }
}

/// 同一个开关（`RootRingSlotTarget`）的第二个口子：包在**读者**外面，不是包在块设备外面。
///
/// 块设备那个口子（`FaultInjectingBlockDevice` + `FaultSchedule::every_read_of_named_root_ring_slots_fails`）
/// 罩得住走 `Vec<(DeviceIdentity, Device)>` 的路径（可写挂载、回退、抬 F）；崩溃点重放枚举出来的崩溃镜像
/// （`crate::memory_pool::MemoryPool` / `CrashImage`）不是块设备、只是 `PoolReader`，那一路要这个口子。
/// 两个口子判「这一次读落在哪个点名的槽里」用的是同一个 `RootRingSlotTarget::slot_covering`。
///
/// 读**持续**失败：点名的槽上每一次读都返回 None，次数不限；`reads_refused` 数得出来它被拦了几次。
pub struct PoolReaderWithUnreadableRootRingSlots<'reader, Reader: PoolReader + ?Sized> {
    inner: &'reader Reader,
    target: RootRingSlotTarget,
    reads_refused: Cell<u64>,
}

impl<'reader, Reader: PoolReader + ?Sized> PoolReaderWithUnreadableRootRingSlots<'reader, Reader> {
    #[must_use]
    pub fn new(inner: &'reader Reader, target: RootRingSlotTarget) -> Self {
        Self {
            inner,
            target,
            reads_refused: Cell::new(0),
        }
    }

    /// 点名的槽上拦下了几次读。持续注入的判据：同一段恢复跑两遍，这个数接着涨、结论不变。
    #[must_use]
    pub fn reads_refused(&self) -> u64 {
        self.reads_refused.get()
    }
}

impl<Reader: PoolReader + ?Sized> PoolReader for PoolReaderWithUnreadableRootRingSlots<'_, Reader> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.inner.device_identities()
    }

    /// 只供测试的那道「根槽读不出」拦在读上，不改几何：盘多大照问内层。
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.inner.device_size_in_bytes(device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.inner.physical_block_size_probed_on_the_device(device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        if self.target.slot_covering(device, offset).is_some() {
            self.reads_refused.set(self.reads_refused.get() + 1);
            return None;
        }
        self.inner.read(device, offset, length)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 注入只挑落点落在这儿的调用（屏障没有落点，恒算命中）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPlacement {
    AnyOffset,
    /// 偏移小于这个数（系统配置两槽住在偏移 0 起的那一段）。
    OffsetBelow(u64),
    OffsetExactly(DeviceOffsetInBytes),
    /// 落在点名的那几个根环槽里（只供测试的开关，见 `RootRingSlotTarget`）。
    WithinNamedRootRingSlots(RootRingSlotTarget),
}

impl FaultPlacement {
    #[must_use]
    fn matches(self, device: DeviceIdentity, offset: DeviceOffsetInBytes) -> bool {
        match self {
            FaultPlacement::AnyOffset => true,
            FaultPlacement::OffsetBelow(end) => offset.0 < end,
            FaultPlacement::OffsetExactly(wanted) => offset == wanted,
            FaultPlacement::WithinNamedRootRingSlots(target) => {
                target.slot_covering(device, offset).is_some()
            }
        }
    }
}

/// 命中的调用按什么数：整个池数一份，还是每块盘各数一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultCounting {
    AcrossThePool,
    PerDevice,
}

/// 注入在第几次命中的调用上（序号从 1 起）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultOccurrence {
    /// 只在第 n 次命中的调用上注入一次，之后放行（随机注入那一路用这一种：一段历史一个注入点）。
    TheNthMatchingCall(u64),
    /// 第 n 次命中起，每一次都注入（手写包装里「之后每次写都报错」「屏障一直报错」那一种）。
    EveryMatchingCallFromTheNthOnward(u64),
}

impl FaultOccurrence {
    /// 每一次命中都注入。
    pub const EVERY_MATCHING_CALL: FaultOccurrence =
        FaultOccurrence::EveryMatchingCallFromTheNthOnward(1);

    #[must_use]
    fn fires_at(self, matching_call_ordinal: u64) -> bool {
        match self {
            FaultOccurrence::TheNthMatchingCall(ordinal) => matching_call_ordinal == ordinal,
            FaultOccurrence::EveryMatchingCallFromTheNthOnward(first) => {
                matching_call_ordinal >= first
            }
        }
    }
}

/// 一次注入怎么摆：注入什么、挑哪块盘、挑哪个落点、按什么数、在第几次上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaultSchedule {
    pub fault: InjectedFault,
    pub device: FaultDeviceSelector,
    pub placement: FaultPlacement,
    pub counting: FaultCounting,
    pub occurrence: FaultOccurrence,
}

impl FaultSchedule {
    /// 整个池上第 `ordinal` 次这一种调用（随机注入那一路摆的形态）。
    #[must_use]
    pub fn the_nth_call_across_the_pool(fault: InjectedFault, ordinal: u64) -> Self {
        Self {
            fault,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
        }
    }

    /// 每一次这一种调用都注入（手写包装里的开关那一种）。
    #[must_use]
    pub fn every_call_across_the_pool(fault: InjectedFault) -> Self {
        Self {
            fault,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        }
    }

    /// 只供测试的开关（`RootRingSlotTarget`）：点名的那几个根环槽上，**每一次**读都报块设备错。
    /// 盘不另挑（点名 (区域, 槽) 已经点定了盘），命中整池数，从第 1 次起每一次都注入。
    #[must_use]
    pub fn every_read_of_named_root_ring_slots_fails(target: RootRingSlotTarget) -> Self {
        Self {
            fault: InjectedFault::ReadFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::WithinNamedRootRingSlots(target),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        }
    }

    #[must_use]
    fn matches(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes) -> bool {
        let device_matches = match self.device {
            FaultDeviceSelector::EveryDevice => true,
            FaultDeviceSelector::OnlyDevice(wanted) => device == wanted,
        };
        device_matches && self.placement.matches(device, offset)
    }

    /// 报告里的名字。
    #[must_use]
    pub fn render(&self) -> String {
        let device = match self.device {
            FaultDeviceSelector::EveryDevice => "每块盘".to_string(),
            FaultDeviceSelector::OnlyDevice(identity) => format!("只盘 {}", identity.0),
        };
        let placement = match self.placement {
            FaultPlacement::AnyOffset => "任意落点".to_string(),
            FaultPlacement::OffsetBelow(end) => format!("偏移小于 {end}"),
            FaultPlacement::OffsetExactly(offset) => format!("偏移恰好 {}", offset.0),
            FaultPlacement::WithinNamedRootRingSlots(target) => {
                format!("点名的根环槽 {}", target.render())
            }
        };
        let counting = match self.counting {
            FaultCounting::AcrossThePool => "整池数",
            FaultCounting::PerDevice => "逐盘数",
        };
        let occurrence = match self.occurrence {
            FaultOccurrence::TheNthMatchingCall(ordinal) => format!("第 {ordinal} 次"),
            FaultOccurrence::EveryMatchingCallFromTheNthOnward(first) => {
                format!("第 {first} 次起每一次")
            }
        };
        format!(
            "{}（{device}、{placement}、{counting}、{occurrence}）",
            self.fault.name()
        )
    }
}

/// 一块盘上真正发生过的调用数：报成功交给里面那块盘的那些。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaultDeviceCounts {
    pub reads: u64,
    pub writes: u64,
    pub written_bytes: u64,
    pub force_unit_access_writes: u64,
    /// 真交给里面那块盘的屏障（被吞掉的那些不算）。
    pub barriers_forwarded: u64,
    pub barriers_swallowed: u64,
    /// 交给里面那块盘、它交回「交给了底层」的 discard（不算进 `writes` 与 `written_bytes`：块层把 discard 另记一类）。
    pub discards_applied: u64,
    /// 交给里面那块盘、它交回「不支持」的 discard。
    pub discards_answered_unsupported: u64,
}

impl FaultDeviceCounts {
    /// 两次快照之差（一段窗口里发生了多少）。
    #[must_use]
    pub fn since(&self, earlier: &FaultDeviceCounts) -> FaultDeviceCounts {
        FaultDeviceCounts {
            reads: self.reads - earlier.reads,
            writes: self.writes - earlier.writes,
            written_bytes: self.written_bytes - earlier.written_bytes,
            force_unit_access_writes: self.force_unit_access_writes
                - earlier.force_unit_access_writes,
            barriers_forwarded: self.barriers_forwarded - earlier.barriers_forwarded,
            barriers_swallowed: self.barriers_swallowed - earlier.barriers_swallowed,
            discards_applied: self.discards_applied - earlier.discards_applied,
            discards_answered_unsupported: self.discards_answered_unsupported
                - earlier.discards_answered_unsupported,
        }
    }
}

/// 整个池上调用方发出的调用数（注入与否都数，注入那一次也算发出过）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaultCallCounts {
    pub reads: u64,
    pub writes: u64,
    pub barriers: u64,
}

impl FaultCallCounts {
    #[must_use]
    pub fn of(&self, kind: FaultCallKind) -> u64 {
        match kind {
            FaultCallKind::Read => self.reads,
            FaultCallKind::Write => self.writes,
            FaultCallKind::Barrier => self.barriers,
        }
    }
}

/// 注入真的发生过的那一次：注入了什么、落在哪块盘的哪个落点上、那次调用是整池第几次。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FiredFault {
    pub fault: InjectedFault,
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length: u64,
    /// 写落在哪一类结构上（按落点分，`crate::segments::FixedGeometry::classify`）；读与屏障没有这一项。
    pub written_structure: Option<StepKind>,
    /// 这次调用是整池第几次这一种调用（1 起）。
    pub call_ordinal_across_the_pool: u64,
}

impl FiredFault {
    /// 报告里的名字：注入了什么、落在哪一类结构上。
    #[must_use]
    pub fn render(&self) -> String {
        let structure = match self.written_structure {
            Some(kind) => kind.name(),
            None => "-",
        };
        format!(
            "{}@{structure}（盘 {}、偏移 {}、整池第 {} 次{}）",
            self.fault.name(),
            self.device.0,
            self.offset.0,
            self.call_ordinal_across_the_pool,
            self.fault.call_kind().name()
        )
    }
}

/// 被吞掉的那一次写本该写下什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwallowedContents {
    Bytes(Vec<u8>),
    /// 写零（`BlockDevice::write_zeroes_at`）：整段全 0。
    Zeroes {
        length: u64,
    },
    /// discard（`BlockDevice::discard_at`）：报了「交给了底层」，其实没交下去。
    Discard {
        length: u64,
    },
}

/// 说谎的设备吞掉的那一次写（[`InjectedFault::WriteIsSwallowed`]）：落在哪、本该写下什么、它之前整池交下去了几次写（含写零）。
/// 按写数、不按「写与屏障」数：录制器在注入层里面，交下去的每一次写它记且只记一步，屏障却会把紧挨着的几道并成一步
/// （`SharedStream` 的 `push`）。屏障不改镜像，插回去时只要排在第这么多次写之后。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwallowedWrite {
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub contents: SwallowedContents,
    pub durability: WriteDurability,
    pub writes_recorded_before: usize,
}

impl SwallowedWrite {
    /// 它要是落了盘，录制流里会多出的那一步。
    #[must_use]
    pub fn as_retained_operation(&self) -> crate::RetainedOperation {
        match &self.contents {
            SwallowedContents::Bytes(bytes) => crate::RetainedOperation {
                operation: RecordedOperation {
                    device: self.device,
                    kind: match self.durability {
                        WriteDurability::Plain => RecordedOperationKind::Write,
                        WriteDurability::ForceUnitAccess => {
                            RecordedOperationKind::WriteForceUnitAccess
                        }
                    },
                    offset: self.offset,
                    length: u64::try_from(bytes.len()).expect("一次写的长度装得进 u64"),
                    content_hash: crate::fnv1a_64(bytes),
                },
                contents: Some(bytes.clone()),
            },
            SwallowedContents::Zeroes { length } => crate::RetainedOperation {
                operation: RecordedOperation {
                    device: self.device,
                    kind: RecordedOperationKind::WriteZeroes,
                    offset: self.offset,
                    length: *length,
                    content_hash: crate::fnv1a_64_of_zeros(*length),
                },
                contents: None,
            },
            SwallowedContents::Discard { length } => crate::RetainedOperation {
                operation: RecordedOperation {
                    device: self.device,
                    kind: RecordedOperationKind::Discard,
                    offset: self.offset,
                    length: *length,
                    content_hash: crate::fnv1a_64_of_zeros(*length),
                },
                contents: None,
            },
        }
    }
}

struct FaultPlanState {
    geometry: FixedGeometry,
    schedule: Option<FaultSchedule>,
    calls: FaultCallCounts,
    matching_calls_across_the_pool: u64,
    matching_calls_per_device: BTreeMap<DeviceIdentity, u64>,
    per_device_counts: BTreeMap<DeviceIdentity, FaultDeviceCounts>,
    /// 每块盘在收到过至少一次写之后收到的第一道屏障，那一刻这块盘的计数（屏障本身不算进去）：真设备那一路按它切「挂载」那一段窗口
    /// （可写挂载里那是取号写之后那一道；取号写之前那一道前面没有写，不算，代码审阅第 19 条加了它）。
    counts_when_the_first_barrier_after_a_write_arrived:
        BTreeMap<DeviceIdentity, FaultDeviceCounts>,
    /// 每块盘每一道「前面比上一次快照多了写」的屏障进来那一刻的计数（屏障本身不算进去），按到达次序：写数逐个严格递增，
    /// 头一项就是 `counts_when_the_first_barrier_after_a_write_arrived` 那一刻。可写挂载取号写两轮（D22（单元原子性怎么合成） 已定项 8），
    /// 真设备那一路的挂载窗口从取号写之后那一道屏障起，按写数在这里找。
    counts_at_each_barrier_after_new_writes: BTreeMap<DeviceIdentity, Vec<FaultDeviceCounts>>,
    fired: Vec<FiredFault>,
    /// 整个池上交给里面那块盘的写（含写零）一共几次：被吞的写在录制流里本该排在第几次写之后，按它数。
    writes_forwarded_across_the_pool: usize,
    swallowed_writes: Vec<SwallowedWrite>,
}

/// 一个池里几块设备共用的注入计划：数调用、按计划注入、记下注入真的发生在哪一次。
/// 装的是 `Rc<RefCell<..>>`，写入口正拿着设备时调用方也开得了、关得掉（手写包装里的开关就是这么用的）。
#[derive(Clone)]
pub struct SharedFaultPlan(Rc<RefCell<FaultPlanState>>);

impl SharedFaultPlan {
    /// 不注入，只数调用。
    #[must_use]
    pub fn unarmed(geometry: FixedGeometry) -> Self {
        Self(Rc::new(RefCell::new(FaultPlanState {
            geometry,
            schedule: None,
            calls: FaultCallCounts::default(),
            matching_calls_across_the_pool: 0,
            matching_calls_per_device: BTreeMap::new(),
            per_device_counts: BTreeMap::new(),
            counts_when_the_first_barrier_after_a_write_arrived: BTreeMap::new(),
            counts_at_each_barrier_after_new_writes: BTreeMap::new(),
            fired: Vec::new(),
            writes_forwarded_across_the_pool: 0,
            swallowed_writes: Vec::new(),
        })))
    }

    /// 说谎的设备吞掉的那几次写，按发生次序。
    #[must_use]
    pub fn swallowed_writes(&self) -> Vec<SwallowedWrite> {
        self.0.borrow().swallowed_writes.clone()
    }

    /// 到此为止吞掉了几次写（[`Self::swallowed_writes`] 的头这么多次）。
    #[must_use]
    pub fn swallowed_write_count(&self) -> usize {
        self.0.borrow().swallowed_writes.len()
    }

    fn note_swallowed_write(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        contents: SwallowedContents,
        durability: WriteDurability,
    ) {
        let mut state = self.0.borrow_mut();
        let writes_recorded_before = state.writes_forwarded_across_the_pool;
        state.swallowed_writes.push(SwallowedWrite {
            device,
            offset,
            contents,
            durability,
            writes_recorded_before,
        });
    }

    /// 开着一条计划起步。
    #[must_use]
    pub fn armed(geometry: FixedGeometry, schedule: FaultSchedule) -> Self {
        let plan = Self::unarmed(geometry);
        plan.arm(schedule);
        plan
    }

    /// 换上一条计划：命中计数从零重数（换计划就是换注入点，旧计划数到哪与新的无关）。
    pub fn arm(&self, schedule: FaultSchedule) {
        let mut state = self.0.borrow_mut();
        state.schedule = Some(schedule);
        state.matching_calls_across_the_pool = 0;
        state.matching_calls_per_device.clear();
    }

    /// 收起计划：之后每次调用都原样交给里面那块盘。
    pub fn disarm(&self) {
        self.0.borrow_mut().schedule = None;
    }

    #[must_use]
    pub fn calls(&self) -> FaultCallCounts {
        self.0.borrow().calls
    }

    #[must_use]
    pub fn counts_of_device(&self, device: DeviceIdentity) -> FaultDeviceCounts {
        self.0
            .borrow()
            .per_device_counts
            .get(&device)
            .copied()
            .unwrap_or_default()
    }

    /// 这块盘收到过至少一次写之后收到的第一道屏障，那一刻的计数；写之后一道屏障都没收到过时 None。
    #[must_use]
    pub fn counts_when_the_first_barrier_after_a_write_arrived_at(
        &self,
        device: DeviceIdentity,
    ) -> Option<FaultDeviceCounts> {
        self.0
            .borrow()
            .counts_when_the_first_barrier_after_a_write_arrived
            .get(&device)
            .copied()
    }

    /// 这块盘收到的写数到了 `writes_before_the_barrier` 之后进来的第一道屏障，那一刻的计数（屏障本身不算进去）；
    /// 取 1 与 [`Self::counts_when_the_first_barrier_after_a_write_arrived_at`] 是同一刻。写数到了之后一道屏障都没收到过时 None。
    ///
    /// # Panics
    /// `writes_before_the_barrier` 是 0（「零次写之后」的屏障不拍快照，问它是调用方写错了）。
    #[must_use]
    pub fn counts_when_the_first_barrier_after_this_many_writes_arrived_at(
        &self,
        device: DeviceIdentity,
        writes_before_the_barrier: u64,
    ) -> Option<FaultDeviceCounts> {
        assert!(
            writes_before_the_barrier > 0,
            "问的是至少一次写之后的屏障：写之前的屏障不拍快照"
        );
        self.0
            .borrow()
            .counts_at_each_barrier_after_new_writes
            .get(&device)?
            .iter()
            .find(|snapshot| snapshot.writes >= writes_before_the_barrier)
            .copied()
    }

    /// 注入真的发生过的那几次，按发生次序。
    #[must_use]
    pub fn fired(&self) -> Vec<FiredFault> {
        self.0.borrow().fired.clone()
    }

    #[must_use]
    pub fn fired_count(&self) -> usize {
        self.0.borrow().fired.len()
    }

    /// 这一次调用要不要注入；要注入就把它记进 `fired`。命中计数在这里推进，所以每次调用只许问一次。
    fn decide(
        &self,
        kind: FaultCallKind,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Option<InjectedFault> {
        let mut state = self.0.borrow_mut();
        match kind {
            FaultCallKind::Read => state.calls.reads += 1,
            FaultCallKind::Write => state.calls.writes += 1,
            FaultCallKind::Barrier => state.calls.barriers += 1,
        }
        let schedule = state.schedule?;
        if schedule.fault.call_kind() != kind || !schedule.matches(device, offset) {
            return None;
        }
        state.matching_calls_across_the_pool += 1;
        let per_device = state.matching_calls_per_device.entry(device).or_insert(0);
        *per_device += 1;
        let ordinal = match schedule.counting {
            FaultCounting::AcrossThePool => state.matching_calls_across_the_pool,
            FaultCounting::PerDevice => *per_device,
        };
        if !schedule.occurrence.fires_at(ordinal) {
            return None;
        }
        let written_structure = match kind {
            FaultCallKind::Write => Some(state.geometry.classify(&RecordedOperation {
                device,
                kind: RecordedOperationKind::Write,
                offset,
                length,
                content_hash: 0,
            })),
            FaultCallKind::Read | FaultCallKind::Barrier => None,
        };
        let call_ordinal_across_the_pool = state.calls.of(kind);
        state.fired.push(FiredFault {
            fault: schedule.fault,
            device,
            offset,
            length,
            written_structure,
            call_ordinal_across_the_pool,
        });
        Some(schedule.fault)
    }

    fn note_forwarded_read(&self, device: DeviceIdentity) {
        self.0
            .borrow_mut()
            .per_device_counts
            .entry(device)
            .or_default()
            .reads += 1;
    }

    fn note_forwarded_write(
        &self,
        device: DeviceIdentity,
        bytes: u64,
        durability: WriteDurability,
    ) {
        let mut state = self.0.borrow_mut();
        state.writes_forwarded_across_the_pool += 1;
        let counts = state.per_device_counts.entry(device).or_default();
        counts.writes += 1;
        counts.written_bytes += bytes;
        match durability {
            WriteDurability::Plain => {}
            WriteDurability::ForceUnitAccess => counts.force_unit_access_writes += 1,
        }
    }

    /// 屏障进来了：这块盘收到过写、还没拍过快照的，先拍（屏障本身不算进去），再记下它是发下去了还是被吞了。
    fn note_barrier_arrived(&self, device: DeviceIdentity) {
        let mut state = self.0.borrow_mut();
        let counts = *state.per_device_counts.entry(device).or_default();
        if counts.writes > 0 {
            state
                .counts_when_the_first_barrier_after_a_write_arrived
                .entry(device)
                .or_insert(counts);
        }
        let snapshots = state
            .counts_at_each_barrier_after_new_writes
            .entry(device)
            .or_default();
        let writes_at_the_latest_snapshot = snapshots.last().map_or(0, |snapshot| snapshot.writes);
        if counts.writes > writes_at_the_latest_snapshot {
            snapshots.push(counts);
        }
    }

    /// 一次 discard 交给了里面那块盘、它交回了 `outcome`。「交给了底层」的在整池写数里算一次（录制器在注入层里面时它在录制流里记一步，
    /// 被吞的写排在第几次写之后按这个数）；「不支持」的不算（录制器不记它）。
    fn note_forwarded_discard(&self, device: DeviceIdentity, outcome: DiscardOutcome) {
        let mut state = self.0.borrow_mut();
        match outcome {
            DiscardOutcome::Applied => {
                state.writes_forwarded_across_the_pool += 1;
                state
                    .per_device_counts
                    .entry(device)
                    .or_default()
                    .discards_applied += 1;
            }
            DiscardOutcome::Unsupported => {
                state
                    .per_device_counts
                    .entry(device)
                    .or_default()
                    .discards_answered_unsupported += 1;
            }
        }
    }

    fn note_forwarded_barrier(&self, device: DeviceIdentity) {
        self.0
            .borrow_mut()
            .per_device_counts
            .entry(device)
            .or_default()
            .barriers_forwarded += 1;
    }

    fn note_swallowed_barrier(&self, device: DeviceIdentity) {
        self.0
            .borrow_mut()
            .per_device_counts
            .entry(device)
            .or_default()
            .barriers_swallowed += 1;
    }
}

/// 注入的那一个错：块设备的 I/O 错，消息里写明是注入的。
#[must_use]
pub fn injected_block_device_error(what: &str) -> BlockDeviceError {
    BlockDeviceError::InputOutput(std::io::Error::other(format!("注入的{what}错")))
}

/// 通用的故障注入设备包装：包在任意一块设备外面，按共用的计划让某一次读 / 写 / 刷盘报错、吞掉、或读回改坏的字节。
/// 几块盘共用同一个 [`SharedFaultPlan`]，序号因此可以整池数（「这个池第 7 次写」只有一次）。
pub struct FaultInjectingBlockDevice<Inner: BlockDevice> {
    inner: Inner,
    device: DeviceIdentity,
    plan: SharedFaultPlan,
}

impl<Inner: BlockDevice> FaultInjectingBlockDevice<Inner> {
    pub fn new(device: DeviceIdentity, inner: Inner, plan: SharedFaultPlan) -> Self {
        Self {
            inner,
            device,
            plan,
        }
    }

    #[must_use]
    pub fn wrapped_device(&self) -> &Inner {
        &self.inner
    }

    pub fn wrapped_device_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    #[must_use]
    pub fn into_inner(self) -> Inner {
        self.inner
    }

    #[must_use]
    pub fn plan(&self) -> &SharedFaultPlan {
        &self.plan
    }

    /// 普通读与两种专用读（读坏之后的重读、这一刻读盘）共用的注入判定与记账：同一次 `decide(FaultCallKind::Read, …)`，
    /// 读失败与读出坏位一样生效，交下去的读一样记进这块盘的读数。交下去的那一读由 `read_from_the_inner_device` 发：
    /// 普通读发里面那块盘的 `read_at`，专用读发它的同名方法（里面那块盘可能是记着这次挂载读过什么的那一层，它在专用读上绕过记下的那一份）。
    fn read_through_the_fault_plan(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
        read_from_the_inner_device: impl FnOnce(
            &Inner,
            DeviceOffsetInBytes,
            &mut [u8],
        ) -> Result<(), BlockDeviceError>,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读的长度装得进 u64");
        match self
            .plan
            .decide(FaultCallKind::Read, self.device, offset, length)
        {
            Some(InjectedFault::ReadFails) => return Err(injected_block_device_error("读")),
            Some(InjectedFault::ReadReturnsCorruptedBytes { flipped_bit }) => {
                read_from_the_inner_device(&self.inner, offset, buffer)?;
                self.plan.note_forwarded_read(self.device);
                if let Some((index, mask)) = flipped_bit.within(buffer.len()) {
                    buffer[index] ^= mask;
                }
                return Ok(());
            }
            // 读的计划只有这两条；写与屏障的计划在 `decide` 里按调用种类挡掉了，走不到这里。
            Some(
                InjectedFault::WriteFails
                | InjectedFault::WriteIsSwallowed
                | InjectedFault::BarrierFails
                | InjectedFault::BarrierIsSwallowed,
            )
            | None => {}
        }
        read_from_the_inner_device(&self.inner, offset, buffer)?;
        self.plan.note_forwarded_read(self.device);
        Ok(())
    }
}

impl<Inner: BlockDevice> BlockDevice for FaultInjectingBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_through_the_fault_plan(offset, buffer, Inner::read_at)
    }

    /// 与普通读同一条注入判定与记账，交下去的是里面那块盘的重读。
    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_through_the_fault_plan(offset, buffer, Inner::reread_at_after_a_bad_read)
    }

    /// 与普通读同一条注入判定与记账，交下去的是里面那块盘的这一刻读盘。
    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_through_the_fault_plan(offset, buffer, Inner::read_at_straight_from_the_disk)
    }

    /// 通知不是读、不碰盘：不走注入判定、不占调用序号、不记读，原样转给里面那块盘。
    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        self.inner.keep_the_self_verified_reread(offset, bytes);
    }

    /// 通知不是读、不碰盘：不走注入判定、不占调用序号、不记读，原样转给里面那块盘。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        self.inner
            .note_the_read_failing_self_verification_and_forget_its_bytes(offset, length_in_bytes);
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("写的长度装得进 u64");
        match self
            .plan
            .decide(FaultCallKind::Write, self.device, offset, length)
        {
            Some(InjectedFault::WriteFails) => return Err(injected_block_device_error("写")),
            // 吞掉：报成功，一个字节都不落盘，也不记进这块盘的计数（真的没写）；本该写下什么另记一笔，判说谎的设备留下了什么时用。
            Some(InjectedFault::WriteIsSwallowed) => {
                self.plan.note_swallowed_write(
                    self.device,
                    offset,
                    SwallowedContents::Bytes(bytes.to_vec()),
                    durability,
                );
                return Ok(());
            }
            Some(
                InjectedFault::ReadFails
                | InjectedFault::ReadReturnsCorruptedBytes { .. }
                | InjectedFault::BarrierFails
                | InjectedFault::BarrierIsSwallowed,
            )
            | None => {}
        }
        self.inner.write_at(offset, bytes, durability)?;
        self.plan
            .note_forwarded_write(self.device, length, durability);
        Ok(())
    }

    /// 写零与写走同一条注入判定（`FaultCallKind::Write`）：对故障计划来说它就是一次写，
    /// 落点是 `offset`、长度是 `length`。
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        match self
            .plan
            .decide(FaultCallKind::Write, self.device, offset, length)
        {
            Some(InjectedFault::WriteFails) => return Err(injected_block_device_error("写零")),
            Some(InjectedFault::WriteIsSwallowed) => {
                // 吞掉：报成功，一个字节都不清零，也不记进这块盘的计数（真的没写）。
                self.plan.note_swallowed_write(
                    self.device,
                    offset,
                    SwallowedContents::Zeroes { length },
                    WriteDurability::Plain,
                );
                return Ok(());
            }
            Some(
                InjectedFault::ReadFails
                | InjectedFault::ReadReturnsCorruptedBytes { .. }
                | InjectedFault::BarrierFails
                | InjectedFault::BarrierIsSwallowed,
            )
            | None => {}
        }
        self.inner.write_zeroes_at(offset, length)?;
        // 清零永远是普通写（块设备抽象里它没有 FUA 那一档），设备一层按普通写记它的调用数与字节数。
        self.plan
            .note_forwarded_write(self.device, length, WriteDurability::Plain);
        Ok(())
    }

    /// discard 与写零一样走写的注入判定（`FaultCallKind::Write`，落点 `offset`、长度 `length`）：崩溃点重放按「范围写零」看它。
    /// `WriteFails` 让它报错、一个字节不动；`WriteIsSwallowed` 让它报「交给了底层」、其实没交下去（另记一笔被吞的写）；
    /// 不注入就原样转给里面那块盘，交回它的结局。
    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        // 长度 0 没有动作（`BlockDevice::discard_at` 的文档：后端照「交给了底层」交回、不记数）：注入层同一口径——不走注入判定、
        // 不占整池调用序号、不数 discard、不数整池转下去的写（录制器那一侧同样不给它记一步），只交给里面那块盘核落点、交回它的结局。
        if length == 0 {
            return self.inner.discard_at(offset, length);
        }
        match self
            .plan
            .decide(FaultCallKind::Write, self.device, offset, length)
        {
            Some(InjectedFault::WriteFails) => {
                return Err(injected_block_device_error("discard"));
            }
            Some(InjectedFault::WriteIsSwallowed) => {
                self.plan.note_swallowed_write(
                    self.device,
                    offset,
                    SwallowedContents::Discard { length },
                    WriteDurability::Plain,
                );
                return Ok(DiscardOutcome::Applied);
            }
            Some(
                InjectedFault::ReadFails
                | InjectedFault::ReadReturnsCorruptedBytes { .. }
                | InjectedFault::BarrierFails
                | InjectedFault::BarrierIsSwallowed,
            )
            | None => {}
        }
        let outcome = self.inner.discard_at(offset, length)?;
        self.plan.note_forwarded_discard(self.device, outcome);
        Ok(outcome)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.plan.note_barrier_arrived(self.device);
        match self.plan.decide(
            FaultCallKind::Barrier,
            self.device,
            DeviceOffsetInBytes(0),
            0,
        ) {
            Some(InjectedFault::BarrierFails) => return Err(injected_block_device_error("屏障")),
            Some(InjectedFault::BarrierIsSwallowed) => {
                self.plan.note_swallowed_barrier(self.device);
                return Ok(());
            }
            Some(
                InjectedFault::WriteFails
                | InjectedFault::WriteIsSwallowed
                | InjectedFault::ReadFails
                | InjectedFault::ReadReturnsCorruptedBytes { .. },
            )
            | None => {}
        }
        self.inner.barrier()?;
        self.plan.note_forwarded_barrier(self.device);
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    /// 提示照里面那块盘答：注入的故障不改哪些槽可能有记录——吞掉或失败的写在里面那块盘上没留字节，全环扫描在那一槽读到的也是 0；
    /// 读失败与读到翻位的槽，按提示扫与全环扫描读到的是同一槽的同一个结果（没写过的槽读出来是 0，翻一位过不了 journal 的校验）。
    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_pool::SparseBlockDevice;

    /// 用例里的几何：与 E142 装置相同（物理块 512、io_min 512 ⇒ 固定结构槽距 4096），journal 环取默认的 768 MiB。
    const TEST_GEOMETRY: FixedGeometry = FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    };

    const DEVICE_BYTES: u64 = 4 << 30;
    /// 单元区里的一个落点（第一个单元槽，`UNIT_AREA_START_SLOT * SLOT_BYTES`）：分类成「写单元」。
    const UNIT_AREA_OFFSET: DeviceOffsetInBytes =
        DeviceOffsetInBytes(singlefs_format::UNIT_AREA_START_SLOT * singlefs_format::SLOT_BYTES);

    fn two_devices(
        plan: &SharedFaultPlan,
    ) -> Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> {
        (0..2u32)
            .map(|device_number| {
                let identity = DeviceIdentity(device_number);
                (
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                        plan.clone(),
                    ),
                )
            })
            .collect()
    }

    fn write(
        device: &mut FaultInjectingBlockDevice<SparseBlockDevice>,
        offset: u64,
        byte: u8,
    ) -> Result<(), BlockDeviceError> {
        device.write_at(
            DeviceOffsetInBytes(offset),
            &[byte; 512],
            WriteDurability::Plain,
        )
    }

    fn read(device: &FaultInjectingBlockDevice<SparseBlockDevice>, offset: u64) -> [u8; 512] {
        let mut buffer = [0u8; 512];
        device
            .read_at(DeviceOffsetInBytes(offset), &mut buffer)
            .expect("这次读没注入");
        buffer
    }

    /// 长度 0 的 discard 没有动作（`BlockDevice::discard_at` 的文档：后端照「交给了底层」交回、不记数）：注入层同一口径——不走注入判定、
    /// 不占整池调用序号、不数 discard。计划排在「整池第 1 次写报错」时，长度 0 的 discard 不吃掉那一次，下一个真写才报错。
    #[test]
    fn zero_length_discard_answers_applied_and_neither_counts_nor_takes_an_injection() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, 1),
        );
        let mut devices = two_devices(&plan);
        assert_eq!(
            devices[0]
                .1
                .discard_at(UNIT_AREA_OFFSET, 0)
                .expect("长度 0 的 discard 不报错"),
            DiscardOutcome::Applied
        );
        assert_eq!(
            plan.calls(),
            FaultCallCounts {
                reads: 0,
                writes: 0,
                barriers: 0
            },
            "不占整池调用序号"
        );
        assert_eq!(
            plan.counts_of_device(DeviceIdentity(0)),
            FaultDeviceCounts::default(),
            "不数 discard、不数写"
        );
        assert!(plan.fired().is_empty(), "注入点没被它吃掉");
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 1).expect_err("整池第 1 次写才报错");
        assert_eq!(plan.fired().len(), 1);
    }

    /// 计划不开时只数调用：写进去的字节原样读得回来，每块盘的计数与发出的调用一一对上。
    #[test]
    fn an_unarmed_plan_forwards_every_call_and_counts_it() {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, 0, 7).expect("写");
        write(&mut devices[1].1, 512, 9).expect("写");
        devices[0].1.barrier().expect("屏障");
        assert_eq!(read(&devices[0].1, 0), [7u8; 512]);
        assert_eq!(
            plan.calls(),
            FaultCallCounts {
                reads: 1,
                writes: 2,
                barriers: 1
            }
        );
        assert_eq!(
            plan.counts_of_device(DeviceIdentity(0)),
            FaultDeviceCounts {
                reads: 1,
                writes: 1,
                written_bytes: 512,
                force_unit_access_writes: 0,
                barriers_forwarded: 1,
                barriers_swallowed: 0,
                discards_applied: 0,
                discards_answered_unsupported: 0,
            }
        );
        assert!(plan.fired().is_empty(), "没开计划就不该注入");
    }

    /// 「整池第 n 次写报错」：前面的照做、第 n 次报错、第 n + 1 次又照做（一次性）；报错那次一个字节都没落盘。
    #[test]
    fn the_nth_write_across_the_pool_fails_once_and_writes_nothing() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, 2),
        );
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, 0, 1).expect("第 1 次写照做");
        let error = write(&mut devices[1].1, 0, 2).expect_err("第 2 次写报错");
        assert!(matches!(error, BlockDeviceError::InputOutput(_)));
        write(&mut devices[1].1, 512, 3).expect("第 3 次写照做");
        assert_eq!(
            read(&devices[1].1, 0),
            [0u8; 512],
            "报错那次一个字节都没落盘"
        );
        assert_eq!(read(&devices[1].1, 512), [3u8; 512]);
        let fired = plan.fired();
        assert_eq!(fired.len(), 1, "一次性：只注入一次");
        assert_eq!(fired[0].device, DeviceIdentity(1));
        assert_eq!(fired[0].call_ordinal_across_the_pool, 2);
        assert_eq!(
            fired[0].written_structure,
            Some(StepKind::SystemConfigurationSlot),
            "偏移 0 落在系统配置两槽里"
        );
    }

    /// 「第 n 次起每一次都注入」加「只这块盘」加「偏移小于某个数」：三样选择合起来只挑得中该挑的那几次。
    #[test]
    fn every_call_from_the_second_onward_on_one_device_below_an_offset() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule {
                fault: InjectedFault::WriteFails,
                device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
                placement: FaultPlacement::OffsetBelow(8192),
                counting: FaultCounting::PerDevice,
                occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(2),
            },
        );
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, 0, 1).expect("盘 0 不在选中的盘里");
        write(&mut devices[1].1, UNIT_AREA_OFFSET.0, 2).expect("落点不在 8192 之下");
        write(&mut devices[1].1, 0, 3).expect("盘 1 第 1 次命中：还没到第 2 次");
        write(&mut devices[1].1, 4096, 4).expect_err("盘 1 第 2 次命中：报错");
        write(&mut devices[1].1, 0, 5).expect_err("第 2 次起每一次都报错");
        assert_eq!(plan.fired().len(), 2);
    }

    /// 读回改坏的字节：报成功、缓冲区里那一位被翻掉，盘上那一份一个字节都没变。
    #[test]
    fn corrupted_read_flips_one_bit_in_the_buffer_and_leaves_the_device_alone() {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 0b0000_0001).expect("写");
        plan.arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::ReadReturnsCorruptedBytes {
                flipped_bit: FlippedBit {
                    byte_index: 3,
                    bit_index: 2,
                },
            },
            1,
        ));
        let corrupted = read(&devices[0].1, UNIT_AREA_OFFSET.0);
        assert_eq!(corrupted[3], 0b0000_0101, "第 3 个字节的第 2 位被翻掉");
        assert_eq!(corrupted[2], 0b0000_0001, "别的字节不动");
        plan.disarm();
        assert_eq!(
            read(&devices[0].1, UNIT_AREA_OFFSET.0),
            [0b0000_0001u8; 512],
            "盘上那一份没被改坏"
        );
    }

    /// 吞掉一次写：报成功，盘上一个字节都没变（设备说谎那一形）。
    #[test]
    fn swallowed_write_reports_success_and_changes_nothing_on_the_device() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteIsSwallowed, 1),
        );
        let mut devices = two_devices(&plan);
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 6).expect("报成功");
        assert_eq!(
            read(&devices[0].1, UNIT_AREA_OFFSET.0),
            [0u8; 512],
            "其实一个字节都没落盘"
        );
        assert_eq!(
            plan.counts_of_device(DeviceIdentity(0)).writes,
            0,
            "吞掉的写不算这块盘真发生过的写"
        );
    }

    /// 吞掉一道屏障：报成功，里面那块盘没收到；数进 `barriers_swallowed`。
    #[test]
    fn swallowed_barrier_reports_success_and_is_counted_apart() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::BarrierIsSwallowed, 1),
        );
        let mut devices = two_devices(&plan);
        devices[0].1.barrier().expect("报成功");
        devices[0].1.barrier().expect("第二道照发");
        let counts = plan.counts_of_device(DeviceIdentity(0));
        assert_eq!(counts.barriers_swallowed, 1);
        assert_eq!(counts.barriers_forwarded, 1);
    }

    /// 每块盘收到过写之后的第一道屏障那一刻的计数：屏障本身不算进去，之后的调用也不算；写之前收到的屏障（可写挂载里取号写之前那一道）
    /// 不拍快照。
    #[test]
    fn the_counts_when_the_first_barrier_after_a_write_arrived_are_frozen_at_that_moment() {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        devices[0].1.barrier().expect("写之前的一道屏障");
        assert_eq!(
            plan.counts_when_the_first_barrier_after_a_write_arrived_at(DeviceIdentity(0)),
            None,
            "写之前的屏障不拍快照"
        );
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 1).expect("写");
        devices[0].1.barrier().expect("写之后第一道屏障");
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0 + 512, 2).expect("写");
        devices[0].1.barrier().expect("写之后第二道屏障");
        let frozen = plan
            .counts_when_the_first_barrier_after_a_write_arrived_at(DeviceIdentity(0))
            .expect("写之后收到过屏障");
        assert_eq!(frozen.writes, 1);
        assert_eq!(
            frozen.barriers_forwarded, 1,
            "写之前那一道算进去，这一道本身不算进去"
        );
        assert_eq!(plan.counts_of_device(DeviceIdentity(0)).writes, 2);
    }

    /// 按写数找屏障那一刻的计数：写数到了之后进来的第一道屏障，屏障本身不算进去；两道屏障之间没有新写的那一道不拍快照，
    /// 问的写数还没到、或到了之后还没收到屏障时没有。可写挂载取号写两轮，挂载窗口从写之后那一道屏障起，问的就是写数 2。
    #[test]
    fn the_counts_when_the_first_barrier_after_this_many_writes_arrived_are_frozen_at_that_moment()
    {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        devices[0].1.barrier().expect("写之前的一道屏障");
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 1).expect("取号第一轮那一写");
        devices[0].1.barrier().expect("第一轮之后的屏障");
        devices[0].1.barrier().expect("中间没有新写的一道屏障");
        assert_eq!(
            plan.counts_when_the_first_barrier_after_this_many_writes_arrived_at(
                DeviceIdentity(0),
                2
            ),
            None,
            "第二次写还没来"
        );
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0 + 512, 2).expect("取号第二轮那一写");
        devices[0].1.barrier().expect("第二轮之后的屏障");
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0 + 1024, 3).expect("之后的写");
        devices[0].1.barrier().expect("之后的屏障");
        let after_one_write = plan
            .counts_when_the_first_barrier_after_this_many_writes_arrived_at(DeviceIdentity(0), 1)
            .expect("一次写之后收到过屏障");
        assert_eq!(
            Some(after_one_write),
            plan.counts_when_the_first_barrier_after_a_write_arrived_at(DeviceIdentity(0)),
            "写数取 1 与写之后第一道屏障同一刻"
        );
        assert_eq!(
            (after_one_write.writes, after_one_write.barriers_forwarded),
            (1, 1),
            "第一轮之后那一刻：一次写，写之前那一道屏障"
        );
        let after_two_writes = plan
            .counts_when_the_first_barrier_after_this_many_writes_arrived_at(DeviceIdentity(0), 2)
            .expect("两次写之后收到过屏障");
        assert_eq!(
            (after_two_writes.writes, after_two_writes.barriers_forwarded),
            (2, 3),
            "第二轮之后那一刻：两次写，之前三道屏障（中间没有新写那一道也发了），这一道本身不算进去"
        );
        assert_eq!(
            plan.counts_when_the_first_barrier_after_this_many_writes_arrived_at(
                DeviceIdentity(0),
                4
            ),
            None,
            "第四次写还没来"
        );
        assert_eq!(
            plan.counts_when_the_first_barrier_after_this_many_writes_arrived_at(
                DeviceIdentity(1),
                1
            ),
            None,
            "另一块盘一次写都没收到"
        );
    }

    /// 屏障报错：交回块设备错，里面那块盘没收到。
    #[test]
    fn failing_barrier_returns_the_block_device_error() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::every_call_across_the_pool(InjectedFault::BarrierFails),
        );
        let mut devices = two_devices(&plan);
        let error = devices[0].1.barrier().expect_err("屏障报错");
        assert!(matches!(error, BlockDeviceError::InputOutput(_)));
        assert_eq!(
            plan.counts_of_device(DeviceIdentity(0)).barriers_forwarded,
            0
        );
    }

    /// 读报错：交回块设备错，缓冲区不动。
    #[test]
    fn failing_read_returns_the_block_device_error_and_leaves_the_buffer_alone() {
        let plan = SharedFaultPlan::armed(
            TEST_GEOMETRY,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::ReadFails, 1),
        );
        let devices = two_devices(&plan);
        let mut buffer = [0xabu8; 512];
        let error = devices[0]
            .1
            .read_at(UNIT_AREA_OFFSET, &mut buffer)
            .expect_err("读报错");
        assert!(matches!(error, BlockDeviceError::InputOutput(_)));
        assert_eq!(buffer, [0xabu8; 512], "报错的读不动缓冲区");
    }

    /// 哪两种是「设备说谎」：报错那四种的盘面不一致照算新发现，说谎那两种单记一格
    /// （`InjectedFault::the_device_lies` 分的就是这一刀）。
    #[test]
    fn only_the_two_swallowing_faults_report_success_without_doing_anything() {
        for (fault, lies) in [
            (InjectedFault::WriteFails, false),
            (InjectedFault::ReadFails, false),
            (
                InjectedFault::ReadReturnsCorruptedBytes {
                    flipped_bit: FlippedBit {
                        byte_index: 0,
                        bit_index: 0,
                    },
                },
                false,
            ),
            (InjectedFault::BarrierFails, false),
            (InjectedFault::WriteIsSwallowed, true),
            (InjectedFault::BarrierIsSwallowed, true),
        ] {
            assert_eq!(fault.the_device_lies(), lies, "{}", fault.name());
        }
    }

    /// 种子数接近 u64 上限时切片照样首尾相接、不重不漏（片号 × 种子数在 u128 里乘，u64 里乘会溢出）。
    #[test]
    fn seed_slices_near_the_top_of_u64_still_tile_the_seed_interval() {
        let seed_count = u64::MAX / 2 + 5;
        let slices = seed_slices(seed_count, 3);
        assert_eq!(slices.len(), 12, "片数取 4 × 线程数");
        assert_eq!(slices.first().map(|slice| slice.start), Some(0));
        assert_eq!(slices.last().map(|slice| slice.end), Some(seed_count));
        for pair in slices.windows(2) {
            assert_eq!(pair[0].end, pair[1].start, "首尾相接");
        }
    }

    /// 种子区间切片：片数取 min(种子数, 4 × 线程数)，首尾相接、不重不漏。
    #[test]
    fn seed_slices_cover_every_seed_exactly_once() {
        for (seed_count, threads) in [(0u64, 4usize), (1, 4), (7, 2), (100, 8)] {
            let slices = seed_slices(seed_count, threads);
            let covered: Vec<u64> = slices.iter().flat_map(std::clone::Clone::clone).collect();
            assert_eq!(covered, (0..seed_count).collect::<Vec<u64>>());
        }
    }
}
