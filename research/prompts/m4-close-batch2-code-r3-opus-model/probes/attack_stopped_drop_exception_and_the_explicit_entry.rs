//! 攻方探针（代码轮 m4-close-batch2-code-r3）：整理因一批出错停着、现行那一版有活意图、回退门关着时，停着删意图那一趟的两格。
//!
//! 一、前台写不进的破例（D26（后台整理与放置回收） 已定项 13 第 5 条「只有全空聚簇段数低于底额、前台已写不进时破例，每趟至少发 1 次」）：
//!     后台入口停着那一趟带着破例那一档的预算起跑，产品代码不判门（`is_this_drop_the_exception`），门关着照发一次删意图。
//!     断言写产品此刻该有的样子（发 1 次、意图与 R 排他都撤了）；在「破例照样判门」的变异副本上这一格应当红。
//!     底额用只供测试的开关抬到每块盘都低于它（造「前台写不进」，不写盘）。
//! 二、显式入口停着、上一次用户改动给的那一份预算扣完、门关着：显式入口的预算 `3S − 4 − 已过发布数` 再取至少 1、不判门，
//!     照样发一次删意图；D26 已定项 13 第 4 条现行字面「那一趟吃发布预算，预算扣完就等下一次用户改动之后那一次调用再删」管的是显式入口与后台入口两个。
//!     后台入口同一刻不起跑（对照）。已过发布数直接写进会话（造「扣完」，不写盘）。
//!
//! 前缀照 `compaction_stopped_intent_drop_judges_the_rollback_gate.rs`：另一次挂载里顺序写第一个文件；重挂选 `gpu`（两张卡都点名、都用得上）；
//! 开关照水位判、越线靠只供测试的开关、选段强制、一批一个；建几个 inode；定时兜底叫起到停在回退门上；挂着切到一张卡都用不上的那一组。
//! 盘是内存稀疏盘：起点（顺序写之后）建一次，从内存里那一份拷。观测行以 `m4-close-batch2-r3-attack:` 起头。
mod common;
mod common_background_gpu_compute;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{build_pool, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, card_named, CpuBackedBackgroundGpuComputeForTests,
    EIGHT_GIBIBYTES,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeBackend, BackgroundComputeSettings, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, ClusterSegmentEntryKey,
    GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuLeftOutUnderItsMemoryLimit,
    GpuMemoryInBytes, RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome,
    UnitChecksumToVerifyOnTheGpu, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{
    foreground_writes_are_below_the_empty_cluster_segments_floor,
    publish_budget_of_a_compaction_run, publishes_left_of_the_budget_given_by_the_last_user_change,
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits, CompactionPacing,
    CompactionRunForcing, CompactionRunStop, CompactionSwitches, CompactionTimerFallback,
    CompactionWatermarkCrossingForcing, CompactionWriteBandwidthLimit, EmptyClusterSegmentsFloor,
    UnitAreaOfOneDevice, WatermarkQuantity,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::mount_writable_with_background_compute_settings;
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, CompactionRunOutcome, MountedSession, UserChange,
};
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 起点那一次顺序写的数据单元数（同回退门那两份用例）。
const DATA_UNITS_OF_THE_STARTING_POINT: usize = 40;
/// 定时兜底间隔。
const INTERVAL_IN_MILLISECONDS: u64 = 30_000;
/// 门关之前至多叫起几次。
const WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST: u64 = 8;
/// 内存稀疏盘的物理块宽（同 `common::crash_state_devices`）。
const PHYSICAL_BLOCK_BYTES: u32 = 512;
/// 「card-eight」答的那一张被晾下的卡（数同 `compaction_stopped_intent_drop_judges_the_rollback_gate.rs`）。
const MEMORY_LIMIT_HOLDING_NO_PIECE: GpuMemoryInBytes = GpuMemoryInBytes(64);
const SMALLEST_PIECE_BYTES: u64 = 972;
/// 只供测试的底额：比任何一块盘的段数都大，前台判成写不进。
const FLOOR_ABOVE_EVERY_DEVICE: u64 = 1 << 20;
/// 这几何每区根槽数 S（同回退门那两份用例）。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;

fn every_participating_gpu_left_out() -> BackgroundGpuComputeFailed {
    BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
        gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
            device: card_named("card-eight"),
            memory_limit: MEMORY_LIMIT_HOLDING_NO_PIECE,
            smallest_piece_bytes: SMALLEST_PIECE_BYTES,
            bytes_already_held_on_the_device: 0,
        }],
    }
}

/// 假实现外面包一层：交进来的最近那一组只点名「card-eight」时答那一组下一张卡都用不上，三段照旧交给里面那一份算。
struct GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
    computed_on_the_cpu: CpuBackedBackgroundGpuComputeForTests,
    settings_handed_in_last: Option<AdmittedGpuSettings>,
}

