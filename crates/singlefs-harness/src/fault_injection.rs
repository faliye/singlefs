//! 故障注入（里程碑「覆盖写、释放、回退与复用」增补 3 第 4 件）：一个通用的设备包装，按种子让任意一次写、读、刷盘返回 `BlockDeviceError`，
//! 或让一次读返回改坏的字节。被测的性质有两条：注入之后 `singlefs-core` 那一侧**返回错误而不是 panic**，
//! 出错之后**重开要恢复到模型允许的版本**。
//!
//! [`FaultInjectingBlockDevice`] 本身不定包在录制器的哪一边，录制流记的是什么由搭栈的一方定。两条要紧的栈次序相反：
//! * 这一件（历史里的盘，`crate::history::HistoryDevice`）：注入在录制器**外面**（调用方与录制器之间）。报错的写、被吞掉的写与
//!   被吞掉的屏障都不进录制流，录制流因此恒等于真正落到内存盘上的那一串——注入之后的镜像可以从录制流重建
//!   （`crate::memory_pool::MemoryPool::apply`），不必把执行器里的两块盘再交出来一份。
//! * 门禁 checker-tier-qemu-device-streams 的装置（`src/bin/new_pool_file_creation_on_device.rs` 的 `CountedDevice`）：录制器在**外面**、注入在里面。录制流记的是
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
//! 第 40 行那一格的判别力由写死的那条用例钉（`tests/fault_injection_fast_tier.rs`）。

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
    crash_recovery_disagreement, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreementAspect,
    ModelInstanceRow, ModelReportedUnreadablePlace, ModelRingPosition, ModelRootKey,
    ModelUnitCopyLocation, ObservedReadBack, ObservedSnapshotReadBack,
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

/// 故障注入的工作线程数从这个环境变量取（十进制正整数）；没设就取每台默认重活线程数
/// （[`crate::heavy_threads::default_heavy_threads`]：`SINGLEFS_THREAD_CAP`，再没设取登记值）。
pub const FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_FAULT_INJECTION_WORKER_THREADS";

/// 每个工作线程摊到的片数：各段历史的长短差得远，片切得比线程多，先跑完的线程接着领下一片。
const SLICES_PER_WORKER_THREAD: usize = 4;

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
    /// 真设备那一路的挂载窗口从第二轮取号写之后那一道屏障起，按写数在这里找。
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

/// 注入点落在这段历史的哪一段上：起点段，还是起点之后的第几步（那一步是哪一类操作）。
///
/// 起点段是 `HistoryPool::start` 里的 mkfs、取号、暖机、第一个文件，在录制流里排在第一个操作之前。
/// 判决第二节改法二把它放开：在那之前 `draw_faults` 从第 1 步起算，起点段结构性地一次都摆不到
/// （判定五：49 次 panic 全在 `history.rs` 起点段那几处 `expect` 上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultedSegment {
    TheStartingPoint,
    Operation {
        /// 起点之后的第几步，从 0 数。
        step_index: usize,
        operation_kind: HistoryOperationKind,
    },
}

impl FaultedSegment {
    /// 这一段在执行器里的位置，拿去与 `FailureObservation::position` 比。
    #[must_use]
    pub const fn position(self) -> StepPosition {
        match self {
            FaultedSegment::TheStartingPoint => StepPosition::StartingPoint,
            FaultedSegment::Operation { step_index, .. } => StepPosition::Operation(step_index),
        }
    }

    /// 这一段是哪一类操作；起点段不是历史里的一步，没有这一项。
    #[must_use]
    pub const fn operation_kind(self) -> Option<HistoryOperationKind> {
        match self {
            FaultedSegment::TheStartingPoint => None,
            FaultedSegment::Operation { operation_kind, .. } => Some(operation_kind),
        }
    }

    /// 跨段可比的那一半（不带步号）：注入点按它分类。
    #[must_use]
    pub const fn kind(self) -> FaultedSegmentKind {
        match self {
            FaultedSegment::TheStartingPoint => FaultedSegmentKind::TheStartingPoint,
            FaultedSegment::Operation { operation_kind, .. } => {
                FaultedSegmentKind::Operation(operation_kind)
            }
        }
    }

    #[must_use]
    pub fn render(self) -> String {
        match self {
            FaultedSegment::TheStartingPoint => {
                "起点段（mkfs、取号、暖机、第一个文件）".to_string()
            }
            FaultedSegment::Operation {
                step_index,
                operation_kind,
            } => format!("第 {step_index} 步（{operation_kind:?}）"),
        }
    }
}

/// 注入点分类用的「哪一段」：起点段，或起点之后的某一类操作。不带步号，几段历史之间可比。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultedSegmentKind {
    TheStartingPoint,
    Operation(HistoryOperationKind),
}

impl FaultedSegmentKind {
    /// 报告里从上到下的次序：起点段排在七类操作之前（它在历史里也排在最前面）。
    #[must_use]
    pub fn every_kind_in_report_order() -> Vec<FaultedSegmentKind> {
        let mut kinds = vec![FaultedSegmentKind::TheStartingPoint];
        kinds.extend(
            HistoryOperationKind::ALL
                .into_iter()
                .map(FaultedSegmentKind::Operation),
        );
        kinds
    }

    #[must_use]
    pub fn render(self) -> String {
        match self {
            FaultedSegmentKind::TheStartingPoint => "TheStartingPoint".to_string(),
            FaultedSegmentKind::Operation(operation_kind) => format!("{operation_kind:?}"),
        }
    }
}

/// 一段历史上注入哪一次调用：注入什么，加上那次调用是整池第几次这一种调用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DrawnFault {
    pub fault: InjectedFault,
    /// 整池第几次这一种调用（1 起）。
    pub call_ordinal: u64,
    /// 这次调用落在哪一段里。
    pub segment: FaultedSegment,
}

impl DrawnFault {
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "{}：整池第 {} 次{}，落在{}",
            self.fault.name(),
            self.call_ordinal,
            self.fault.call_kind().name(),
            self.segment.render()
        )
    }
}

/// 一个注入点：哪一段上的哪一种调用（写再按落点分成哪一类结构）。验收要的「每个注入点的命中次数」按它数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InjectionPoint {
    pub segment: FaultedSegmentKind,
    pub call: FaultCallKind,
    /// 写落在哪一类结构上；读与屏障没有这一项。
    pub written_structure: Option<StepKind>,
}

/// 写落在哪一类结构上，四种（`StepKind::Barrier` 不是写）。
const WRITTEN_STRUCTURES: [StepKind; 4] = [
    StepKind::UnitWrite,
    StepKind::JournalRecord,
    StepKind::RootRecordFua,
    StepKind::SystemConfigurationSlot,
];

impl InjectionPoint {
    /// 全部注入点：（起点段 + 七类操作） × （四类写落点 + 读 + 屏障）。里面有几格在今天的实现上摆不出来
    /// （冷启动只读不写、零单元发布不写单元），报告里按「没命中」列名，不当失败。
    #[must_use]
    pub fn every_injection_point() -> Vec<InjectionPoint> {
        let mut points = Vec::new();
        for segment in FaultedSegmentKind::every_kind_in_report_order() {
            for structure in WRITTEN_STRUCTURES {
                points.push(InjectionPoint {
                    segment,
                    call: FaultCallKind::Write,
                    written_structure: Some(structure),
                });
            }
            for call in [FaultCallKind::Read, FaultCallKind::Barrier] {
                points.push(InjectionPoint {
                    segment,
                    call,
                    written_structure: None,
                });
            }
        }
        points
    }

    #[must_use]
    pub fn render(&self) -> String {
        match self.written_structure {
            Some(structure) => format!("{}/{}", self.segment.render(), structure.name()),
            None => format!("{}/{}", self.segment.render(), self.call.name()),
        }
    }
}

/// 注入之后重开走到的那一版落在哪一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReopenedVersion {
    /// 走到的是模型认下来的某一版（注入那一段历史跑完时模型根环里的那几条，`HistoryRun::committed_versions_at_the_end`）。
    CommittedByTheModel,
    /// 模型没认这一版，但它正是不注入时失败那一步会写出的那一版（增补 2 收口表第 40 行 / C381 在争的那一格；这里只记不判）。
    TheVersionTheFaultedStepWasWriting,
    /// 两样都不是：`crash_recovery_disagreement` 判出对不上。
    OutsideEverythingTheModelAllows,
    /// 重开走读失败，而这一次注入之后盘上本来就没有一版立得住（`failed_reopen_is_the_right_answer`）：
    /// 走读失败正是要的结果，只记不判。悄悄读回一版模型从没提交过的内容照样判红。
    NothingWasLeftToRecoverAndTheReopenSaidSo,
}

/// 注入之后这一次怎么收尾。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultOutcome {
    /// 注入那一步返回了 `Err`（或冷启动读回报错），历史到此为止：要的就是这一条。
    SurfacedAsAnError,
    /// 注入了，那一步照样做成了（读错被容错、吞掉的写没被用到），整段历史跑完。
    ToleratedAndTheHistoryFinished,
    /// 注入了，历史却停在别的东西上（不是注入那一步返回的错）：这一格逐条分类，有真缺口就在这里。
    StoppedOnSomethingOtherThanTheInjectedError,
    /// 注入点没走到（这一段历史在注入那一次调用之前就停了）。
    NeverFired,
}

impl FaultOutcome {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            FaultOutcome::SurfacedAsAnError => "返回了错误",
            FaultOutcome::ToleratedAndTheHistoryFinished => "被容错、历史跑完",
            FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError => "历史停在别的东西上",
            FaultOutcome::NeverFired => "没走到",
        }
    }
}

/// 一次注入上的一条失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultFinding {
    pub signature: FailureSignature,
    pub seed: HistorySeed,
    pub drawn: DrawnFault,
    pub observation: FailureObservation,
    /// 重开走到了哪一版（判红的那一次才带）。
    pub reopen: String,
}

impl FaultFinding {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "种子 {} {}：{:?}\n",
            self.seed.0,
            self.drawn.render(),
            self.signature
        );
        let _ = writeln!(text, "  重开：{}", self.reopen);
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic：{} —— {}", panic.location, panic.message);
        }
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        text
    }
}

/// 只有故障注入撞得到的「已知红」形态：随机历史不注入，撞不到；两边共用的那份清单是 `history::KNOWN_RED_FORMS`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnownRedFormOnlyAfterAFault {
    /// 起点段里 mkfs 自己把错交回来，录制流里一次系统配置槽写都没落盘；注入之后的镜像上池级 checker 只判红
    /// I-8.5（tail 槽有效集非空）一条，重开走读失败（[`ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo`]）。
    /// 用户 2026-10-03 定：盘上全池没有一个自证过的系统配置槽，就是 I-8.5 那一行写的「全池皆废」，红是对的，照判、记在这里；
    /// 它不是要修的缺口。
    MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
}

impl KnownRedFormOnlyAfterAFault {
    /// 报告里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded => {
                "mkfs 在系统配置槽落盘之前报错，全池皆废、只红 I-8.5"
            }
        }
    }
}

/// 一次注入以「已知红」收尾时撞上的是哪一条：两边共用的清单里第几条，或只有故障注入撞得到的那几条之一。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnownRedFormAtAFault {
    /// `history::KNOWN_RED_FORMS` 的第几条（从 0 数）。
    SharedWithRandomHistories(usize),
    OnlyAfterAFault(KnownRedFormOnlyAfterAFault),
}

impl KnownRedFormAtAFault {
    /// 报告里的一段：共用清单写第几条，故障注入自己那几条写名字。
    #[must_use]
    pub fn render(self) -> String {
        match self {
            KnownRedFormAtAFault::SharedWithRandomHistories(form) => format!("第 {form} 条"),
            KnownRedFormAtAFault::OnlyAfterAFault(form) => {
                format!("故障注入那一条（{}）", form.name())
            }
        }
    }
}

/// 判故障注入自己那几条「已知红」要看的事实，都是 [`inject_one_fault_into_the_segment`] 一路判到这一步时本来就拿着的：不另读盘、不另跑实现。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactsAfterTheFault {
    /// 注入那一段历史停在起点段某一步把错交回来上（`HarnessJudgement::TheStartingPointReturnedAnError`）时是哪一步；
    /// 别的结局（跑完了、停在起点之后的某一步、停在 panic 上）是 None。
    pub starting_point_step_that_returned_an_error: Option<StartingPointStep>,
    /// 录制流（只记真正落到盘上的写）里写在系统配置槽上的写有几次（[`StepKind::SystemConfigurationSlot`]；整段清零不算）。
    pub system_configuration_slot_writes_that_landed: usize,
    /// 重开走到了哪一格。
    pub reopened: ReopenedVersion,
}

/// 这条失败是不是只有故障注入撞得到的那几条「已知红」之一；不是就交回 None，照共用清单与新发现往下判。
///
/// mkfs 没做完那一条要五样同时成立，少一样就不认（照报新发现，不放宽）：
/// 1. 停在起点段、把错交回来的是 mkfs 自己（取号、暖机、第一个文件报错时 mkfs 已经写下了系统配置槽）；
/// 2. 录制流里一次系统配置槽写都没落盘（mkfs 写完系统配置之后才报错的，盘上有自证过的槽，I-8.5 红就不是这一形）；
/// 3. 重开走读失败、而盘上本来就没有一版立得住；
/// 4. 这条失败里没有 panic、执行器判出的失败、模型对不上、记录核对器判红；
/// 5. checker（连同 RefFS 借违例那一栏带的那一票）判红的恰好一条，就是 I-8.5。
#[must_use]
pub fn known_red_form_only_after_a_fault(
    observation: &FailureObservation,
    facts: &FactsAfterTheFault,
) -> Option<KnownRedFormOnlyAfterAFault> {
    let make_filesystem_returned_the_error = match facts.starting_point_step_that_returned_an_error
    {
        Some(StartingPointStep::MakeFilesystem) => true,
        Some(
            StartingPointStep::ReadThePoolForTheSession
            | StartingPointStep::AcquireInstance
            | StartingPointStep::WarmUp
            | StartingPointStep::PublishFirstFile,
        )
        | None => false,
    };
    let no_system_configuration_slot_landed =
        facts.system_configuration_slot_writes_that_landed == 0;
    let the_reopen_found_nothing_to_recover = match facts.reopened {
        ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo => true,
        ReopenedVersion::CommittedByTheModel
        | ReopenedVersion::TheVersionTheFaultedStepWasWriting
        | ReopenedVersion::OutsideEverythingTheModelAllows => false,
    };
    let only_the_checker_flagged_it = observation.panic.is_none()
        && observation.harness_judgement.is_none()
        && observation.model_disagreement.is_none()
        && crate::history::record_check_aspects(&observation.record_check).is_empty();
    let only_some_device_carries_a_self_verified_system_configuration_is_violated = matches!(
        observation.violations.as_slice(),
        [(invariant, _)] if *invariant == SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
    );
    (make_filesystem_returned_the_error
        && no_system_configuration_slot_landed
        && the_reopen_found_nothing_to_recover
        && only_the_checker_flagged_it
        && only_some_device_carries_a_self_verified_system_configuration_is_violated)
        .then_some(
            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
        )
}

/// 「已知红」收尾的一次注入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRedAtAFault {
    pub seed: HistorySeed,
    pub form: KnownRedFormAtAFault,
    pub drawn: DrawnFault,
}

/// 跑过的注入的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FaultInjectionTally {
    pub histories: u64,
    /// 测量跑（不注入）就以「已知红」收尾的段数：注入点只摆在测量跑跑完的那几步里。
    pub histories_whose_measurement_run_stopped_early: u64,
    pub histories_without_any_injection_point: u64,
    pub faults: u64,
    pub faults_by_kind: BTreeMap<&'static str, u64>,
    pub faults_by_outcome: BTreeMap<&'static str, u64>,
    pub faults_by_injection_point: BTreeMap<String, u64>,
    /// 注入之后整段历史里 `singlefs-core` panic 的次数：这是被测的性质，一次都不许有。
    pub faults_that_panicked: u64,
    /// 说谎的设备（[`InjectedFault::the_device_lies`]）吞掉写之后留下的失败、按注入点认得出是它留下的次数
    /// （[`left_by_the_lying_device`]）：不算新发现，按签名列进报告。认不出的不进这一格，照报成新发现。
    /// 一次都没有说明说谎那两种注得进去却什么也没碰着，判别力要重新量。
    pub faults_where_a_lying_device_left_the_image_inconsistent: u64,
    /// 上一格逐条的签名：判红的是哪几条不变量 / 哪一处模型对不上。
    pub lying_device_signatures: BTreeMap<String, u64>,
    /// 注入那一步返回的错误成员（`StepOutcome::Refused` 的成员名）。
    pub refusal_members: BTreeMap<String, u64>,
    /// 起点段哪一步把错交回来了（`StartingPointStep::name`）：改法二把 `HistoryPool::start` 那四处 `expect`
    /// 改成可失败之后才有这一格，改之前起点段一次都摆不到。
    pub starting_point_failures: BTreeMap<&'static str, u64>,
    /// 取号之后挂载报错、新号没回卷的次数，按 [`AcquiredInstanceLeftAfterAFailedMount::name`] 分（C378；认下的已知行为，
    /// 不算新发现，在报告里点名）。
    pub acquired_instances_left_after_a_failed_mount: BTreeMap<&'static str, u64>,
    pub reopens: u64,
    pub reopens_reading_a_file: u64,
    pub reopens_without_a_file: u64,
    pub reopens_that_failed: u64,
    pub reopened_into_a_version_the_model_committed: u64,
    /// 重开走到了失败那一步正在写、模型没提交的那一版（收口表第 40 行那一格；只记不判）。
    pub reopened_into_the_version_the_faulted_step_was_writing: u64,
    pub reopens_outside_everything_the_model_allows: u64,
    /// 重开走读失败，而盘上本来就没有一版立得住（`failed_reopen_is_the_right_answer`）：只记不判。
    pub reopens_that_correctly_found_nothing_to_recover: u64,
    /// 重开上 RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计；RefFS 弃权的按理由另计。
    pub reopens_by_the_reference_filesystem_and_model_verdicts:
        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
    pub reference_filesystem_abstentions_at_the_reopen: BTreeMap<&'static str, u64>,
    /// harness 在注入之后重开的盘上就地跑池级 checker 的次数：那一遍归 checker 档两段，harness 一次都不跑，这一格恒为 0
    /// （用例照它核 harness 没把就地判偷回来）；下面三格（判绿、不适用、部分没判的条次）同样恒空。
    pub checker_runs_on_the_image_after_the_fault: u64,
    /// 注入之后重开的盘经钩子交出去的次数（[`ImagesAfterAFault`]，每个注入点一次；池级 checker 那一遍归 checker 档两段）。
    /// 合并入口（[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`]）交给判的那几次不记在这一格。
    pub images_after_the_fault_handed_out: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    /// 判成「部分没判」（`singlefs_checker::image::InvariantVerdict::PartiallyJudged`）的条次：不红，也不算判绿。
    pub invariant_partially_judged: BTreeMap<&'static str, u64>,
    pub faults_ending_known_red: BTreeMap<KnownRedFormAtAFault, u64>,
    pub faults_ending_new_finding: u64,
}

