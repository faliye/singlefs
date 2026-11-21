//! 运行库后台整理一批拆三段（D26（后台整理与放置回收） 已定项 13 第 4、5 条）的验收：锁外那一段等 GPU 时前台入口取得到会话锁，
//! 维护线程自己的 `publish_when_due` 与重发冻结照跑，卸载与丢弃不等那一批（活意图留在盘上、下次挂载续做）；
//! 回锁时有冻结着的发布、盘表不是挂载那一份，这一批都不发、这一拍停；卸载排空被拒交回的运行库照常整理；替身在锁外那一段交回调用方在停的那个错、
//! 回锁之后整理停下报警；回锁剔掉的单元数累加进观测；剔完一个不剩、选中的段没了资格之后这一拍接着叫。
//! GPU 是用例的替身：照核心层的 CPU 写法算（甲 `checksum::crc32_castagnoli`，乙 `region_scan::scan_region_over_the_identity_index` 对单条记录，
//! 丙 `compaction_plan::live_slots_per_cluster_segment_by_the_records`），在点名的那种调用里挡住，用例放行或调用方在停才交回；调用方在停时每一段调用一进来就放弃。
//! 替身多半只挡一次。挡在数活槽（丙）：那一批没有活意图，选段打分在读盘之前；挡在一批里没数活槽就分类记录（乙）：那一批有活意图。
//! 强制选段的几条每一批分类记录都挡，用例一次放行一次：每一条意图签发那一批挡一次，之后每一批有活意图的各一次。
//! 起线程的几条用测试摆的钟：维护线程挂载之后那一拍推整理，之后钟不走就不再到期，锁外期间的 `publish_when_due` 由用例拨钟触发。
mod common_pool_on_memory_devices;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use common_pool_on_memory_devices::{
    parameters, pool_with_holes_in_the_data_segments, PoolOnMemoryDevices, RecordedMemoryDevice,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{DeviceIdentity, FileOffsetInBytes, InodeNumber, SlotNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeBackend, BackgroundComputeError,
    BackgroundComputeSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed,
    BackgroundGpuComputeHandle, BackgroundTask, BackgroundTaskAfterTheComputeError,
    CallerIsStoppingTheBackgroundGpu, ClusterSegmentEntryKey, GpuDeviceIdentifier,
    GpuEntryDecision, GpuInventory, GpuMemoryInBytes, GpuSubmissionsOfTheBatch,
    RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    live_slots_per_cluster_segment_by_the_records, CallerMonotonicClockReading,
    CompactionBatchSize, CompactionBatchUnits, CompactionMountParameters, CompactionRunStop,
    CompactionSwitches, CompactionWatermarks, EmptyClusterSegmentsWatermark,
    FragmentationWatermark, UnitAreaOfOneDevice, UnreclaimableBytesWatermark,
    COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, BackgroundCompactionNotStarted, CompactionRunOutcome, UserChange,
};
use singlefs_core::region_scan::{
    scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
    CentralMappingIdentityIndex,
};
use singlefs_core::transaction::{FirstFile, PartialWriteBytes, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::CLUSTER_SEGMENT_SLOTS;
use singlefs_mount_runtime::{
    BackgroundGpuNotOpened, ClockOfTheRuntime, IgnoringTheMaintenanceEvents, MaintenanceEvent,
    MaintenancePass, MountedRuntime, OpenedBackgroundGpu, OpensTheBackgroundGpu,
    ReceiverOfTheMaintenanceEvents, RuntimeMounted, RuntimeParameters, UnmountRefused,
};

/// 默认的整理周期：钟摆到这一刻挂载，维护线程挂载之后那一拍就到期。
const ONE_PERIOD: Duration = Duration::from_secs(30);
/// 起线程的用例里 `publish_when_due` 的周期（真钟）：锁外那一段等着时维护线程隔这么久回锁里重发冻结、`publish_when_due` 一次。
const PUBLISH_POLL_PERIOD_OF_THE_THREAD: Duration = Duration::from_millis(5);
/// 等线程做成一件事的上限：只防挂住，健康时远用不了，这个长度只决定红得多快。
const THREAD_DEADLINE: Duration = Duration::from_secs(300);
/// 替身挡着时前台入口、锁外期间的发布、卸载与丢弃该在多久之内做成：健康时是毫秒级，不等替身放行；红了就是在等替身。
const DEADLINE_WHILE_THE_GPU_IS_HELD_BACK: Duration = Duration::from_secs(60);
/// 替身挡着时隔多久读一次调用方在停的标记。
const POLL_OF_THE_CALLER_STOPPING: Duration = Duration::from_millis(1);
/// 替身至多挡多久：用例忘了放行时让替身那一侧红，不挂住。
const CALL_HELD_BACK_AT_MOST: Duration = Duration::from_secs(600);
/// 让后面每一个写都报错（远多于这一拍会写的次数）。
const WRITES_FAILING_UNTIL_THE_TEST_CLEARS_THEM: u64 = 1 << 40;
/// 替身那张卡的显存总量，也是给它的上限。
const MEMORY_OF_THE_CARD_OF_THE_STAND_IN: GpuMemoryInBytes = GpuMemoryInBytes(8 << 30);
/// 等替身挡住下一次的时候隔多久看一眼那一拍结束了没有：那一拍不再挡就结束了，不等满 [`THREAD_DEADLINE`]。
const POLL_OF_THE_PASS_THREAD: Duration = Duration::from_millis(10);

/// 测试摆的钟。
#[derive(Debug)]
struct ClockMovedByTheTest {
    reading: Mutex<Duration>,
}

impl ClockMovedByTheTest {
    fn starting_at(reading: Duration) -> Self {
        ClockMovedByTheTest {
            reading: Mutex::new(reading),
        }
    }

    fn advance_by(&self, elapsed: Duration) {
        *self.reading.lock().expect("钟的锁里不 panic") += elapsed;
    }
}

impl ClockOfTheRuntime for ClockMovedByTheTest {
    fn reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(*self.reading.lock().expect("钟的锁里不 panic"))
    }
}

/// 替身挡住的是哪一次调用。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallHeldBack {
    /// 数活槽（丙）：那一批没有活意图（有活意图的一批不选段打分），选段打分在读盘之前。
    CountOfTheLiveSlots,
    /// 一批里没数活槽就分类记录（乙）：那一批有活意图。
    ClassificationInABatchWithALiveIntent,
    /// 强制选段时每一批分类记录（乙）：强制选段不数活槽，签发那一批与之后有活意图的那几批都分类一次 R 里的记录（R 里没有未释放的记录时不分类）。
    ClassificationOfTheForcedRegion,
}

/// 替身挡几次。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HowOftenTheCallIsHeldBack {
    /// 只挡头一次。
    Once,
    /// 点名的那种调用每一次都挡，用例一次放行一次（[`GateOfTheCallHeldBack::release_one_call`]）；门开了之后不再挡。
    EveryTimeUntilTheGateOpens,
}

/// 挡住的那一次怎样交回。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeldBackCallEnded {
    ReleasedByTheTest,
    GaveUpForTheCallerStopping,
}

/// 用例与替身共用的那一扇门：替身挡住时记下，等用例放行或调用方在停。替身在 GPU 实现那把锁里等，用例不经那把锁读写这一扇门。
#[derive(Debug, Default)]
struct GateOfTheCallHeldBack {
    state: Mutex<StateOfTheGate>,
    changed: Condvar,
}

#[derive(Debug, Default)]
struct StateOfTheGate {
    is_open: bool,
    is_a_call_held_back: bool,
    has_given_up_for_the_caller_stopping: bool,
    /// 每一次都挡的那一种（[`HowOftenTheCallIsHeldBack::EveryTimeUntilTheGateOpens`]）挡过几次。
    calls_held_back_one_by_one: u64,
    /// 用例一次一次放行了几次。
    calls_released_one_by_one: u64,
}

impl GateOfTheCallHeldBack {
    fn open(&self) {
        self.state.lock().expect("门的锁里不 panic").is_open = true;
        self.changed.notify_all();
    }

    fn is_open(&self) -> bool {
        self.state.lock().expect("门的锁里不 panic").is_open
    }

    fn has_given_up_for_the_caller_stopping(&self) -> bool {
        self.state
            .lock()
            .expect("门的锁里不 panic")
            .has_given_up_for_the_caller_stopping
    }

