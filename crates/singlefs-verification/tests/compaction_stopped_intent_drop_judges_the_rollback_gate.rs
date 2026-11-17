//! 整理因一批出错停着时后台入口单发的删意图也是一次整理发布（D26（后台整理与放置回收） 已定项 13 第 4、5 条）：开关照水位判时发之前过回退门，
//! 门关着就不发、一个写都不发，意图与 R 排他留着；之后每有一次用户改动叫起一次，门开了的那一次才删。后台入口头一次判起跑从根环读挂载那一份预算，
//! 排在停没停那一判之前：刚挂上就停着时，那一次调用同样读到根环里最新那个用户状态之后已过去的发布数。
//! 全空聚簇段数低于底额、前台写不进时破例（已定项 13 第 5 条「只有全空聚簇段数低于底额（已定项 3）、前台已写不进时破例，每趟至少发 1 次」）：
//! 后台入口停着那一趟带破例那一档的预算起跑，删意图那一次不过回退门，门关着照发。
//! 显式入口停着时预算取至少 1、不判回退门：上一次用户改动给的那一份扣完、门关着，照样单发一次删意图（后台入口同一刻不起跑）。
//!
//! 历史（攻方探针 `research/prompts/m4-close-batch2-code-r2-opus-s2-model/probes/attack_ab_stopped_drop_judges_the_rollback_gate.rs` 的造法）：
//! 另一次挂载里顺序写第一个文件；重挂选 `gpu`（两张卡都点名、都用得上）；开关照水位判、越线靠只供测试的开关、选段强制、一批一个；
//! 建几个 inode；后台入口定时兜底叫起到停在回退门上（意图还活着、R 排他）；挂着切到一张卡都用不上的那一组，整理从下一次叫起停着。
//! 盘是内存稀疏盘：起点（顺序写之后）建一次，从内存里那一份拷。
//! 门开没开另按用例自己的读法判，不经产品的门：根环里读得出、不低于 F 的根按用户改动分状态（同 `compaction_rollback_gate_keeps_four_user_states.rs`）。
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
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
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
    BackgroundCompactionCall, BackgroundCompactionNotStarted, BackgroundCompactionTrigger,
    CompactionRun, CompactionRunOutcome, MountedSession, UserChange,
};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::ROOT_RING_REGIONS;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 起点那一次顺序写的数据单元数（同 `compaction_rollback_gate_keeps_four_user_states.rs`）。
const DATA_UNITS_OF_THE_STARTING_POINT: usize = 40;
/// 回退门保住的不同用户状态数（D16（发布语义） 已定项 1）。
const ROLLBACK_STATES_KEPT: usize = 4;
/// 回退门给根环留的余量：这几何每区根槽数 S = 8，余量取 S 槽（D26（后台整理与放置回收） 已定项 13 第 5 条）。
const MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS: u64 = 8;
/// 定时兜底间隔。
const INTERVAL_IN_MILLISECONDS: u64 = 30_000;
/// 门关之前至多叫起几次。
const WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST: u64 = 8;
/// 门关之后至多再建几个 inode（每建一个叫起一次）等门开。
const USER_CHANGES_AFTER_THE_CLOSURE_AT_MOST: usize = 8;
/// 内存稀疏盘的物理块宽（同 `common::crash_state_devices`）。
const PHYSICAL_BLOCK_BYTES: u32 = 512;
/// 「card-eight」答的那一张被晾下的卡：给它的上限与最小的一片（数是用例自己定的，同
/// `compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs`）。
const MEMORY_LIMIT_HOLDING_NO_PIECE: GpuMemoryInBytes = GpuMemoryInBytes(64);
const SMALLEST_PIECE_BYTES: GpuMemoryInBytes = GpuMemoryInBytes(972);
/// 只供测试的全空聚簇段底额：比任何一块盘的段数都大，每块盘都低于它，前台判成写不进（不写盘）。
const FLOOR_ABOVE_EVERY_DEVICE: u64 = 1 << 20;