impl FaultInjectionTally {
    /// 把另一份计数并进来（按片的次序相加，结果与线程数无关）。
    pub fn absorb(&mut self, other: &FaultInjectionTally) {
        self.histories += other.histories;
        self.histories_whose_measurement_run_stopped_early +=
            other.histories_whose_measurement_run_stopped_early;
        self.histories_without_any_injection_point += other.histories_without_any_injection_point;
        self.faults += other.faults;
        for (key, count) in &other.faults_by_kind {
            *self.faults_by_kind.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.faults_by_outcome {
            *self.faults_by_outcome.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.faults_by_injection_point {
            *self
                .faults_by_injection_point
                .entry(key.clone())
                .or_insert(0) += count;
        }
        self.faults_that_panicked += other.faults_that_panicked;
        self.faults_where_a_lying_device_left_the_image_inconsistent +=
            other.faults_where_a_lying_device_left_the_image_inconsistent;
        for (key, count) in &other.lying_device_signatures {
            *self.lying_device_signatures.entry(key.clone()).or_insert(0) += count;
        }
        for (key, count) in &other.refusal_members {
            *self.refusal_members.entry(key.clone()).or_insert(0) += count;
        }
        for (key, count) in &other.starting_point_failures {
            *self.starting_point_failures.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.acquired_instances_left_after_a_failed_mount {
            *self
                .acquired_instances_left_after_a_failed_mount
                .entry(key)
                .or_insert(0) += count;
        }
        self.reopens += other.reopens;
        self.reopens_reading_a_file += other.reopens_reading_a_file;
        self.reopens_without_a_file += other.reopens_without_a_file;
        self.reopens_that_failed += other.reopens_that_failed;
        self.reopened_into_a_version_the_model_committed +=
            other.reopened_into_a_version_the_model_committed;
        self.reopened_into_the_version_the_faulted_step_was_writing +=
            other.reopened_into_the_version_the_faulted_step_was_writing;
        self.reopens_outside_everything_the_model_allows +=
            other.reopens_outside_everything_the_model_allows;
        self.reopens_that_correctly_found_nothing_to_recover +=
            other.reopens_that_correctly_found_nothing_to_recover;
        for (verdict, count) in &other.reopens_by_the_reference_filesystem_and_model_verdicts {
            *self
                .reopens_by_the_reference_filesystem_and_model_verdicts
                .entry(*verdict)
                .or_insert(0) += count;
        }
        for (reason, count) in &other.reference_filesystem_abstentions_at_the_reopen {
            *self
                .reference_filesystem_abstentions_at_the_reopen
                .entry(reason)
                .or_insert(0) += count;
        }
        self.checker_runs_on_the_image_after_the_fault +=
            other.checker_runs_on_the_image_after_the_fault;
        self.images_after_the_fault_handed_out += other.images_after_the_fault_handed_out;
        for (key, count) in &other.invariant_holds {
            *self.invariant_holds.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.invariant_partially_judged {
            *self.invariant_partially_judged.entry(key).or_insert(0) += count;
        }
        for (form, count) in &other.faults_ending_known_red {
            *self.faults_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.faults_ending_new_finding += other.faults_ending_new_finding;
    }

    /// 一个注入点都没命中的那几格（验收要的「没命中的逐个列名」）。
    #[must_use]
    pub fn injection_points_never_hit(&self) -> Vec<String> {
        InjectionPoint::every_injection_point()
            .into_iter()
            .map(|point| point.render())
            .filter(|name| !self.faults_by_injection_point.contains_key(name))
            .collect()
    }

    /// 这一段上注入过几次（验收要的「每个发布步骤与挂载步骤上各注入过至少一次」与改法二的「起点段注入点大于 0」按它判）。
    #[must_use]
    pub fn faults_on_segment(&self, segment: FaultedSegmentKind) -> u64 {
        let prefix = format!("{}/", segment.render());
        self.faults_by_injection_point
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, count)| count)
            .sum()
    }

    /// 给人看的一整块：绝对数。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：测量跑就提前停的 {} 段、摆不出注入点的 {} 段；注入 {} 次",
            self.histories,
            self.histories_whose_measurement_run_stopped_early,
            self.histories_without_any_injection_point,
            self.faults
        );
        for (kind, count) in &self.faults_by_kind {
            let _ = writeln!(text, "  注入 {kind}：{count} 次");
        }
        for (outcome, count) in &self.faults_by_outcome {
            let _ = writeln!(text, "  收尾「{outcome}」：{count} 次");
        }
        let _ = writeln!(
            text,
            "注入之后 core panic 的次数：{}（被测的性质：返回错误而不是 panic）",
            self.faults_that_panicked
        );
        let _ = writeln!(
            text,
            "说谎的设备丢掉一份内容之后盘面不一致：{} 次（设备丢的，不算新发现）",
            self.faults_where_a_lying_device_left_the_image_inconsistent
        );
        for (signature, count) in &self.lying_device_signatures {
            let _ = writeln!(text, "    {signature}：{count} 次");
        }
        for segment in FaultedSegmentKind::every_kind_in_report_order() {
            let _ = writeln!(
                text,
                "  {} 上注入 {} 次",
                segment.render(),
                self.faults_on_segment(segment)
            );
        }
        for (point, count) in &self.faults_by_injection_point {
            let _ = writeln!(text, "    注入点 {point}：{count} 次");
        }
        let never_hit = self.injection_points_never_hit();
        let _ = writeln!(
            text,
            "  没命中的注入点 {} 个：{}",
            never_hit.len(),
            never_hit.join("、")
        );
        for (member, count) in &self.refusal_members {
            let _ = writeln!(text, "  返回的错误成员 {member}：{count} 次");
        }
        for (step, count) in &self.starting_point_failures {
            let _ = writeln!(text, "  起点段的{step}把错交回来：{count} 次");
        }
        let _ = writeln!(
            text,
            "取号之后挂载报错、新号不回卷（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）：用户 2026-09-23 认下的已知行为，D23（journal 的角色与格式） 已定项 16）：{} 次",
            self.acquired_instances_left_after_a_failed_mount
                .values()
                .sum::<u64>()
        );
        for kind in [
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot,
        ] {
            let _ = writeln!(
                text,
                "  {}：{} 次",
                kind.name(),
                self.acquired_instances_left_after_a_failed_mount
                    .get(kind.name())
                    .copied()
                    .unwrap_or(0)
            );
        }
        let _ = writeln!(
            text,
            "重开 {} 次：读回文件 {}、没有文件 {}、走读失败 {}",
            self.reopens,
            self.reopens_reading_a_file,
            self.reopens_without_a_file,
            self.reopens_that_failed
        );
        let _ = writeln!(
            text,
            "  走到模型认下的某一版 {} 次、走到失败那一步正在写的那一版 {} 次（收口表第 40 行那一格，只记不判）、模型都不允许的 {} 次、本来就没一版立得住而走读失败 {} 次（只记不判）",
            self.reopened_into_a_version_the_model_committed,
            self.reopened_into_the_version_the_faulted_step_was_writing,
            self.reopens_outside_everything_the_model_allows,
            self.reopens_that_correctly_found_nothing_to_recover
        );
        let _ = writeln!(
            text,
            "重开上 RefFS 与 O1 判官两票（任一红即红）：{:?}；RefFS 弃权的理由 {:?}",
            self.reopens_by_the_reference_filesystem_and_model_verdicts
                .iter()
                .map(|(verdict, count)| (verdict.name(), *count))
                .collect::<Vec<_>>(),
            self.reference_filesystem_abstentions_at_the_reopen
        );
        let _ = writeln!(
            text,
            "注入之后重开的盘交出去 {} 次；harness 就地在上面跑池级 checker {} 次：判绿 {} 条次、不适用 {} 条次、部分没判 {} 条次",
            self.images_after_the_fault_handed_out,
            self.checker_runs_on_the_image_after_the_fault,
            self.invariant_holds.values().sum::<u64>(),
            self.invariant_not_applicable.values().sum::<u64>(),
            self.invariant_partially_judged.values().sum::<u64>()
        );
        let _ = writeln!(
            text,
            "以「已知红」收尾 {:?}、新发现 {}",
            self.faults_ending_known_red
                .iter()
                .map(|(form, count)| (form.render(), *count))
                .collect::<BTreeMap<_, _>>(),
            self.faults_ending_new_finding
        );
        text
    }
}

/// 测量跑里每一步跑完时记下的东西：这一步是什么、到这一步为止读 / 写 / 刷盘各调了多少次、模型到这一步提交过哪些版本、
/// 这一步跑完时盘上择到的系统配置带的实例代号（C378 那一格按它判「这一步取了号」）。
struct StepMark {
    position: StepPosition,
    operation_kind: Option<HistoryOperationKind>,
    calls: FaultCallCounts,
    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    system_configuration_instance: Option<InstanceGeneration>,
}

/// 取号之后这一步报了错、写进系统配置的新号没有回卷时，重开之后盘上是哪一格（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）；
/// 用户 2026-09-23 定「认了，写成已知行为」，D23（journal 的角色与格式） 已定项 16 的射程）。这不是失败，是认下来的行为：
/// 只记账、在报告里点名，不进新发现。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AcquiredInstanceLeftAfterAFailedMount {
    /// 写行那次发布的根没落盘：新号一条根都没有，就此烧掉（下一次可写挂载按 max + 1 取下一个，给它写一行）。
    WithoutAnyRoot,
    /// 写行那次的根已落盘，之后暖机（或写行那次的系统配置槽轮换）报错：新号有根，挂载照样报错返回。
    WithTheRowPublishRoot,
}

impl AcquiredInstanceLeftAfterAFailedMount {
    /// 计数与报告里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot => "新号一条根都没有（烧掉）",
            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot => "新号已有写行那次的根",
        }
    }
}

/// 一次注入落在取号之后、挂载报错、新号没回卷的那一格：注入点、这一步之前与重开之后盘上系统配置的实例代号、根环里最大的实例代号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcquiredInstanceLeft {
    pub drawn: DrawnFault,
    pub kind: AcquiredInstanceLeftAfterAFailedMount,
    pub system_configuration_instance_before_the_step: InstanceGeneration,
    pub system_configuration_instance_after_the_reopen: InstanceGeneration,
    pub highest_root_instance_after_the_reopen: Option<InstanceGeneration>,
}

/// 一段历史注入故障之后交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryFaultInjection {
    pub seed: HistorySeed,
    /// 摆出来的注入点，按调用序号从小到大。
    pub drawn_faults: Vec<DrawnFault>,
    pub tally: FaultInjectionTally,
    pub known_red_hits: Vec<KnownRedAtAFault>,
    /// 同一段历史里同一个签名只留第一个。
    pub new_findings: Vec<FaultFinding>,
    /// 取号之后挂载报错、新号没回卷的那几次（C378；认下的已知行为，不算新发现）。
    pub acquired_instances_left: Vec<AcquiredInstanceLeft>,
}

/// 注入那一步是不是一次挂载（起点段里有取号与暖机、可写挂载），是就交回这一步之前盘上系统配置的实例代号。
/// 起点段之前是 mkfs 写的 0；别的步取测量跑里上一步跑完时的那个。不是挂载的几类操作不取号，交回 `None`
/// （管理员回退是挂着时的一次向前发布，不取号）。挂着时整理那一步按以批间崩溃收尾的那一种算挂载（臂上的注释写了为什么）。
fn system_configuration_instance_before_a_mount_step(
    marks: &[StepMark],
    segment: FaultedSegment,
) -> Option<InstanceGeneration> {
    match segment {
        FaultedSegment::TheStartingPoint => Some(MKFS_INSTANCE_GENERATION),
        FaultedSegment::Operation { operation_kind, .. } => match operation_kind {
            // 崩溃恢复抛弃根、每块盘最新那一槽系统配置读回全 0 的那一次也是可写挂载：取号、写行、暖机；
            // 掉电在 journal 记录之前的那次覆盖写之后也挂一次（掉电那一截不写系统配置）。
            HistoryOperationKind::CloseAndMountWritable
            | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
            | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
            | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            // 挂着时整理以批间崩溃收尾的那一种（`CompactionRunEnding::CrashesAfterBatchesThenMountsWritable`）推完批之后，执行器
            // （`history::apply_compact_while_mounted`）调可写挂载那一步同一个 `apply_mount_writable` 挂一次：取号、写行、暖机。
            // 整理那几次发布用的是会话开着时那次挂载取的号、不换号，所以这一步之前的号就是上一步跑完时的那个。推到停的那一种不挂载、不取号，
            // 重开读回的号不比这一步之前大，`acquired_instance_left_after_a_failed_mount` 照样交回 `None`。故障注入的比重
            // `GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED`（`broad-compaction`）会抽到这一步。
            | HistoryOperationKind::CompactWhileMounted => {
                let index = marks
                    .iter()
                    .position(|mark| mark.position == segment.position())?;
                marks[index.checked_sub(1)?].system_configuration_instance
            }
            HistoryOperationKind::PublishFirstFile
            | HistoryOperationKind::PublishOverwrite
            | HistoryOperationKind::PublishWithoutUnits
            | HistoryOperationKind::RollBackWhileMounted
            | HistoryOperationKind::RaiseRollbackFloor
            | HistoryOperationKind::ColdStartRecover
            | HistoryOperationKind::Unlink
            | HistoryOperationKind::CreateInode
            | HistoryOperationKind::CreateSnapshot
            | HistoryOperationKind::ReadSnapshot
            | HistoryOperationKind::DestroySnapshot
            // 截断经挂着的会话发，不挂载、不取号。
            | HistoryOperationKind::Truncate
            // 建 inode、按 inode 号写同样经挂着的会话发。
            | HistoryOperationKind::CreateInodes
            | HistoryOperationKind::WriteOfAnInode
            // 按偏移写同样经挂着的会话发。
            | HistoryOperationKind::WriteAtOffset => None,
        },
    }
}

/// 一次挂载报了错之后重开的盘上，系统配置的实例代号比这一步之前大（这一步取了号、没回卷）：按根环里有没有带新号的根分两格。
/// 系统配置择不出、或者号没变大（取号自己那几次写报错、回卷做成了），交回 `None`。
fn acquired_instance_left_after_a_failed_mount(
    image: &MemoryPool,
    marks: &[StepMark],
    drawn: DrawnFault,
) -> Option<AcquiredInstanceLeft> {
    let before = system_configuration_instance_before_a_mount_step(marks, drawn.segment)?;
    let system_configuration = choose_system_configuration(image).ok()?;
    let after = system_configuration.quantities.journal_instance;
    if after <= before {
        return None;
    }
    let roots = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let kind = if roots.iter().any(|root| root.instance == after) {
        AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot
    } else {
        AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot
    };
    Some(AcquiredInstanceLeft {
        drawn,
        kind,
        system_configuration_instance_before_the_step: before,
        system_configuration_instance_after_the_reopen: after,
        highest_root_instance_after_the_reopen: roots.iter().map(|root| root.instance).max(),
    })
}

/// 这个镜像上，最新那条根带的回退下界 F 落不落在回退留下的空档里（与活盘面、崩溃状态两路同一个谓词、同一份实现）。
fn raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(
    image: &MemoryPool,
) -> Option<bool> {
    let system_configuration = choose_system_configuration(image).ok()?;
    let newest_root = choose_root(image, &system_configuration)?;
    raised_floor_lands_only_on_abandoned_roots(image, newest_root.rollback_floor)
}

/// 随机注入抽 [`InjectedFault`] 的全部六种，等概率。前四种是里程碑「增补 3」第 4 件逐字写的
/// 「让任意一次写、读、刷盘返回 `BlockDeviceError`，或让一次读返回改坏的字节」；
/// 后两种是**设备说谎**（[`InjectedFault::WriteIsSwallowed`]、[`InjectedFault::BarrierIsSwallowed`]），
/// 判决第二节改法一把它们也抽进样本——它们原先只有各自的孤立单测，那两条只证明注得进去，
/// 不证明被测代码在这两种故障下的行为被看过。
///
/// 说谎那两种的判定档按 [`InjectedFault::the_device_lies`] 走：panic 与「重开悄悄读回没提交过的内容」照判，
/// 盘上被丢掉一份内容留下的不一致单记一格、不算新发现。一次被吞的屏障在这套装置里不改变任何东西
/// （录制流留着每一次真正落盘的写，重建镜像不看屏障），它落在「注入了、什么也没发生」那一格，
/// 计数里照样有它一笔——`fired` 记的是注入发没发生，不是有没有后果。
fn draw_injected_fault(random: &mut SeededRandomSource) -> InjectedFault {
    let variety = usize::try_from(random.below(6)).expect("小于 6");
    match variety {
        0 => InjectedFault::WriteFails,
        1 => InjectedFault::ReadFails,
        2 => InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit {
                byte_index: random.next_word(),
                bit_index: u32::try_from(random.below(8)).expect("小于 8"),
            },
        },
        3 => InjectedFault::BarrierFails,
        4 => InjectedFault::WriteIsSwallowed,
        5 => InjectedFault::BarrierIsSwallowed,
        // `below(6)` 的值域就是 0..6，第七种写不出来。
        other => unreachable!("按 below(6) 抽出来的只有 0..6，抽到了 {other}"),
    }
}

/// 这一段之前整池发过多少次读 / 写 / 刷盘：第 0 段（起点段）之前一次都没有。
fn calls_before(marks: &[StepMark], index: usize) -> FaultCallCounts {
    match index.checked_sub(1) {
        None => FaultCallCounts::default(),
        Some(previous) => marks[previous].calls,
    }
}