    /// 等到替身挡住一次调用，交回等到没有。
    fn wait_until_a_call_is_held_back(&self, deadline: Duration) -> bool {
        let given_up_at = Instant::now() + deadline;
        let mut state = self.state.lock().expect("门的锁里不 panic");
        while !state.is_a_call_held_back {
            let now = Instant::now();
            if now >= given_up_at {
                return false;
            }
            state = self
                .changed
                .wait_timeout(state, given_up_at - now)
                .expect("门的锁里不 panic")
                .0;
        }
        true
    }

    /// 替身挡住这一次：记下挡住了，等用例放行或调用方在停（读把手交来的标记）。
    fn hold_back(
        &self,
        caller_is_stopping: Option<&CallerIsStoppingTheBackgroundGpu>,
    ) -> HeldBackCallEnded {
        let given_up_at = Instant::now() + CALL_HELD_BACK_AT_MOST;
        let mut state = self.state.lock().expect("门的锁里不 panic");
        state.is_a_call_held_back = true;
        self.changed.notify_all();
        loop {
            if state.is_open {
                return HeldBackCallEnded::ReleasedByTheTest;
            }
            if caller_is_stopping
                .is_some_and(CallerIsStoppingTheBackgroundGpu::is_the_caller_stopping)
            {
                state.has_given_up_for_the_caller_stopping = true;
                self.changed.notify_all();
                return HeldBackCallEnded::GaveUpForTheCallerStopping;
            }
            assert!(
                Instant::now() < given_up_at,
                "用例没放行替身挡住的那一次调用"
            );
            state = self
                .changed
                .wait_timeout(state, POLL_OF_THE_CALLER_STOPPING)
                .expect("门的锁里不 panic")
                .0;
        }
    }

    /// 每一次都挡的那一种：记下又挡了一次，等用例放行这一次（放行的次数到了它的序号）、门开了，或调用方在停。
    fn hold_back_until_released_one_by_one(
        &self,
        caller_is_stopping: Option<&CallerIsStoppingTheBackgroundGpu>,
    ) -> HeldBackCallEnded {
        let given_up_at = Instant::now() + CALL_HELD_BACK_AT_MOST;
        let mut state = self.state.lock().expect("门的锁里不 panic");
        state.calls_held_back_one_by_one += 1;
        let number_of_this_call = state.calls_held_back_one_by_one;
        self.changed.notify_all();
        loop {
            if state.is_open || state.calls_released_one_by_one >= number_of_this_call {
                return HeldBackCallEnded::ReleasedByTheTest;
            }
            if caller_is_stopping
                .is_some_and(CallerIsStoppingTheBackgroundGpu::is_the_caller_stopping)
            {
                state.has_given_up_for_the_caller_stopping = true;
                self.changed.notify_all();
                return HeldBackCallEnded::GaveUpForTheCallerStopping;
            }
            assert!(
                Instant::now() < given_up_at,
                "用例没放行替身挡住的第 {number_of_this_call} 次调用"
            );
            state = self
                .changed
                .wait_timeout(state, POLL_OF_THE_CALLER_STOPPING)
                .expect("门的锁里不 panic")
                .0;
        }
    }

    /// 等到每一次都挡的那一种挡过 `calls` 次（第 `calls` 次正挡着或已放行），交回等到没有。
    fn wait_until_the_calls_held_back_one_by_one_reach(
        &self,
        calls: u64,
        deadline: Duration,
    ) -> bool {
        let given_up_at = Instant::now() + deadline;
        let mut state = self.state.lock().expect("门的锁里不 panic");
        while state.calls_held_back_one_by_one < calls {
            let now = Instant::now();
            if now >= given_up_at {
                return false;
            }
            state = self
                .changed
                .wait_timeout(state, given_up_at - now)
                .expect("门的锁里不 panic")
                .0;
        }
        true
    }

    /// 放行每一次都挡的那一种里挡着的下一次。
    fn release_one_call(&self) {
        self.state
            .lock()
            .expect("门的锁里不 panic")
            .calls_released_one_by_one += 1;
        self.changed.notify_all();
    }
}

/// 用例的 GPU 替身：三段照 CPU 写法算、都算确认，点名的那种调用挡住（挡几次照 `how_often`）。调用方在停时每一段调用一进来就放弃
/// （产品 GPU 包等卡的每一处都读停标记），挡着的那一次也放弃。
struct GpuStandInHoldingBackOneCall {
    call_held_back: CallHeldBack,
    how_often: HowOftenTheCallIsHeldBack,
    gate: Arc<GateOfTheCallHeldBack>,
    caller_is_stopping: Option<CallerIsStoppingTheBackgroundGpu>,
    has_held_back_a_call: bool,
    /// 这一批数过活槽没有：一批算完核心层取一次这一批的提交记录，在那里清掉。
    has_this_batch_counted_the_live_slots: bool,
}

fn card_of_the_stand_in() -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(b"card-of-the-stand-in".to_vec())
}

impl GpuStandInHoldingBackOneCall {
    /// 调用方在停（把手交来的标记置着）就当场放弃这一段调用。
    fn give_up_once_the_caller_is_stopping(&self) -> Result<(), BackgroundGpuComputeFailed> {
        if self
            .caller_is_stopping
            .as_ref()
            .is_some_and(CallerIsStoppingTheBackgroundGpu::is_the_caller_stopping)
        {
            return Err(BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping);
        }
        Ok(())
    }

    fn hold_back_if_this_is_the_call(
        &mut self,
        call: CallHeldBack,
    ) -> Result<(), BackgroundGpuComputeFailed> {
        if call != self.call_held_back {
            return Ok(());
        }
        let ended = match self.how_often {
            HowOftenTheCallIsHeldBack::Once => {
                if self.has_held_back_a_call {
                    return Ok(());
                }
                self.has_held_back_a_call = true;
                self.gate.hold_back(self.caller_is_stopping.as_ref())
            }
            HowOftenTheCallIsHeldBack::EveryTimeUntilTheGateOpens => self
                .gate
                .hold_back_until_released_one_by_one(self.caller_is_stopping.as_ref()),
        };
        match ended {
            HeldBackCallEnded::ReleasedByTheTest => Ok(()),
            HeldBackCallEnded::GaveUpForTheCallerStopping => {
                Err(BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping)
            }
        }
    }
}

impl GpuInventory for GpuStandInHoldingBackOneCall {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        vec![(card_of_the_stand_in(), MEMORY_OF_THE_CARD_OF_THE_STAND_IN)]
    }
}

impl BackgroundGpuCompute for GpuStandInHoldingBackOneCall {
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        self.give_up_once_the_caller_is_stopping()?;
        Ok(units
            .iter()
            .map(|unit| {
                let computed_crc32c = crc32_castagnoli(unit.unit_bytes);
                GpuEntryDecision::ConfirmedByBothKernels(UnitChecksumOutcome {
                    computed_crc32c,
                    matches_the_expected_crc32c: computed_crc32c == unit.expected_crc32c,
                })
            })
            .collect())
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        self.give_up_once_the_caller_is_stopping()?;
        if !self.has_this_batch_counted_the_live_slots {
            self.hold_back_if_this_is_the_call(
                CallHeldBack::ClassificationInABatchWithALiveIntent,
            )?;
        }
        self.hold_back_if_this_is_the_call(CallHeldBack::ClassificationOfTheForcedRegion)?;
        let mapping_index = CentralMappingIdentityIndex::of_the_mapping_entries(mapping_entries);
        Ok(records
            .iter()
            .map(|to_classify| {
                let mut scan = scan_region_over_the_identity_index(
                    region,
                    std::slice::from_ref(&to_classify.record),
                    &mapping_index,
                    &BTreeSet::new(),
                    &BTreeSet::new(),
                    |_device, _slot, _unit_bytes| to_classify.unit_bytes.map(<[u8]>::to_vec),
                )
                .expect("单条记录：区域扫描交来的记录不跨 R 的边界，一条记录也不会两条同 key");
                GpuEntryDecision::ConfirmedByBothKernels(
                    scan.allocations
                        .pop()
                        .expect("区域扫描交来的记录起点在 R 里、未释放：扫出恰好一条")
                        .class,
                )
            })
            .collect())
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        self.give_up_once_the_caller_is_stopping()?;
        self.has_this_batch_counted_the_live_slots = true;
        self.hold_back_if_this_is_the_call(CallHeldBack::CountOfTheLiveSlots)?;
        let live_slots_per_segment =
            live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area);
        assert_eq!(
            live_slots_per_segment.len(),
            segments.len(),
            "条目键一个整段一个"
        );
        Ok(live_slots_per_segment
            .into_iter()
            .map(GpuEntryDecision::ConfirmedByBothKernels)
            .collect())
    }

    fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}

    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        self.has_this_batch_counted_the_live_slots = false;
        GpuSubmissionsOfTheBatch::default()
    }

    fn give_up_waiting_for_the_cards_once_the_caller_is_stopping(
        &mut self,
        caller_is_stopping: CallerIsStoppingTheBackgroundGpu,
    ) {
        self.caller_is_stopping = Some(caller_is_stopping);
    }
}

