//! 故障注入（里程碑「第二个事务」增补 3 第 4 件）：一个通用的设备包装，按种子让任意一次写、读、刷盘返回 `BlockDeviceError`，
//! 或让一次读返回改坏的字节。被测的性质有两条：注入之后 `singlefs-core` 那一侧**返回错误而不是 panic**，
//! 出错之后**重开要恢复到模型允许的版本**。
//!
//! 包装的位置在录制器**外面**（调用方与录制器之间）：报错的写、被吞掉的写与被吞掉的屏障都不进录制流，
//! 录制流因此恒等于真正落到盘上的那一串——注入之后的镜像可以从录制流重建（`crate::crash::MemoryPool::apply`），
//! 不必把执行器里的两块盘再交出来一份。
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
//! 三方三轮判完、改法未定，这里不替它定，只把落在那一格的次数记成一笔（`FaultInjectionTally::reopened_into_the_version_the_failed_step_was_writing`）。
//! 第 40 行那一格的判别力由写死的那条用例钉（`tests/second_transaction_supplement_three_fault_injection.rs`）。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MKFS_INSTANCE_GENERATION;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, recover, JournalPolicy, PoolReader,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_format::{ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION_MAXIMUM};

use crate::crash::{newest_persisted_root, writes_and_segments, MemoryPool, RecordCheck};
use crate::history::{
    classify_failure, execute_history_with_faults, generate_history_with_weights,
    newest_ring_root_and_slot_count, raised_floor_lands_only_on_abandoned_roots, AppliedEffect,
    ColdStartReadBack, FailureObservation, FailureSignature, GeneratedHistory, GenerationWeights,
    HarnessJudgement, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
    HistoryRun, HistorySeed, PerStepChecker, SeededRandomSource, StepOutcome, StepPosition,
};
use crate::model::{crash_recovery_disagreement, ModelRootKey, ObservedReadBack};
use crate::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordedOperation, RecordedOperationKind, SharedStream};