fn every_participating_gpu_left_out() -> BackgroundGpuComputeFailed {
    BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
        gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
            device: card_named("card-eight"),
            memory_limit: MEMORY_LIMIT_HOLDING_NO_PIECE,
            smallest_piece_bytes: SMALLEST_PIECE_BYTES,
            bytes_already_held_on_the_device: GpuMemoryInBytes(0),
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

/// 只点名「card-eight」、上限是它的显存总量的一组 `gpu` 设置（校验照收；包着的那一层答它一张卡都用不上）。
fn settings_naming_only_the_card_with_eight_gibibytes() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_named("card-eight")]),
        background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

/// 后台入口的调用方：钟由用例拨，前台没有 fsync 在等。
struct CallerWithAClockMovedByTheTest {
    clock_in_milliseconds: Cell<u64>,
}

impl CallerOfTheBackgroundCompaction for CallerWithAClockMovedByTheTest {
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

/// 现行那一版 logged_ops 里那条活意图的 R；没有活意图交回 `None`。
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

/// 起点那一份内存镜像拷成两块内存稀疏盘。
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

/// 每块盘此刻的内容（逐字节比用）。
fn images_of(devices: &Devices) -> Vec<SparseDevice> {
    devices
        .iter()
        .map(|(_identity, device)| device.image.clone())
        .collect()
}

/// 可写挂载选 `gpu`、两张卡都点名、都用得上。
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

/// 开关照水位判（门在这一支判）、越线靠只供测试的开关；选段强制、一批一个（同回退门那份用例的造法）。
fn force_the_compaction_over_the_watermark(session: &mut MountedSession, region: CompactionRegion) {
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing: _,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(1).expect("至少 1"),
            ),
            forced_region: Some(region),
            run_forcing: CompactionRunForcing::FollowTheWatermarks,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    session
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
}

fn create_one_inode(session: &mut MountedSession, devices: &mut Devices) -> CheckpointTxg {
    session
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("建一个 inode");
    current_of(session).root.checkpoint_txg
}