/// 开成：交回用例拿着的那一份把手。
struct OpenerHandingOverTheStandIn {
    gpu_compute: BackgroundGpuComputeHandle,
}

impl OpensTheBackgroundGpu for OpenerHandingOverTheStandIn {
    fn open_the_background_gpu(
        &self,
        _settings: &BackgroundComputeSettings,
    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
        Ok(OpenedBackgroundGpu {
            gpu_compute: self.gpu_compute.clone(),
            report_of_the_opening: String::new(),
            gpus_leaked: None,
        })
    }
}

/// 运行库的参数：选 `gpu`、点替身那张卡；全空聚簇段线取整段数的 100%（有一段不全空就越线），另两条关；`publish_when_due` 的周期 5 毫秒。
fn parameters_on_the_stand_in_over_the_watermark() -> RuntimeParameters {
    let mut runtime_parameters = RuntimeParameters::by_default();
    runtime_parameters.background_compute = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_of_the_stand_in()]),
        background_gpu_memory_limit: Some(vec![MEMORY_OF_THE_CARD_OF_THE_STAND_IN]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    runtime_parameters.compaction = CompactionMountParameters {
        batch_units: COMPACTION_BATCH_UNITS_BY_DEFAULT,
        watermarks: CompactionWatermarks {
            empty_cluster_segments: EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
                percent: 100,
                lowest_line_in_segments: 2,
            },
            fragmentation: FragmentationWatermark::Off,
            unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
        },
    };
    runtime_parameters.publish_poll_period = PUBLISH_POLL_PERIOD_OF_THE_THREAD;
    runtime_parameters
}

/// 用例留着的几样：门、替身的把手、摆的钟。
struct StandInOfTheTest {
    gate: Arc<GateOfTheCallHeldBack>,
    gpu_compute: BackgroundGpuComputeHandle,
    clock: Arc<ClockMovedByTheTest>,
}

fn stand_in_holding_back(call_held_back: CallHeldBack) -> StandInOfTheTest {
    stand_in_holding_back_as_often_as(call_held_back, HowOftenTheCallIsHeldBack::Once)
}

fn stand_in_holding_back_as_often_as(
    call_held_back: CallHeldBack,
    how_often: HowOftenTheCallIsHeldBack,
) -> StandInOfTheTest {
    let gate = Arc::new(GateOfTheCallHeldBack::default());
    let gpu_compute =
        BackgroundGpuComputeHandle::of_an_implementation(GpuStandInHoldingBackOneCall {
            call_held_back,
            how_often,
            gate: Arc::clone(&gate),
            caller_is_stopping: None,
            has_held_back_a_call: false,
            has_this_batch_counted_the_live_slots: false,
        });
    StandInOfTheTest {
        gate,
        gpu_compute,
        clock: Arc::new(ClockMovedByTheTest::starting_at(ONE_PERIOD)),
    }
}

/// 起维护线程挂载：挂载之后那一拍就推整理，挡在替身点名的那一次调用。
fn mounted_on_a_thread(
    pool: PoolOnMemoryDevices,
    stand_in: &StandInOfTheTest,
) -> MountedRuntime<RecordedMemoryDevice> {
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount(
        &parameters(),
        pool.devices,
        parameters_on_the_stand_in_over_the_watermark(),
        Some(&OpenerHandingOverTheStandIn {
            gpu_compute: stand_in.gpu_compute.clone(),
        }),
        Arc::clone(&stand_in.clock) as Arc<dyn ClockOfTheRuntime>,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("开卡器开成、全合法的参数：挂载照常做成");
    runtime
}

/// 不起线程挂载：用例在别的线程上调一拍，挡在替身点名的那一次调用。
fn mounted_without_a_thread(
    pool: PoolOnMemoryDevices,
    stand_in: &StandInOfTheTest,
) -> MountedRuntime<RecordedMemoryDevice> {
    mounted_without_a_thread_sending_the_events_to(
        pool,
        stand_in,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
}

/// 同 [`mounted_without_a_thread`]，维护事件送 `receiver`。
fn mounted_without_a_thread_sending_the_events_to(
    pool: PoolOnMemoryDevices,
    stand_in: &StandInOfTheTest,
    receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
) -> MountedRuntime<RecordedMemoryDevice> {
    let RuntimeMounted { runtime, .. } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        pool.devices,
        parameters_on_the_stand_in_over_the_watermark(),
        Some(&OpenerHandingOverTheStandIn {
            gpu_compute: stand_in.gpu_compute.clone(),
        }),
        Arc::clone(&stand_in.clock) as Arc<dyn ClockOfTheRuntime>,
        receiver,
    )
    .expect("开卡器开成、全合法的参数：挂载照常做成");
    runtime
}

/// 在 `deadline` 之内等到 `condition` 成立，交回等到没有。
fn waited_until(deadline: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let given_up_at = Instant::now() + deadline;
    while !condition() {
        if Instant::now() >= given_up_at {
            return false;
        }
        std::thread::yield_now();
    }
    true
}

fn pending_changes_of(runtime: &MountedRuntime<RecordedMemoryDevice>) -> u64 {
    runtime
        .with_the_session_and_devices(|locked| locked.session.pending_changes_accepted())
        .expect("运行库没坏")
}

/// 现行那一版里有没有活意图（盘上的权威）。
fn is_an_intent_live_in(runtime: &MountedRuntime<RecordedMemoryDevice>) -> bool {
    runtime
        .with_the_session_and_devices(|locked| {
            live_compaction_intent_of_the_entries(
                &locked
                    .session
                    .session()
                    .current
                    .version_of_trees()
                    .expect("有树的一版")
                    .logged_operations,
            )
            .is_some()
        })
        .expect("运行库没坏")
}

/// 前台四个入口这一串的结局。
#[derive(Debug, PartialEq, Eq)]
struct ForegroundEntriesAnswered {
    is_the_change_accepted: bool,
    is_the_fsync_answered: bool,
    is_the_user_change_published: bool,
    pending_changes_read: u64,
}

/// 验收第 1 条：锁外那一段等 GPU 时（替身挡着），收下改动、fsync、单发改动、读都立刻取到会话锁，不等替身放行。
// 不切片并行：一个池、一条维护线程，被测的就是锁外那一段与前台在一把锁上的交错，没有可切的区间。
#[test]
fn foreground_entries_take_the_session_lock_while_the_gpu_of_the_batch_out_of_the_lock_is_held_back(
) {
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_on_a_thread(pool_with_holes_in_the_data_segments(), &stand_in);
    assert!(
        stand_in
            .gate
            .wait_until_a_call_is_held_back(THREAD_DEADLINE),
        "前提：维护线程把一批交到锁外，替身挡在数活槽那一次"
    );
    let (answered_sender, answered_receiver) = channel::<ForegroundEntriesAnswered>();
    let (answered_while_held_back, is_still_held_back) = std::thread::scope(|scope| {
        scope.spawn(|| {
            let is_the_change_accepted = runtime
                .accept(UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 21,
                })
                .is_ok();
            let is_the_fsync_answered = runtime.publish_pending_now().is_ok();
            let is_the_user_change_published = runtime
                .publish_user_change(UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 22,
                })
                .is_ok();
            let pending_changes_read = pending_changes_of(&runtime);
            answered_sender
                .send(ForegroundEntriesAnswered {
                    is_the_change_accepted,
                    is_the_fsync_answered,
                    is_the_user_change_published,
                    pending_changes_read,
                })
                .expect("用例在等前台这一串");
        });
        let answered = answered_receiver.recv_timeout(DEADLINE_WHILE_THE_GPU_IS_HELD_BACK);
        let is_still_held_back = !stand_in.gate.is_open();
        if answered.is_err() {
            // 前台等着锁：叫替身放弃那一等，算的那一段交回、前台取到锁，这一串照常收尾再判红。
            stand_in
                .gpu_compute
                .tell_the_implementation_the_caller_is_stopping();
        }
        (answered, is_still_held_back)
    });
    assert_eq!(
        answered_while_held_back,
        Ok(ForegroundEntriesAnswered {
            is_the_change_accepted: true,
            is_the_fsync_answered: true,
            is_the_user_change_published: true,
            pending_changes_read: 0,
        }),
        "替身挡着时四个入口都立刻取到会话锁、做成（fsync 把收下的那一个发了）"
    );
    assert!(is_still_held_back, "前台做成的时候替身还挡着");
    stand_in.gate.open();
    runtime.unmount().expect("放行之后卸载干净");
}