impl GpuInventory for GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.computed_on_the_cpu.devices_with_total_memory()
    }
}

impl BackgroundGpuCompute
    for GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes
{
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed> {
        self.computed_on_the_cpu.verify_unit_checksums(units)
    }

    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
        self.computed_on_the_cpu
            .classify_region_records(region, records, mapping_entries)
    }

    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
        self.computed_on_the_cpu
            .count_live_slots_per_cluster_segment(allocation_records, unit_area, segments)
    }

    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.settings_handed_in_last = Some(settings.clone());
        self.computed_on_the_cpu
            .use_the_settings_from_the_next_batch(settings);
    }

    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        let settings = self.settings_handed_in_last.as_ref()?;
        let names_only_the_card_with_eight_gibibytes = settings
            .participating_devices()
            .iter()
            .map(|participating| participating.device.clone())
            .eq(std::iter::once(card_named("card-eight")));
        names_only_the_card_with_eight_gibibytes.then(every_participating_gpu_left_out)
    }
}

fn gpu_compute_unusable_under_the_settings_naming_only_the_card_with_eight_gibibytes(
) -> BackgroundGpuComputeHandle {
    BackgroundGpuComputeHandle::of_a_shared_implementation(Arc::new(Mutex::new(
        GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
            computed_on_the_cpu: CpuBackedBackgroundGpuComputeForTests::with_nothing_injected(),
            settings_handed_in_last: None,
        },
    )))
}

fn settings_naming_only_the_card_with_eight_gibibytes() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_named("card-eight")]),
        background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

/// 后台入口的调用方：钟由探针拨，前台没有 fsync 在等。
struct CallerWithAClockMovedByTheProbe {
    clock_in_milliseconds: Cell<u64>,
}

impl CallerOfTheBackgroundCompaction for CallerWithAClockMovedByTheProbe {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::from_millis(self.clock_in_milliseconds.get()))
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

fn pacing_with_the_timer_fallback() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::EveryMilliseconds(
            NonZeroU64::new(INTERVAL_IN_MILLISECONDS).expect("不是 0"),
        ),
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

fn region_of_the_live_intent(session: &MountedSession) -> Option<CompactionRegion> {
    live_compaction_intent_of_the_entries(&current_of(session).logged_operations)
        .map(|(_, intent)| intent.region)
}

/// 起点：另一次挂载（CPU）里顺序写第一个文件之后丢掉会话，整条录制流施加到内存镜像上。
fn memory_pool_after_a_sequential_write(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let mut devices = pool.reopen_recorded();
    let mounted =
        singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content: Vec<u8> = (0..DATA_UNITS_OF_THE_STARTING_POINT * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("顺序写第一个文件");
    drop(session);
    pool.devices = Some(devices);
    pool.memory_pool()
}

fn devices_on(starting_point: &MemoryPool) -> Devices {
    starting_point
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

fn mounted_on_both_cards(devices: &mut Devices) -> MountedSession {
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(gpu_compute_unusable_under_the_settings_naming_only_the_card_with_eight_gibibytes()),
    )
    .expect("两张卡都点名、都用得上：挂载照常做成");
    let (_, session) = MountedSession::of_the_mount(mounted);
    session
}

fn fullest_data_segment_of(session: &MountedSession) -> CompactionRegion {
    let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
    for record in current_of(session)
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == DeviceIdentity(0) && !record.is_released && record.span_slots == 2
        })
    {
        *data_units_by_segment
            .entry(record.slot.0 / 64 * 64)
            .or_default() += 1;
    }
    let (segment, _data_units) = data_units_by_segment
        .into_iter()
        .max_by_key(|(_segment, data_units)| *data_units)
        .expect("写了数据单元");
    CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(segment))
        .expect("按 64 取整的段起点")
}

/// 开关照水位判（门在这一支判）、越线靠只供测试的开关；选段强制、一批一个；底额照产品值。
fn force_the_compaction_over_the_watermark(session: &mut MountedSession, region: CompactionRegion) {
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(1).expect("至少 1"),
            ),
            forced_region: Some(region),
            run_forcing: CompactionRunForcing::FollowTheWatermarks,
            ..CompactionSwitches::PRODUCT
        });
    session
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
}

fn create_one_inode(session: &mut MountedSession, devices: &mut Devices) {
    session
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("建一个 inode");
}

fn the_gate_closed_last(session: &MountedSession) -> bool {
    matches!(
        session.compaction.observations.last_stop,
        Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. })
    )
}