/// 根环里读得出、不低于 F、txg 不低于 `oldest_root_kept` 的根属于几个用户状态（同回退门那份用例的读法，不经产品的门）。
fn user_states_among_the_roots_from(
    devices: &Devices,
    txgs_of_the_user_changes: &[CheckpointTxg],
    oldest_root_kept: CheckpointTxg,
) -> usize {
    let pool_parameters = parameters();
    let roots = readable_roots(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    let floor = effective_rollback_floor(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    txgs_of_the_user_changes
        .iter()
        .enumerate()
        .filter(|(index, txg)| {
            let next = txgs_of_the_user_changes.get(index + 1);
            roots.iter().any(|root| {
                root.checkpoint_txg >= floor
                    && root.checkpoint_txg >= oldest_root_kept
                    && root.checkpoint_txg >= **txg
                    && next.is_none_or(|next_txg| root.checkpoint_txg < *next_txg)
            })
        })
        .count()
}

/// 用例自己的读法认为回退门此刻拒下一次整理发布：从现行那一版起再发一次、连同余量的根一走（txg 为 x 的根由 x 加根环槽数那一次发布盖掉），
/// 根环里不同用户状态少于 4 个、且比此刻少。
fn the_gate_refuses_the_next_publish_by_the_test(
    devices: &Devices,
    txgs_of_the_user_changes: &[CheckpointTxg],
    current: CheckpointTxg,
) -> bool {
    let root_ring_slots =
        ROOT_RING_REGIONS * parameters().geometry.root_ring_slots_per_region.count();
    let states_now =
        user_states_among_the_roots_from(devices, txgs_of_the_user_changes, CheckpointTxg(0));
    let states_left_past_the_next_publish_and_the_margin = user_states_among_the_roots_from(
        devices,
        txgs_of_the_user_changes,
        CheckpointTxg(
            (current.0 + MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS + 2).saturating_sub(root_ring_slots),
        ),
    );
    states_left_past_the_next_publish_and_the_margin < ROLLBACK_STATES_KEPT
        && states_left_past_the_next_publish_and_the_margin < states_now
}

/// 根环里读得出、不低于 F、txg 比 `txg` 新的根有几条（用例自己数）。
fn roots_newer_than(devices: &Devices, txg: CheckpointTxg) -> usize {
    let pool_parameters = parameters();
    let roots = readable_roots(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    let floor = effective_rollback_floor(
        devices,
        &pool_parameters.region_devices,
        &pool_parameters.geometry,
        &pool_parameters.filesystem_identifier,
    );
    roots
        .iter()
        .filter(|root| root.checkpoint_txg >= floor && root.checkpoint_txg > txg)
        .count()
}

/// 上一趟停在回退门上（不发布的那一种停）。
fn the_gate_closed_last(session: &MountedSession) -> bool {
    matches!(
        session.compaction.observations.last_stop,
        Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. })
    )
}

/// 定时兜底往后拨一个间隔，叫起一次后台入口。
fn wake(
    session: &mut MountedSession,
    devices: &mut Devices,
    caller: &CallerWithAClockMovedByTheTest,
) -> BackgroundCompactionCall {
    caller
        .clock_in_milliseconds
        .set(caller.clock_in_milliseconds.get() + INTERVAL_IN_MILLISECONDS);
    session
        .run_compaction_in_the_background(devices, pacing_with_the_timer_fallback(), caller)
        .expect("盘表是挂载的那一份")
}

/// 门前的那一段：挂载、开关、建 `user_changes_before` 个 inode、定时兜底叫起到停在回退门上（那几趟都不出错）。
/// 交回会话、各次用户改动的 txg、调用方与强制选的 R。
fn prefix_up_to_the_closed_gate(
    devices: &mut Devices,
    user_changes_before: usize,
) -> (
    MountedSession,
    Vec<CheckpointTxg>,
    CallerWithAClockMovedByTheTest,
    CompactionRegion,
) {
    let mut session = mounted_on_both_cards(devices);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, region);
    let mut txgs = Vec::new();
    for _change in 0..user_changes_before {
        txgs.push(create_one_inode(&mut session, devices));
    }
    let caller = CallerWithAClockMovedByTheTest {
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
    (session, txgs, caller, region)
}

/// 门前的那一段（[`prefix_up_to_the_closed_gate`]）推到停在回退门上、意图还活着且在强制选的那一段上之后，挂着切到一张卡都用不上的那一组：
/// 整理从下一次叫起停着。交回同 [`prefix_up_to_the_closed_gate`]。
fn prefix_up_to_the_closed_gate_and_the_stop(
    devices: &mut Devices,
    user_changes_before: usize,
) -> (
    MountedSession,
    Vec<CheckpointTxg>,
    CallerWithAClockMovedByTheTest,
    CompactionRegion,
) {
    let (mut session, txgs, caller, region) =
        prefix_up_to_the_closed_gate(devices, user_changes_before);
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
    (session, txgs, caller, region)
}

/// 全空聚簇段底额抬到 [`FLOOR_ABOVE_EVERY_DEVICE`]，别的开关照此刻的：每块盘都低于底额，前台判成写不进（不写盘）。
fn raise_the_empty_cluster_segments_floor_above_every_device(session: &mut MountedSession) {
    let CompactionSwitches {
        batch_size,
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: _,
        relocation_publish_order,
    } = session.allocator.compaction().switches;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size,
            forced_region,
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor: EmptyClusterSegmentsFloor::ForcedForTests(
                FLOOR_ABOVE_EVERY_DEVICE,
            ),
            relocation_publish_order,
        });
}

/// `run` 这一趟单发了一次只删意图的发布、之后停在出错停着、一批都不搬：删意图那一版不搬单元、logged_ops 里没有整理意图，
/// 是现行那一版，分配器这一侧意图连同 R 排他都撤了。`situation` 写进失败信息。
fn assert_the_run_dropped_the_intent_once_and_stopped(
    session: &MountedSession,
    run: &CompactionRun,
    situation: &str,
) {
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0,
        "{situation}：单发一次删意图之后停在出错停着、一批都不搬：{:?}，搬了 {} 批",
        run.outcome,
        run.batches_published
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "{situation}：恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && current_of(session) == dropped
            && session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "{situation}：删意图那一版不搬单元、logged_ops 里没有整理意图，分配器这一侧意图连同 R 排他都撤了"
    );
}