/// 验收第 3 条：替身挡着时前台收下的改动，到期之后在下一个 `publish_poll_period` 被维护线程发出去（`T_time` 之后，D16（发布语义） 已定项 5 的丢失窗口），
/// 不等替身放行。
// 不切片并行：一个池、一条维护线程，被测的是锁外那一段期间维护线程自己的发布，没有可切的区间。
#[test]
fn a_change_accepted_while_the_gpu_is_held_back_is_published_by_the_poll_once_it_is_due() {
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_on_a_thread(pool_with_holes_in_the_data_segments(), &stand_in);
    assert!(
        stand_in
            .gate
            .wait_until_a_call_is_held_back(THREAD_DEADLINE),
        "前提：维护线程把一批交到锁外，替身挡在数活槽那一次"
    );
    let time_threshold = runtime
        .with_the_session_and_devices(|locked| locked.session.thresholds().time_threshold())
        .expect("运行库没坏");
    runtime
        .accept(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 23,
        })
        .expect("收进待发");
    assert_eq!(
        pending_changes_of(&runtime),
        1,
        "前提：收下时没到期，留在待发里"
    );
    stand_in.clock.advance_by(time_threshold);
    let is_published_while_held_back = waited_until(DEADLINE_WHILE_THE_GPU_IS_HELD_BACK, || {
        pending_changes_of(&runtime) == 0
    });
    // 待发在会话锁里清零，发成的那一次放锁之后才记进观测：先等记上再读，读数照下面那条断言判。
    waited_until(DEADLINE_WHILE_THE_GPU_IS_HELD_BACK, || {
        runtime.observations().pending_publishes >= 1
    });
    let is_still_held_back = !stand_in.gate.is_open();
    let pending_publishes = runtime.observations().pending_publishes;
    stand_in.gate.open();
    runtime.unmount().expect("放行之后卸载干净");
    assert!(
        is_published_while_held_back,
        "到期之后维护线程在下一个 P 回锁里发了待发，不等锁外那一段"
    );
    assert!(is_still_held_back, "发出去的时候替身还挡着");
    assert!(
        pending_publishes >= 1,
        "维护线程记下发成的那一次：{pending_publishes}"
    );
}

/// 验收第 3 条：替身挡着时一次用户改动写盘报错、发布路径冻结，维护线程在下一个 `publish_poll_period` 重发，不等替身放行。
// 不切片并行：一个池、一条维护线程，被测的是锁外那一段期间维护线程自己的重发，没有可切的区间。
#[test]
fn a_publish_frozen_while_the_gpu_is_held_back_is_resent_by_the_poll_before_the_gpu_answers() {
    let pool = pool_with_holes_in_the_data_segments();
    let faults = pool.faults.clone();
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_on_a_thread(pool, &stand_in);
    assert!(
        stand_in
            .gate
            .wait_until_a_call_is_held_back(THREAD_DEADLINE),
        "前提：维护线程把一批交到锁外，替身挡在数活槽那一次"
    );
    faults
        .writes_left_to_fail
        .store(1, std::sync::atomic::Ordering::SeqCst);
    assert!(
        runtime
            .publish_user_change(UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 24,
            })
            .is_err(),
        "前提：写盘报错，这次改动没发成、发布路径冻结"
    );
    let is_resent_while_held_back = waited_until(DEADLINE_WHILE_THE_GPU_IS_HELD_BACK, || {
        runtime.observations().frozen_publishes_resent >= 1
    });
    let is_still_held_back = !stand_in.gate.is_open();
    stand_in.gate.open();
    runtime.unmount().expect("放行之后卸载干净");
    assert!(
        is_resent_while_held_back,
        "维护线程在下一个 P 重发了冻结的那一次，不等锁外那一段"
    );
    assert!(is_still_held_back, "重发的时候替身还挡着");
}

/// 验收第 4 条：有活意图的一批在锁外、替身挡着时卸载，不等替身放行就返回（替身读到调用方在停、放弃那一等），那一批不回锁：
/// 活意图留在盘上，再挂续做它（D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交」）。
// 不切片并行：一个池、一条维护线程，被测的是卸载与锁外那一段的交错，没有可切的区间。
#[test]
fn unmount_while_the_gpu_of_a_batch_with_a_live_intent_is_held_back_returns_without_waiting_and_the_next_mount_continues_the_intent(
) {
    let stand_in = stand_in_holding_back(CallHeldBack::ClassificationInABatchWithALiveIntent);
    let runtime = mounted_on_a_thread(pool_with_holes_in_the_data_segments(), &stand_in);
    assert!(
        stand_in
            .gate
            .wait_until_a_call_is_held_back(THREAD_DEADLINE),
        "前提：维护线程签了意图、把有活意图的一批交到锁外，替身挡在那一批分类记录"
    );
    assert!(is_an_intent_live_in(&runtime), "前提：盘上有活意图");
    let (unmounted_sender, unmounted_receiver) = channel();
    let unmounting = std::thread::spawn(move || {
        unmounted_sender
            .send(runtime.unmount())
            .expect("用例在等卸载的结局");
    });
    let unmounted_while_held_back =
        unmounted_receiver.recv_timeout(DEADLINE_WHILE_THE_GPU_IS_HELD_BACK);
    let is_still_held_back = !stand_in.gate.is_open();
    let has_returned_while_held_back = unmounted_while_held_back.is_ok();
    let unmounted = match unmounted_while_held_back {
        Ok(unmounted) => unmounted,
        Err(_timed_out) => {
            stand_in.gate.open();
            unmounted_receiver.recv().expect("放行之后卸载返回")
        }
    };
    unmounting.join().expect("卸载那条线程不 panic");
    assert!(has_returned_while_held_back, "卸载不等替身放行就返回");
    assert!(is_still_held_back, "卸载返回的时候替身还挡着");
    assert!(
        stand_in.gate.has_given_up_for_the_caller_stopping(),
        "替身读到调用方在停、放弃了那一等"
    );
    let unmounted = unmounted.expect("卸载干净");
    let RuntimeMounted {
        output,
        runtime: runtime_of_the_next_mount,
    } = MountedRuntime::mount_without_the_maintenance_thread(
        &parameters(),
        unmounted.devices,
        RuntimeParameters::by_default(),
        None,
        Arc::new(ClockMovedByTheTest::starting_at(ONE_PERIOD)) as Arc<dyn ClockOfTheRuntime>,
        Arc::new(IgnoringTheMaintenanceEvents),
    )
    .expect("再挂");
    assert!(
        output.compaction_continuation.run.is_some(),
        "卸载时那一批没回锁：活意图留在盘上，再挂续做它"
    );
    runtime_of_the_next_mount.unmount().expect("卸载干净");
}

/// 验收第 4 条：替身挡着时丢掉运行库，不等替身放行就返回（替身读到调用方在停、放弃那一等）。
// 不切片并行：一个池、一条维护线程，被测的是丢弃与锁外那一段的交错，没有可切的区间。
#[test]
fn dropping_the_runtime_while_the_gpu_is_held_back_returns_without_waiting() {
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_on_a_thread(pool_with_holes_in_the_data_segments(), &stand_in);
    assert!(
        stand_in
            .gate
            .wait_until_a_call_is_held_back(THREAD_DEADLINE),
        "前提：维护线程把一批交到锁外，替身挡在数活槽那一次"
    );
    let (dropped_sender, dropped_receiver) = channel::<()>();
    let dropping = std::thread::spawn(move || {
        drop(runtime);
        dropped_sender.send(()).expect("用例在等丢弃返回");
    });
    let has_returned_while_held_back = dropped_receiver
        .recv_timeout(DEADLINE_WHILE_THE_GPU_IS_HELD_BACK)
        .is_ok();
    let is_still_held_back = !stand_in.gate.is_open();
    stand_in.gate.open();
    dropping.join().expect("丢弃那条线程不 panic");
    assert!(has_returned_while_held_back, "丢弃不等替身放行就返回");
    assert!(is_still_held_back, "丢弃返回的时候替身还挡着");
    assert!(
        stand_in.gate.has_given_up_for_the_caller_stopping(),
        "替身读到调用方在停、放弃了那一等"
    );
}