/// 按种子在测量跑记下的调用里摆注入点：先抽一种注入，再在「发过这一种调用」的那几段里均匀抽一段，
/// 最后在那一段发出的那几次调用里均匀抽一次。先抽段再抽调用（不是在整条调用流里均匀抽）是为了摊开到各类操作上——
/// 一次发布发几十次写、一次冷启动只发几十次读，整条流上均匀抽会把注入点全堆在写得最多的那几步里。
/// 同一个 (种类, 序号) 只摆一次。
///
/// 候选从第 0 段起算：第 0 段是起点段（mkfs、取号、暖机、第一个文件）。判决第二节改法二把它放开，
/// 同一次把 `HistoryPool::start` 那四处 `expect` 改成可失败——只放开候选区间会把「注入之后 core panic 0 次」
/// 这个读数弄脏（判定五：攻方在只改候选区间的探针上实测 49 次 panic，全在起点段那几处 `expect` 上）。
fn draw_faults(
    seed: HistorySeed,
    marks: &[StepMark],
    faults_per_history: usize,
) -> Vec<DrawnFault> {
    let mut random = SeededRandomSource::from_seed(seed.0 ^ 0x46_41_55_4c_54_00_00_01);
    let mut drawn: Vec<DrawnFault> = Vec::new();
    let mut taken: BTreeSet<(FaultCallKind, u64)> = BTreeSet::new();
    for _ in 0..faults_per_history {
        let fault = draw_injected_fault(&mut random);
        let kind = fault.call_kind();
        let candidate_steps: Vec<usize> = (0..marks.len())
            .filter(|index| marks[*index].calls.of(kind) > calls_before(marks, *index).of(kind))
            .collect();
        if candidate_steps.is_empty() {
            continue;
        }
        let picked = candidate_steps[usize::try_from(
            random.below(u64::try_from(candidate_steps.len()).expect("段数")),
        )
        .expect("下标")];
        let first = calls_before(marks, picked).of(kind) + 1;
        let last = marks[picked].calls.of(kind);
        let call_ordinal = first + random.below(last - first + 1);
        if !taken.insert((kind, call_ordinal)) {
            continue;
        }
        let segment = match (marks[picked].position, marks[picked].operation_kind) {
            (StepPosition::StartingPoint, None | Some(_)) => FaultedSegment::TheStartingPoint,
            (StepPosition::Operation(step_index), Some(operation_kind)) => {
                FaultedSegment::Operation {
                    step_index,
                    operation_kind,
                }
            }
            // 每一步的观察都带操作（`execute_history_with_faults` 在 `StepPosition::Operation` 上一律给 `Some`）；
            // 真出现这一格就是观察者那边变了，那一次不摆。
            (StepPosition::Operation(_), None) => continue,
        };
        drawn.push(DrawnFault {
            fault,
            call_ordinal,
            segment,
        });
    }
    drawn.sort();
    drawn
}

/// 跑一段历史，按种子在它起点之后的读 / 写 / 刷盘里摆 `faults_per_history` 个注入点，每个注入点重跑一遍这段历史，
/// 判「返回错误而不是 panic」与「重开恢复到模型允许的版本」。
#[must_use]
pub fn inject_faults_into_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    faults_per_history: usize,
) -> HistoryFaultInjection {
    inject_faults_into_history_handing_out_the_images(
        history,
        execution,
        faults_per_history,
        &mut |_| {},
    )
}

/// 同 [`inject_faults_into_history`]，另把每个注入点重开的盘交给 `hand_out`（[`ImagesAfterAFault`]）：harness 不在上面跑池级 checker，
/// 那一遍归 checker 档两段，结论交回 [`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 归类。交出去的次序是注入点的次序。
#[must_use]
pub fn inject_faults_into_history_handing_out_the_images(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    faults_per_history: usize,
    hand_out: &mut dyn FnMut(ImagesAfterAFault),
) -> HistoryFaultInjection {
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };
    let marks = measure_history(history, execution, &mut tally);
    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
    inject_the_drawn_faults(
        history,
        execution,
        &marks,
        drawn_faults,
        tally,
        PoolCheckerOfTheImagesAfterAFault::HandedOutToTheCheckerTier(hand_out),
    )
}

/// checker 档判回的、一个注入点交出去的那几张盘上池级 checker 判红的（[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`]
/// 收它）。违例细节可以是占位的一句（GPU 判的、从判定库读回的没有细节）：归类只看判红的是哪几条。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolCheckerVerdictsAfterAFault {
    /// 与 [`ImagesAfterAFault::step_images_of_the_run_with_the_fault`] 逐张对应。
    pub violations_of_the_step_images: Vec<Vec<(&'static str, String)>>,
    /// 重开那张盘（[`ImagesAfterAFault::image_after_the_fault`]）上判红的。
    pub violations_after_the_fault: Vec<(&'static str, String)>,
    /// 被吞的写插回去之后那张盘（[`ImagesAfterAFault::image_with_the_swallowed_writes_landed`]）上判红的不变量；没吞写时空。
    pub invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet<&'static str>,
}

/// 同 [`inject_faults_into_history`]，池级 checker 那一遍由 checker 档判（`pool_checker_verdicts_of` 收一个注入点交出去的盘，交回 checker 档
/// 从判定库读回的结论），照 harness 原先就地判、判红就停的那一档的口径并进 harness 自己的判定，交回与那一档逐字相同的失败观察与归类
/// （那一档删了，用户 2026-10-14 定；要它的产物不变的调用方是 E158 的故障注入）：
///
/// 1. 每个注入点先照不判池级 checker 跑一遍，交出去判；
/// 2. 带着注入跑的那一遍里一张每步镜像都不红：历史与判红就停那一档跑的是同一段，重开那张盘与被吞的写插回去那张盘的结论并进归类；
/// 3. 有一张判红：判红就停那一档的历史停在那一步（之后的步不跑，注入点在那之后的就没走到，重开的是那一步的盘）。照那几张的结论把这个注入点
///    截短重跑一遍（每步镜像那个钩子收调用方给的结论，`history::execute_history_stopping_at_the_given_pool_checker_violations`），
///    重跑那一遍交出去的盘再交一次判，重开那张盘与被吞的写插回去那张盘的结论并进归类。
///
/// 一个注入点因此交出去一次或两次。判红就停那一档里池级 checker 自己 panic 会被历史接住、记成那一步的 panic；这里 checker 档判不出
/// panic 来，那一格不罩（推的：E158 入库产物里没有这种行）。判红就停那一档在重开的盘上记的每条不变量判绿、不适用的计数
/// （[`FaultInjectionTally::invariant_holds`] 那几格）不记：结论里只有判红的。
///
/// # Panics
/// 交回的每步镜像结论与交出去的每步镜像张数不同；截短重跑那一遍交出去的每步镜像与整段那一遍判红那一张之前（含）逐张不同
/// （同一个种子、同一个注入点重跑，不同就是执行器不确定了）。
#[must_use]
pub fn inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    faults_per_history: usize,
    pool_checker_verdicts_of: &mut dyn FnMut(&ImagesAfterAFault) -> PoolCheckerVerdictsAfterAFault,
) -> HistoryFaultInjection {
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };
    let marks = measure_history(history, execution, &mut tally);
    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
    inject_the_drawn_faults(
        history,
        execution,
        &marks,
        drawn_faults,
        tally,
        PoolCheckerOfTheImagesAfterAFault::MergedFromTheCheckerTier(pool_checker_verdicts_of),
    )
}

/// 同 [`inject_faults_into_history`]，注入点不按种子抽，由调用方写死：`segment` 那一段发出的第 `call_within_the_segment` 次
/// `fault` 那一种调用（1 起）。写死的用例用它把注入点摆在点名的那一次调用上（C378：取号之后的写行、暖机）。
///
/// # Panics
/// 测量跑没走到那一段，或者那一段发出的这一种调用不到 `call_within_the_segment` 次（写死的用例把注入点摆错了）。
#[must_use]
pub fn inject_one_fault_into_the_segment(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    fault: InjectedFault,
    segment: FaultedSegment,
    call_within_the_segment: u64,
) -> HistoryFaultInjection {
    inject_one_fault_into_the_segment_handing_out_the_images(
        history,
        execution,
        fault,
        segment,
        call_within_the_segment,
        &mut |_| {},
    )
}

/// 同 [`inject_one_fault_into_the_segment`]，另把那个注入点重开的盘交给 `hand_out`（同 [`inject_faults_into_history_handing_out_the_images`]）。
///
/// # Panics
/// 同 [`inject_one_fault_into_the_segment`]。
#[must_use]
pub fn inject_one_fault_into_the_segment_handing_out_the_images(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    fault: InjectedFault,
    segment: FaultedSegment,
    call_within_the_segment: u64,
    hand_out: &mut dyn FnMut(ImagesAfterAFault),
) -> HistoryFaultInjection {
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };
    let marks = measure_history(history, execution, &mut tally);
    let index = marks
        .iter()
        .position(|mark| mark.position == segment.position())
        .unwrap_or_else(|| panic!("测量跑没走到{}", segment.render()));
    let kind = fault.call_kind();
    let calls_in_the_segment = marks[index].calls.of(kind) - calls_before(&marks, index).of(kind);
    assert!(
        (1..=calls_in_the_segment).contains(&call_within_the_segment),
        "{}只发了 {calls_in_the_segment} 次{}，摆不到第 {call_within_the_segment} 次",
        segment.render(),
        kind.name()
    );
    let drawn = DrawnFault {
        fault,
        call_ordinal: calls_before(&marks, index).of(kind) + call_within_the_segment,
        segment,
    };
    inject_the_drawn_faults(
        history,
        execution,
        &marks,
        vec![drawn],
        tally,
        PoolCheckerOfTheImagesAfterAFault::HandedOutToTheCheckerTier(hand_out),
    )
}

/// 测量跑：不注入，不跑池级 checker（那一档由随机历史与崩溃注入罩着），只要每一步的调用数、模型目录与系统配置的实例代号。
fn measure_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    tally: &mut FaultInjectionTally,
) -> Vec<StepMark> {
    let geometry = execution.device_width.fixed_geometry();
    let measurement_execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: execution.device_width,
        space_admission: execution.space_admission,
    };
    let measurement_plan = SharedFaultPlan::unarmed(geometry);
    let measurement_stream = SharedStream::new();
    let mut marks: Vec<StepMark> = Vec::new();
    let mut committed_so_far: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let measurement_run = execute_history_with_faults(
        history,
        measurement_execution,
        &measurement_stream,
        &measurement_plan,
        &mut |observation| {
            for (key, file) in observation.model.committed_versions() {
                committed_so_far.insert(key, file);
            }
            marks.push(StepMark {
                position: observation.position,
                operation_kind: observation.operation.map(HistoryOperation::kind),
                calls: measurement_plan.calls(),
                committed_versions: committed_so_far.clone(),
                system_configuration_instance: choose_system_configuration(observation.image)
                    .ok()
                    .map(|system_configuration| system_configuration.quantities.journal_instance),
            });
        },
    );
    match measurement_run.ending {
        HistoryEnding::Completed => {}
        HistoryEnding::KnownRed { .. } | HistoryEnding::NewFinding { .. } => {
            tally.histories_whose_measurement_run_stopped_early = 1;
        }
    }
    marks
}

/// 每个注入点重开的盘上池级 checker 那一遍怎么办（[`inject_the_drawn_faults`]）。harness 自己一条不变量都不判。
enum PoolCheckerOfTheImagesAfterAFault<'hook> {
    /// 盘交给钩子，那一遍归 checker 档两段，结论交回 [`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 归类。
    HandedOutToTheCheckerTier(&'hook mut dyn FnMut(ImagesAfterAFault)),
    /// checker 档判回的结论交进来，照判红就停那一档的口径并进 harness 的判定
    /// （[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`]）。
    MergedFromTheCheckerTier(
        &'hook mut dyn FnMut(&ImagesAfterAFault) -> PoolCheckerVerdictsAfterAFault,
    ),
}

/// 摆好的每个注入点各重跑一遍这段历史（[`run_one_fault_up_to_the_pool_checker`]），池级 checker 那一遍照 `pool_checker` 交出去或并进来，
/// 攒成一段的结果。
fn inject_the_drawn_faults(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn_faults: Vec<DrawnFault>,
    mut tally: FaultInjectionTally,
    pool_checker: PoolCheckerOfTheImagesAfterAFault<'_>,
) -> HistoryFaultInjection {
    if drawn_faults.is_empty() {
        tally.histories_without_any_injection_point = 1;
    }
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<FaultFinding> = Vec::new();
    let mut acquired_instances_left: Vec<AcquiredInstanceLeft> = Vec::new();
    {
        let mut accumulator = FaultInjectionAccumulator {
            tally: &mut tally,
            known_red_hits: &mut known_red_hits,
            new_findings: &mut new_findings,
            acquired_instances_left: &mut acquired_instances_left,
        };
        match pool_checker {
            PoolCheckerOfTheImagesAfterAFault::HandedOutToTheCheckerTier(hand_out) => {
                for drawn in &drawn_faults {
                    let one_fault = run_one_fault_up_to_the_pool_checker(
                        history, execution, marks, drawn, None,
                    );
                    settle_one_fault_with_the_pool_checker_verdicts(
                        &one_fault,
                        Vec::new(),
                        BTreeSet::new(),
                        &mut accumulator,
                    );
                    // 重开的盘交出去：池级 checker 那一遍归 checker 档两段
                    accumulator.tally.images_after_the_fault_handed_out += 1;
                    hand_out(one_fault.images);
                }
            }
            PoolCheckerOfTheImagesAfterAFault::MergedFromTheCheckerTier(
                pool_checker_verdicts_of,
            ) => {
                for drawn in &drawn_faults {
                    merge_one_fault_with_the_pool_checker_verdicts(
                        history,
                        execution,
                        marks,
                        drawn,
                        pool_checker_verdicts_of,
                        &mut accumulator,
                    );
                }
            }
        }
    }
    HistoryFaultInjection {
        seed: history.seed,
        drawn_faults,
        tally,
        known_red_hits,
        new_findings,
        acquired_instances_left,
    }
}

/// 一批注入攒出来的东西：计数、「已知红」命中、新发现、取号之后挂载报错而新号没回卷的那几次。
struct FaultInjectionAccumulator<'run> {
    tally: &'run mut FaultInjectionTally,
    known_red_hits: &'run mut Vec<KnownRedAtAFault>,
    new_findings: &'run mut Vec<FaultFinding>,
    acquired_instances_left: &'run mut Vec<AcquiredInstanceLeft>,
}

impl FaultInjectionAccumulator<'_> {
    /// 一条注入之后的失败归好类之后记账：「已知红」照形态计、说谎的设备留下的按签名计、新发现同一个签名只留第一个。
    fn settle_a_failure_after_a_fault(
        &mut self,
        seed: HistorySeed,
        drawn: DrawnFault,
        classified: FailureAfterAFault,
        reopen_text: &str,
    ) {
        match classified {
            FailureAfterAFault::KnownRed(form) => self.record_a_known_red_hit(seed, form, drawn),
            FailureAfterAFault::LeftByTheLyingDevice { signature } => {
                self.tally
                    .faults_where_a_lying_device_left_the_image_inconsistent += 1;
                *self
                    .tally
                    .lying_device_signatures
                    .entry(format!("{signature:?}"))
                    .or_insert(0) += 1;
            }
            FailureAfterAFault::NewFinding {
                signature,
                observation,
            } => {
                self.tally.faults_ending_new_finding += 1;
                if !self
                    .new_findings
                    .iter()
                    .any(|finding| finding.signature == signature)
                {
                    self.new_findings.push(FaultFinding {
                        signature,
                        seed,
                        drawn,
                        observation: *observation,
                        reopen: reopen_text.to_string(),
                    });
                }
            }
        }
    }

    /// 一次注入以「已知红」收尾：按形态计一次，再记下是哪个种子上的哪一次注入。
    fn record_a_known_red_hit(
        &mut self,
        seed: HistorySeed,
        form: KnownRedFormAtAFault,
        drawn: DrawnFault,
    ) {
        *self.tally.faults_ending_known_red.entry(form).or_insert(0) += 1;
        self.known_red_hits
            .push(KnownRedAtAFault { seed, form, drawn });
    }
}

/// 注入那一段的结局是不是「入口返回了错误」：没 panic、checker 没判红，而且那一段要么返回了 `Err`，
/// 要么是冷启动或读快照、读回报了错。注入的块设备错在模型里没有对应的拒绝理由（`ObservedRefusalReason::Unexplained`），
/// 模型因此必然报「该成却拒了」——那一格不算失败，它就是这一件要的结果。
///
/// 起点段（改法二放开的那一段）按 `HarnessJudgement::TheStartingPointReturnedAnError` 认：起点段的四个入口
/// 在执行器里不再是 `expect`，报错时把是哪一步、报的什么交回来，那正是这一件要的结果。
fn the_fault_surfaced_as_an_error(
    run: &HistoryRun,
    observation: &FailureObservation,
    segment: FaultedSegment,
) -> bool {
    if observation.position != segment.position()
        || observation.panic.is_some()
        || !observation.violations.is_empty()
    {
        return false;
    }
    if let FaultedSegment::TheStartingPoint = segment {
        return matches!(
            observation.harness_judgement,
            Some(HarnessJudgement::TheStartingPointReturnedAnError { .. })
        );
    }
    match run.outcomes.last() {
        Some(StepOutcome::Refused { .. }) => true,
        Some(StepOutcome::Applied(AppliedEffect::Recovered { read_back, .. })) => match read_back {
            ColdStartReadBack::Failed => true,
            ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => false,
        },
        Some(StepOutcome::Applied(
            AppliedEffect::Published { .. }
            | AppliedEffect::Mounted { .. }
            | AppliedEffect::RaisedFloor { .. }
            | AppliedEffect::RolledBack { .. }
            | AppliedEffect::SnapshotPublished { .. }
            | AppliedEffect::Compacted { .. },
        ))
        | Some(StepOutcome::NotApplicable(_))
        | None => false,
        // 读快照读失败（注入的读错落在它要读的那几处上）：同冷启动读回报错，是注入的错浮出来了。
        Some(StepOutcome::Applied(AppliedEffect::SnapshotRead { read_back, .. })) => {
            match read_back {
                ObservedSnapshotReadBack::Failed { .. } => true,
                ObservedSnapshotReadBack::NoFile
                | ObservedSnapshotReadBack::File(_)
                | ObservedSnapshotReadBack::NoSnapshotWithThisTree => false,
            }
        }
    }
}