/// 门前用户改动 `user_changes_before` 次的那一段历史：停在回退门上、整理停着之后，先照产品底额叫起一次——前台写得进，
/// 停着删意图那一趟停在回退门上、一版都不发、意图留着；再把底额抬到每块盘都低于它（前台写不进）叫起一次——用例的读法下门仍关着，
/// 起跑的仍是删停着时的活意图那一趟，预算取破例那一档，删意图那一次不过回退门：门关着照发一次，意图与 R 排他都撤了。
fn the_stopped_drop_passes_the_closed_gate_while_the_foreground_cannot_write(
    user_changes_before: usize,
    tag: &str,
) {
    let starting_point = memory_pool_after_a_sequential_write(tag);
    let mut devices = devices_on(&starting_point);
    let (mut session, txgs, caller, region) =
        prefix_up_to_the_closed_gate_and_the_stop(&mut devices, user_changes_before);
    assert!(
        !foreground_writes_are_below_the_empty_cluster_segments_floor(&session.allocator),
        "前提：照产品底额前台写得进"
    );
    let held_at_the_gate = wake(&mut session, &mut devices, &caller);
    let BackgroundCompactionCall::Ran {
        trigger: trigger_at_the_closed_gate,
        run: run_at_the_closed_gate,
    } = &held_at_the_gate
    else {
        panic!("前提：停着而有活意图、预算还剩，后台入口起跑删意图那一趟：{held_at_the_gate:?}");
    };
    assert!(
        *trigger_at_the_closed_gate
            == BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError
            && matches!(
                run_at_the_closed_gate.outcome,
                CompactionRunOutcome::Stopped(
                    CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
                )
            )
            && run_at_the_closed_gate.versions_published.is_empty()
            && region_of_the_live_intent(&session) == Some(region),
        "前提：前台写得进时停着删意图那一趟停在回退门上、一版都不发、意图留着：{:?}",
        run_at_the_closed_gate.outcome
    );
    assert!(
        the_gate_refuses_the_next_publish_by_the_test(
            &devices,
            &txgs,
            current_of(&session).root.checkpoint_txg
        ),
        "前提：用例自己的读法下门仍关着"
    );
    raise_the_empty_cluster_segments_floor_above_every_device(&mut session);
    assert!(
        foreground_writes_are_below_the_empty_cluster_segments_floor(&session.allocator),
        "前提：底额抬上去之后前台判成写不进"
    );
    let through_the_closed_gate = wake(&mut session, &mut devices, &caller);
    let BackgroundCompactionCall::Ran { trigger, run } = &through_the_closed_gate else {
        panic!("前台写不进、停着而有活意图：后台入口起跑删意图那一趟：{through_the_closed_gate:?}");
    };
    assert_eq!(
        *trigger,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "前台写不进时起跑的仍是删停着时的活意图那一趟"
    );
    assert_the_run_dropped_the_intent_once_and_stopped(
        &session,
        run,
        "前台写不进、门关着：破例那一档不过回退门",
    );
}