/// 一拍在别的线程上推，替身挡住时用例做 `while_held_back`，再放行；交回挡住了没有、`while_held_back` 交回的与那一拍的结局。
fn one_pass_with_the_foreground_acting_while_held_back<Answer>(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    stand_in: &StandInOfTheTest,
    while_held_back: impl FnOnce() -> Answer,
) -> (bool, Option<Answer>, MaintenancePass) {
    std::thread::scope(|scope| {
        let pass_thread = scope.spawn(|| runtime.run_one_maintenance_pass());
        let is_held_back = stand_in
            .gate
            .wait_until_a_call_is_held_back(THREAD_DEADLINE);
        let answer = is_held_back.then(while_held_back);
        stand_in.gate.open();
        let pass = pass_thread
            .join()
            .expect("一拍那条线程不 panic")
            .expect("运行库没坏");
        (is_held_back, answer, pass)
    })
}

/// 回锁时有冻结着的发布（锁外那一段期间一次用户改动写盘报错）：这一批丢掉、一个写都不发，这一拍停在这一次调用（下一拍先重发）。
// 不切片并行：一个池，被测的是锁外那一段与前台那一次写错冻结的先后，没有可切的区间。
#[test]
fn a_batch_relocked_after_a_publish_froze_out_of_the_lock_is_dropped_and_ends_the_pass() {
    let pool = pool_with_holes_in_the_data_segments();
    let faults = pool.faults.clone();
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_without_a_thread(pool, &stand_in);
    let (is_held_back, is_the_user_change_refused, pass) =
        one_pass_with_the_foreground_acting_while_held_back(&runtime, &stand_in, || {
            faults.writes_left_to_fail.store(
                WRITES_FAILING_UNTIL_THE_TEST_CLEARS_THEM,
                std::sync::atomic::Ordering::SeqCst,
            );
            runtime
                .publish_user_change(UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 25,
                })
                .is_err()
        });
    faults
        .writes_left_to_fail
        .store(0, std::sync::atomic::Ordering::SeqCst);
    assert!(
        is_held_back,
        "前提：一拍把一批交到锁外，替身挡在数活槽那一次"
    );
    assert_eq!(
        is_the_user_change_refused,
        Some(true),
        "前提：锁外那一段期间写盘报错，发布路径冻结"
    );
    let observations = runtime.observations();
    assert_eq!(
        observations.last_stop,
        Some(CompactionRunStop::BatchDroppedWhileAPublishIsFrozenAtTheRelock),
        "回锁时冻结着：这一批丢掉"
    );
    assert_eq!(
        observations.batches_dropped_while_a_publish_is_frozen_at_the_relock,
        1
    );
    assert_eq!(pass.compaction_publishes, 0, "一个写都不发：{pass:?}");
    assert_eq!(
        pass.compaction_calls, 1,
        "丢批这一次之后这一拍不接着调：{pass:?}"
    );
}

/// 回锁时交进去的盘表不是挂载核过的那一份（锁外那一段期间用例把两块盘换了次序）：核心层不收这一批，运行库记下、这一拍停；
/// 那一批丢了，这个会话之后每一拍取输入都拒（有一批在锁外）。
// 不切片并行：一个池，被测的是锁外那一段期间换了盘表次序、之后几拍的先后，没有可切的区间。
#[test]
fn a_batch_relocked_with_a_device_table_other_than_the_mount_is_not_taken_back_and_later_inputs_are_refused(
) {
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_without_a_thread(pool_with_holes_in_the_data_segments(), &stand_in);
    let swap_the_two_devices = || {
        runtime
            .with_the_session_and_devices(|locked| locked.devices.swap(0, 1))
            .expect("运行库没坏");
    };
    let (is_held_back, _swapped, pass) = one_pass_with_the_foreground_acting_while_held_back(
        &runtime,
        &stand_in,
        swap_the_two_devices,
    );
    assert!(
        is_held_back,
        "前提：一拍把一批交到锁外，替身挡在数活槽那一次"
    );
    assert_eq!(
        runtime
            .observations()
            .batches_out_of_the_lock_refused_at_the_relock,
        1,
        "回锁没收下这一批"
    );
    assert_eq!(pass.compaction_publishes, 0, "一个写都不发：{pass:?}");
    assert_eq!(pass.compaction_calls, 1, "这一拍停在这一次调用：{pass:?}");
    swap_the_two_devices();
    stand_in.clock.advance_by(ONE_PERIOD);
    let later_pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    assert_eq!(
        later_pass.compaction_publishes, 0,
        "之后一拍一批都不发：{later_pass:?}"
    );
    assert_eq!(
        runtime.observations().last_not_started,
        Some(BackgroundCompactionNotStarted::ABatchOfThisSessionIsOutOfTheLock),
        "这个会话还记着有一批在锁外：取输入拒"
    );
}

fn publishes_since_the_last_user_change_of(runtime: &MountedRuntime<RecordedMemoryDevice>) -> u64 {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .compaction
                .publishes_since_the_last_user_change
        })
        .expect("运行库没坏")
}

/// 卸载时那一批在锁外、替身挡着（调用方在停，替身放弃那一等），写盘报错让排空待发被拒、运行库交回：交回之前撤了 GPU 停标记、
/// 会话上不再记有一批在锁外，丢掉的那一批所在的这一趟留着没完、发布预算不扣；放行替身、写盘不再报错之后同步推一拍，后台整理照常取到输入、发成一批。
/// 交回的运行库不起维护线程（同卸载排空被拒今天的做法）。
// 不切片并行：一个池、一条维护线程，被测的是卸载、锁外那一段与交回之后那一拍的先后，没有可切的区间。
#[test]
fn a_runtime_handed_back_by_an_unmount_refused_at_the_drain_compacts_again_on_the_next_pass() {
    let pool = pool_with_holes_in_the_data_segments();
    let faults = pool.faults.clone();
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_on_a_thread(pool, &stand_in);
    assert!(
        stand_in
            .gate
            .wait_until_a_call_is_held_back(THREAD_DEADLINE),
        "前提：维护线程把一批交到锁外，替身挡在数活槽那一次"
    );
    runtime
        .accept(UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 28,
        })
        .expect("收进待发");
    assert_eq!(
        pending_changes_of(&runtime),
        1,
        "前提：收下时没到期，留在待发里"
    );
    let publishes_since_the_last_user_change_before_the_unmount =
        publishes_since_the_last_user_change_of(&runtime);
    faults
        .writes_left_to_fail
        .store(WRITES_FAILING_UNTIL_THE_TEST_CLEARS_THEM, Ordering::SeqCst);
    let (unmounted_sender, unmounted_receiver) = channel();
    let unmounting = std::thread::spawn(move || {
        unmounted_sender
            .send(runtime.unmount())
            .expect("用例在等卸载的结局");
    });
    let unmounted = match unmounted_receiver.recv_timeout(DEADLINE_WHILE_THE_GPU_IS_HELD_BACK) {
        Ok(unmounted) => unmounted,
        Err(_timed_out) => {
            stand_in.gate.open();
            unmounted_receiver.recv().expect("放行之后卸载返回")
        }
    };
    unmounting.join().expect("卸载那条线程不 panic");
    faults.writes_left_to_fail.store(0, Ordering::SeqCst);
    let runtime_handed_back = match unmounted {
        Err(refused) => match *refused {
            UnmountRefused::PendingChangesNotPublished {
                runtime: runtime_returned,
                ..
            } => runtime_returned,
            UnmountRefused::Failed { failure, .. } => {
                panic!("前提：排空待发被拒、运行库交回，不是运行库坏了：{failure:?}")
            }
            UnmountRefused::FrozenPublishResendFailed { refusal, .. } => {
                panic!("前提：排空待发被拒、运行库交回，不是重发冻结报错：{refusal:?}")
            }
            UnmountRefused::UnmountFailed { refusal, .. } => {
                panic!("前提：排空待发被拒、运行库交回，不是核心层卸载没做成：{refusal:?}")
            }
        },
        Ok(unmounted) => panic!("前提：写盘报错，排空待发被拒：{unmounted:?}"),
    };
    let has_given_up_for_the_caller_stopping = stand_in.gate.has_given_up_for_the_caller_stopping();
    stand_in.gate.open();
    let (is_a_batch_out_of_the_lock_after_the_unmount, is_the_trip_under_way_after_the_unmount) =
        runtime_handed_back
            .with_the_session_and_devices(|locked| {
                let compaction = &locked.session.session().compaction;
                (
                    compaction.has_a_batch_out_of_the_lock(),
                    compaction.has_a_background_trip_under_way(),
                )
            })
            .expect("运行库没坏");
    let publishes_since_the_last_user_change_after_the_unmount =
        publishes_since_the_last_user_change_of(&runtime_handed_back);
    stand_in.clock.advance_by(ONE_PERIOD);
    let pass = runtime_handed_back
        .run_one_maintenance_pass()
        .expect("运行库没坏");
    let observations = runtime_handed_back.observations();
    assert!(
        has_given_up_for_the_caller_stopping,
        "前提：卸载时替身读到调用方在停、放弃了那一等"
    );
    assert!(
        !is_a_batch_out_of_the_lock_after_the_unmount,
        "交回的会话上不再记有一批在锁外"
    );
    assert!(
        is_the_trip_under_way_after_the_unmount,
        "丢掉的那一批所在的这一趟留着没完"
    );
    assert_eq!(
        publishes_since_the_last_user_change_after_the_unmount,
        publishes_since_the_last_user_change_before_the_unmount,
        "丢掉那一批不扣发布预算"
    );
    assert_ne!(
        observations.last_not_started,
        Some(BackgroundCompactionNotStarted::ABatchOfThisSessionIsOutOfTheLock),
        "交回之后那一拍取输入没被「有一批在锁外」拒：{observations:?}"
    );
    assert_eq!(
        observations.alarms_received, 0,
        "停标记撤了：替身照常算，整理不因调用方在停的那个错停下：{observations:?}"
    );
    assert!(
        pass.compaction_publishes >= 1,
        "交回的运行库推一拍：后台整理取到输入、发成一批：{pass:?}、{observations:?}"
    );
    runtime_handed_back
        .unmount()
        .expect("写盘不再报错：卸载干净");
}