/// 说谎的设备吞掉了几次写之后，拿来判「这条失败是不是它留下的」的这几样东西（按注入点认，主 agent 2026-09-26 定，实七）：
///
/// - `invariants_still_violated_if_the_swallowed_writes_had_landed`：录制流里把被吞的那几次写按原位插回去、重建镜像
///   （实现以为自己写下的那一份），在它上面 checker 判红的那几条。违例点名的（盘, 槽）是被吞那次写的、或红在被吞的根槽里留下的旧根上，
///   那几次写一落盘它就消失；插回去仍红的那一条不是注入点造成的。
/// - `ceiling_without_the_swallowed_roots`：被吞的写里有根槽写时，拿历史停下那一步之前的模型、去掉那几条根重算的抬 F 的上限
///   （实现从盘上现读根环，看不见那几条根）；没有被吞的根槽写、或没有那一步之前的模型时 None。
/// - `root_ring_slots_of_the_swallowed_root_slot_writes`：被吞的根槽写各落在根环的哪个槽（实八）。同一个进程写过、FUA 返回过的槽，
///   算抬 F 的上限、或挂着时回退判候选集读根环时读坏就重读一次、仍坏就拒（D16（发布语义） 已定项 1「根槽这一次读坏」那一行；
///   回退照同一个读法，`crates/singlefs-core/src/mount.rs` 的 `rollback_candidate`）；读坏的正是被吞那次写的槽，这个拒绝就是条款要的结局，是说谎的设备的投影。
/// - `instance_rows_with_the_swallowed_roots_rebuilt_from_their_records`：被吞的写里有根槽写时，拿历史停下那一步之前的模型答
///   「那几条根的根槽没落、记录与单元落了」时可写挂载写的行（实八，[`crate::model::IdealModel::answer_mount_writable_with_the_roots_rebuilt_from_their_records`]：
///   恢复施加它们的记录、W 取施加的记录的事务号，D23（journal 的角色与格式） 已定项 14 第 4 条）；没有被吞的根槽写、没有那一步之前的模型、
///   或模型答不了时 None。
/// - `byte_ranges_of_the_swallowed_writes`：每一次被吞的写落在哪块盘、哪一段字节（不分写的是什么结构）。管理员回退在任何写之前逐盘验
///   回退目标引用的单元（复活集里的、隔离的）每一份，读不出或整单元 CRC-32C 对不上就拒（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格）；
///   对不上的那一份正落在被吞的写上，这个拒绝就是条款要的结局，是说谎的设备的投影（`research/prompts/code-backlog-b2-code-r1-main-verification.md`
///   「更正」一节 A4：模型不知道那一份没落、要它成）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhatTheSwallowedWritesExplain {
    pub invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet<&'static str>,
    pub ceiling_without_the_swallowed_roots: Option<Option<ModelCheckpointTxg>>,
    pub root_ring_slots_of_the_swallowed_root_slot_writes: BTreeSet<ModelRingPosition>,
    pub instance_rows_with_the_swallowed_roots_rebuilt_from_their_records:
        Option<Vec<ModelInstanceRow>>,
    pub byte_ranges_of_the_swallowed_writes: BTreeSet<SwallowedWriteByteRange>,
}

impl WhatTheSwallowedWritesExplain {
    /// 同一份，池级 checker 那一项（插回去之后仍红的不变量）换成 `invariants_still_violated`：交出去的那一份里那一项是空的，归类那一刻填。
    #[must_use]
    pub fn with_the_invariants_still_violated(
        self,
        invariants_still_violated: BTreeSet<&'static str>,
    ) -> Self {
        Self {
            invariants_still_violated_if_the_swallowed_writes_had_landed: invariants_still_violated,
            ceiling_without_the_swallowed_roots: self.ceiling_without_the_swallowed_roots,
            root_ring_slots_of_the_swallowed_root_slot_writes: self
                .root_ring_slots_of_the_swallowed_root_slot_writes,
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: self
                .instance_rows_with_the_swallowed_roots_rebuilt_from_their_records,
            byte_ranges_of_the_swallowed_writes: self.byte_ranges_of_the_swallowed_writes,
        }
    }
}

/// 一次被吞的写落在哪块盘的哪一段字节：[起, 止)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SwallowedWriteByteRange {
    pub device: ModelDeviceIdentity,
    pub start_in_bytes: u64,
    pub end_in_bytes: u64,
}

impl SwallowedWriteByteRange {
    /// 被吞的那一次写落的那一段。
    #[must_use]
    pub fn of(write: &SwallowedWrite) -> Self {
        let length = write.as_retained_operation().operation.length;
        Self {
            device: ModelDeviceIdentity(write.device.0),
            start_in_bytes: write.offset.0,
            end_in_bytes: write
                .offset
                .0
                .checked_add(length)
                .expect("一次写的末端装得进 u64：设备包装先按设备契约查过偏移与长度"),
        }
    }

    /// 这一段与一个单元在同一块盘上的那一份有没有重叠的字节。
    #[must_use]
    pub fn overlaps(self, unit_copy: ModelUnitCopyLocation) -> bool {
        let copy = unit_copy.byte_range_on_the_device();
        self.device == unit_copy.device
            && self.start_in_bytes < copy.end
            && copy.start < self.end_in_bytes
    }
}

/// 这条失败是不是说谎的设备留下的：没有 panic、没有执行器判出的、记录核对器没判红；checker 判红的每一条在「被吞的写都落了盘」的
/// 镜像上都不红；模型对不上的只许是这四格之一：
/// - 抬 F 的上限，而且实现报的上限等于去掉被吞的根之后模型重算的那一个；
/// - 「模型说该成、实现拒了」或「拒绝的理由」，而实现拒的是「算抬 F 的上限、或挂着时回退判候选集时一个知道住着根的根环槽读坏、
///   重读仍坏」、拒之前一个写都没发、点名的那个槽正是一次被吞的根槽写的槽（实八：同一个进程里根槽写被吞之后再抬 F 或回退，
///   D16（发布语义） 已定项 1「根槽这一次读坏」那一行要的就是这个拒绝，模型不知道根槽没落、要它成或按上限拒）；
/// - 「模型说该成、实现拒了」或「拒绝的理由」，而实现拒的是管理员回退「回退目标引用的一个单元（复活集里的或隔离的）在一块盘上的那一份
///   读不出或对不上」、拒之前一个写都没发、点名的那一份与一次被吞的写有重叠的字节（A4：设备吞了那一份的写，回退照条款在任何写之前拒）；
/// - 写的实例表行，而且实现写的行等于模型按「被吞根槽的那几条根由记录重建」重算的行（实八：W 取施加的那条记录的事务号，
///   D23（journal 的角色与格式） 已定项 14 第 4 条；模型不知道根槽没落、答 W = 0）。
///
/// 一条不是注入点造成的违例（插回去仍红）、一次读坏的槽不是被吞那次写的、一行与重算的对不上，照报新发现——这是判别力那一半。
///
/// 它替下的是按签名写死两组的白名单（主 agent 2026-09-21 定的收严，`records/2026-09-21-增补3第4件与checker补三条.md`「签名白名单写死两组、名单外照报」那一句）：
/// 白名单早于 I-3.10 的实现，大档撞出的六组里有四组（I-3.10、I-9.1、旧根复活连带的五条、抬 F 的上限）都是说谎的设备丢一份内容的投影、
/// 却不在名单里（实四丙交回第二节）。
#[must_use]
pub fn left_by_the_lying_device(
    observation: &FailureObservation,
    explained: &WhatTheSwallowedWritesExplain,
) -> bool {
    if observation.panic.is_some()
        || observation.harness_judgement.is_some()
        || !crate::history::record_check_aspects(&observation.record_check).is_empty()
    {
        return false;
    }
    let every_violation_is_explained = observation.violations.iter().all(|(invariant, _)| {
        !explained
            .invariants_still_violated_if_the_swallowed_writes_had_landed
            .contains(invariant)
    });
    let disagreement_is_explained = match &observation.model_disagreement {
        None => true,
        Some(disagreement) => match disagreement.aspect {
            ModelDisagreementAspect::RollbackFloorCeiling => matches!(
                (
                    disagreement.implementation_reported_ceiling,
                    explained.ceiling_without_the_swallowed_roots
                ),
                (Some(reported), Some(Some(recomputed))) if reported == recomputed
            ),
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
            | ModelDisagreementAspect::RefusalReason => {
                match disagreement.implementation_reported_unreadable_place {
                    Some(ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread(
                        ring_slot,
                    )) => explained
                        .root_ring_slots_of_the_swallowed_root_slot_writes
                        .contains(&ring_slot),
                    Some(ModelReportedUnreadablePlace::UnitCopyUnreadableOrMismatched(
                        unit_copy,
                    )) => explained
                        .byte_ranges_of_the_swallowed_writes
                        .iter()
                        .any(|swallowed| swallowed.overlaps(unit_copy)),
                    None => false,
                }
            }
            ModelDisagreementAspect::InstanceRows => matches!(
                (
                    &disagreement.implementation_rows_written,
                    &explained.instance_rows_with_the_swallowed_roots_rebuilt_from_their_records
                ),
                (Some(written), Some(recomputed)) if written == recomputed
            ),
            ModelDisagreementAspect::SessionState
            | ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
            | ModelDisagreementAspect::WroteBeforeRefusing
            | ModelDisagreementAspect::PublishCount
            | ModelDisagreementAspect::RootIdentity
            | ModelDisagreementAspect::JournalCounter
            | ModelDisagreementAspect::RollbackFloor
            | ModelDisagreementAspect::FilePresence
            | ModelDisagreementAspect::FileContent
            | ModelDisagreementAspect::InstanceTableOfTheVersion
            | ModelDisagreementAspect::AllocationGeneration
            | ModelDisagreementAspect::InstanceGeneration
            | ModelDisagreementAspect::ColdStartReadBack
            | ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg
            | ModelDisagreementAspect::Snapshots
            | ModelDisagreementAspect::DeadlistEntries
            | ModelDisagreementAspect::PinnedMappingEntries
            | ModelDisagreementAspect::SnapshotReadBack
            | ModelDisagreementAspect::CompactionIntentOfThePublish
            | ModelDisagreementAspect::LoggedCompactionIntent
            | ModelDisagreementAspect::FirstFileRecordTruncationField
            | ModelDisagreementAspect::InodesOtherThanTheFirst
            | ModelDisagreementAspect::EffectKindOfTheOperation
            | ModelDisagreementAspect::NotModeled => false,
        },
    };
    every_violation_is_explained && disagreement_is_explained
}

/// 一张重开的盘在录制流里的样子：空盘叠上 `operations` 就是它（录制流只记落了盘的写）。checker 档照它建给定状态表：
/// mkfs 写的那几步进基线（`operations_written_by_make_filesystem`），掉电接缝照 `earliest_stream_index_after_each_power_loss` 断开记录核对器的发布表。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedStreamOfAnImage {
    pub operations: Vec<crate::RetainedOperation>,
    /// 录制流头上 mkfs 写的步数（[`HistoryRun::operations_written_by_make_filesystem`]）。
    pub operations_written_by_make_filesystem: usize,
    /// 每次掉电之后录制流里第一步的下标（[`HistoryRun::earliest_stream_index_after_each_power_loss`]）。
    pub earliest_stream_index_after_each_power_loss: Vec<usize>,
}

/// 一个注入点重开的盘经钩子交出去的那一份（[`inject_faults_into_history_handing_out_the_images`]）：池级 checker 那一遍归 checker 档两段
/// （用户 2026-10-13 定：镜像作为给定状态写进判定库、判段从库读、判在 GPU），判出的违例交回
/// [`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 照这里带的事实归类（故障注入自己那条「已知红」、说谎的设备留下的不一致、新发现）。
/// 模型、RefFS 与执行器判出的失败 harness 自己归类过，不在这里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagesAfterAFault {
    pub seed: HistorySeed,
    pub drawn: DrawnFault,
    pub device_width: HistoryDeviceWidth,
    /// 注入之后重开的那张盘。
    pub image_after_the_fault: RecordedStreamOfAnImage,
    /// 带着注入跑的那一遍每一步的镜像（[`crate::history::StepImage`]，次序同历史）：都是 `image_after_the_fault` 录制流的前缀。
    pub step_images_of_the_run_with_the_fault: Vec<StepImageInTheRecordedStream>,
    /// 说谎的设备吞掉了写时：被吞的写按原位插回去之后的那张盘（实现以为自己写下的那一份；判「这条失败是不是它留下的」）。
    pub image_with_the_swallowed_writes_landed: Option<RecordedStreamOfAnImage>,
    /// 说谎的设备吞掉的每一次写，按发生的先后（每步镜像各记着它那一刻之前吞了几次，按步对照那张盘照它插：
    /// [`ImagesAfterAFault::image_at_the_step_image_with_the_swallowed_writes_before_it_landed`]）。
    pub swallowed_writes: Vec<SwallowedWrite>,
    /// 实现交回的每一份隔离，跨重挂累积（[`crate::history::HistoryRun::copies_quarantined_by_the_implementation`]；每步镜像各记着它那一刻之前交回了几份）：
    /// 归类器认「插回去那张盘上仍红的那一份是说谎的设备逼出来的隔离」（[`unit_start_slots_quarantined_over_a_swallowed_write`]）。
    pub copies_quarantined_by_the_implementation: Vec<CopyQuarantinedByTheImplementation>,
    /// 这个注入点上池级 checker 判红时那一条失败除违例之外的样子（位置、根环、抬 F 落没落在空档里），违例一栏是空的。
    pub observation_without_the_pool_checker_violations: FailureObservation,
    pub facts_after_the_fault: FactsAfterTheFault,
    /// 说谎的设备吞掉了写时判「这条失败是不是它留下的」要的那几样，池级 checker 那一项（插回去之后仍红的不变量）是空的、由归类那一刻填。
    pub what_the_swallowed_writes_explain_without_the_pool_checker:
        Option<WhatTheSwallowedWritesExplain>,
    /// 重开走到了哪一版（给人看的一句，新发现带着它）。
    pub reopen: String,
}

/// 带着注入跑的那一遍里一张每步镜像：空盘叠上重开那张盘录制流的头 `operations_recorded` 步，连同 harness 交出去那一刻读的根环
/// （池级 checker 在它上面判红时那一条失败要带）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepImageInTheRecordedStream {
    pub position: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
    pub operations_recorded: usize,
    pub newest_ring_root_txg: Option<u64>,
    pub root_ring_slot_count: Option<u64>,
    /// 这一刻之前说谎的设备吞了几次写（[`ImagesAfterAFault::swallowed_writes`] 的头这么多次）。
    pub swallowed_writes_before_it: usize,
    /// 这一刻之前实现交回了几份隔离（[`ImagesAfterAFault::copies_quarantined_by_the_implementation`] 的头这么多份）。
    pub copies_quarantined_by_the_implementation_before_it: usize,
}

impl ImagesAfterAFault {
    /// 按步对照的那张盘（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3 与「用户定案」：归类器对照第一张红的那一步镜像那一刻
    /// 插回去的盘）：带着注入跑的那一遍第 `step_image_index` 张每步镜像那一刻的录制流前缀，把那一刻之前被吞的写按原位插回去。
    /// 那一刻之前一次写都没吞时交 `None`（插回去的盘就是那张每步镜像本身，那一张上判红的没有一条是被吞的写留下的）。
    /// 掉电接缝只留落在前缀里的那几道。
    ///
    /// # Panics
    /// `step_image_index` 不是一张交出去的每步镜像。
    #[must_use]
    pub fn image_at_the_step_image_with_the_swallowed_writes_before_it_landed(
        &self,
        step_image_index: usize,
    ) -> Option<RecordedStreamOfAnImage> {
        let step_image = self
            .step_images_of_the_run_with_the_fault
            .get(step_image_index)
            .unwrap_or_else(|| {
                panic!(
                    "交出去的每步镜像只有 {} 张，没有第 {step_image_index} 张",
                    self.step_images_of_the_run_with_the_fault.len()
                )
            });
        let swallowed_before_it = &self.swallowed_writes[..step_image.swallowed_writes_before_it];
        if swallowed_before_it.is_empty() {
            return None;
        }
        let prefix = RecordedStreamOfAnImage {
            operations: self.image_after_the_fault.operations[..step_image.operations_recorded]
                .to_vec(),
            operations_written_by_make_filesystem: self
                .image_after_the_fault
                .operations_written_by_make_filesystem
                .min(step_image.operations_recorded),
            earliest_stream_index_after_each_power_loss: self
                .image_after_the_fault
                .earliest_stream_index_after_each_power_loss
                .iter()
                .copied()
                .filter(|earliest_stream_index| {
                    *earliest_stream_index < step_image.operations_recorded
                })
                .collect(),
        };
        Some(stream_with_the_swallowed_writes_landed(
            &prefix,
            swallowed_before_it,
        ))
    }
}

/// 一条注入之后的失败归到哪一格（[`classify_a_failure_after_a_fault`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureAfterAFault {
    KnownRed(KnownRedFormAtAFault),
    /// 说谎的设备吞掉写之后盘面不一致：设备丢的，不是实现的缺口（[`left_by_the_lying_device`]）；按签名计数、不算新发现。
    LeftByTheLyingDevice {
        signature: FailureSignature,
    },
    NewFinding {
        signature: FailureSignature,
        observation: Box<FailureObservation>,
    },
}

/// 一条注入之后的失败归类：故障注入自己那几条「已知红」先认，再对共用清单，清单外的说谎的设备留下的不一致单记，其余是新发现。
/// harness 就地判的失败与 checker 档两段判出的池级 checker 违例走同一个归类。
#[must_use]
pub fn classify_a_failure_after_a_fault(
    observation: FailureObservation,
    facts: &FactsAfterTheFault,
    fault: InjectedFault,
    what_the_swallowed_writes_explain: Option<&WhatTheSwallowedWritesExplain>,
) -> FailureAfterAFault {
    // 「已知红」先认：清单里那几条是认下来的形态（共用清单是实现与 checker 读法对不上的老账，增补 2 收口表第 ②、43 行；
    // 故障注入自己那一条是用户 2026-10-03 定的「mkfs 没写下系统配置槽，全池皆废判红是对的」），
    // 与设备说不说谎无关，说谎的设备把它撞出来时照样记进「已知红」那一格，不许被下面那一条接走。
    if let Some(form) = known_red_form_only_after_a_fault(&observation, facts) {
        return FailureAfterAFault::KnownRed(KnownRedFormAtAFault::OnlyAfterAFault(form));
    }
    match classify_failure(observation) {
        HistoryEnding::KnownRed { form, .. } => {
            FailureAfterAFault::KnownRed(KnownRedFormAtAFault::SharedWithRandomHistories(form))
        }
        HistoryEnding::NewFinding {
            signature,
            observation,
        } => {
            // 说谎的设备吞掉写之后盘面不一致：设备丢的，不是实现的缺口（`InjectedFault::the_device_lies`）。
            // 按注入点认（`left_by_the_lying_device`，主 agent 2026-09-26 定，实七）：被吞的写插回去就消失的违例、
            // 去掉被吞的根之后模型重算对得上的上限才豁免，别的照报成新发现。
            // panic 不走这一格——说谎的设备也不许把实现打 panic，那一条照样报成新发现。
            if fault.the_device_lies()
                && what_the_swallowed_writes_explain
                    .is_some_and(|explained| left_by_the_lying_device(&observation, explained))
            {
                FailureAfterAFault::LeftByTheLyingDevice { signature }
            } else {
                FailureAfterAFault::NewFinding {
                    signature,
                    observation: Box::new(observation),
                }
            }
        }
        HistoryEnding::Completed => {
            unreachable!("classify_failure 只交回「已知红」或新发现，不交回跑完")
        }
    }
}

/// 一张盘上池级 checker 判红的，连同违例点名的（不变量, 盘, 槽）（`singlefs_checker::walk::check_pool_image_naming_the_violation_sites` 交回的那两样；
/// 点名槽的只有 I-3.1、I-3.11、I-3.10 与 I-3.15，别的不变量判红时这里没有它的行）。违例细节可以是占位的一句：归类只看判红的是哪几条、点名了哪几处。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViolationsNamingTheSites {
    pub violations: Vec<(&'static str, String)>,
    pub violation_sites: BTreeSet<ViolationSite>,
}

