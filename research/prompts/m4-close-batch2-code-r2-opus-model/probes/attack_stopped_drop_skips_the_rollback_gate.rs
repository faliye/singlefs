//! 攻方探针（代码轮 m4-close-batch2-code-r2，攻击面二：A1 的修补本身）：后台入口判出整理停着而有活意图时起跑的那一趟
//! （`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`）在一趟开头单发删意图，排在回退门之前，门不判。
//! 历史：另一次挂载里顺序写第一个文件；重挂选 `gpu`（两张卡都用得上）；开关照水位判（门在这一支判）、越线靠只供测试的开关、选段强制、一批一个；
//! 建 `user_changes_before` 个 inode；后台入口定时兜底叫起到门关（门关时意图还活着、R 排他）。之后三臂：
//! 切到一张卡都用不上的那一组再叫起一次；切换之后先建一个 inode 再叫起一次；不切换再叫起一次（对照）。
//! 判门的读法独立于产品：根环里读得出、不低于 F 的根按用户改动分状态（同 `compaction_rollback_gate_keeps_four_user_states.rs`）。
//! 观测行以 `m4-close-batch2-r2-attack:` 起头。断言写在打中那一边：绿 = 打中（门关着的那一版上，停着的那一趟照发了一次删意图）。
mod common;
mod common_background_gpu_compute;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
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
use singlefs_core::compaction_plan::{
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits, CompactionPacing,
    CompactionRunForcing, CompactionRunStop, CompactionSwitches, CompactionTimerFallback,
    CompactionWatermarkCrossingForcing, CompactionWriteBandwidthLimit, UnitAreaOfOneDevice,
    WatermarkQuantity,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::mount_writable_with_background_compute_settings;
use singlefs_core::mounted_session::{BackgroundCompactionCall, MountedSession, UserChange};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::ROOT_RING_REGIONS;

type Devices = Vec<(DeviceIdentity, Recorded)>;

const PREFIX: &str = "m4-close-batch2-r2-attack:";
/// 起点那一次顺序写的数据单元数（同 `compaction_rollback_gate_keeps_four_user_states.rs`）。
const DATA_UNITS_OF_THE_STARTING_POINT: usize = 40;
/// 回退门保住的状态数。
const ROLLBACK_STATES_KEPT: usize = 4;
/// 回退门给根环留的余量：这几何 S = 8，余量取 S 槽（D26 已定项 13 第 5 条）。
const MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS: u64 = 8;
/// 定时兜底间隔。
const INTERVAL_IN_MILLISECONDS: u64 = 30_000;
/// 门关之前至多叫起几次。
const WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST: u64 = 8;
/// 「card-eight」答的那一张被晾下的卡（同 `compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs`）。
const MEMORY_LIMIT_HOLDING_NO_PIECE: GpuMemoryInBytes = GpuMemoryInBytes(64);
const SMALLEST_PIECE_BYTES: u64 = 972;

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

/// 只点名「card-eight」、上限是它的显存总量的一组 `gpu` 设置（校验照收；包着的那一层答它一张卡都用不上）。
fn settings_naming_only_the_card_with_eight_gibibytes() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_named("card-eight")]),
        background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

struct CallerOfTheHistories {
    clock_in_milliseconds: Cell<u64>,
}

impl CallerOfTheBackgroundCompaction for CallerOfTheHistories {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::from_millis(self.clock_in_milliseconds.get()))
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