/// 替身在锁外那一段交回调用方在停的那个错（`BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping`），运行库自己没在停、照常回锁：
/// 回锁之后照停下那个后台任务那一组办（`BackgroundComputeError::reach_of_the_error`）——这一批放弃、整理停下、报警带这个错，之后叫起报停着。
/// 同步入口推（不起线程）：用例先经把手置停标记，替身每一段调用一进来就放弃。
// 不切片并行：一个池、同步的几拍，被测的是放弃的错回锁之后整理停下与之后叫起的先后，没有可切的区间。
#[test]
fn a_gpu_giving_up_for_the_caller_stopping_in_a_batch_relocked_stops_the_compaction_and_alarms() {
    let stand_in = stand_in_holding_back(CallHeldBack::CountOfTheLiveSlots);
    let runtime = mounted_without_a_thread(pool_with_holes_in_the_data_segments(), &stand_in);
    stand_in
        .gpu_compute
        .tell_the_implementation_the_caller_is_stopping();
    let pass = runtime.run_one_maintenance_pass().expect("运行库没坏");
    let observations_after_the_error = runtime.observations();
    stand_in.clock.advance_by(ONE_PERIOD);
    runtime.run_one_maintenance_pass().expect("运行库没坏");
    let observations_after_the_next_wake = runtime.observations();
    assert_eq!(
        pass.compaction_publishes, 0,
        "这一批放弃、一个单元都没搬：{pass:?}"
    );
    assert_eq!(
        observations_after_the_error.last_stop,
        Some(
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped,
            }
        ),
        "回锁时出错先于一切：这一批放弃、整理停下"
    );
    let alarm = observations_after_the_error
        .last_alarm
        .expect("整理停下，叫起报警");
    assert_eq!(alarm.task, BackgroundTask::CompactionRelocation);
    assert!(
        matches!(
            alarm.error_that_stopped_the_task,
            BackgroundComputeError::GpuSegmentFailed {
                failure: BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping,
                ..
            }
        ),
        "报警带的是调用方在停的那个错：{:?}",
        alarm.error_that_stopped_the_task
    );
    assert_eq!(
        observations_after_the_next_wake.last_not_started,
        Some(BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError),
        "之后叫起报整理停着"
    );
}

/// 一次后台调用的结局（维护事件里每一次调用按先后记）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndOfACompactionCall {
    Stopped(CompactionRunStop),
    Failed,
    NotStarted(BackgroundCompactionNotStarted),
}

/// 按先后记下每一次后台调用结局的接收方。
#[derive(Debug, Default)]
struct ReceiverRecordingTheEndOfEachCompactionCall {
    ends: Mutex<Vec<EndOfACompactionCall>>,
}

impl ReceiverRecordingTheEndOfEachCompactionCall {
    fn ends_recorded(&self) -> Vec<EndOfACompactionCall> {
        self.ends.lock().expect("记结局的锁里不 panic").clone()
    }
}

impl ReceiverOfTheMaintenanceEvents for ReceiverRecordingTheEndOfEachCompactionCall {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>) {
        let end = match event {
            MaintenanceEvent::CompactionCallReturned(BackgroundCompactionCall::Ran {
                run, ..
            }) => match &run.outcome {
                CompactionRunOutcome::Stopped(stop) => EndOfACompactionCall::Stopped(*stop),
                CompactionRunOutcome::Failed(_) => EndOfACompactionCall::Failed,
            },
            MaintenanceEvent::CompactionCallReturned(BackgroundCompactionCall::NotStarted {
                reason,
                ..
            }) => EndOfACompactionCall::NotStarted(*reason),
            MaintenanceEvent::PendingChangesPublished(_)
            | MaintenanceEvent::PublishWhenDueRefused(_)
            | MaintenanceEvent::CompactionCallRefused(_)
            | MaintenanceEvent::FrozenPublishResent
            | MaintenanceEvent::FrozenPublishResendFailed(_)
            | MaintenanceEvent::FloorRaisesPushedByAnAccept(_)
            | MaintenanceEvent::MaintenanceThreadPanicked { .. } => return,
        };
        self.ends.lock().expect("记结局的锁里不 panic").push(end);
    }
}

/// 强制选的那一段（[`force_the_segment_holding_only_data_units_of_the_first_file`] 交回）。
struct SegmentOfTheFirstFileForced {
    segment_start_slot: u64,
    /// 那一段里第一个文件那几个数据单元在文件里的下标，按它们在第一块盘上那一份的槽号从低到高。
    data_units_in_slot_order: Vec<u64>,
}

/// 只供测试的开关强制选段与批大小（`None` 照挂载的批大小）：第一块盘上未释放的记录全是第一个文件数据单元在这块盘上那一份的那几段里，
/// 这样的数据单元最多的那一段。覆盖写第一个文件就把那一段里的记录都放掉。
fn force_the_segment_holding_only_data_units_of_the_first_file(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    batch_units: Option<CompactionBatchUnits>,
) -> SegmentOfTheFirstFileForced {
    runtime
        .with_the_session_and_devices(|locked| {
            let session = locked.session.session_for_tests_only();
            let version = session.current.version_of_trees().expect("有树的一版");
            let data_unit_at_the_slot_on_the_first_device: BTreeMap<u64, u64> = version
                .data_pointers
                .iter()
                .enumerate()
                .flat_map(|(index_in_the_file, pointer)| {
                    let data_unit_index =
                        u64::try_from(index_in_the_file).expect("文件的单元下标装得进 u64");
                    pointer
                        .locations
                        .iter()
                        .filter(|location| location.device == DeviceIdentity(0))
                        .map(move |location| (location.slot.0, data_unit_index))
                })
                .collect();
            let mut unreleased_slots_by_segment: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
            for record in version
                .allocation_records
                .iter()
                .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
            {
                unreleased_slots_by_segment
                    .entry(record.slot.0 / CLUSTER_SEGMENT_SLOTS * CLUSTER_SEGMENT_SLOTS)
                    .or_default()
                    .push(record.slot.0);
            }
            let (segment_start_slot, mut slots_in_the_segment) = unreleased_slots_by_segment
                .into_iter()
                .filter(|(_segment_start_slot, slots)| {
                    slots
                        .iter()
                        .all(|slot| data_unit_at_the_slot_on_the_first_device.contains_key(slot))
                })
                .max_by_key(|(_segment_start_slot, slots)| slots.len())
                .expect("前提：第一块盘上至少有一段未释放的记录全是第一个文件的数据单元");
            slots_in_the_segment.sort_unstable();
            let data_units_in_slot_order = slots_in_the_segment
                .iter()
                .map(|slot| data_unit_at_the_slot_on_the_first_device[slot])
                .collect();
            let mut switches: CompactionSwitches = session.allocator.compaction().switches;
            switches.forced_region = Some(
                CompactionRegion::of_one_cluster_segment(
                    DeviceIdentity(0),
                    SlotNumber(segment_start_slot),
                )
                .expect("按整段取整的段起点"),
            );
            if let Some(units) = batch_units {
                switches.batch_size = CompactionBatchSize::ForcedForTests(units);
            }
            session.allocator.set_compaction_switches(switches);
            SegmentOfTheFirstFileForced {
                segment_start_slot,
                data_units_in_slot_order,
            }
        })
        .expect("运行库没坏")
}