/// checker 档两段判回的、一个注入点交出去的那几张盘上池级 checker 判红的（[`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 收它）。
/// 违例点名的（不变量, 盘, 槽）只在拿来比的那几张上要：第一张判红的每步镜像与它的按步对照盘、重开那张盘与被吞的写都插回去那张盘。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolCheckerVerdictsOfTheImagesAfterAFault {
    /// 与 [`ImagesAfterAFault::step_images_of_the_run_with_the_fault`] 逐张对应。
    pub violations_of_the_step_images: Vec<Vec<(&'static str, String)>>,
    /// 第一张判红的每步镜像上违例点名的（不变量, 盘, 槽）；一张都不红时空。
    pub violation_sites_of_the_first_red_step_image: BTreeSet<ViolationSite>,
    /// 重开那张盘（[`ImagesAfterAFault::image_after_the_fault`]）上判红的。
    pub after_the_fault: ViolationsNamingTheSites,
    /// 第一张判红的每步镜像的按步对照盘（[`ImagesAfterAFault::image_at_the_step_image_with_the_swallowed_writes_before_it_landed`]）上判红的：
    /// 那个方法交回 `Some` 时给，交回 `None`（一张都不红、或那一刻之前一次写都没吞）时是 `None`。
    pub at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed:
        Option<ViolationsNamingTheSites>,
    /// 被吞的写都插回去之后那张盘（[`ImagesAfterAFault::image_with_the_swallowed_writes_landed`]）上判红的：那张盘在时给，不在时是 `None`。
    pub with_the_swallowed_writes_landed: Option<ViolationsNamingTheSites>,
}

/// 隔离判据判的那三条（I-3.1、I-3.11 与随 I-3.1 那一份的 I-3.15，`crates/singlefs-checker/invariants/05-空间记账.md`）：
/// 对照盘上落在 [`unit_start_slots_quarantined_over_a_swallowed_write`] 那几个起点槽上的点名，归类器先减掉再比。
const INVARIANTS_JUDGED_BY_THE_QUARANTINE_CRITERION: [&str; 3] = [
    "I-3.1",
    "I-3.11",
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
];

/// 实现交回的隔离里，那一份与一次被吞的写有重叠字节的那几个单元的起点槽（两盘同槽，按槽认、不分盘）。被吞的写插回去之后这几个单元每一份都对得上、
/// 记录却留在已分配：那是实现在说谎的设备上读盘核出对不上才隔离的，不是实现的缺口（判决 `research/prompts/m4-i315-r2-main-verification.md`
/// 甲二「把实现交出、与被吞的写有重叠的单元当隔离单元」，第三轮判决第三节 C3 收齐四条路的清单）。
#[must_use]
pub fn unit_start_slots_quarantined_over_a_swallowed_write(
    copies_quarantined_by_the_implementation: &[CopyQuarantinedByTheImplementation],
    swallowed_writes: &[SwallowedWrite],
) -> BTreeSet<u64> {
    let swallowed_ranges: Vec<SwallowedWriteByteRange> = swallowed_writes
        .iter()
        .map(SwallowedWriteByteRange::of)
        .collect();
    copies_quarantined_by_the_implementation
        .iter()
        .filter(|quarantined| {
            let copy = quarantined.copy;
            let start_in_bytes = copy.placement.slot.to_device_offset().0;
            let end_in_bytes = copy
                .placement
                .span
                .checked_mul(SLOT_BYTES)
                .and_then(|length_in_bytes| start_in_bytes.checked_add(length_in_bytes))
                .expect("一个落点的末端装得进 u64：分配器只在盘内取落点");
            swallowed_ranges.iter().any(|swallowed| {
                swallowed.device.0 == copy.device.0
                    && swallowed.start_in_bytes < end_in_bytes
                    && start_in_bytes < swallowed.end_in_bytes
            })
        })
        .map(|quarantined| quarantined.copy.placement.slot.0)
        .collect()
}

/// 被比的那一张上判红的每一条里，拿对照盘比之后仍红的那几条（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3：按（不变量, 盘, 槽）比）：
/// - 被比那一张上这一条点了槽、对照盘上这一条也点了槽：被比那一张点名的（盘, 槽）里有一处在对照盘上这一条下仍被点名，才算仍红；
///   隔离判据那三条（[`INVARIANTS_JUDGED_BY_THE_QUARANTINE_CRITERION`]），对照盘上落在 `quarantined_over_a_swallowed_write` 那几个起点槽上的点名不算；
/// - 被比那一张上这一条没点槽（I-3.x 之外的不变量，或判红的不是哪一个单元），或对照盘上这一条一个槽都没点：按名比，对照盘上这一条红就算仍红。
///
/// `comparison` 是 `None`（这一刻之前一次写都没吞，插回去就是被比那一张本身）时每一条都算仍红。
#[must_use]
pub fn invariants_still_violated_on_the_comparison_image(
    observed: &ViolationsNamingTheSites,
    comparison: Option<&ViolationsNamingTheSites>,
    quarantined_over_a_swallowed_write: &BTreeSet<u64>,
) -> BTreeSet<&'static str> {
    let observed_invariants = observed.violations.iter().map(|(invariant, _)| *invariant);
    let Some(comparison) = comparison else {
        return observed_invariants.collect();
    };
    let violated_on_the_comparison: BTreeSet<&'static str> = comparison
        .violations
        .iter()
        .map(|(invariant, _)| *invariant)
        .collect();
    observed_invariants
        .filter(|invariant| {
            let sites_named_for = |sites: &BTreeSet<ViolationSite>| -> Vec<ViolationSite> {
                sites
                    .iter()
                    .filter(|site| site.invariant == *invariant)
                    .copied()
                    .collect()
            };
            let observed_sites = sites_named_for(&observed.violation_sites);
            let comparison_sites = sites_named_for(&comparison.violation_sites);
            if observed_sites.is_empty() || comparison_sites.is_empty() {
                return violated_on_the_comparison.contains(invariant);
            }
            let judged_by_the_quarantine_criterion =
                INVARIANTS_JUDGED_BY_THE_QUARANTINE_CRITERION.contains(invariant);
            observed_sites.iter().any(|site| {
                comparison_sites.contains(site)
                    && !(judged_by_the_quarantine_criterion
                        && quarantined_over_a_swallowed_write.contains(&site.slot))
            })
        })
        .collect()
}

/// checker 档两段判出的池级 checker 结论归类（harness 不判，那一遍的结论交回这里；要判红就停那一档口径的调用方改走
/// [`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`]）。交回归好类的失败：每步镜像里第一张判红的
/// （harness 就地判时历史停在那一步，之后的步不跑）、重开那张判红的，各一条；都不红交回空。
///
/// 「这条失败是不是说谎的设备留下的」（[`left_by_the_lying_device`]）要的「插回去仍红的不变量」按步取对照盘、按（不变量, 盘, 槽）比
/// （判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3 与「用户定案」，[`invariants_still_violated_on_the_comparison_image`]）：
/// 第一张判红的每步镜像拿它那一刻的按步对照盘比（那一刻之前被吞的写插回去），减掉那一刻之前实现交回的、压在那一刻之前被吞的写上的隔离；
/// 重开那张拿被吞的写都插回去那张比，减掉整段交回的、压在被吞的写上的隔离。
///
/// # Panics
/// `violations_of_the_step_images` 与每步镜像张数不同；按步对照盘该给没给、不该给给了；被吞的写都插回去那张盘在而没给结论、不在而给了。
#[must_use]
pub fn classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
    images: &ImagesAfterAFault,
    verdicts: PoolCheckerVerdictsOfTheImagesAfterAFault,
) -> Vec<FailureAfterAFault> {
    let PoolCheckerVerdictsOfTheImagesAfterAFault {
        mut violations_of_the_step_images,
        violation_sites_of_the_first_red_step_image,
        after_the_fault,
        at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed,
        with_the_swallowed_writes_landed,
    } = verdicts;
    assert_eq!(
        violations_of_the_step_images.len(),
        images.step_images_of_the_run_with_the_fault.len(),
        "每步镜像的结论与交出去的每步镜像逐张对应"
    );
    let first_red_step_image = violations_of_the_step_images
        .iter()
        .position(|violations| !violations.is_empty());
    let a_write_was_swallowed_before_the_first_red_step_image =
        first_red_step_image.is_some_and(|index| {
            images.step_images_of_the_run_with_the_fault[index].swallowed_writes_before_it > 0
        });
    assert_eq!(
        at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed.is_some(),
        a_write_was_swallowed_before_the_first_red_step_image,
        "第一张判红的每步镜像那一刻之前吞过写，就要给它的按步对照盘的结论；没吞过就不给"
    );
    assert_eq!(
        with_the_swallowed_writes_landed.is_some(),
        images.image_with_the_swallowed_writes_landed.is_some(),
        "被吞的写都插回去那张盘在就要给它的结论；不在就不给"
    );
    let explained_with = |invariants_still_violated: BTreeSet<&'static str>| {
        images
            .what_the_swallowed_writes_explain_without_the_pool_checker
            .clone()
            .map(|explained| {
                explained.with_the_invariants_still_violated(invariants_still_violated)
            })
    };
    let mut failures = Vec::new();
    if let Some(index) = first_red_step_image {
        let step_image = &images.step_images_of_the_run_with_the_fault[index];
        let observed = ViolationsNamingTheSites {
            violations: std::mem::take(&mut violations_of_the_step_images[index]),
            violation_sites: violation_sites_of_the_first_red_step_image,
        };
        let quarantined_over_a_swallowed_write =
            unit_start_slots_quarantined_over_a_swallowed_write(
                &images.copies_quarantined_by_the_implementation
                    [..step_image.copies_quarantined_by_the_implementation_before_it],
                &images.swallowed_writes[..step_image.swallowed_writes_before_it],
            );
        let invariants_still_violated = invariants_still_violated_on_the_comparison_image(
            &observed,
            at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed.as_ref(),
            &quarantined_over_a_swallowed_write,
        );
        let observation = FailureObservation {
            position: step_image.position,
            operation_kind: step_image.operation_kind,
            violations: observed.violations,
            panic: None,
            newest_ring_root_txg: step_image.newest_ring_root_txg,
            root_ring_slot_count: step_image.root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        failures.push(classify_a_failure_after_a_fault(
            observation,
            &images.facts_after_the_fault,
            images.drawn.fault,
            explained_with(invariants_still_violated).as_ref(),
        ));
    }
    if !after_the_fault.violations.is_empty() {
        let quarantined_over_a_swallowed_write =
            unit_start_slots_quarantined_over_a_swallowed_write(
                &images.copies_quarantined_by_the_implementation,
                &images.swallowed_writes,
            );
        let invariants_still_violated = invariants_still_violated_on_the_comparison_image(
            &after_the_fault,
            with_the_swallowed_writes_landed.as_ref(),
            &quarantined_over_a_swallowed_write,
        );
        let template = &images.observation_without_the_pool_checker_violations;
        let observation = FailureObservation {
            position: template.position,
            operation_kind: template.operation_kind,
            violations: after_the_fault.violations,
            panic: None,
            newest_ring_root_txg: template.newest_ring_root_txg,
            root_ring_slot_count: template.root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: template
                .raised_floor_lands_only_on_abandoned_roots,
            record_check: RecordCheck::default(),
        };
        failures.push(classify_a_failure_after_a_fault(
            observation,
            &images.facts_after_the_fault,
            images.drawn.fault,
            explained_with(invariants_still_violated).as_ref(),
        ));
    }
    failures
}

/// 被吞的那一次写要是落了盘，录制流里它排在第几步之前（插进去的下标）：录制流里第 `writes_recorded_before` 次写之后（屏障不改镜像，
/// 排在那几道屏障前后都一样）。
fn stream_index_the_swallowed_write_lands_at(
    operations: &[crate::RetainedOperation],
    write: &SwallowedWrite,
) -> usize {
    if write.writes_recorded_before == 0 {
        return 0;
    }
    operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .nth(write.writes_recorded_before - 1)
        .map(|(index, _)| index + 1)
        .unwrap_or_else(|| {
            panic!(
                "被吞的写之前记着 {} 次写，录制流里的写不够这么多：注入层与录制器数的写对不上",
                write.writes_recorded_before
            )
        })
}

/// 被吞的那几次写要是都落了盘，录制流该是哪一串：各自插在 [`stream_index_the_swallowed_write_lands_at`] 那一处。
/// 从写数最大的那一次往前插：插进去的写只挪它后面的，排在前面的那几次的位置不变。
fn operations_with_the_swallowed_writes_landed(
    operations: &[crate::RetainedOperation],
    swallowed: &[SwallowedWrite],
) -> Vec<crate::RetainedOperation> {
    let mut landed = operations.to_vec();
    let mut by_position: Vec<&SwallowedWrite> = swallowed.iter().collect();
    by_position.sort_by_key(|write| write.writes_recorded_before);
    for write in by_position.into_iter().rev() {
        let stream_index = stream_index_the_swallowed_write_lands_at(operations, write);
        landed.insert(stream_index, write.as_retained_operation());
    }
    landed
}

/// 被吞的写插回去之后的那张盘（[`RecordedStreamOfAnImage`]）：mkfs 的步数与掉电之后第一步的下标跟着插进去的写往后挪。
/// 插在 mkfs 那几步之后第一处（恰好插在边界上）的算 mkfs 之后写的；插在掉电之后第一步之前（恰好插在接缝上）的算掉电之前写的。
/// 写死一段历史、照入口的样子拼 [`ImagesAfterAFault`] 的用例也经它插（同一份插法）。
///
/// # Panics
/// `swallowed` 里有一次写之前记着的写数比 `stream` 里的写多（注入层与录制器数的写对不上）。
#[must_use]
pub fn stream_with_the_swallowed_writes_landed(
    stream: &RecordedStreamOfAnImage,
    swallowed: &[SwallowedWrite],
) -> RecordedStreamOfAnImage {
    let landing_indexes: Vec<usize> = swallowed
        .iter()
        .map(|write| stream_index_the_swallowed_write_lands_at(&stream.operations, write))
        .collect();
    let landed_strictly_before = |boundary: usize| {
        landing_indexes
            .iter()
            .filter(|landing_index| **landing_index < boundary)
            .count()
    };
    let landed_at_or_before = |stream_index: usize| {
        landing_indexes
            .iter()
            .filter(|landing_index| **landing_index <= stream_index)
            .count()
    };
    RecordedStreamOfAnImage {
        operations: operations_with_the_swallowed_writes_landed(&stream.operations, swallowed),
        operations_written_by_make_filesystem: stream.operations_written_by_make_filesystem
            + landed_strictly_before(stream.operations_written_by_make_filesystem),
        earliest_stream_index_after_each_power_loss: stream
            .earliest_stream_index_after_each_power_loss
            .iter()
            .map(|earliest_stream_index| {
                earliest_stream_index + landed_at_or_before(*earliest_stream_index)
            })
            .collect(),
    }
}

/// 「重开走读失败」是不是这一次注入的正确答案：盘上本来就没有一版立得住。两种：
///
/// - 注入的是说谎的设备（[`InjectedFault::the_device_lies`]）：那一份内容无声地丢了，
///   单元头里的校验和是旧那一份的，读出来发现对不上、报错正是要的结果。
/// - 注入落在起点段而 mkfs 自己就报了错：盘上没有一个做完了的文件系统，模型一版都没提交过，
///   重开走到任何一版才是缺口。
///
/// 别的一律照判：报错的设备不改盘上已有的字节，重开该走到之前某一条根上，走读失败就是缺口。
fn failed_reopen_is_the_right_answer(
    fault: InjectedFault,
    ending_observation: Option<&FailureObservation>,
) -> bool {
    fault.the_device_lies() || make_filesystem_left_no_finished_filesystem(ending_observation)
}

/// 历史停在起点而 mkfs 自己报了错、盘上没有一个做完了的文件系统（[`failed_reopen_is_the_right_answer`] 的后一种）。
fn make_filesystem_left_no_finished_filesystem(
    ending_observation: Option<&FailureObservation>,
) -> bool {
    matches!(
        ending_observation.and_then(|observation| observation.harness_judgement.as_ref()),
        Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. })
            if !step.finished_filesystem_is_on_the_devices()
    )
}