fn wake(
    session: &mut MountedSession,
    devices: &mut Devices,
    caller: &CallerWithAClockMovedByTheProbe,
) -> BackgroundCompactionCall {
    caller
        .clock_in_milliseconds
        .set(caller.clock_in_milliseconds.get() + INTERVAL_IN_MILLISECONDS);
    session
        .run_compaction_in_the_background(devices, pacing_with_the_timer_fallback(), caller)
        .expect("盘表是挂载的那一份")
}

/// 前缀：挂载、开关、建 `user_changes_before` 个 inode、定时兜底叫起到停在回退门上（那几趟都不出错），
/// 再挂着切到一张卡都用不上的那一组（整理从下一次叫起停着）。交回会话、调用方与强制选的 R。
fn prefix_up_to_the_closed_gate_and_the_stop(
    devices: &mut Devices,
    user_changes_before: usize,
) -> (
    MountedSession,
    CallerWithAClockMovedByTheProbe,
    CompactionRegion,
) {
    let mut session = mounted_on_both_cards(devices);
    assert_eq!(
        parameters().geometry.root_ring_slots_per_region.count(),
        ROOT_RING_SLOTS_PER_REGION,
        "前提：几何同回退门那两份用例（S = 8）"
    );
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, region);
    for _change in 0..user_changes_before {
        create_one_inode(&mut session, devices);
    }
    let caller = CallerWithAClockMovedByTheProbe {
        clock_in_milliseconds: Cell::new(0),
    };
    for _wake in 0..WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST {
        let call = wake(&mut session, devices, &caller);
        if let BackgroundCompactionCall::Ran { run, .. } = &call {
            assert!(
                !matches!(run.outcome, CompactionRunOutcome::Failed(_)),
                "前提：叫起到门关的那几趟不出错：{:?}",
                run.outcome
            );
        }
        if the_gate_closed_last(&session) {
            break;
        }
    }
    assert!(
        the_gate_closed_last(&session) && region_of_the_live_intent(&session) == Some(region),
        "前提：门前用户改动 {user_changes_before} 次之后叫起到停在回退门上，意图还活着、在强制选的那一段上"
    );
    session
        .switch_background_compute_from_the_next_batch(
            settings_naming_only_the_card_with_eight_gibibytes(),
            None,
        )
        .expect("只点名 card-eight 的一组照卡清单校验合法：不拒切换");
    (session, caller, region)
}

/// 一次后台入口调用的读数：起没起跑、触发、停在哪、发了几版。
fn reading_of_the_background_call(
    call: &BackgroundCompactionCall,
) -> (bool, String, String, usize) {
    match call {
        BackgroundCompactionCall::Ran { trigger, run } => (
            true,
            format!("{trigger:?}"),
            format!("{:?}", run.outcome),
            run.versions_published.len(),
        ),
        BackgroundCompactionCall::NotStarted { reason, .. } => {
            (false, "-".to_string(), format!("NotStarted({reason:?})"), 0)
        }
    }
}

/// 第一格：门关着、停着、有活意图。先照产品底额叫起一次（对照：停在门上、一版不发），再把底额抬到每块盘都低于它（前台写不进）叫起一次：
/// 破例那一档不判门，单发一次删意图，意图与 R 排他都撤了。交回破例那一次发了几版。
fn the_stopped_drop_while_the_foreground_cannot_write(user_changes_before: usize, tag: &str) {
    let starting_point = memory_pool_after_a_sequential_write(tag);
    let mut devices = devices_on(&starting_point);
    let (mut session, caller, region) =
        prefix_up_to_the_closed_gate_and_the_stop(&mut devices, user_changes_before);
    let below_before =
        foreground_writes_are_below_the_empty_cluster_segments_floor(&session.allocator);
    let contrast = wake(&mut session, &mut devices, &caller);
    let (contrast_started, contrast_trigger, contrast_outcome, contrast_published) =
        reading_of_the_background_call(&contrast);
    let live_intent_after_the_contrast = region_of_the_live_intent(&session) == Some(region);
    let switches = session.allocator.compaction().switches;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            empty_cluster_segments_floor: EmptyClusterSegmentsFloor::ForcedForTests(
                FLOOR_ABOVE_EVERY_DEVICE,
            ),
            ..switches
        });
    let below = foreground_writes_are_below_the_empty_cluster_segments_floor(&session.allocator);
    let exception = wake(&mut session, &mut devices, &caller);
    let (started, trigger, outcome, published) = reading_of_the_background_call(&exception);
    let live_intent_after = region_of_the_live_intent(&session).is_some();
    let region_excluded_after = session.allocator.region_excluded_from_placement().is_some();
    println!(
        "m4-close-batch2-r3-attack: stopped_drop_exception user_changes_before={user_changes_before} \
         contrast_below_the_floor={below_before} contrast_started={contrast_started} contrast_trigger={contrast_trigger} \
         contrast_outcome={contrast_outcome} contrast_publishes={contrast_published} live_intent_after_the_contrast={live_intent_after_the_contrast} \
         below_the_floor={below} started={started} trigger={trigger} outcome={outcome} publishes_of_the_wake={published} \
         live_intent_after={live_intent_after} region_excluded_after={region_excluded_after}"
    );
    assert!(
        !below_before && contrast_published == 0 && live_intent_after_the_contrast,
        "前提：照产品底额前台写得进，这一次叫起停在门上、一版不发、意图留着（{contrast_outcome}）"
    );
    assert!(below, "前提：底额抬上去之后前台判成写不进");
    assert!(
        started && published == 1 && !live_intent_after && !region_excluded_after,
        "前台写不进时停着删意图破例、不判门：应当单发一次删意图、意图与 R 排他都撤了；实际 started={started} outcome={outcome} 发了 {published} 版"
    );
}