fn pacing_of_the_histories() -> CompactionPacing {
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

/// 顺序写之后丢掉会话（CPU 挂载）：段都差不多是满的，数据单元最多的那一段一批一个搬得久。
fn pool_after_a_sequential_write(tag: &str) -> BuiltPool {
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
    pool
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

/// 从 `current` 起再发 `publishes_ahead` 次发布之后还留在根环里的根里有几个用户状态：txg 为 x 的根由 x 加根环槽数那一次发布盖掉。
fn user_states_left_after_the_next_publishes(
    devices: &Devices,
    txgs_of_the_user_changes: &[CheckpointTxg],
    current: CheckpointTxg,
    publishes_ahead: u64,
) -> usize {
    let root_ring_slots =
        ROOT_RING_REGIONS * parameters().geometry.root_ring_slots_per_region.count();
    user_states_among_the_roots_from(
        devices,
        txgs_of_the_user_changes,
        CheckpointTxg((current.0 + publishes_ahead + 1).saturating_sub(root_ring_slots)),
    )
}

/// 门关之后用户（与调用方）做什么。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AfterTheGateCloses {
    /// 挂着切到一张卡都用不上的那一组，再叫起一次。
    SwitchToNoUsableGpuThenWake,
    /// 同上，切换之后先建一个 inode（一次用户改动、一个新用户状态），再叫起一次。
    SwitchToNoUsableGpuThenAUserChangeThenWake,
    /// 不切换，再叫起一次（对照：门照判）。
    WakeWithoutASwitch,
}

/// 一段历史的读数。
#[derive(Debug)]
struct ReadingOfAHistory {
    gate_closed_before: bool,
    live_intent_before: bool,
    states_now_before: usize,
    states_left_past_the_next_publish_and_the_margin_before: usize,
    started: bool,
    trigger: String,
    outcome: String,
    publishes_of_the_wake: usize,
    states_left_past_the_margin_after: usize,
    live_intent_after: bool,
}

fn history(user_changes_before: usize, after: AfterTheGateCloses) -> ReadingOfAHistory {
    let tag = format!("attack-stopped-drop-gate-{user_changes_before}-{after:?}");
    let mut pool = pool_after_a_sequential_write(&tag);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(gpu_compute_unusable_under_the_settings_naming_only_the_card_with_eight_gibibytes()),
    )
    .expect("两张卡都点名、都用得上：挂载照常做成");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, region);
    let mut txgs = Vec::new();
    for _change in 0..user_changes_before {
        txgs.push(create_one_inode(&mut session, &mut devices));
    }
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let mut clock = 0_u64;
    for _wake in 0..WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST {
        clock += INTERVAL_IN_MILLISECONDS;
        caller.clock_in_milliseconds.set(clock);
        let call = session
            .run_compaction_in_the_background(&mut devices, pacing_of_the_histories(), &caller)
            .expect("盘表是挂载的那一份");
        if let BackgroundCompactionCall::Ran { run, .. } = &call {
            assert!(
                !matches!(
                    run.outcome,
                    singlefs_core::mounted_session::CompactionRunOutcome::Failed(_)
                ),
                "前提：叫起到门关的那几趟不出错：{:?}",
                run.outcome
            );
        }
        if matches!(
            session.compaction.observations.last_stop,
            Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. })
        ) {
            break;
        }
    }
    let gate_closed_before = matches!(
        session.compaction.observations.last_stop,
        Some(CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates { .. })
    );
    let live_intent_before =
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations).is_some();
    let current_before = current_of(&session).root.checkpoint_txg;
    let states_now_before = user_states_among_the_roots_from(&devices, &txgs, CheckpointTxg(0));
    let states_left_past_the_next_publish_and_the_margin_before =
        user_states_left_after_the_next_publishes(
            &devices,
            &txgs,
            current_before,
            MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS + 1,
        );
    match after {
        AfterTheGateCloses::SwitchToNoUsableGpuThenWake => {
            session
                .switch_background_compute_from_the_next_batch(
                    settings_naming_only_the_card_with_eight_gibibytes(),
                    None,
                )
                .expect("只点名 card-eight 的一组照卡清单校验合法：不拒切换");
        }
        AfterTheGateCloses::SwitchToNoUsableGpuThenAUserChangeThenWake => {
            session
                .switch_background_compute_from_the_next_batch(
                    settings_naming_only_the_card_with_eight_gibibytes(),
                    None,
                )
                .expect("只点名 card-eight 的一组照卡清单校验合法：不拒切换");
            txgs.push(create_one_inode(&mut session, &mut devices));
        }
        AfterTheGateCloses::WakeWithoutASwitch => {}
    }
    clock += INTERVAL_IN_MILLISECONDS;
    caller.clock_in_milliseconds.set(clock);
    let call = session
        .run_compaction_in_the_background(&mut devices, pacing_of_the_histories(), &caller)
        .expect("盘表是挂载的那一份");
    let (started, trigger, outcome, publishes_of_the_wake) = match &call {
        BackgroundCompactionCall::NotStarted { reason, .. } => (
            false,
            String::from("-"),
            format!("NotStarted({reason:?})"),
            0,
        ),
        BackgroundCompactionCall::Ran { trigger, run } => (
            true,
            format!("{trigger:?}"),
            format!("{:?}", run.outcome),
            run.versions_published.len(),
        ),
    };
    let current_after = current_of(&session).root.checkpoint_txg;
    let states_left_past_the_margin_after = user_states_left_after_the_next_publishes(
        &devices,
        &txgs,
        current_after,
        MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS,
    );
    let live_intent_after =
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations).is_some();
    drop(session);
    pool.devices = Some(devices);
    ReadingOfAHistory {
        gate_closed_before,
        live_intent_before,
        states_now_before,
        states_left_past_the_next_publish_and_the_margin_before,
        started,
        trigger,
        outcome,
        publishes_of_the_wake,
        states_left_past_the_margin_after,
        live_intent_after,
    }
}