/// 门前用户改动 `user_changes_before` 次的那一段历史：停在回退门上之后切到一张卡都用不上的那一组（整理停着），叫起——
/// 起跑的是停着删意图那一趟，停在回退门上，一版都不发、盘上逐字节不变，意图与 R 排他都留着；之后每建一个 inode 叫起一次：
/// 用例的读法判门还关着的那几次照旧停在门上、一版都不发，判门开了的那一次单发一次删意图，意图与 R 排他都撤了。
fn the_stopped_drop_holds_at_the_closed_gate_and_drops_once_it_opens(
    user_changes_before: usize,
    tag: &str,
) {
    let starting_point = memory_pool_after_a_sequential_write(tag);
    let mut devices = devices_on(&starting_point);
    let (mut session, mut txgs, caller, region) =
        prefix_up_to_the_closed_gate_and_the_stop(&mut devices, user_changes_before);
    assert!(
        the_gate_refuses_the_next_publish_by_the_test(
            &devices,
            &txgs,
            current_of(&session).root.checkpoint_txg
        ),
        "前提：用例自己的读法下门关着"
    );
    let images_before_the_wake = images_of(&devices);
    let held_at_the_gate = wake(&mut session, &mut devices, &caller);
    let BackgroundCompactionCall::Ran {
        trigger: trigger_at_the_closed_gate,
        run: run_at_the_closed_gate,
    } = &held_at_the_gate
    else {
        panic!("停着而有活意图、预算还剩：后台入口起跑删意图那一趟：{held_at_the_gate:?}");
    };
    assert_eq!(
        *trigger_at_the_closed_gate,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "起跑是为了删停着时的活意图"
    );
    assert!(
        matches!(
            run_at_the_closed_gate.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
            )
        ) && run_at_the_closed_gate.versions_published.is_empty(),
        "门关着：删意图那一次也是整理发布，停在回退门上、一版都不发：{:?}，发了 {} 版",
        run_at_the_closed_gate.outcome,
        run_at_the_closed_gate.versions_published.len()
    );
    assert!(
        region_of_the_live_intent(&session) == Some(region)
            && session.allocator.region_excluded_from_placement() == Some(region),
        "门关着：意图与 R 排他都留着"
    );
    assert_eq!(
        images_of(&devices),
        images_before_the_wake,
        "门关着：盘上逐字节不变"
    );
    for user_changes_after_the_closure in 1..=USER_CHANGES_AFTER_THE_CLOSURE_AT_MOST {
        txgs.push(create_one_inode(&mut session, &mut devices));
        let gate_refuses_by_the_test = the_gate_refuses_the_next_publish_by_the_test(
            &devices,
            &txgs,
            current_of(&session).root.checkpoint_txg,
        );
        let images_before_the_next_wake = images_of(&devices);
        let after_a_user_change = wake(&mut session, &mut devices, &caller);
        let BackgroundCompactionCall::Ran { trigger, run } = &after_a_user_change else {
            panic!(
                "门关之后第 {user_changes_after_the_closure} 次用户改动给了新的一份预算、意图还活着：后台入口起跑：{after_a_user_change:?}"
            );
        };
        assert_eq!(
            *trigger,
            BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
            "门关之后第 {user_changes_after_the_closure} 次用户改动之后起跑的仍是删停着时的活意图那一趟"
        );
        if gate_refuses_by_the_test {
            assert!(
                matches!(
                    run.outcome,
                    CompactionRunOutcome::Stopped(
                        CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. }
                    )
                ) && run.versions_published.is_empty()
                    && region_of_the_live_intent(&session) == Some(region),
                "门关之后第 {user_changes_after_the_closure} 次用户改动之后门仍关着：停在门上、一版都不发、意图留着：{:?}",
                run.outcome
            );
            assert_eq!(
                images_of(&devices),
                images_before_the_next_wake,
                "门仍关着：这一次叫起盘上逐字节不变"
            );
            continue;
        }
        assert!(
            matches!(
                run.outcome,
                CompactionRunOutcome::Stopped(
                    CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
                )
            ) && run.batches_published == 0,
            "门开了：单发一次删意图之后停在出错停着，一批都不搬：{:?}",
            run.outcome
        );
        let [dropped] = run.versions_published.as_slice() else {
            panic!(
                "门开了的那一次叫起恰好发一次只删意图的发布，实际发了 {} 次",
                run.versions_published.len()
            );
        };
        assert!(
            dropped.units_relocated_by_this_publish.is_empty()
                && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
                && current_of(&session) == dropped
                && session.allocator.compaction().live_intent().is_none()
                && session.allocator.region_excluded_from_placement().is_none(),
            "门开了：删意图那一版不搬单元、logged_ops 里没有整理意图，分配器这一侧意图连同 R 排他都撤了"
        );
        return;
    }
    panic!(
        "门关之后建了 {USER_CHANGES_AFTER_THE_CLOSURE_AT_MOST} 个 inode，用例的读法下门一直没开（造法要改，不是读数）"
    );
}