#[test]
fn attack_the_stopped_drop_after_four_user_changes_passes_the_closed_gate_while_the_foreground_cannot_write(
) {
    the_stopped_drop_while_the_foreground_cannot_write(
        4,
        "attack-r3-stopped-drop-exception-after-four-user-changes",
    );
}

#[test]
fn attack_the_stopped_drop_after_six_user_changes_passes_the_closed_gate_while_the_foreground_cannot_write(
) {
    the_stopped_drop_while_the_foreground_cannot_write(
        6,
        "attack-r3-stopped-drop-exception-after-six-user-changes",
    );
}

#[test]
fn attack_the_stopped_drop_after_eight_user_changes_passes_the_closed_gate_while_the_foreground_cannot_write(
) {
    the_stopped_drop_while_the_foreground_cannot_write(
        8,
        "attack-r3-stopped-drop-exception-after-eight-user-changes",
    );
}

/// 第二格：门关着、停着、有活意图，上一次用户改动给的那一份预算扣完（已过发布数直接写成 `3S − 4`）。
/// 后台入口不起跑（对照，D26 已定项 13 第 4 条「预算扣完就等」）；同一刻显式入口照发一次删意图（预算取至少 1、不判门）。
#[test]
fn attack_the_explicit_entry_left_stopped_drops_the_intent_with_the_budget_spent_and_the_gate_closed(
) {
    let starting_point = memory_pool_after_a_sequential_write(
        "attack-r3-explicit-entry-left-stopped-with-the-budget-spent",
    );
    let mut devices = devices_on(&starting_point);
    let (mut session, caller, region) = prefix_up_to_the_closed_gate_and_the_stop(&mut devices, 4);
    let whole_budget = publish_budget_of_a_compaction_run(ROOT_RING_SLOTS_PER_REGION, 0);
    session.compaction.publishes_since_the_last_user_change = session
        .compaction
        .publishes_since_the_last_user_change
        .max(whole_budget);
    let left = publishes_left_of_the_budget_given_by_the_last_user_change(
        ROOT_RING_SLOTS_PER_REGION,
        session.compaction.publishes_since_the_last_user_change,
    );
    let background = wake(&mut session, &mut devices, &caller);
    let (background_started, background_trigger, background_outcome, background_published) =
        reading_of_the_background_call(&background);
    let live_intent_after_the_background_call = region_of_the_live_intent(&session) == Some(region);
    let explicit = session
        .run_compaction(&mut devices)
        .expect("盘表是挂载的那一份、现行那一版有树");
    let explicit_published = explicit.versions_published.len();
    let live_intent_after_the_explicit_entry = region_of_the_live_intent(&session).is_some();
    println!(
        "m4-close-batch2-r3-attack: explicit_entry_with_the_budget_spent budget_left={left} \
         background_started={background_started} background_trigger={background_trigger} background_outcome={background_outcome} \
         background_publishes={background_published} live_intent_after_the_background_call={live_intent_after_the_background_call} \
         explicit_outcome={:?} explicit_publishes={explicit_published} live_intent_after_the_explicit_entry={live_intent_after_the_explicit_entry}",
        explicit.outcome
    );
    assert!(
        left == 0
            && !background_started
            && background_published == 0
            && live_intent_after_the_background_call,
        "对照：预算扣完、前台写得进，后台入口不起跑、意图留着（{background_outcome}）"
    );
    assert!(
        explicit_published == 1
            && !live_intent_after_the_explicit_entry
            && matches!(
                explicit.outcome,
                CompactionRunOutcome::Stopped(
                    CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
                )
            ),
        "显式入口停着、预算扣完、门关着：照发一次删意图（与已定项 13 第 4 条字面「预算扣完就等」对不上）；实际 {:?} 发了 {explicit_published} 版",
        explicit.outcome
    );
}