/// 第一块盘上起点在 `segment_start_slot` 那一段里的未释放记录数（现行那一版）。
fn unreleased_records_in_the_segment(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    segment_start_slot: u64,
) -> usize {
    runtime
        .with_the_session_and_devices(|locked| {
            locked
                .session
                .session()
                .current
                .version_of_trees()
                .expect("有树的一版")
                .allocation_records
                .iter()
                .filter(|record| {
                    record.device == DeviceIdentity(0)
                        && !record.is_released
                        && record.slot.0 / CLUSTER_SEGMENT_SLOTS * CLUSTER_SEGMENT_SLOTS
                            == segment_start_slot
                })
                .count()
        })
        .expect("运行库没坏")
}

/// 覆盖写第一个文件：它原来的数据单元都放掉（新内容在一个数据单元以内）。
fn overwrite_the_first_file(runtime: &MountedRuntime<RecordedMemoryDevice>) {
    let content = vec![0x5A_u8; 2500];
    runtime
        .publish_user_change(UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 27,
        }))
        .expect("覆盖写第一个文件");
}

/// 部分写第一个文件下标 `data_unit_index` 那一个数据单元的头一个字节。部分写从偏移所在的单元起到文件末尾一起重写
/// （`singlefs_core::transaction::publish_partial_write`）：下标不小于 `data_unit_index` 的数据单元旧落点都放掉，它们的记录都变了。
fn rewrite_the_first_file_from_the_data_unit_to_the_end(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    data_unit_index: u64,
) {
    let one_byte = [0xA5_u8];
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷装得进 u64");
    runtime
        .publish_user_change(UserChange::WriteAtOffset {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            offset: FileOffsetInBytes(data_unit_index * payload),
            bytes: PartialWriteBytes::new(&one_byte).expect("一个字节不是空的"),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 26,
        })
        .expect("部分写一个数据单元");
}

/// 等每一次都挡的替身挡过 `calls` 次；那一拍先结束了（不会再挡）就不等满 [`THREAD_DEADLINE`]。交回等到没有。
fn held_back_before_the_pass_ends<Answer>(
    gate: &GateOfTheCallHeldBack,
    calls: u64,
    pass_thread: &std::thread::ScopedJoinHandle<'_, Answer>,
) -> bool {
    let given_up_at = Instant::now() + THREAD_DEADLINE;
    loop {
        if gate.wait_until_the_calls_held_back_one_by_one_reach(calls, POLL_OF_THE_PASS_THREAD) {
            return true;
        }
        if pass_thread.is_finished() || Instant::now() >= given_up_at {
            return gate.wait_until_the_calls_held_back_one_by_one_reach(calls, Duration::ZERO);
        }
    }
}

/// 强制选段、每一批分类记录都挡的替身、维护事件按先后记，不起线程挂载。
fn mounted_with_the_segment_of_the_first_file_forced(
    batch_units: Option<CompactionBatchUnits>,
) -> (
    MountedRuntime<RecordedMemoryDevice>,
    StandInOfTheTest,
    Arc<ReceiverRecordingTheEndOfEachCompactionCall>,
    SegmentOfTheFirstFileForced,
) {
    let receiver = Arc::new(ReceiverRecordingTheEndOfEachCompactionCall::default());
    let stand_in = stand_in_holding_back_as_often_as(
        CallHeldBack::ClassificationOfTheForcedRegion,
        HowOftenTheCallIsHeldBack::EveryTimeUntilTheGateOpens,
    );
    let runtime = mounted_without_a_thread_sending_the_events_to(
        pool_with_holes_in_the_data_segments(),
        &stand_in,
        Arc::clone(&receiver) as Arc<dyn ReceiverOfTheMaintenanceEvents>,
    );
    let forced = force_the_segment_holding_only_data_units_of_the_first_file(&runtime, batch_units);
    (runtime, stand_in, receiver, forced)
}

/// 强制选段、没有活意图：签发那一批在锁外时（替身挡着）覆盖写第一个文件，那一段里的记录都放掉；回锁时选中的段在现行那一版上没了资格、这一次不签
/// （`CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock`），这一拍接着叫（钉运行库 `the_pass_calls_again_after` 的判法）。
// 不切片并行：一个池，被测的是锁外那一段与前台改动的先后，没有可切的区间。
#[test]
fn after_the_chosen_segment_is_no_longer_eligible_at_the_relock_the_pass_calls_again() {
    let (runtime, stand_in, receiver, forced) =
        mounted_with_the_segment_of_the_first_file_forced(None);
    let (is_held_back, unreleased_records_left_by_the_overwrite, pass) =
        std::thread::scope(|scope| {
            let pass_thread = scope.spawn(|| runtime.run_one_maintenance_pass());
            let is_held_back = held_back_before_the_pass_ends(&stand_in.gate, 1, &pass_thread);
            let unreleased_records_left_by_the_overwrite = is_held_back.then(|| {
                overwrite_the_first_file(&runtime);
                unreleased_records_in_the_segment(&runtime, forced.segment_start_slot)
            });
            stand_in.gate.open();
            let pass = pass_thread
                .join()
                .expect("一拍那条线程不 panic")
                .expect("运行库没坏");
            (is_held_back, unreleased_records_left_by_the_overwrite, pass)
        });
    let ends = receiver.ends_recorded();
    assert!(
        is_held_back,
        "前提：签发那一批交到锁外，替身挡在分类强制选的那一段：{ends:?}"
    );
    assert_eq!(
        unreleased_records_left_by_the_overwrite,
        Some(0),
        "前提：覆盖写第一个文件之后那一段里没有未释放的记录"
    );
    let position_of_the_stop = ends
        .iter()
        .position(|end| {
            *end == EndOfACompactionCall::Stopped(
                CompactionRunStop::ChosenSegmentNoLongerEligibleAtTheRelock,
            )
        })
        .unwrap_or_else(|| panic!("前提：回锁时选中的段没了资格、这一次不签：{ends:?}"));
    assert!(
        position_of_the_stop + 1 < ends.len(),
        "没了资格那一次之后这一拍接着叫：{ends:?}、{pass:?}"
    );
}