/// 故障注入的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。
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
/// 罩得住走 `Vec<(DeviceIdentity, Device)>` 的路径（可写挂载、回退、抬 F）；层 0 枚举出来的崩溃镜像
/// （`crate::crash::MemoryPool` / `CrashImage`）不是块设备、只是 `PoolReader`，那一路要这个口子。
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

    #[must_use]
    pub fn target(&self) -> RootRingSlotTarget {
        self.target
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

struct FaultPlanState {
    geometry: FixedGeometry,
    schedule: Option<FaultSchedule>,
    calls: FaultCallCounts,
    matching_calls_across_the_pool: u64,
    matching_calls_per_device: BTreeMap<DeviceIdentity, u64>,
    per_device_counts: BTreeMap<DeviceIdentity, FaultDeviceCounts>,
    /// 每块盘收到第一道屏障那一刻这块盘的计数（屏障本身不算进去）：真设备那一路按它切「挂载」那一段窗口。
    counts_when_the_first_barrier_arrived: BTreeMap<DeviceIdentity, FaultDeviceCounts>,
    fired: Vec<FiredFault>,
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
            counts_when_the_first_barrier_arrived: BTreeMap::new(),
            fired: Vec::new(),
        })))
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

    /// 这块盘收到第一道屏障那一刻的计数；一道屏障都没收到过时 None。
    #[must_use]
    pub fn counts_when_the_first_barrier_arrived_at(
        &self,
        device: DeviceIdentity,
    ) -> Option<FaultDeviceCounts> {
        self.0
            .borrow()
            .counts_when_the_first_barrier_arrived
            .get(&device)
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
        let counts = state.per_device_counts.entry(device).or_default();
        counts.writes += 1;
        counts.written_bytes += bytes;
        match durability {
            WriteDurability::Plain => {}
            WriteDurability::ForceUnitAccess => counts.force_unit_access_writes += 1,
        }
    }

    /// 屏障进来了：先给这块盘拍第一道屏障的快照（屏障本身不算进去），再记下它是发下去了还是被吞了。
    fn note_barrier_arrived(&self, device: DeviceIdentity) {
        let mut state = self.0.borrow_mut();
        let counts = *state.per_device_counts.entry(device).or_default();
        state
            .counts_when_the_first_barrier_arrived
            .entry(device)
            .or_insert(counts);
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
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut Inner {
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
}

impl<Inner: BlockDevice> BlockDevice for FaultInjectingBlockDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读的长度装得进 u64");
        match self
            .plan
            .decide(FaultCallKind::Read, self.device, offset, length)
        {
            Some(InjectedFault::ReadFails) => return Err(injected_block_device_error("读")),
            Some(InjectedFault::ReadReturnsCorruptedBytes { flipped_bit }) => {
                self.inner.read_at(offset, buffer)?;
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
        self.inner.read_at(offset, buffer)?;
        self.plan.note_forwarded_read(self.device);
        Ok(())
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
            // 吞掉：报成功，一个字节都不落盘，也不记进这块盘的计数（真的没写）。
            Some(InjectedFault::WriteIsSwallowed) => return Ok(()),
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
    pub fn all() -> Vec<FaultedSegmentKind> {
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
    pub fn all() -> Vec<InjectionPoint> {
        let mut points = Vec::new();
        for segment in FaultedSegmentKind::all() {
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
    /// 重开走读失败，而这一次注入之后盘上本来就没有一版立得住（`a_failed_reopen_is_the_right_answer`）：
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

/// 「已知红」收尾的一次注入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownRedAtAFault {
    pub seed: HistorySeed,
    pub form: usize,
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
    /// 说谎的设备（[`InjectedFault::the_device_lies`]）丢掉一份内容之后盘面不一致、而且落在白名单
    /// （[`INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE`]）里的次数：不算新发现，按签名列进报告。
    /// 白名单之外的不进这一格，照报成新发现。一次都没有说明说谎那两种注得进去却什么也没碰着，判别力要重新量。
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
    /// 重开走读失败，而盘上本来就没有一版立得住（`a_failed_reopen_is_the_right_answer`）：只记不判。
    pub reopens_that_correctly_found_nothing_to_recover: u64,
    pub checker_runs_on_the_image_after_the_fault: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    pub faults_ending_known_red: BTreeMap<usize, u64>,
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
        self.checker_runs_on_the_image_after_the_fault +=
            other.checker_runs_on_the_image_after_the_fault;
        for (key, count) in &other.invariant_holds {
            *self.invariant_holds.entry(key).or_insert(0) += count;
        }
        for (key, count) in &other.invariant_not_applicable {
            *self.invariant_not_applicable.entry(key).or_insert(0) += count;
        }
        for (form, count) in &other.faults_ending_known_red {
            *self.faults_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.faults_ending_new_finding += other.faults_ending_new_finding;
    }

    /// 一个注入点都没命中的那几格（验收要的「没命中的逐个列名」）。
    #[must_use]
    pub fn injection_points_never_hit(&self) -> Vec<String> {
        InjectionPoint::all()
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
        for segment in FaultedSegmentKind::all() {
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
            "注入之后的镜像上跑池级 checker {} 次：判绿 {} 条次、不适用 {} 条次",
            self.checker_runs_on_the_image_after_the_fault,
            self.invariant_holds.values().sum::<u64>(),
            self.invariant_not_applicable.values().sum::<u64>()
        );
        let _ = writeln!(
            text,
            "以「已知红」收尾 {:?}、新发现 {}",
            self.faults_ending_known_red, self.faults_ending_new_finding
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

/// 注入那一步是不是一次挂载（起点段里有取号与暖机、可写挂载、回退挂载），是就交回这一步之前盘上系统配置的实例代号。
/// 起点段之前是 mkfs 写的 0；别的步取测量跑里上一步跑完时的那个。不是挂载的几类操作不取号，交回 `None`。
fn system_configuration_instance_before_a_mount_step(
    marks: &[StepMark],
    segment: FaultedSegment,
) -> Option<InstanceGeneration> {
    match segment {
        FaultedSegment::TheStartingPoint => Some(MKFS_INSTANCE_GENERATION),
        FaultedSegment::Operation { operation_kind, .. } => match operation_kind {
            HistoryOperationKind::CloseAndMountWritable
            | HistoryOperationKind::CloseAndMountRollback => {
                let index = marks
                    .iter()
                    .position(|mark| mark.position == segment.position())?;
                marks[index.checked_sub(1)?].system_configuration_instance
            }
            HistoryOperationKind::PublishFirstFile
            | HistoryOperationKind::PublishOverwrite
            | HistoryOperationKind::PublishWithoutUnits
            | HistoryOperationKind::RaiseRollbackFloor
            | HistoryOperationKind::ColdStartRecover => None,
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

/// 对一份镜像跑池级 checker，记每条不变量判绿、不适用各几次，交回判红的那几条（与 `history.rs` 的 `violations_on` 同一条口径，
/// 计数进的是故障注入自己的那一份）。
fn checker_violations_on(
    image: &MemoryPool,
    tally: &mut FaultInjectionTally,
) -> Vec<(&'static str, String)> {
    tally.checker_runs_on_the_image_after_the_fault += 1;
    let mut violations = Vec::new();
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Holds => *tally.invariant_holds.entry(invariant).or_insert(0) += 1,
            InvariantVerdict::NotApplicable(_) => {
                *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
            }
            InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
        }
    }
    violations
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
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };
    let marks = measure_history(history, execution, &mut tally);
    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
    inject_the_drawn_faults(history, execution, &marks, drawn_faults, tally)
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
    inject_the_drawn_faults(history, execution, &marks, vec![drawn], tally)
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

/// 摆好的每个注入点各重跑一遍这段历史（[`inject_one_fault`]），攒成一段的结果。
fn inject_the_drawn_faults(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn_faults: Vec<DrawnFault>,
    mut tally: FaultInjectionTally,
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
        for drawn in &drawn_faults {
            inject_one_fault(history, execution, marks, drawn, &mut accumulator);
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

/// 注入那一段的结局是不是「入口返回了错误」：没 panic、checker 没判红，而且那一段要么返回了 `Err`，
/// 要么是冷启动、读回报了错。注入的块设备错在模型里没有对应的拒绝理由（`ObservedRefusalReason::Unexplained`），
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
            | AppliedEffect::RaisedFloor { .. },
        ))
        | Some(StepOutcome::NotApplicable(_))
        | None => false,
    }
}

/// 说谎的设备丢掉一份内容之后，盘面**只许**留下这两组违例。每一组都是「那一份内容没落盘」的直接投影：
///
/// - `["I-2.1", "I-4.8", "I-7.4"]`：那一份没落盘的单元读出来是旧内容（或是全零），单元头里的校验和跟树上记的对不上
///   ⇒ 单元自证不过（I-2.1）、树指着的那一份取不回来（I-4.8）、最新那条根走不完（I-7.4）。丢一份内容必然长这样。
/// - `["I-3.1"]`：丢掉的是记账树那一份，盘上记的「已分配」与遍历全部有效根得到的对不上。丢一份记账内容必然长这样。
///
/// **白名单之外的签名照报成新发现**（用户 2026-09-21 定的收严）：别的签名（I-3.9、I-5.4、I-9.14 这一类）意味着
/// 丢一份内容之外还发生了别的事，那一格不默认豁免——那才可能是实现的缺口。
/// 白名单少一组，被它罩住的那几次当场变成新发现。快档那 24 段里说谎的设备留下的不一致全落在前一组，`["I-3.1"]` 那一组一次都没有；
/// 那一组的取样点是故障注入那个二进制里的 `a_swallowed_write_after_which_the_checker_flags_only_i_3_1_is_excused_as_what_a_lying_device_may_leave`
/// （`crates/mutations.tsv` 里钉着把那一组去掉的变异）。
const INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE: &[&[&str]] =
    &[&["I-2.1", "I-4.8", "I-7.4"], &["I-3.1"]];

/// 这条签名在不在 [`INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE`] 里。
/// checker 的违例之外一概不在：panic、执行器判出的、模型对不上、记录核对器判红都不是「丢一份内容」的投影。
fn a_lying_device_may_leave(signature: &FailureSignature) -> bool {
    match signature {
        FailureSignature::CheckerViolations { invariants } => {
            INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE.contains(&invariants.as_slice())
        }
        FailureSignature::Panic { .. }
        | FailureSignature::HarnessJudgement { .. }
        | FailureSignature::ModelDisagreement { .. }
        | FailureSignature::RecordCheck { .. } => false,
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
fn a_failed_reopen_is_the_right_answer(
    fault: InjectedFault,
    ending_observation: Option<&FailureObservation>,
) -> bool {
    if fault.the_device_lies() {
        return true;
    }
    matches!(
        ending_observation.and_then(|observation| observation.harness_judgement.as_ref()),
        Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. })
            if !step.a_finished_filesystem_is_on_the_devices()
    )
}

/// 注入一次：armed 的计划重跑这段历史，判 panic、判那一段的结局、再重开判恢复到的那一版。
fn inject_one_fault(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn: &DrawnFault,
    accumulator: &mut FaultInjectionAccumulator<'_>,
) {
    let geometry = execution.device_width.fixed_geometry();
    let plan = SharedFaultPlan::armed(
        geometry,
        FaultSchedule::the_nth_call_across_the_pool(drawn.fault, drawn.call_ordinal),
    );
    let stream = SharedStream::retaining_contents();
    let mut committed_by_the_armed_run: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let run = execute_history_with_faults(history, execution, &stream, &plan, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed_by_the_armed_run.insert(key, file);
        }
    });
    let fired = plan.fired();
    accumulator.tally.faults += 1;
    *accumulator
        .tally
        .faults_by_kind
        .entry(drawn.fault.name())
        .or_insert(0) += 1;
    if let Some(first) = fired.first() {
        let point = InjectionPoint {
            segment: drawn.segment.kind(),
            call: drawn.fault.call_kind(),
            written_structure: first.written_structure,
        };
        *accumulator
            .tally
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
            *accumulator
                .tally
                .refusal_members
                .entry(member.clone())
                .or_insert(0) += 1;
        }
        if let Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) =
            ending_observation.and_then(|observation| observation.harness_judgement.as_ref())
        {
            *accumulator
                .tally
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
    *accumulator
        .tally
        .faults_by_outcome
        .entry(outcome.name())
        .or_insert(0) += 1;

    // 二、重开：拿录制流（只记真正落到盘上的那些写）重建镜像，跑恢复，问模型走到的这一版允不允许。
    let operations = stream.retained_operations();
    let mut image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    image.apply(&operations);
    let (writes, _segments) = writes_and_segments(&operations, &geometry);
    let all_persisted = vec![true; writes.len()];
    let newest_persisted = newest_persisted_root(&writes, &all_persisted)
        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    let recovery = recover(&image, JournalPolicy::Consult);
    accumulator.tally.reopens += 1;
    match &recovery.outcome {
        RecoveryOutcome::FileRead { .. } => accumulator.tally.reopens_reading_a_file += 1,
        RecoveryOutcome::NoFile { .. } => accumulator.tally.reopens_without_a_file += 1,
        RecoveryOutcome::Failed { .. } => accumulator.tally.reopens_that_failed += 1,
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
    let nothing_was_left_to_recover = matches!(read_back, ObservedReadBack::Failed { .. })
        && a_failed_reopen_is_the_right_answer(drawn.fault, ending_observation);
    let disagreement = if nothing_was_left_to_recover {
        None
    } else {
        crash_recovery_disagreement(&allowed, newest_persisted, &read_back)
    };
    let reopened = if nothing_was_left_to_recover {
        accumulator
            .tally
            .reopens_that_correctly_found_nothing_to_recover += 1;
        ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo
    } else if crash_recovery_disagreement(&committed_by_the_model, newest_persisted, &read_back)
        .is_none()
    {
        accumulator
            .tally
            .reopened_into_a_version_the_model_committed += 1;
        ReopenedVersion::CommittedByTheModel
    } else if disagreement.is_none() {
        accumulator
            .tally
            .reopened_into_the_version_the_faulted_step_was_writing += 1;
        ReopenedVersion::TheVersionTheFaultedStepWasWriting
    } else {
        accumulator
            .tally
            .reopens_outside_everything_the_model_allows += 1;
        ReopenedVersion::OutsideEverythingTheModelAllows
    };
    let violations_after_the_fault = checker_violations_on(&image, accumulator.tally);
    let reopen_text = format!("{reopened:?}（{}）", describe_read_back(&read_back));
    // C378：注入落在一次挂载里、那一步返回了错误，而重开的盘上系统配置的实例代号比这一步之前大——取号之后的写行或暖机报了错，
    // 新号没回卷。认下的已知行为：记进这一格、在报告里点名，不进新发现，也不挡下面的判定。
    if surfaced {
        if let Some(left) = acquired_instance_left_after_a_failed_mount(&image, marks, *drawn) {
            *accumulator
                .tally
                .acquired_instances_left_after_a_failed_mount
                .entry(left.kind.name())
                .or_insert(0) += 1;
            accumulator.acquired_instances_left.push(left);
        }
    }

    // 三、判：注入那一步返回错误的那一格不算失败；别的失败（panic、checker 判红、重开走到模型不允许的版本）逐条分类。
    let mut failures: Vec<FailureObservation> = Vec::new();
    if !surfaced {
        if let Some(observation) = ending_observation {
            failures.push(observation.clone());
        }
    }
    if !violations_after_the_fault.is_empty() || disagreement.is_some() {
        let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
        failures.push(FailureObservation {
            position: drawn.segment.position(),
            operation_kind: drawn.segment.operation_kind(),
            violations: violations_after_the_fault,
            panic: None,
            newest_ring_root_txg,
            root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: disagreement,
            raised_floor_lands_only_on_abandoned_roots:
                raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(&image),
            record_check: RecordCheck::default(),
        });
    }
    for observation in failures {
        if observation.panic.is_some() {
            accumulator.tally.faults_that_panicked += 1;
        }
        // 「已知红」先认：清单里那两条是实现与 checker 读法对不上的老账（增补 2 收口表第 ②、43 行），
        // 与设备说不说谎无关，说谎的设备把它撞出来时照样记进「已知红」那一格，不许被下面那一条接走。
        match classify_failure(observation) {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, .. } => {
                *accumulator
                    .tally
                    .faults_ending_known_red
                    .entry(form)
                    .or_insert(0) += 1;
                accumulator.known_red_hits.push(KnownRedAtAFault {
                    seed: history.seed,
                    form,
                    drawn: *drawn,
                });
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                // 说谎的设备丢掉一份内容之后盘面不一致：设备丢的，不是实现的缺口（`InjectedFault::the_device_lies`）。
                // 豁免只给白名单里那两种投影（`a_lying_device_may_leave`），白名单之外的照报成新发现。
                // panic 不走这一格——说谎的设备也不许把实现打 panic，那一条照样报成新发现。
                if observation.panic.is_none()
                    && drawn.fault.the_device_lies()
                    && a_lying_device_may_leave(&signature)
                {
                    accumulator
                        .tally
                        .faults_where_a_lying_device_left_the_image_inconsistent += 1;
                    *accumulator
                        .tally
                        .lying_device_signatures
                        .entry(format!("{signature:?}"))
                        .or_insert(0) += 1;
                    continue;
                }
                accumulator.tally.faults_ending_new_finding += 1;
                if !accumulator
                    .new_findings
                    .iter()
                    .any(|finding| finding.signature == signature)
                {
                    accumulator.new_findings.push(FaultFinding {
                        signature,
                        seed: history.seed,
                        drawn: *drawn,
                        observation,
                        reopen: reopen_text.clone(),
                    });
                }
            }
        }
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
    /// 没设环境变量，取 `available_parallelism`。
    FromAvailableParallelism(usize),
    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
    AvailableParallelismUnknown,
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl FaultInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取 `available_parallelism`。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_value(
            std::env::var(FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE),
            std::thread::available_parallelism,
        )
    }

    /// [`Self::from_the_environment`] 的判定本身：环境变量读到什么、`available_parallelism` 报什么都由调用方给
    /// （用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数。
    #[must_use]
    fn from_the_environment_value(
        variable: Result<String, std::env::VarError>,
        available_parallelism: impl Fn() -> std::io::Result<std::num::NonZeroUsize>,
    ) -> Self {
        match variable {
            Ok(text) => {
                let count: usize = text.trim().parse().unwrap_or_else(|error| {
                    panic!(
                        "{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE}={text} 不是十进制正整数：{error}"
                    )
                });
                assert!(
                    count > 0,
                    "{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE}={text}：至少要 1 个线程"
                );
                Self::FromTheEnvironmentVariable(count)
            }
            Err(std::env::VarError::NotPresent) => match available_parallelism() {
                Ok(count) => Self::FromAvailableParallelism(count.get()),
                Err(_) => Self::AvailableParallelismUnknown,
            },
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")
            }
        }
    }

    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::FromTheEnvironmentVariable(count)
            | Self::FromAvailableParallelism(count)
            | Self::GivenByCaller(count) => count,
            Self::AvailableParallelismUnknown => 1,
        }
    }

    #[must_use]
    pub fn source_name(self) -> &'static str {
        match self {
            Self::FromTheEnvironmentVariable(_) => "environment_variable",
            Self::FromAvailableParallelism(_) => "available_parallelism",
            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
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
        let mut known_red_by_form: BTreeMap<usize, Vec<&KnownRedAtAFault>> = BTreeMap::new();
        for hit in &self.known_red_hits {
            known_red_by_form.entry(hit.form).or_default().push(hit);
        }
        for (form, hits) in &known_red_by_form {
            let first = hits.first().expect("这一条至少有一次命中");
            let _ = writeln!(
                text,
                "「已知红」第 {form} 条：{} 次，第一次在种子 {}（{}）",
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
        first_seed + seed_count,
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
                        inject_faults_into_history(&history, execution, faults_per_history)
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
        first_seed + seed_count,
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
    (0..slice_count)
        .map(|slice_index| {
            let start = seed_count * slice_index / slice_count;
            let end = seed_count * (slice_index + 1) / slice_count;
            start..end
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash::SparseBlockDevice;
    use crate::history::StartingPointStep;

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
    fn a_corrupted_read_flips_one_bit_in_the_buffer_and_leaves_the_device_alone() {
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
    fn a_swallowed_write_reports_success_and_changes_nothing_on_the_device() {
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
    fn a_swallowed_barrier_reports_success_and_is_counted_apart() {
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

    /// 每块盘收到第一道屏障那一刻的计数：屏障本身不算进去，之后的调用也不算。
    #[test]
    fn the_counts_when_the_first_barrier_arrived_are_frozen_at_that_moment() {
        let plan = SharedFaultPlan::unarmed(TEST_GEOMETRY);
        let mut devices = two_devices(&plan);
        assert_eq!(
            plan.counts_when_the_first_barrier_arrived_at(DeviceIdentity(0)),
            None,
            "一道屏障都没收到过"
        );
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0, 1).expect("写");
        devices[0].1.barrier().expect("第一道屏障");
        write(&mut devices[0].1, UNIT_AREA_OFFSET.0 + 512, 2).expect("写");
        devices[0].1.barrier().expect("第二道屏障");
        let frozen = plan
            .counts_when_the_first_barrier_arrived_at(DeviceIdentity(0))
            .expect("收到过屏障");
        assert_eq!(frozen.writes, 1);
        assert_eq!(frozen.barriers_forwarded, 0, "屏障本身不算进去");
        assert_eq!(plan.counts_of_device(DeviceIdentity(0)).writes, 2);
    }

    /// 屏障报错：交回块设备错，里面那块盘没收到。
    #[test]
    fn a_failing_barrier_returns_a_block_device_error() {
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
    fn a_failing_read_returns_a_block_device_error_and_leaves_the_buffer_alone() {
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

    /// 线程数从环境变量取，没设取 `available_parallelism`；设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn worker_thread_count_comes_from_the_environment_variable_or_available_parallelism() {
        let four = || Ok(std::num::NonZeroUsize::new(4).expect("4"));
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(Ok("3".to_string()), four),
            FaultInjectionWorkerThreads::FromTheEnvironmentVariable(3)
        );
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                four
            ),
            FaultInjectionWorkerThreads::FromAvailableParallelism(4)
        );
        assert_eq!(
            FaultInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                || Err(std::io::Error::other("报不出来"))
            ),
            FaultInjectionWorkerThreads::AvailableParallelismUnknown
        );
        assert_eq!(
            FaultInjectionWorkerThreads::AvailableParallelismUnknown.count(),
            1
        );
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

    /// 说谎的设备许可留下的盘面不一致是一张白名单：名单里那两组豁免，名单外的（多一条不变量、少一条不变量、
    /// 换一条不变量、以及 checker 违例之外的那几类签名）一概照报成新发现
    /// （用户 2026-09-21 定的收严；`a_lying_device_may_leave`）。
    #[test]
    fn only_the_two_whitelisted_projections_of_a_lost_write_are_excused() {
        let checker_violations =
            |invariants: &[&'static str]| FailureSignature::CheckerViolations {
                invariants: invariants.to_vec(),
            };
        for allowed in [
            checker_violations(&["I-2.1", "I-4.8", "I-7.4"]),
            checker_violations(&["I-3.1"]),
        ] {
            assert!(
                a_lying_device_may_leave(&allowed),
                "白名单里这一组该豁免：{allowed:?}"
            );
        }
        for refused in [
            // 多一条：丢一份内容之外还发生了别的事。
            checker_violations(&["I-2.1", "I-4.8", "I-7.4", "I-5.4"]),
            checker_violations(&["I-3.1", "I-3.9"]),
            // 少一条、换一条：不是那一组投影。
            checker_violations(&["I-2.1", "I-4.8"]),
            checker_violations(&["I-9.14"]),
            // 次序不同不算同一组（签名按 checker 报出来的次序排，翻了次序就是另一回事）。
            checker_violations(&["I-7.4", "I-4.8", "I-2.1"]),
            // checker 违例之外的四类签名一概不豁免。
            FailureSignature::Panic {
                location: "crates/singlefs-core/src/transaction.rs:1".to_string(),
            },
            FailureSignature::HarnessJudgement {
                judgement: "checker 判绿的镜像上冷启动恢复报错",
            },
            FailureSignature::ModelDisagreement {
                aspect: "冷启动读回",
            },
            FailureSignature::RecordCheck {
                aspects: vec!["root_without_record"],
            },
        ] {
            assert!(
                !a_lying_device_may_leave(&refused),
                "白名单外这一组不许豁免：{refused:?}"
            );
        }
    }

    /// 「重开走读失败」什么时候是正确答案：说谎的那两种一律是；报错的那四种只有在 mkfs 自己就报错
    /// （盘上还没有文件系统）时才是。这道豁免放宽一格，报错的设备之后的走读失败就没人看了。
    #[test]
    fn a_failed_reopen_is_excused_only_by_a_lying_device_or_by_a_failed_make_filesystem() {
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
                a_failed_reopen_is_the_right_answer(lying, None),
                "{} 之后走读失败是对的",
                lying.name()
            );
        }
        assert!(
            !a_failed_reopen_is_the_right_answer(InjectedFault::WriteFails, None),
            "写报错之后走读失败是缺口，不许豁免"
        );
        assert!(
            a_failed_reopen_is_the_right_answer(
                InjectedFault::WriteFails,
                Some(&failed_make_filesystem)
            ),
            "mkfs 自己就报错时盘上没有文件系统，走读失败是对的"
        );
        assert!(
            !a_failed_reopen_is_the_right_answer(
                InjectedFault::WriteFails,
                Some(&failed_first_file)
            ),
            "mkfs 做完了，第一个文件那一步报错之后重开该走到创世根上"
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