/// 一个注入点跑到池级 checker 之前的样子（[`run_one_fault_up_to_the_pool_checker`]）：harness 自己判完的那几样（理想模型、RefFS、执行器）
/// 连同计数，池级 checker 那一遍的结论还没并进来（[`settle_one_fault_with_the_pool_checker_verdicts`] 并）。
struct OneFaultBeforeThePoolChecker {
    /// 这一个注入点自己的计数，并进来时整份加进这一段的计数（截短重跑时整段那一遍的计数不加）。
    tally: FaultInjectionTally,
    /// C378 那一格（取号之后挂载报错、新号没回卷），没落进去是 None。
    acquired_instance_left: Option<AcquiredInstanceLeft>,
    /// 历史停在别的东西上（不是注入那一步返回的错）时停下那一步的那一条失败。
    failure_where_the_history_stopped: Option<FailureObservation>,
    /// 重开读回的那一版理想模型不允许时对不上的那一格。
    model_disagreement_at_the_reopen: Option<crate::model::ModelDisagreement>,
    /// 重开上 RefFS 判红的那一项（借违例那一栏带着走）。
    reference_filesystem_violation_at_the_reopen: Option<(&'static str, String)>,
    /// 交给 checker 档判的盘与归类要的事实。
    images: ImagesAfterAFault,
}

/// 照 checker 档判回的结论把一个注入点并进来（[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_checker_tier`] 的那三步）：
/// 不判池级 checker 跑一遍、交出去判；每步镜像里有判红的，照那几张的结论截短重跑一遍、再交一次判；并进来的是历史与判红就停那一档跑的那一遍。
fn merge_one_fault_with_the_pool_checker_verdicts(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn: &DrawnFault,
    pool_checker_verdicts_of: &mut dyn FnMut(&ImagesAfterAFault) -> PoolCheckerVerdictsAfterAFault,
    accumulator: &mut FaultInjectionAccumulator<'_>,
) {
    let run_through = run_one_fault_up_to_the_pool_checker(history, execution, marks, drawn, None);
    let verdicts = pool_checker_verdicts_of(&run_through.images);
    assert_eq!(
        verdicts.violations_of_the_step_images.len(),
        run_through
            .images
            .step_images_of_the_run_with_the_fault
            .len(),
        "checker 档交回的每步镜像结论与交出去的每步镜像逐张对应（种子 {} {}）",
        history.seed.0,
        drawn.render()
    );
    let Some(first_red_step_image) = verdicts
        .violations_of_the_step_images
        .iter()
        .position(|violations| !violations.is_empty())
    else {
        settle_one_fault_with_the_pool_checker_verdicts(
            &run_through,
            verdicts.violations_after_the_fault,
            verdicts.invariants_still_violated_if_the_swallowed_writes_had_landed,
            accumulator,
        );
        return;
    };
    let stopped = run_one_fault_up_to_the_pool_checker(
        history,
        execution,
        marks,
        drawn,
        Some(&verdicts.violations_of_the_step_images[..=first_red_step_image]),
    );
    assert_eq!(
        stopped
            .images
            .step_images_of_the_run_with_the_fault
            .as_slice(),
        &run_through.images.step_images_of_the_run_with_the_fault[..=first_red_step_image],
        "截短重跑那一遍停在判红的那一张、之前（含）逐张与整段那一遍相同（种子 {} {}）",
        history.seed.0,
        drawn.render()
    );
    let verdicts_of_the_stopped_run = pool_checker_verdicts_of(&stopped.images);
    assert_eq!(
        verdicts_of_the_stopped_run
            .violations_of_the_step_images
            .len(),
        stopped.images.step_images_of_the_run_with_the_fault.len(),
        "checker 档交回的每步镜像结论与截短重跑那一遍交出去的每步镜像逐张对应（种子 {} {}）",
        history.seed.0,
        drawn.render()
    );
    settle_one_fault_with_the_pool_checker_verdicts(
        &stopped,
        verdicts_of_the_stopped_run.violations_after_the_fault,
        verdicts_of_the_stopped_run.invariants_still_violated_if_the_swallowed_writes_had_landed,
        accumulator,
    );
}

/// 一个注入点跑到池级 checker 之前判完的那几样，连同池级 checker 在重开那张盘与被吞的写插回去那张盘上的结论，归好类记进这一段：
/// 停下那一步的失败（历史停在别的东西上时）、重开那张盘上的失败（池级 checker、RefFS 判红或理想模型对不上时），各一条。
fn settle_one_fault_with_the_pool_checker_verdicts(
    one_fault: &OneFaultBeforeThePoolChecker,
    pool_checker_violations_after_the_fault: Vec<(&'static str, String)>,
    invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet<&'static str>,
    accumulator: &mut FaultInjectionAccumulator<'_>,
) {
    accumulator.tally.absorb(&one_fault.tally);
    if let Some(left) = one_fault.acquired_instance_left {
        accumulator.acquired_instances_left.push(left);
    }
    let images = &one_fault.images;
    // 三、判：注入那一步返回错误的那一格不算失败；别的失败（panic、checker 判红、重开走到模型或 RefFS 不允许的版本）逐条分类。
    // RefFS 判红的那一项借违例那一栏带着走（注入真落了的每一档 RefFS 都弃权，只有注入点没走到的那几次投票；没走到时说谎的设备
    // 什么也没丢，不会落进 `left_by_the_lying_device` 的豁免）。
    let mut violations_after_the_fault = pool_checker_violations_after_the_fault;
    violations_after_the_fault.extend(
        one_fault
            .reference_filesystem_violation_at_the_reopen
            .clone(),
    );
    let mut failures: Vec<FailureObservation> = Vec::new();
    failures.extend(one_fault.failure_where_the_history_stopped.clone());
    if !violations_after_the_fault.is_empty()
        || one_fault.model_disagreement_at_the_reopen.is_some()
    {
        let template = &images.observation_without_the_pool_checker_violations;
        failures.push(FailureObservation {
            position: template.position,
            operation_kind: template.operation_kind,
            violations: violations_after_the_fault,
            panic: None,
            newest_ring_root_txg: template.newest_ring_root_txg,
            root_ring_slot_count: template.root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: one_fault.model_disagreement_at_the_reopen.clone(),
            raised_floor_lands_only_on_abandoned_roots: template
                .raised_floor_lands_only_on_abandoned_roots,
            record_check: RecordCheck::default(),
        });
    }
    let what_the_swallowed_writes_explain = images
        .what_the_swallowed_writes_explain_without_the_pool_checker
        .clone()
        .map(|explained| WhatTheSwallowedWritesExplain {
            invariants_still_violated_if_the_swallowed_writes_had_landed,
            ceiling_without_the_swallowed_roots: explained.ceiling_without_the_swallowed_roots,
            root_ring_slots_of_the_swallowed_root_slot_writes: explained
                .root_ring_slots_of_the_swallowed_root_slot_writes,
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: explained
                .instance_rows_with_the_swallowed_roots_rebuilt_from_their_records,
            byte_ranges_of_the_swallowed_writes: explained.byte_ranges_of_the_swallowed_writes,
        });
    for observation in failures {
        if observation.panic.is_some() {
            accumulator.tally.faults_that_panicked += 1;
        }
        accumulator.settle_a_failure_after_a_fault(
            images.seed,
            images.drawn,
            classify_a_failure_after_a_fault(
                observation,
                &images.facts_after_the_fault,
                images.drawn.fault,
                what_the_swallowed_writes_explain.as_ref(),
            ),
            &images.reopen,
        );
    }
}

/// 注入一次：armed 的计划重跑这段历史，判 panic、判那一段的结局、再重开判恢复到的那一版（与理想模型、RefFS 比），
/// 交回判完的这几样连同交给 checker 档判的盘（[`OneFaultBeforeThePoolChecker`]）。harness 不判池级 checker：
/// `pool_checker_violations_of_the_step_images` 是 None 时历史不因池级 checker 停；是 `Some` 时那是 checker 档在整段那一遍交出去的每步镜像上
/// 判红的（到第一张判红的为止），历史照判红就停那一档的口径停在第一张判红的那一步（合并入口截短重跑的那一遍）。
///
/// # Panics
/// 给了每步镜像的结论而历史没停在最后那一张上（交出去的镜像比给的结论多：同一个种子、同一个注入点重跑，执行器不确定了）。
// gpu-exempt: comparison-with-the-ideal-model
// gpu-exempt: reference-filesystem
fn run_one_fault_up_to_the_pool_checker(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn: &DrawnFault,
    pool_checker_violations_of_the_step_images: Option<&[Vec<(&'static str, String)>]>,
) -> OneFaultBeforeThePoolChecker {
    let mut tally = FaultInjectionTally::default();
    let geometry = execution.device_width.fixed_geometry();
    let plan = SharedFaultPlan::armed(
        geometry,
        FaultSchedule::the_nth_call_across_the_pool(drawn.fault, drawn.call_ordinal),
    );
    let stream = SharedStream::retaining_contents();
    let mut committed_by_the_armed_run: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    // 最后一次观察那一刻的模型与录制流长度：历史停下的那一步不调观察者，它之前那一步的这两样就是「停下那一步开始时」的。
    let mut model_at_the_last_observation: Option<crate::model::IdealModel> = None;
    let mut stream_length_at_the_last_observation = 0usize;
    // RefFS 的输入（G1 用户定案：重开的判定 = RefFS ∧ O1 判官全四条）：开着注入跑的那一遍逐步攒，停下的那一步跑完之后补。
    let mut reference_filesystem_translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    // 带着注入跑的那一遍每一步的镜像（池级 checker 本来在那一刻判的那几张）：只记它在录制流里的位置与根环，盘本身随重开那一份交出去
    let mut step_images_of_the_run_with_the_fault: Vec<StepImageInTheRecordedStream> = Vec::new();
    let run = execute_history_stopping_at_the_given_pool_checker_violations(
        history,
        execution,
        &stream,
        &plan,
        &mut |observation| {
            for (key, file) in observation.model.committed_versions() {
                committed_by_the_armed_run.insert(key, file);
            }
            model_at_the_last_observation = Some(observation.model.clone());
            stream_length_at_the_last_observation = stream.operation_count();
            reference_filesystem_translation
                .note_an_observed_step(observation, stream_length_at_the_last_observation);
        },
        &mut |step_image| {
            let (newest_ring_root_txg, root_ring_slot_count) =
                newest_ring_root_and_slot_count(step_image.image);
            step_images_of_the_run_with_the_fault.push(StepImageInTheRecordedStream {
                position: step_image.position,
                operation_kind: step_image.operation_kind,
                operations_recorded: step_image.operations_recorded,
                newest_ring_root_txg,
                root_ring_slot_count,
                swallowed_writes_before_it: plan.swallowed_write_count(),
                copies_quarantined_by_the_implementation_before_it: step_image
                    .copies_quarantined_by_the_implementation_up_to_this_step,
            });
        },
        &mut |step_image_index| {
            match pool_checker_violations_of_the_step_images {
            None => Vec::new(),
            Some(violations_of_the_step_images) => violations_of_the_step_images
                .get(step_image_index)
                .cloned()
                .unwrap_or_else(|| {
                    panic!(
                        "截短重跑那一遍要停在给了结论的最后一张上，交出去了第 {step_image_index} 张（给了 {} 张的结论）",
                        violations_of_the_step_images.len()
                    )
                }),
        }
        },
    );
    reference_filesystem_translation.note_the_step_the_history_stopped_in(
        &history.operations,
        &run,
        stream.operation_count(),
    );
    let fired = plan.fired();
    tally.faults += 1;
    *tally.faults_by_kind.entry(drawn.fault.name()).or_insert(0) += 1;
    if let Some(first) = fired.first() {
        let point = InjectionPoint {
            segment: drawn.segment.kind(),
            call: drawn.fault.call_kind(),
            written_structure: first.written_structure,
        };
        *tally
            .faults_by_injection_point
            .entry(point.render())
            .or_insert(0) += 1;
    }

    let ending_observation = match &run.ending {
        HistoryEnding::Completed => None,
        HistoryEnding::KnownRed { observation, .. }
        | HistoryEnding::NewFinding { observation, .. } => Some(observation),
    };
    let surfaced = ending_observation.is_some_and(|observation| {
        the_fault_surfaced_as_an_error(&run, observation, drawn.segment)
    });
    if surfaced {
        if let Some(StepOutcome::Refused { member }) = run.outcomes.last() {
            *tally.refusal_members.entry(member.clone()).or_insert(0) += 1;
        }
        if let Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) =
            ending_observation.and_then(|observation| observation.harness_judgement.as_ref())
        {
            *tally
                .starting_point_failures
                .entry(step.name())
                .or_insert(0) += 1;
        }
    }
    let outcome = if fired.is_empty() {
        FaultOutcome::NeverFired
    } else if surfaced {
        FaultOutcome::SurfacedAsAnError
    } else {
        match &run.ending {
            HistoryEnding::Completed => FaultOutcome::ToleratedAndTheHistoryFinished,
            HistoryEnding::KnownRed { .. } | HistoryEnding::NewFinding { .. } => {
                FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError
            }
        }
    };
    *tally.faults_by_outcome.entry(outcome.name()).or_insert(0) += 1;

    // 二、重开：拿录制流（只记真正落到盘上的那些写）重建镜像，跑恢复，问模型走到的这一版允不允许。
    let operations = stream.retained_operations();
    let mut image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    image.apply(&operations);
    let (writes, _segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &geometry);
    let all_persisted = vec![true; writes.len()];
    let newest_persisted = newest_persisted_root(&writes, &all_persisted)
        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    let recovery = recover(&image, JournalPolicy::Consult);
    tally.reopens += 1;
    match &recovery.outcome {
        RecoveryOutcome::FileRead { .. } => tally.reopens_reading_a_file += 1,
        RecoveryOutcome::NoFile { .. } => tally.reopens_without_a_file += 1,
        RecoveryOutcome::Failed { .. } => tally.reopens_that_failed += 1,
    }
    let read_back = observed_read_back_after_a_crash(&recovery);
    // 模型认下来的每一版：这段历史（开着注入跑的那一遍）停下时模型根环里的那几条。观察者攒的那一份漏掉失败那一步——
    // 判出失败的那一步不调观察者，而模型常常已经认下那一版了，两份并起来才完整。
    let mut committed_by_the_model = committed_by_the_armed_run.clone();
    for (key, file) in &run.committed_versions_at_the_end {
        committed_by_the_model.insert(
            *key,
            file.as_ref().map(|content| Rc::from(content.as_slice())),
        );
    }
    // 再放行一样：不注入时失败那一步会写出的那几版（测量跑记的那一份）。
    let mut allowed = committed_by_the_model.clone();
    if let Some(index) = marks
        .iter()
        .position(|mark| mark.position == drawn.segment.position())
    {
        for (key, file) in &marks[index].committed_versions {
            allowed.insert(*key, file.clone());
        }
    }
    // 再放行一样（主 agent 2026-09-26 定，实七）：历史停下的那一步里实现已经写出的根——最后一次观察之后录制流里落盘的根槽写
    // （说谎的设备让历史停在注入之后的另一步时，那一步写出的根不在上面两份里；实四丙交回 ④ offset 284）。
    // 那条根的内容不另信实现：读回的正是它、而且读回的内容是模型认过的某一版，或停下那一步这次发布自己的内容，才放行。
    let roots_written_by_the_stopping_step: BTreeSet<ModelRootKey> = writes
        .iter()
        .zip(&stream_indexes)
        .filter(|(write, stream_index)| {
            write.kind == StepKind::RootRecordFua
                && **stream_index >= stream_length_at_the_last_observation
        })
        .filter_map(|(write, _)| write.bytes().map(root_identity_written_by))
        .map(|(instance, checkpoint_txg)| model_root_key(instance, checkpoint_txg))
        .collect();
    let content_published_by_the_stopping_step: Option<Vec<u8>> = ending_observation
        .and_then(|observation| match observation.position {
            StepPosition::Operation(step_index) => history.operations.get(step_index),
            StepPosition::StartingPoint => None,
        })
        .and_then(|operation| match operation {
            HistoryOperation::PublishFirstFile(content)
            | HistoryOperation::PublishOverwrite(content) => Some(content.bytes()),
            // 按 inode 号写点名的号执行时才取（`history::WriteOfAnInodeChoice::inode_among`），这里分不出写的是不是第一个文件：照写第一个文件放行
            // 这一步交的内容（写的是别的文件时第一个文件的内容不变，模型认过的那几版里本来就有它）。
            HistoryOperation::WriteOfAnInode(choice) => Some(choice.content.bytes()),
            HistoryOperation::PublishWithoutUnits
            | HistoryOperation::CloseAndMountWritable
            | HistoryOperation::RollBackWhileMounted(_)
            | HistoryOperation::RaiseRollbackFloor(_)
            | HistoryOperation::ColdStartRecover
            | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
            | HistoryOperation::Unlink
            | HistoryOperation::CreateInode
            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_)
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            // 建快照、读快照、删快照不改文件内容（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2、S4）。
            | HistoryOperation::CreateSnapshot
            | HistoryOperation::ReadSnapshot(_)
            | HistoryOperation::DestroySnapshot(_)
            // 整理只搬单元的落点，不改文件内容（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）。
            | HistoryOperation::CompactWhileMounted(_)
            // 截断改到的长度执行时按那一刻的 size 取（`history::TruncationChoice`），这里取不到它的内容：照不改内容的那几种只放行模型认过的
            // 那几版与测量跑记的那几版（截断那一步不注入时写出的截后内容在后一份里）。
            | HistoryOperation::Truncate(_)
            // 建 inode 不改第一个文件的内容。
            | HistoryOperation::CreateInodes(_)
            // 按偏移写的号、偏移与长度执行时才取（`history::PartialWriteChoice`），这里取不到写之后的内容：照截断那样只放行模型认过的那几版与
            // 测量跑记的那几版。
            | HistoryOperation::WriteAtOffset(_) => None,
        });
    let (read_back_root, read_back_content): (Option<ModelRootKey>, Option<&[u8]>) =
        match &read_back {
            ObservedReadBack::FileRead { root, content } => (Some(*root), Some(content.as_slice())),
            ObservedReadBack::NoFile { root } => (Some(*root), None),
            ObservedReadBack::Failed { .. } => (None, None),
        };
    if let Some(root) = read_back_root {
        let content_is_one_the_model_knows = allowed
            .values()
            .any(|file| file.as_deref() == read_back_content)
            || (read_back_content.is_some()
                && read_back_content == content_published_by_the_stopping_step.as_deref());
        if roots_written_by_the_stopping_step.contains(&root)
            && !allowed.contains_key(&root)
            && content_is_one_the_model_knows
        {
            allowed.insert(root, read_back_content.map(Rc::from));
        }
    }
    let nothing_was_left_to_recover = matches!(read_back, ObservedReadBack::Failed { .. })
        && failed_reopen_is_the_right_answer(drawn.fault, ending_observation);
    let disagreement = if nothing_was_left_to_recover {
        None
    } else {
        crash_recovery_disagreement(&allowed, newest_persisted, &read_back)
    };
    // RefFS 那一票：重开 = 停下之后崩、全部步都返回过（停下那一步报错的按「写了之后报错」两可，panic 的按在途）。注入真落了的每一档
    // RefFS 都弃权、O1 判官照旧：U1 射程句「带故障的状态（说谎的设备、丢整块盘 C519）不适用」管全部带故障的状态，括号里是举例
    // （调度记录第 12 批「12-2 交回」那一条）；注入点没走到的不带故障、照判（代码轮第三轮判决第三节第 1 条），说谎的设备没说谎，
    // 盘上有没有一版立得住只看 mkfs 做没做完。弃权按理由计进 `reference_filesystem_abstentions_at_the_reopen`。
    let no_version_stands_for_the_reference_filesystem = match outcome {
        FaultOutcome::NeverFired => make_filesystem_left_no_finished_filesystem(ending_observation),
        FaultOutcome::SurfacedAsAnError
        | FaultOutcome::ToleratedAndTheHistoryFinished
        | FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError => {
            failed_reopen_is_the_right_answer(drawn.fault, ending_observation)
        }
    };
    let reference_filesystem_vote = reference_filesystem_translation
        .translate(&operations, &geometry)
        .judge_a_reopen_after_an_injected_fault(
            drawn.fault,
            outcome,
            if no_version_stands_for_the_reference_filesystem {
                WhatTheReopenMustRecover::NothingBecauseNoVersionStands
            } else {
                WhatTheReopenMustRecover::AVersionTheHistoryLeft
            },
            &read_back,
        );
    *tally
        .reopens_by_the_reference_filesystem_and_model_verdicts
        .entry(JointVerdictOfTheReferenceFileSystemAndTheModel::of(
            &reference_filesystem_vote,
            disagreement.as_ref(),
        ))
        .or_insert(0) += 1;
    if let ReferenceFileSystemVote::Abstained(reason) = &reference_filesystem_vote {
        *tally
            .reference_filesystem_abstentions_at_the_reopen
            .entry(reason.name())
            .or_insert(0) += 1;
    }
    let reopened = if nothing_was_left_to_recover {
        tally.reopens_that_correctly_found_nothing_to_recover += 1;
        ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo
    } else if crash_recovery_disagreement(&committed_by_the_model, newest_persisted, &read_back)
        .is_none()
    {
        tally.reopened_into_a_version_the_model_committed += 1;
        ReopenedVersion::CommittedByTheModel
    } else if disagreement.is_none() {
        tally.reopened_into_the_version_the_faulted_step_was_writing += 1;
        ReopenedVersion::TheVersionTheFaultedStepWasWriting
    } else {
        tally.reopens_outside_everything_the_model_allows += 1;
        ReopenedVersion::OutsideEverythingTheModelAllows
    };
    // 池级 checker 那一遍归 checker 档两段（重开的盘随交回的那一份交出去），harness 一条不变量都不判
    let reopen_text = format!("{reopened:?}（{}）", describe_read_back(&read_back));
    // C378：注入落在一次挂载里、那一步返回了错误，而重开的盘上系统配置的实例代号比这一步之前大——取号之后的写行或暖机报了错，
    // 新号没回卷。认下的已知行为：记进这一格、在报告里点名，不进新发现，也不挡下面的判定。
    let acquired_instance_left = if surfaced {
        acquired_instance_left_after_a_failed_mount(&image, marks, *drawn)
    } else {
        None
    };
    if let Some(left) = &acquired_instance_left {
        *tally
            .acquired_instances_left_after_a_failed_mount
            .entry(left.kind.name())
            .or_insert(0) += 1;
    }

    // 说谎的设备吞掉的写（只有 `WriteIsSwallowed` 有）要是都落了盘，盘面与抬 F 的上限该是什么样：判「这条失败是不是它留下的」要的两样。
    let swallowed_writes = plan.swallowed_writes();
    let stream_after_the_fault = RecordedStreamOfAnImage {
        operations,
        operations_written_by_make_filesystem: run.operations_written_by_make_filesystem,
        earliest_stream_index_after_each_power_loss: run
            .earliest_stream_index_after_each_power_loss
            .clone(),
    };
    let stream_with_the_swallowed_writes_landed = (!swallowed_writes.is_empty()).then(|| {
        stream_with_the_swallowed_writes_landed(&stream_after_the_fault, &swallowed_writes)
    });
    let what_the_swallowed_writes_explain = (!swallowed_writes.is_empty()).then(|| {
        // 池级 checker 那一项（插回去之后仍红的不变量）在并进来那一刻由 checker 档的结论填：那张盘随交回的那一份交出去判
        let invariants_still_violated_if_the_swallowed_writes_had_landed = BTreeSet::new();
        let swallowed_root_slot_writes: Vec<&SwallowedWrite> = swallowed_writes
            .iter()
            .filter(|write| {
                geometry.classify(&write.as_retained_operation().operation)
                    == StepKind::RootRecordFua
            })
            .collect();
        let swallowed_roots: Vec<ModelRootKey> = swallowed_root_slot_writes
            .iter()
            .filter_map(|write| match &write.contents {
                SwallowedContents::Bytes(bytes) => Some(root_identity_written_by(bytes)),
                SwallowedContents::Zeroes { .. } | SwallowedContents::Discard { .. } => None,
            })
            .map(|(instance, checkpoint_txg)| model_root_key(instance, checkpoint_txg))
            .collect();
        let ceiling_without_the_swallowed_roots = if swallowed_roots.is_empty() {
            None
        } else {
            model_at_the_last_observation
                .as_ref()
                .map(|model| model.rollback_floor_ceiling_without_the_roots(&swallowed_roots))
        };
        let parameters = execution.device_width.parameters();
        let every_slot_of_the_ring = RootRingSlotTarget {
            named_slots: NamedRootRingSlots::every_slot_in_the_ring(
                parameters.geometry.root_ring_slots_per_region,
            ),
            region_devices: parameters.region_devices,
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        };
        let root_ring_slots_of_the_swallowed_root_slot_writes = swallowed_root_slot_writes
            .iter()
            .filter_map(|write| every_slot_of_the_ring.slot_covering(write.device, write.offset))
            .map(model_ring_position)
            .collect();
        let instance_rows_with_the_swallowed_roots_rebuilt_from_their_records =
            if swallowed_roots.is_empty() {
                None
            } else {
                model_at_the_last_observation
                    .as_ref()
                    .and_then(|model| {
                        model
                            .answer_mount_writable_with_the_roots_rebuilt_from_their_records(
                                &swallowed_roots,
                            )
                            .ok()
                    })
                    .and_then(|answer| answer.expected_mount)
                    .map(|(_, rows)| rows)
            };
        WhatTheSwallowedWritesExplain {
            invariants_still_violated_if_the_swallowed_writes_had_landed,
            ceiling_without_the_swallowed_roots,
            root_ring_slots_of_the_swallowed_root_slot_writes,
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records,
            byte_ranges_of_the_swallowed_writes: swallowed_writes
                .iter()
                .map(SwallowedWriteByteRange::of)
                .collect(),
        }
    });

    let failure_where_the_history_stopped = if surfaced {
        None
    } else {
        ending_observation.cloned()
    };
    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
    let raised_floor_lands_only_on_abandoned_roots =
        raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(&image);
    let facts_after_the_fault = FactsAfterTheFault {
        starting_point_step_that_returned_an_error: ending_observation.and_then(|observation| {
            match &observation.harness_judgement {
                Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => Some(*step),
                Some(_) | None => None,
            }
        }),
        system_configuration_slot_writes_that_landed: writes
            .iter()
            .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
            .count(),
        reopened,
    };

    // 四、交回：判完的这几样，连同交给 checker 档判的盘（池级 checker 那一遍归 checker 档两段）
    let images = ImagesAfterAFault {
        seed: history.seed,
        drawn: *drawn,
        device_width: execution.device_width,
        image_after_the_fault: stream_after_the_fault,
        step_images_of_the_run_with_the_fault,
        image_with_the_swallowed_writes_landed: stream_with_the_swallowed_writes_landed,
        swallowed_writes,
        copies_quarantined_by_the_implementation: run.copies_quarantined_by_the_implementation,
        observation_without_the_pool_checker_violations: FailureObservation {
            position: drawn.segment.position(),
            operation_kind: drawn.segment.operation_kind(),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg,
            root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots,
            record_check: RecordCheck::default(),
        },
        facts_after_the_fault,
        what_the_swallowed_writes_explain_without_the_pool_checker:
            what_the_swallowed_writes_explain,
        reopen: reopen_text,
    };
    OneFaultBeforeThePoolChecker {
        tally,
        acquired_instance_left,
        failure_where_the_history_stopped,
        model_disagreement_at_the_reopen: disagreement,
        reference_filesystem_violation_at_the_reopen: reference_filesystem_vote.violation(),
        images,
    }
}

/// 重开读回了什么，给人看的一行。
fn describe_read_back(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => {
            format!(
                "走到实例 {} 第 {} 代根、没有文件",
                root.instance.0, root.checkpoint_txg.0
            )
        }
        ObservedReadBack::FileRead { root, content } => format!(
            "走到实例 {} 第 {} 代根、读回 {} 字节",
            root.instance.0,
            root.checkpoint_txg.0,
            content.len()
        ),
        ObservedReadBack::Failed { what } => format!("走读失败：{what}"),
    }
}

/// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultInjectionWorkerThreads {
    /// 环境变量 [`FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    FromTheEnvironmentVariable(usize),
    /// 没设环境变量，取每台默认重活线程数（[`crate::heavy_threads::default_heavy_threads`]）。
    FromTheHeavyThreadDefault(usize),
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl FaultInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数同样停。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_value(
            std::env::var(FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE),
            process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        )
    }

    /// [`Self::from_the_environment`] 的判定本身：环境变量读到什么、`SINGLEFS_THREAD_CAP` 的值（`None` 是没设）都由调用方给
    /// （用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串；解析经数值环境变量共用的那一份，消息带变量名与原值），或不是 UTF-8；
    /// 落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
    #[must_use]
    fn from_the_environment_value(
        variable: Result<String, std::env::VarError>,
        thread_cap_value: Option<&str>,
    ) -> Self {
        let value = match variable {
            Ok(text) => Some(text),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")
            }
        };
        match number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
            FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE,
            value.as_deref(),
        ) {
            Some(count) => Self::FromTheEnvironmentVariable(count.get()),
            None => Self::FromTheHeavyThreadDefault(
                default_heavy_threads_of_the_cap_value(thread_cap_value).get(),
            ),
        }
    }

    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::FromTheEnvironmentVariable(count)
            | Self::FromTheHeavyThreadDefault(count)
            | Self::GivenByCaller(count) => count,
        }
    }

    #[must_use]
    pub fn source_name(self) -> &'static str {
        match self {
            Self::FromTheEnvironmentVariable(_) => "environment_variable",
            Self::FromTheHeavyThreadDefault(_) => "heavy_thread_default",
            Self::GivenByCaller(_) => "given_by_caller",
        }
    }
}