/// 强制选段：签了意图之后有活意图的那一批在锁外时（替身挡着）覆盖写第一个文件，计划里的单元记录都变了；回锁时剔完一个不剩、这一批不发
/// （`CompactionRunStop::BatchPrunedToNothingAtTheRelock`），这一拍接着叫（钉运行库 `the_pass_calls_again_after` 的判法），
/// 观测 `units_pruned_at_the_relock` 恰好加上那一次剔掉的单元数。
// 不切片并行：一个池，被测的是锁外那一段与前台改动的先后，没有可切的区间。
#[test]
fn after_a_batch_pruned_to_nothing_at_the_relock_the_pass_calls_again_and_the_units_pruned_are_observed(
) {
    let (runtime, stand_in, receiver, forced) =
        mounted_with_the_segment_of_the_first_file_forced(None);
    let (is_the_issue_held_back, is_the_live_intent_batch_held_back, unreleased_records_left, pass) =
        std::thread::scope(|scope| {
            let pass_thread = scope.spawn(|| runtime.run_one_maintenance_pass());
            let is_the_issue_held_back =
                held_back_before_the_pass_ends(&stand_in.gate, 1, &pass_thread);
            stand_in.gate.release_one_call();
            let is_the_live_intent_batch_held_back =
                held_back_before_the_pass_ends(&stand_in.gate, 2, &pass_thread);
            let unreleased_records_left = is_the_live_intent_batch_held_back.then(|| {
                overwrite_the_first_file(&runtime);
                unreleased_records_in_the_segment(&runtime, forced.segment_start_slot)
            });
            stand_in.gate.open();
            let pass = pass_thread
                .join()
                .expect("一拍那条线程不 panic")
                .expect("运行库没坏");
            (
                is_the_issue_held_back,
                is_the_live_intent_batch_held_back,
                unreleased_records_left,
                pass,
            )
        });
    let ends = receiver.ends_recorded();
    assert!(
        is_the_issue_held_back && is_the_live_intent_batch_held_back,
        "前提：签发那一批与之后有活意图的那一批先后交到锁外：{ends:?}"
    );
    assert_eq!(
        unreleased_records_left,
        Some(0),
        "前提：覆盖写第一个文件之后那一段里没有未释放的记录"
    );
    let (position_of_the_stop, units_pruned) = ends
        .iter()
        .enumerate()
        .find_map(|(position, end)| {
            if let EndOfACompactionCall::Stopped(
                CompactionRunStop::BatchPrunedToNothingAtTheRelock { units_pruned },
            ) = end
            {
                Some((position, *units_pruned))
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("前提：回锁时剔完一个不剩、这一批不发：{ends:?}"));
    assert!(units_pruned >= 1, "前提：计划里本来有要搬的单元");
    assert!(
        position_of_the_stop + 1 < ends.len(),
        "剔完一个不剩那一次之后这一拍接着叫：{ends:?}、{pass:?}"
    );
    assert_eq!(
        runtime.observations().units_pruned_at_the_relock,
        units_pruned,
        "回锁剔掉几个，观测恰好加几个"
    );
}

/// [`one_pass_rewriting_the_forced_segment_while_its_live_intent_batch_is_held_back`] 那一拍替身挡几次：签发那一批与之后有活意图的那一批各一次。
const CALLS_HELD_BACK_IN_A_PASS_REWRITING_THE_FORCED_SEGMENT: u64 = 2;

/// [`one_pass_rewriting_the_forced_segment_while_its_live_intent_batch_is_held_back`] 交回的。
struct PassRewritingTheForcedSegment {
    /// 签发那一批与之后有活意图的那一批都挡到了时，有活意图的那一批挡着、部分写之后强制选的那一段里还剩几条未释放的记录；没挡到两批是 `None`。
    unreleased_records_left_by_the_rewrite: Option<usize>,
    /// 有活意图的那一批放行之后这一拍又挡住了一次（挡住了就开门，让这一拍做完）。
    is_held_back_after_the_rewrite: bool,
    pass: MaintenancePass,
}

/// 推一拍：替身挡过的次数从 `calls_held_back_before` 数起，签发那一批挡着时放行；之后有活意图的那一批挡着时部分写强制选的那一段里文件下标最小的那个数据单元
/// （从它起到文件末尾一起重写，那一段里的记录都变了），再放行。
fn one_pass_rewriting_the_forced_segment_while_its_live_intent_batch_is_held_back(
    runtime: &MountedRuntime<RecordedMemoryDevice>,
    gate: &GateOfTheCallHeldBack,
    forced: &SegmentOfTheFirstFileForced,
    calls_held_back_before: u64,
) -> PassRewritingTheForcedSegment {
    let lowest_data_unit_index_in_the_segment = forced
        .data_units_in_slot_order
        .iter()
        .copied()
        .min()
        .expect("前提：强制选的那一段里有数据单元");
    std::thread::scope(|scope| {
        let pass_thread = scope.spawn(move || runtime.run_one_maintenance_pass());
        let mut unreleased_records_left_by_the_rewrite = None;
        if held_back_before_the_pass_ends(gate, calls_held_back_before + 1, &pass_thread) {
            gate.release_one_call();
            if held_back_before_the_pass_ends(gate, calls_held_back_before + 2, &pass_thread) {
                rewrite_the_first_file_from_the_data_unit_to_the_end(
                    runtime,
                    lowest_data_unit_index_in_the_segment,
                );
                unreleased_records_left_by_the_rewrite = Some(unreleased_records_in_the_segment(
                    runtime,
                    forced.segment_start_slot,
                ));
                gate.release_one_call();
            }
        }
        let is_held_back_after_the_rewrite = held_back_before_the_pass_ends(
            gate,
            calls_held_back_before + CALLS_HELD_BACK_IN_A_PASS_REWRITING_THE_FORCED_SEGMENT + 1,
            &pass_thread,
        );
        if is_held_back_after_the_rewrite {
            gate.open();
        }
        let pass = pass_thread
            .join()
            .expect("一拍那条线程不 panic")
            .expect("运行库没坏");
        PassRewritingTheForcedSegment {
            unreleased_records_left_by_the_rewrite,
            is_held_back_after_the_rewrite,
            pass,
        }
    })
}

/// 强制选段、批大小 1，推两拍，每一拍做完一条意图：签发那一批放行；之后有活意图的那一批在锁外时（替身挡着）部分写那一段里文件下标最小的那个数据单元，
/// 那一段里的记录都变了，回锁时剔掉计划里那 1 个、剔完一个不剩（`CompactionRunStop::BatchPrunedToNothingAtTheRelock`），这一拍接着叫、那一段已空，意图做完。
/// 拨钟、在现行那一版上另选一段，再推一拍照做。两次回锁各剔 1 个；观测 `units_pruned_at_the_relock` 把两次累加起来。
/// 两次剔分在两条意图里：部分写从偏移所在的单元起到文件末尾一起重写，同一条意图里让计划取的那一个变了，R 里文件下标比它大的也都跟着变了。
// 不切片并行：一个池，被测的是两拍锁外那一段与前台改动的先后，没有可切的区间。
#[test]
fn units_pruned_at_two_relocks_add_up_in_the_observations() {
    let batch_of_one_unit = Some(CompactionBatchUnits::new(1).expect("至少 1"));
    let (runtime, stand_in, receiver, segment_forced_at_the_mount) =
        mounted_with_the_segment_of_the_first_file_forced(batch_of_one_unit);
    let pass_over_the_segment_forced_at_the_mount =
        one_pass_rewriting_the_forced_segment_while_its_live_intent_batch_is_held_back(
            &runtime,
            &stand_in.gate,
            &segment_forced_at_the_mount,
            0,
        );
    let ends_after_the_pass_over_the_segment_forced_at_the_mount = receiver.ends_recorded();
    assert_eq!(
        pass_over_the_segment_forced_at_the_mount.unreleased_records_left_by_the_rewrite,
        Some(0),
        "前提：签发那一批与之后有活意图的那一批先后交到锁外，部分写之后强制选的那一段里没有未释放的记录：{ends_after_the_pass_over_the_segment_forced_at_the_mount:?}、{:?}",
        pass_over_the_segment_forced_at_the_mount.pass
    );
    assert!(
        !pass_over_the_segment_forced_at_the_mount.is_held_back_after_the_rewrite,
        "前提：那一段空了，这一拍之后不再分类那一段的记录：{ends_after_the_pass_over_the_segment_forced_at_the_mount:?}"
    );
    stand_in.clock.advance_by(ONE_PERIOD);
    let segment_forced_after_the_intent_is_done =
        force_the_segment_holding_only_data_units_of_the_first_file(&runtime, batch_of_one_unit);
    let pass_over_the_segment_forced_after_the_intent_is_done =
        one_pass_rewriting_the_forced_segment_while_its_live_intent_batch_is_held_back(
            &runtime,
            &stand_in.gate,
            &segment_forced_after_the_intent_is_done,
            CALLS_HELD_BACK_IN_A_PASS_REWRITING_THE_FORCED_SEGMENT,
        );
    let ends = receiver.ends_recorded();
    assert_eq!(
        pass_over_the_segment_forced_after_the_intent_is_done.unreleased_records_left_by_the_rewrite,
        Some(0),
        "前提：拨钟之后另选的那一段（起点 {}，挂载时选的起点 {}）签发那一批与之后有活意图的那一批先后交到锁外，部分写之后那一段里没有未释放的记录：{ends:?}、{:?}",
        segment_forced_after_the_intent_is_done.segment_start_slot,
        segment_forced_at_the_mount.segment_start_slot,
        pass_over_the_segment_forced_after_the_intent_is_done.pass
    );
    let units_pruned_at_each_relock: Vec<u64> = ends
        .iter()
        .filter_map(|end| {
            if let EndOfACompactionCall::Stopped(
                CompactionRunStop::BatchPrunedToNothingAtTheRelock { units_pruned },
            ) = end
            {
                Some(*units_pruned)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        units_pruned_at_each_relock,
        vec![1, 1],
        "前提：两拍各有一次回锁剔完一个不剩、各剔 1 个（批大小 1）：{ends:?}"
    );
    assert_eq!(
        runtime.observations().units_pruned_at_the_relock,
        1 + 1,
        "两次回锁各剔 1 个，观测累加成 2"
    );
}