/// 打中那一边：门关着（独立读法：下一次发布加余量之后不同用户状态少于 4 个、且比此刻少）、意图活着时，
/// 切到一张卡都用不上那一组之后的那一次叫起照样起跑、单发一次删意图；不切换的对照臂同一刻一版不发。
#[test]
fn attack_the_background_trip_left_stopped_drops_the_intent_past_a_closed_rollback_gate() {
    let mut hits = 0_usize;
    let mut arms_judged = 0_usize;
    for user_changes_before in [4_usize, 6, 8] {
        let mut readings = Vec::new();
        for after in [
            AfterTheGateCloses::SwitchToNoUsableGpuThenWake,
            AfterTheGateCloses::SwitchToNoUsableGpuThenAUserChangeThenWake,
            AfterTheGateCloses::WakeWithoutASwitch,
        ] {
            let reading = history(user_changes_before, after);
            println!(
                "{PREFIX} stopped_drop_gate user_changes_before={user_changes_before} after={after:?} gate_closed_before={} live_intent_before={} states_now_before={} states_left_past_the_next_publish_and_the_margin_before={} started={} trigger={} outcome={} publishes_of_the_wake={} states_left_past_the_margin_after={} live_intent_after={}",
                reading.gate_closed_before,
                reading.live_intent_before,
                reading.states_now_before,
                reading.states_left_past_the_next_publish_and_the_margin_before,
                reading.started,
                reading.trigger,
                reading.outcome,
                reading.publishes_of_the_wake,
                reading.states_left_past_the_margin_after,
                reading.live_intent_after,
            );
            readings.push((after, reading));
        }
        let [(_, switched), _, (_, contrast)] = readings.as_slice() else {
            panic!("三臂");
        };
        let is_this_geometry_judged = switched.gate_closed_before
            && switched.live_intent_before
            && contrast.gate_closed_before
            && contrast.live_intent_before;
        if !is_this_geometry_judged {
            println!(
                "{PREFIX} stopped_drop_gate_not_judged user_changes_before={user_changes_before} reason=gate_not_closed_or_no_live_intent"
            );
            continue;
        }
        arms_judged += 1;
        let gate_refuses_the_next_publish =
            switched.states_left_past_the_next_publish_and_the_margin_before < ROLLBACK_STATES_KEPT
                && switched.states_left_past_the_next_publish_and_the_margin_before
                    < switched.states_now_before;
        let is_a_hit = gate_refuses_the_next_publish
            && switched.started
            && switched.trigger == "LiveIntentWhileStoppedAfterAComputeError"
            && switched.publishes_of_the_wake == 1
            && !switched.live_intent_after
            && switched.states_left_past_the_margin_after < ROLLBACK_STATES_KEPT
            && contrast.publishes_of_the_wake == 0;
        println!(
            "{PREFIX} stopped_drop_gate_verdict user_changes_before={user_changes_before} gate_refuses_the_next_publish={gate_refuses_the_next_publish} hit={is_a_hit}"
        );
        if is_a_hit {
            hits += 1;
        }
    }
    println!("{PREFIX} stopped_drop_gate_summary geometries_judged={arms_judged} hits={hits}");
    assert!(
        arms_judged > 0,
        "前提不成立：三种用户改动数下都没叫起到门关、或门关时意图已不在（造法要改，不是没打中）"
    );
    assert_eq!(
        hits, arms_judged,
        "没打中：门关着时停着的那一趟没有越门发删意图（看观测行）"
    );
}