/// 一次故障注入跑什么：种子区间、每段几步、每段注入几次、比重、历史怎么跑、线程数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaultInjectionCampaign {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub faults_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub worker_threads: FaultInjectionWorkerThreads,
}

/// 一批种子跑下来的故障注入报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultInjectionReport {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub faults_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub tally: FaultInjectionTally,
    pub known_red_hits: Vec<KnownRedAtAFault>,
    pub new_findings: Vec<FaultFinding>,
}

impl FaultInjectionReport {
    /// 给人看的一整块：跑了什么、计数、「已知红」命中、新发现。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!(
            "故障注入：种子 [{}, {})（种子基是这个测试周期写死的那一个）、每段 {} 步、每段注入 {} 次、比重 {}、{}\n",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.faults_per_history,
            self.weights.name,
            self.execution.name()
        );
        text.push_str(&self.tally.render());
        let mut known_red_by_form: BTreeMap<KnownRedFormAtAFault, Vec<&KnownRedAtAFault>> =
            BTreeMap::new();
        for hit in &self.known_red_hits {
            known_red_by_form.entry(hit.form).or_default().push(hit);
        }
        for (form, hits) in &known_red_by_form {
            let first = hits.first().expect("这一条至少有一次命中");
            let _ = writeln!(
                text,
                "「已知红」{}：{} 次，第一次在种子 {}（{}）",
                form.render(),
                hits.len(),
                first.seed.0,
                first.drawn.render()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render());
        }
        text
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，
/// 线程按片号从小到大领片，调用线程收到一片就打一行 `FAULT_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），
/// 再按片号从小到大并计数。计数按片的次序相加、新发现按种子从小到大留第一个，所以
/// [`FaultInjectionReport::render`] 与线程数、调度次序无关（进度行是按到达次序打的，只报跑到哪了，不带判定）。
///
/// # Panics
/// 有一片领了却没交回。
#[must_use]
pub fn run_fault_injection_campaign(campaign: &FaultInjectionCampaign) -> FaultInjectionReport {
    run_fault_injection_campaign_handing_out_the_images(campaign, &|_| {})
}