/// 门前用户改动 4 次：停着删意图那一趟在关着的回退门上不发，门开了才删。
/// 判别力：`crates/mutations.tsv`「停着删意图之前不判回退门」那一行。
#[test]
fn the_stopped_drop_after_four_user_changes_holds_at_the_closed_gate_and_drops_once_it_opens() {
    the_stopped_drop_holds_at_the_closed_gate_and_drops_once_it_opens(
        4,
        "compaction-stopped-drop-at-the-closed-gate-after-four-user-changes",
    );
}

/// 门前用户改动 6 次：同上。
#[test]
fn the_stopped_drop_after_six_user_changes_holds_at_the_closed_gate_and_drops_once_it_opens() {
    the_stopped_drop_holds_at_the_closed_gate_and_drops_once_it_opens(
        6,
        "compaction-stopped-drop-at-the-closed-gate-after-six-user-changes",
    );
}

/// 门前用户改动 8 次：同上。
#[test]
fn the_stopped_drop_after_eight_user_changes_holds_at_the_closed_gate_and_drops_once_it_opens() {
    the_stopped_drop_holds_at_the_closed_gate_and_drops_once_it_opens(
        8,
        "compaction-stopped-drop-at-the-closed-gate-after-eight-user-changes",
    );
}

/// 门前用户改动 4 次：前台写不进时停着删意图那一趟不过回退门，门关着照发一次删意图。
/// 判别力：`crates/mutations.tsv`「停着删意图的破例那一档也判回退门」那一行。
#[test]
fn the_stopped_drop_after_four_user_changes_passes_the_closed_gate_while_the_foreground_cannot_write(
) {
    the_stopped_drop_passes_the_closed_gate_while_the_foreground_cannot_write(
        4,
        "compaction-stopped-drop-through-the-closed-gate-after-four-user-changes",
    );
}

/// 门前用户改动 6 次：同上。
/// 判别力：`crates/mutations.tsv`「停着删意图那一趟起跑时不给破例那一档的预算」那一行。
#[test]
fn the_stopped_drop_after_six_user_changes_passes_the_closed_gate_while_the_foreground_cannot_write(
) {
    the_stopped_drop_passes_the_closed_gate_while_the_foreground_cannot_write(
        6,
        "compaction-stopped-drop-through-the-closed-gate-after-six-user-changes",
    );
}

/// 门前用户改动 8 次：同上。
/// 判别力：`crates/mutations.tsv`「停着删意图的破例认错那一档预算」那一行。
#[test]
fn the_stopped_drop_after_eight_user_changes_passes_the_closed_gate_while_the_foreground_cannot_write(
) {
    the_stopped_drop_passes_the_closed_gate_while_the_foreground_cannot_write(
        8,
        "compaction-stopped-drop-through-the-closed-gate-after-eight-user-changes",
    );
}