/// 同 [`run_fault_injection_campaign`]，另把每个注入点重开的盘交给 `hand_out`（[`inject_faults_into_history_handing_out_the_images`]）：
/// 工作线程各自调它，交出去的次序按线程领片的次序、不按种子（调用方要按种子归并就照 [`ImagesAfterAFault::seed`] 与注入点排）。
///
/// # Panics
/// 同 [`run_fault_injection_campaign`]。
#[must_use]
pub fn run_fault_injection_campaign_handing_out_the_images(
    campaign: &FaultInjectionCampaign,
    hand_out: &(dyn Fn(ImagesAfterAFault) + Sync),
) -> FaultInjectionReport {
    let FaultInjectionCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights,
        execution,
        worker_threads,
    } = *campaign;
    let slices = seed_slices(seed_count, worker_threads.count());
    let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
    let started = Instant::now();
    println!(
        "FAULT_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
        first_seed.wrapping_add(seed_count),
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name()
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryFaultInjection>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            scope.spawn(move || loop {
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let injections: Vec<HistoryFaultInjection> = slice
                    .clone()
                    .map(|offset| {
                        let history = generate_history_with_weights(
                            HistorySeed(first_seed.wrapping_add(offset)),
                            operations_per_history,
                            &weights,
                        );
                        inject_faults_into_history_handing_out_the_images(
                            &history,
                            execution,
                            faults_per_history,
                            &mut |images| hand_out(images),
                        )
                    })
                    .collect();
                if sender.send((slice_index, injections)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut waiting_for_earlier_slices: BTreeMap<usize, Vec<HistoryFaultInjection>> =
            BTreeMap::new();
        let mut in_order: Vec<HistoryFaultInjection> = Vec::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_histories = 0u64;
        for (finished_slice_count, (slice_index, injections)) in receiver.iter().enumerate() {
            let slice = &slices[slice_index];
            finished_histories += slice.end - slice.start;
            println!(
                "FAULT_INJECTION_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
                slice_index + 1,
                slices.len(),
                first_seed.wrapping_add(slice.start),
                first_seed.wrapping_add(slice.end),
                finished_slice_count + 1,
                slices.len(),
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(slice_index, injections);
            while let Some(ready) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                in_order.extend(ready);
                next_slice_to_merge += 1;
            }
        }
        (in_order, next_slice_to_merge)
    });
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "FAULT_INJECTION_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}",
        first_seed.wrapping_add(seed_count),
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut tally = FaultInjectionTally::default();
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<FaultFinding> = Vec::new();
    let mut signatures_seen: BTreeSet<FailureSignature> = BTreeSet::new();
    for injection in &finished_slices {
        tally.absorb(&injection.tally);
        known_red_hits.extend(injection.known_red_hits.iter().cloned());
        for finding in &injection.new_findings {
            if signatures_seen.insert(finding.signature.clone()) {
                new_findings.push(finding.clone());
            }
        }
    }
    FaultInjectionReport {
        first_seed,
        seed_count,
        operations_per_history,
        faults_per_history,
        weights,
        execution,
        tally,
        known_red_hits,
        new_findings,
    }
}

/// 把 [0, `seed_count`) 切成首尾相接的种子区间：片数取 min(种子数, 4 × 线程数)，各片长度相差至多 1。
fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Range<u64>> {
    if seed_count == 0 {
        return Vec::new();
    }
    let wanted = u64::try_from(worker_threads.max(1) * SLICES_PER_WORKER_THREAD).expect("片数");
    let slice_count = wanted.min(seed_count);
    // 种子数 × 片号在 u128 里乘：种子数是用户给的，接近 u64 上限时 u64 里乘就溢出
    let bound_of = |slice_index: u64| -> u64 {
        u64::try_from(u128::from(seed_count) * u128::from(slice_index) / u128::from(slice_count))
            .expect("种子数 × 片号 ÷ 片数不超过种子数，装得回 u64")
    };
    (0..slice_count)
        .map(|slice_index| bound_of(slice_index)..bound_of(slice_index + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{CapturedPanic, StartingPointStep};
    use crate::memory_pool::SparseBlockDevice;
    use crate::model::ModelDisagreement;

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
    fn a_zero_length_discard_answers_applied_and_neither_counts_nor_takes_an_injection() {
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
    /// 问的写数还没到、或到了之后还没收到屏障时没有。可写挂载取号写两轮，挂载窗口从第二轮写之后那一道屏障起，问的就是写数 2。
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

    /// 线程数从环境变量取，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取登记值，不看本机核数）；
    /// 设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn worker_thread_count_comes_from_the_environment_variable_or_the_heavy_thread_default() {
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(Ok("3".to_string()), Some("5")),
            FaultInjectionWorkerThreads::FromTheEnvironmentVariable(3),
            "自己的变量先读"
        );
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                Some("5")
            ),
            FaultInjectionWorkerThreads::FromTheHeavyThreadDefault(5),
            "没设自己的变量读 SINGLEFS_THREAD_CAP"
        );
        let registered = crate::heavy_threads::registered_heavy_threads().get();
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                None
            ),
            FaultInjectionWorkerThreads::FromTheHeavyThreadDefault(registered),
            "都没设取 .claude/heavy-thread-default 的登记值"
        );
        assert_eq!(
            FaultInjectionWorkerThreads::FromTheHeavyThreadDefault(registered).source_name(),
            "heavy_thread_default"
        );
    }

    /// 线程数变量写成认不出的字样：经数值环境变量共用的那一份解析 panic，消息带变量名与原值，不悄悄退回默认。
    #[test]
    #[should_panic(expected = "SINGLEFS_FAULT_INJECTION_WORKER_THREADS=\"twelve\" 认不出")]
    fn an_unreadable_fault_injection_worker_thread_count_panics_naming_the_variable_and_the_value()
    {
        let _threads =
            FaultInjectionWorkerThreads::from_the_environment_value(Ok("twelve".to_string()), None);
    }

    /// 随机抽的六种全抽得到：`draw_injected_fault` 的 `below(6)` 改回 `below(4)`，说谎那两种一次都抽不到，这里必红
    /// （判决第二节改法一钉的那一格；`crates/mutations.tsv` 同名的那一条）。
    #[test]
    fn the_random_draw_reaches_all_six_injected_faults() {
        let mut random = SeededRandomSource::from_seed(0x46_41_55_4c_54_44_52_57);
        let mut drawn: BTreeSet<&'static str> = BTreeSet::new();
        for _ in 0..200 {
            drawn.insert(draw_injected_fault(&mut random).name());
        }
        assert_eq!(
            drawn.len(),
            6,
            "六种注入都该抽得到，200 次里抽到的是 {drawn:?}"
        );
    }

    /// 按注入点认说谎的设备留下的失败（`left_by_the_lying_device`，主 agent 2026-09-26 定，实七）：被吞的写插回去就消失的违例、
    /// 去掉被吞的根之后模型重算对得上的上限才认；插回去仍红的违例、重算对不上的上限、别的模型格、panic、执行器判出的一概照报。
    #[test]
    fn only_failures_that_the_swallowed_writes_explain_are_left_by_the_lying_device() {
        let observation =
            |violations: &[&'static str],
             disagreement: Option<ModelDisagreement>,
             panic: Option<CapturedPanic>| FailureObservation {
                position: StepPosition::Operation(4),
                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                violations: violations
                    .iter()
                    .map(|invariant| (*invariant, "盘 0 槽 50180".to_string()))
                    .collect(),
                panic,
                newest_ring_root_txg: None,
                root_ring_slot_count: None,
                harness_judgement: None,
                model_disagreement: disagreement,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            };
        let ceiling_disagreement = |reported: u64| ModelDisagreement {
            aspect: ModelDisagreementAspect::RollbackFloorCeiling,
            model_answer: "上限 Some(3)".to_string(),
            implementation_answer: format!("实现报上限 {reported}"),
            implementation_reported_ceiling: Some(ModelCheckpointTxg(reported)),
            implementation_reported_unreadable_place: None,
            implementation_rows_written: None,
        };
        let explained = |still_violated: &[&'static str], ceiling: Option<Option<u64>>| {
            WhatTheSwallowedWritesExplain {
                invariants_still_violated_if_the_swallowed_writes_had_landed: still_violated
                    .iter()
                    .copied()
                    .collect(),
                ceiling_without_the_swallowed_roots: ceiling
                    .map(|recomputed| recomputed.map(ModelCheckpointTxg)),
                root_ring_slots_of_the_swallowed_root_slot_writes: BTreeSet::new(),
                instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: None,
                byte_ranges_of_the_swallowed_writes: BTreeSet::new(),
            }
        };
        let lost_unit = ["I-2.1", "I-3.10", "I-4.8", "I-7.4"];
        assert!(left_by_the_lying_device(
            &observation(&lost_unit, None, None),
            &explained(&[], None)
        ));
        assert!(
            !left_by_the_lying_device(
                &observation(&lost_unit, None, None),
                &explained(&["I-3.10"], None)
            ),
            "被吞的写插回去 I-3.10 仍红：不是注入点造成的"
        );
        assert!(left_by_the_lying_device(
            &observation(&[], Some(ceiling_disagreement(0)), None),
            &explained(&[], Some(Some(0)))
        ));
        for (recomputed, why) in [
            (Some(Some(1)), "去掉被吞的根之后重算的上限不等于实现报的"),
            (Some(None), "去掉被吞的根之后模型算不出上限"),
            (None, "被吞的写里没有根槽写，或没有停下那一步之前的模型"),
        ] {
            assert!(
                !left_by_the_lying_device(
                    &observation(&[], Some(ceiling_disagreement(0)), None),
                    &explained(&[], recomputed)
                ),
                "{why}"
            );
        }
        let read_back_disagreement = ModelDisagreement {
            aspect: ModelDisagreementAspect::ColdStartReadBack,
            model_answer: "读回 (5,19)".to_string(),
            implementation_answer: "读回 (5,18)".to_string(),
            implementation_reported_ceiling: None,
            implementation_reported_unreadable_place: None,
            implementation_rows_written: None,
        };
        assert!(
            !left_by_the_lying_device(
                &observation(&[], Some(read_back_disagreement), None),
                &explained(&[], Some(Some(0)))
            ),
            "上限之外的模型格不认"
        );
        assert!(
            !left_by_the_lying_device(
                &observation(
                    &lost_unit,
                    None,
                    Some(CapturedPanic {
                        location: "crates/singlefs-core/src/transaction.rs:1".to_string(),
                        message: "断言".to_string(),
                    })
                ),
                &explained(&[], None)
            ),
            "说谎的设备也不许把实现打 panic"
        );
    }

    /// 按注入点认的另两格（实八）：实现拒抬 F、报读坏重读仍坏的那个根环槽正是被吞的根槽写的槽，才认——不管模型要它成还是按上限拒；
    /// 槽对不上、实现没报槽（别的拒绝，或拒之前写了盘）一概照报。实现写的实例表行等于「被吞的根由记录重建」时模型重算的行，才认；
    /// 行对不上、模型重算不出，照报。
    #[test]
    fn a_refused_floor_raise_and_the_rows_written_are_left_by_the_lying_device_only_when_the_swallowed_root_slot_write_explains_them(
    ) {
        let observation = |disagreement: ModelDisagreement| FailureObservation {
            position: StepPosition::Operation(5),
            operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: Some(disagreement),
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let swallowed_slot = ModelRingPosition {
            region: 1,
            slot_in_region: 1,
        };
        let another_slot = ModelRingPosition {
            region: 2,
            slot_in_region: 1,
        };
        let refusal = |aspect: ModelDisagreementAspect, reported: Option<ModelRingPosition>| {
            ModelDisagreement {
                aspect,
                model_answer: "RaiseRollbackFloor 该成".to_string(),
                implementation_answer:
                    "拒了：MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread"
                        .to_string(),
                implementation_reported_ceiling: None,
                implementation_reported_unreadable_place: reported
                    .map(ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread),
                implementation_rows_written: None,
            }
        };
        let row = |applied_transaction_high_water: u64| ModelInstanceRow {
            instance: crate::model::ModelInstanceGeneration(1),
            selected_root_txg: ModelCheckpointTxg(4),
            applied_transaction_high_water,
        };
        let rows_disagreement = |written: Option<Vec<ModelInstanceRow>>| ModelDisagreement {
            aspect: ModelDisagreementAspect::InstanceRows,
            model_answer: "写行 [(1, 4, 0)]".to_string(),
            implementation_answer: "写行 [(1, 4, 2)]".to_string(),
            implementation_reported_ceiling: None,
            implementation_reported_unreadable_place: None,
            implementation_rows_written: written,
        };
        let explained = |slots: &[ModelRingPosition], rows: Option<Vec<ModelInstanceRow>>| {
            WhatTheSwallowedWritesExplain {
                invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
                ceiling_without_the_swallowed_roots: None,
                root_ring_slots_of_the_swallowed_root_slot_writes: slots.iter().copied().collect(),
                instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: rows,
                byte_ranges_of_the_swallowed_writes: BTreeSet::new(),
            }
        };
        for aspect in [
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            ModelDisagreementAspect::RefusalReason,
        ] {
            assert!(
                left_by_the_lying_device(
                    &observation(refusal(aspect, Some(swallowed_slot))),
                    &explained(&[swallowed_slot], None)
                ),
                "{aspect:?}：读坏的正是被吞那次写的槽"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, Some(another_slot))),
                    &explained(&[swallowed_slot], None)
                ),
                "{aspect:?}：读坏的槽不是被吞那次写的"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, None)),
                    &explained(&[swallowed_slot], None)
                ),
                "{aspect:?}：实现没报读坏的槽（别的拒绝，或拒之前写了盘）"
            );
        }
        assert!(
            left_by_the_lying_device(
                &observation(rows_disagreement(Some(vec![row(2)]))),
                &explained(&[], Some(vec![row(2)]))
            ),
            "实现写的行等于被吞的根由记录重建之后重算的行"
        );
        for (written, recomputed, why) in [
            (
                Some(vec![row(2)]),
                Some(vec![row(0)]),
                "重算的行与实现写的不同",
            ),
            (Some(vec![row(2)]), None, "模型重算不出"),
            (None, Some(vec![row(2)]), "对不上的那一格没带实现写的行"),
        ] {
            assert!(
                !left_by_the_lying_device(
                    &observation(rows_disagreement(written)),
                    &explained(&[], recomputed)
                ),
                "{why}"
            );
        }
    }

    /// 按注入点认的回退那一格（`research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4）：实现拒管理员回退、
    /// 点名的「回退目标引用的一个单元在一块盘上的那一份」与一次被吞的写有重叠的字节，才认——不管模型要它成还是要别的理由；
    /// 被吞的写落在另一块盘上、紧挨着那一份而不重叠、实现没点名那一份（别的拒绝，或拒之前写了盘），一概照报。
    #[test]
    fn a_refused_rollback_is_left_by_the_lying_device_only_when_the_unit_copy_it_names_overlaps_a_swallowed_write(
    ) {
        const UNIT_SLOT: u64 = 51_000;
        let observation = |disagreement: ModelDisagreement| FailureObservation {
            position: StepPosition::Operation(18),
            operation_kind: Some(HistoryOperationKind::RollBackWhileMounted),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: Some(disagreement),
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let unit_copy_on = |device: u32| ModelUnitCopyLocation {
            device: ModelDeviceIdentity(device),
            span_in_slots: 1,
            first_slot: UNIT_SLOT,
        };
        let refusal = |aspect: ModelDisagreementAspect, reported: Option<ModelUnitCopyLocation>| {
            let mut disagreement = ModelDisagreement::new(
                aspect,
                "RollbackWhileMounted 该成".to_string(),
                "拒了：RollbackError::IsolatedUnitCopyUnreadableOrMismatched".to_string(),
            );
            disagreement.note_the_unit_copy_a_rollback_refused_on(reported, true);
            disagreement
        };
        let swallowed_unit_write_on = |device: u32, first_slot: u64| {
            SwallowedWriteByteRange::of(&SwallowedWrite {
                device: DeviceIdentity(device),
                offset: DeviceOffsetInBytes(first_slot * singlefs_format::SLOT_BYTES),
                contents: SwallowedContents::Bytes(vec![
                    0xAB;
                    usize::try_from(
                        singlefs_format::SLOT_BYTES
                    )
                    .expect("槽宽装得进 usize")
                ]),
                durability: WriteDurability::Plain,
                writes_recorded_before: 146,
            })
        };
        let explained = |swallowed: &[SwallowedWriteByteRange]| WhatTheSwallowedWritesExplain {
            invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
            ceiling_without_the_swallowed_roots: None,
            root_ring_slots_of_the_swallowed_root_slot_writes: BTreeSet::new(),
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: None,
            byte_ranges_of_the_swallowed_writes: swallowed.iter().copied().collect(),
        };
        let swallowed_on_the_unit = swallowed_unit_write_on(1, UNIT_SLOT);
        for aspect in [
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            ModelDisagreementAspect::RefusalReason,
        ] {
            assert!(
                left_by_the_lying_device(
                    &observation(refusal(aspect, Some(unit_copy_on(1)))),
                    &explained(&[swallowed_on_the_unit])
                ),
                "{aspect:?}：对不上的那一份正是被吞那次写落的地方"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, Some(unit_copy_on(0)))),
                    &explained(&[swallowed_on_the_unit])
                ),
                "{aspect:?}：被吞的写在另一块盘上"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, Some(unit_copy_on(1)))),
                    &explained(&[
                        swallowed_unit_write_on(1, UNIT_SLOT - 1),
                        swallowed_unit_write_on(1, UNIT_SLOT + 1)
                    ])
                ),
                "{aspect:?}：被吞的写紧挨着那一份的前后、一个字节都不重叠"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, None)),
                    &explained(&[swallowed_on_the_unit])
                ),
                "{aspect:?}：实现没点名那一份（别的拒绝）"
            );
        }
        let mut written_before_refusing = ModelDisagreement::new(
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            "RollbackWhileMounted 该成".to_string(),
            "拒了：RollbackError::IsolatedUnitCopyUnreadableOrMismatched".to_string(),
        );
        written_before_refusing
            .note_the_unit_copy_a_rollback_refused_on(Some(unit_copy_on(1)), false);
        assert_eq!(
            written_before_refusing.implementation_reported_unreadable_place, None,
            "拒之前写了盘：不记那一份"
        );
        assert!(!left_by_the_lying_device(
            &observation(written_before_refusing),
            &explained(&[swallowed_on_the_unit])
        ));
        let mut read_back = ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            "读回 (5,19)".to_string(),
            "读回 (5,18)".to_string(),
        );
        read_back.note_the_unit_copy_a_rollback_refused_on(Some(unit_copy_on(1)), true);
        assert_eq!(
            read_back.implementation_reported_unreadable_place, None,
            "拒绝之外的格不记那一份"
        );
        assert_eq!(
            SwallowedWriteByteRange::of(&SwallowedWrite {
                device: DeviceIdentity(0),
                offset: DeviceOffsetInBytes(8192),
                contents: SwallowedContents::Zeroes { length: 4096 },
                durability: WriteDurability::Plain,
                writes_recorded_before: 0,
            }),
            SwallowedWriteByteRange {
                device: ModelDeviceIdentity(0),
                start_in_bytes: 8192,
                end_in_bytes: 12_288,
            },
            "写零那一次照它的长度算"
        );
    }

    /// 「重开走读失败」什么时候是正确答案：说谎的那两种一律是；报错的那四种只有在 mkfs 自己就报错
    /// （盘上还没有文件系统）时才是。这道豁免放宽一格，报错的设备之后的走读失败就没人看了。
    #[test]
    fn failed_reopen_is_excused_only_by_the_lying_device_or_by_the_failed_make_filesystem() {
        let stopped_at = |step| FailureObservation {
            position: StepPosition::StartingPoint,
            operation_kind: None,
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: Some(HarnessJudgement::TheStartingPointReturnedAnError {
                step,
                error: "注入的写错".to_string(),
            }),
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let failed_make_filesystem = stopped_at(StartingPointStep::MakeFilesystem);
        let failed_first_file = stopped_at(StartingPointStep::PublishFirstFile);
        for lying in [
            InjectedFault::WriteIsSwallowed,
            InjectedFault::BarrierIsSwallowed,
        ] {
            assert!(
                failed_reopen_is_the_right_answer(lying, None),
                "{} 之后走读失败是对的",
                lying.name()
            );
        }
        assert!(
            !failed_reopen_is_the_right_answer(InjectedFault::WriteFails, None),
            "写报错之后走读失败是缺口，不许豁免"
        );
        assert!(
            failed_reopen_is_the_right_answer(
                InjectedFault::WriteFails,
                Some(&failed_make_filesystem)
            ),
            "mkfs 自己就报错时盘上没有文件系统，走读失败是对的"
        );
        assert!(
            !failed_reopen_is_the_right_answer(InjectedFault::WriteFails, Some(&failed_first_file)),
            "mkfs 做完了，第一个文件那一步报错之后重开该走到创世根上"
        );
    }

    /// 注入之后的镜像上 checker 判红的就是 `violated_invariants` 那几条、模型对不上的是 `model_disagreement` 的一条失败，停在起点段。
    fn failure_after_the_fault_at_the_starting_point(
        violated_invariants: &[&'static str],
        model_disagreement: Option<ModelDisagreement>,
    ) -> FailureObservation {
        FailureObservation {
            position: StepPosition::StartingPoint,
            operation_kind: None,
            violations: violated_invariants
                .iter()
                .map(|invariant| (*invariant, "池里 2 块盘一个都没自证过".to_string()))
                .collect(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        }
    }

    /// 注入之后的镜像上只红 I-8.5 的那条失败（mkfs 没做完那一形的签名）。
    fn failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
    ) -> FailureObservation {
        failure_after_the_fault_at_the_starting_point(
            &[SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
            None,
        )
    }

    /// 判那一形要的三样事实，逐样由调用方给。
    fn facts_after_the_fault(
        starting_point_step_that_returned_an_error: Option<StartingPointStep>,
        system_configuration_slot_writes_that_landed: usize,
        reopened: ReopenedVersion,
    ) -> FactsAfterTheFault {
        FactsAfterTheFault {
            starting_point_step_that_returned_an_error,
            system_configuration_slot_writes_that_landed,
            reopened,
        }
    }

    /// mkfs 报错、重开走读失败；落盘的系统配置槽写有几次由调用方给。
    fn make_filesystem_failed_with_system_configuration_slot_writes_landed(
        system_configuration_slot_writes_that_landed: usize,
    ) -> FactsAfterTheFault {
        facts_after_the_fault(
            Some(StartingPointStep::MakeFilesystem),
            system_configuration_slot_writes_that_landed,
            ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
        )
    }

    /// 对照（用户 2026-10-03 定的那一形只认「mkfs 在系统配置槽落盘之前报错」）：同一个签名（只红 I-8.5、重开走读失败、mkfs 报的错），
    /// 而录制流里系统配置槽写已经落了盘——mkfs 写完系统配置之后才报错，盘上本该有自证过的槽，I-8.5 红就不是「mkfs 没做完」能交代的，
    /// 照报新发现。同一条失败换成一次都没落盘时认成那一形，证明这一格不是因为判法什么都不认才绿。
    #[test]
    fn i85_only_failure_after_make_filesystem_wrote_its_system_configuration_slots_stays_a_new_finding(
    ) {
        let observation =
            failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
            );
        let slots_written_by_make_filesystem =
            2 * usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                .expect("每盘槽数装得进 usize");
        assert_eq!(
            known_red_form_only_after_a_fault(
                &observation,
                &make_filesystem_failed_with_system_configuration_slot_writes_landed(0)
            ),
            Some(
                KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded
            ),
            "mkfs 在系统配置槽落盘之前报错、只红 I-8.5、重开走读失败：这是用户定的那一形，要认"
        );
        assert_eq!(
            known_red_form_only_after_a_fault(
                &observation,
                &make_filesystem_failed_with_system_configuration_slot_writes_landed(
                    slots_written_by_make_filesystem
                )
            ),
            None,
            "系统配置槽已经落盘（{slots_written_by_make_filesystem} 次）之后的 I-8.5 红不是 mkfs 没做完那一形"
        );
        match classify_failure(observation) {
            HistoryEnding::NewFinding { signature, .. } => assert_eq!(
                signature,
                FailureSignature::CheckerViolations {
                    invariants: vec![SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
                },
                "共用清单接不走它，签名照旧是只红 I-8.5"
            ),
            ending @ (HistoryEnding::Completed | HistoryEnding::KnownRed { .. }) => {
                panic!("只红 I-8.5 的失败不在共用清单里，该是新发现：{ending:?}")
            }
        }
    }

    /// mkfs 没做完那一形不放宽：别的步报错、多红一条或换一条、重开走到别的格、带着模型对不上的，一律不认。
    #[test]
    fn make_filesystem_known_red_form_takes_no_other_step_signature_or_reopen() {
        let observation =
            failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
            );
        let facts = make_filesystem_failed_with_system_configuration_slot_writes_landed(0);
        assert!(
            known_red_form_only_after_a_fault(&observation, &facts).is_some(),
            "基准那一条要认，下面每一格只改一样"
        );
        for step in [
            StartingPointStep::ReadThePoolForTheSession,
            StartingPointStep::AcquireInstance,
            StartingPointStep::WarmUp,
            StartingPointStep::PublishFirstFile,
        ] {
            let facts_of_another_step = facts_after_the_fault(
                Some(step),
                0,
                ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
            );
            assert_eq!(
                known_red_form_only_after_a_fault(&observation, &facts_of_another_step),
                None,
                "{}报错时 mkfs 已经做完，不是这一形",
                step.name()
            );
        }
        let facts_without_a_starting_point_error = facts_after_the_fault(
            None,
            0,
            ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
        );
        assert_eq!(
            known_red_form_only_after_a_fault(&observation, &facts_without_a_starting_point_error),
            None,
            "历史没停在起点段的报错上，不是这一形"
        );
        for reopened in [
            ReopenedVersion::CommittedByTheModel,
            ReopenedVersion::TheVersionTheFaultedStepWasWriting,
            ReopenedVersion::OutsideEverythingTheModelAllows,
        ] {
            let facts_of_another_reopen =
                facts_after_the_fault(Some(StartingPointStep::MakeFilesystem), 0, reopened);
            assert_eq!(
                known_red_form_only_after_a_fault(&observation, &facts_of_another_reopen),
                None,
                "重开走到 {reopened:?}，不是这一形"
            );
        }
        let with_one_more_violation = failure_after_the_fault_at_the_starting_point(
            &[
                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
                "I-4.1",
            ],
            None,
        );
        assert_eq!(
            known_red_form_only_after_a_fault(&with_one_more_violation, &facts),
            None,
            "除了 I-8.5 还红别的，不是这一形"
        );
        let with_another_violation =
            failure_after_the_fault_at_the_starting_point(&["I-4.1"], None);
        assert_eq!(
            known_red_form_only_after_a_fault(&with_another_violation, &facts),
            None,
            "红的不是 I-8.5，不是这一形"
        );
        let with_a_model_disagreement = failure_after_the_fault_at_the_starting_point(
            &[SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
            Some(ModelDisagreement {
                aspect: ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
                model_answer: "该成".to_string(),
                implementation_answer: "拒了".to_string(),
                implementation_reported_ceiling: None,
                implementation_reported_unreadable_place: None,
                implementation_rows_written: None,
            }),
        );
        assert_eq!(
            known_red_form_only_after_a_fault(&with_a_model_disagreement, &facts),
            None,
            "带着模型对不上，不是这一形"
        );
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