/// 门前用户改动 4 次、停在回退门上、整理停着，上一次用户改动给的那一份预算扣完（会话记的已过发布数直接写成 `3S − 4`，不写盘）：
/// 后台入口不起跑、意图留着（前台写得进，那一份扣完就等）；同一刻显式入口预算取至少 1、不判回退门，单发一次删意图，意图与 R 排他都撤了。
/// 判别力：`crates/mutations.tsv`「回退门在显式入口也判」那一行。
#[test]
fn the_explicit_entry_left_stopped_drops_the_intent_with_the_budget_spent_and_the_gate_closed() {
    let starting_point = memory_pool_after_a_sequential_write(
        "compaction-explicit-entry-left-stopped-drops-the-intent-with-the-budget-spent",
    );
    let mut devices = devices_on(&starting_point);
    let (mut session, txgs, caller, region) =
        prefix_up_to_the_closed_gate_and_the_stop(&mut devices, 4);
    let root_ring_slots_per_region = parameters().geometry.root_ring_slots_per_region.count();
    session.compaction.publishes_since_the_last_user_change =
        session.compaction.publishes_since_the_last_user_change.max(
            publish_budget_of_a_compaction_run(root_ring_slots_per_region, 0),
        );
    assert_eq!(
        publishes_left_of_the_budget_given_by_the_last_user_change(
            root_ring_slots_per_region,
            session.compaction.publishes_since_the_last_user_change,
        ),
        0,
        "前提：上一次用户改动给的那一份预算扣完"
    );
    assert!(
        !foreground_writes_are_below_the_empty_cluster_segments_floor(&session.allocator),
        "前提：照产品底额前台写得进"
    );
    let background_call = wake(&mut session, &mut devices, &caller);
    assert!(
        matches!(
            background_call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ) && region_of_the_live_intent(&session) == Some(region),
        "前提：预算扣完、前台写得进，后台入口不起跑、意图留着：{background_call:?}"
    );
    assert!(
        the_gate_refuses_the_next_publish_by_the_test(
            &devices,
            &txgs,
            current_of(&session).root.checkpoint_txg
        ),
        "前提：用例自己的读法下门关着"
    );
    let explicit_run = session
        .run_compaction(&mut devices)
        .expect("盘表是挂载的那一份、现行那一版有树");
    assert_the_run_dropped_the_intent_once_and_stopped(
        &session,
        &explicit_run,
        "显式入口停着、预算扣完、门关着",
    );
}

/// 上一次挂载推到停在回退门上时丢掉会话；重挂（两张卡都用得上，挂载续做照常），切到一张卡都用不上的那一组，叫起一次后台入口：
/// 这一次停着、不起跑，挂载那一份预算照样在这一次从根环读——会话记的已过发布数取根环里最新那个用户状态之后的根数（用例自己数），
/// 不停在挂载时记的那个数上。
/// 判别力：`crates/mutations.tsv`「停没停那一判排回读挂载预算之前」那一行。
#[test]
fn the_first_background_call_left_stopped_after_a_mount_reads_the_publishes_of_the_mount_from_the_root_ring(
) {
    let starting_point = memory_pool_after_a_sequential_write(
        "compaction-first-background-call-left-stopped-reads-the-publishes-of-the-mount",
    );
    let mut devices = devices_on(&starting_point);
    let (session_of_the_earlier_mount, txgs, _caller_of_the_earlier_mount, _region) =
        prefix_up_to_the_closed_gate(&mut devices, 4);
    assert!(
        the_gate_closed_last(&session_of_the_earlier_mount),
        "前提：上一次挂载叫起到停在回退门上"
    );
    drop(session_of_the_earlier_mount);
    let mut session = mounted_on_both_cards(&mut devices);
    let publishes_counted_at_the_mount = session.compaction.publishes_since_the_last_user_change;
    let roots_after_the_last_user_change = u64::try_from(roots_newer_than(
        &devices,
        *txgs.last().expect("建了 inode"),
    ))
    .expect("根数装得进 u64");
    assert!(
        roots_after_the_last_user_change > publishes_counted_at_the_mount,
        "前提：根环里最新那个用户状态之后的根（{roots_after_the_last_user_change}）比挂载时会话记的已过发布数（{publishes_counted_at_the_mount}）多"
    );
    session
        .switch_background_compute_from_the_next_batch(
            settings_naming_only_the_card_with_eight_gibibytes(),
            None,
        )
        .expect("只点名 card-eight 的一组照卡清单校验合法：不拒切换");
    let caller = CallerWithAClockMovedByTheTest {
        clock_in_milliseconds: Cell::new(0),
    };
    let first_call = wake(&mut session, &mut devices, &caller);
    assert!(
        matches!(
            first_call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "前提：重挂之后整理停着、现行那一版没有活意图，这一次不起跑：{first_call:?}"
    );
    assert_eq!(
        session.compaction.publishes_since_the_last_user_change,
        roots_after_the_last_user_change,
        "停着的头一次后台调用照样从根环读挂载那一份预算：已过发布数取根环里最新那个用户状态之后的根数"
    );
}
