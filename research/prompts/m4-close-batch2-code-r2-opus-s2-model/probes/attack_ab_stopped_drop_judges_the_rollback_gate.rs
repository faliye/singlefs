//! 攻方探针（代码轮 m4-close-batch2-code-r2 承接：判决第三节第 1 条改法的 A/B）。同一份探针放进两份副本跑：不打补丁的那一份（甲臂）与打了
//! `patches/stopped-drop-reads-the-mount-budget-and-judges-the-gate.patch` 的那一份（乙臂）。补丁做两件事：后台入口判起跑时挂载那一份预算读根环排到
//! 停没停那一判之前；停着那一趟单发删意图之前，后台入口、照水位判时先过回退门，门关着就不发、意图与 R 排他留着。
//! 历史（同 `attack_stopped_drop_skips_the_rollback_gate.rs`）：另一次挂载里顺序写第一个文件；重挂选 `gpu`（两张卡都用得上）；开关照水位判、
//! 越线靠只供测试的开关、选段强制、一批一个；建 `user_changes_before` 个 inode；后台入口定时兜底叫起到门关（意图还活着、R 排他）。之后三种：
//! 切到一张卡都用不上的那一组再叫起；切换之后先建一个 inode 再叫起；不切换再叫起（对照）。前两种接着不改动再叫起一次，再一次一个 inode、
//! 每建一个叫起一次，直到意图没了或建满上限。盘是内存稀疏盘：起点（顺序写之后）只建一次，每段历史从内存里那一份拷。
//! 另一个测试量挂载那一份预算：上一次挂载推到门关、意图活着时丢掉会话，重挂（续做照常），切到一张卡都用不上，读头一次后台调用之后的已过发布数。
//! 判门的读法独立于产品：根环里读得出、不低于 F 的根按用户改动分状态（同 `compaction_rollback_gate_keeps_four_user_states.rs`）。
//! 观测行以 `m4-close-batch2-r2-ab:` 起头。断言只核前提（造到了门关），两臂的结局都看观测行。
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
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionBatchUnits, CompactionPacing,
    CompactionRunForcing, CompactionRunStop, CompactionSwitches, CompactionTimerFallback,
    CompactionWatermarkCrossingForcing, CompactionWriteBandwidthLimit, UnitAreaOfOneDevice,
    WatermarkQuantity,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::mount_writable_with_background_compute_settings;
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, CompactionRunOutcome, MountedSession, UserChange,
};
use singlefs_core::recovery::{effective_rollback_floor, readable_roots};
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::ROOT_RING_REGIONS;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

const PREFIX: &str = "m4-close-batch2-r2-ab:";
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
/// 门关之后至多再建几个 inode（每建一个叫起一次）等意图删掉。
const USER_CHANGES_AFTER_THE_CLOSURE_AT_MOST: usize = 8;
/// 内存稀疏盘的物理块宽（同 `common::crash_state_devices`）。
const PHYSICAL_BLOCK_BYTES: u32 = 512;
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

fn has_a_live_intent(session: &MountedSession) -> bool {
    live_compaction_intent_of_the_entries(&current_of(session).logged_operations).is_some()
}

/// 起点：另一次挂载（CPU）里顺序写第一个文件之后丢掉会话，整条录制流施加到内存镜像上。只建这一次，每段历史从它拷。
fn memory_pool_after_a_sequential_write() -> MemoryPool {
    let mut pool = build_pool("attack-ab-stopped-drop-gate-starting-point");
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

/// 根环里读得出、不低于 F、txg 比 `txg` 新的根有几条（探针自己数；不扣实例表抛弃的根，也不认整理换了用户可见树根的那几条算状态）。
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

/// 一次叫起的读数。叫起之前的两样按探针自己的读法（用户改动分状态）算。
#[derive(Debug)]
struct ReadingOfAWake {
    user_changes_after_the_closure: usize,
    states_now_before: usize,
    states_left_past_the_next_publish_and_the_margin_before: usize,
    started: bool,
    trigger: String,
    outcome: String,
    publishes_of_the_wake: usize,
    states_left_past_the_margin_after: usize,
    live_intent_after: bool,
}

impl ReadingOfAWake {
    /// 探针自己的读法认为门该拒这一次叫起的下一次发布：下一次发布加余量之后不同用户状态少于 4 个、且比此刻少。
    fn gate_refuses_by_the_probe(&self) -> bool {
        self.states_left_past_the_next_publish_and_the_margin_before < ROLLBACK_STATES_KEPT
            && self.states_left_past_the_next_publish_and_the_margin_before < self.states_now_before
    }
}

/// 叫起一次后台入口（定时兜底往后拨一个间隔），前后各读一遍。
fn wake(
    session: &mut MountedSession,
    devices: &mut Devices,
    caller: &CallerOfTheHistories,
    txgs: &[CheckpointTxg],
    user_changes_after_the_closure: usize,
) -> ReadingOfAWake {
    let current_before = current_of(session).root.checkpoint_txg;
    let states_now_before = user_states_among_the_roots_from(devices, txgs, CheckpointTxg(0));
    let states_left_past_the_next_publish_and_the_margin_before =
        user_states_left_after_the_next_publishes(
            devices,
            txgs,
            current_before,
            MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS + 1,
        );
    caller
        .clock_in_milliseconds
        .set(caller.clock_in_milliseconds.get() + INTERVAL_IN_MILLISECONDS);
    let call = session
        .run_compaction_in_the_background(devices, pacing_of_the_histories(), caller)
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
    let current_after = current_of(session).root.checkpoint_txg;
    ReadingOfAWake {
        user_changes_after_the_closure,
        states_now_before,
        states_left_past_the_next_publish_and_the_margin_before,
        started,
        trigger,
        outcome,
        publishes_of_the_wake,
        states_left_past_the_margin_after: user_states_left_after_the_next_publishes(
            devices,
            txgs,
            current_after,
            MARGIN_OF_THE_GATE_IN_ROOT_RING_SLOTS,
        ),
        live_intent_after: has_a_live_intent(session),
    }
}

/// 挂载、开关、建 `user_changes_before` 个 inode、定时兜底叫起到门关（前提：那几趟都不出错）。交回会话、各次用户改动的 txg 与调用方。
fn prefix_up_to_the_closed_gate(
    devices: &mut Devices,
    user_changes_before: usize,
) -> (MountedSession, Vec<CheckpointTxg>, CallerOfTheHistories) {
    let mut session = mounted_on_both_cards(devices);
    let region = fullest_data_segment_of(&session);
    force_the_compaction_over_the_watermark(&mut session, region);
    let mut txgs = Vec::new();
    for _change in 0..user_changes_before {
        txgs.push(create_one_inode(&mut session, devices));
    }
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    for _wake in 0..WAKES_UNTIL_THE_GATE_CLOSES_AT_MOST {
        caller
            .clock_in_milliseconds
            .set(caller.clock_in_milliseconds.get() + INTERVAL_IN_MILLISECONDS);
        let call = session
            .run_compaction_in_the_background(devices, pacing_of_the_histories(), &caller)
            .expect("盘表是挂载的那一份");
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
    (session, txgs, caller)
}

/// 门关之后用户（与调用方）做什么。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AfterTheGateCloses {
    /// 挂着切到一张卡都用不上的那一组，再叫起。
    SwitchToNoUsableGpuThenWake,
    /// 同上，切换之后先建一个 inode（一次用户改动、一个新用户状态），再叫起。
    SwitchToNoUsableGpuThenAUserChangeThenWake,
    /// 不切换，再叫起一次（对照：整理没停，门照判）。
    WakeWithoutASwitch,
}

/// 一段历史的读数：门关那一刻、门关之后头一次叫起、（切换的两种）不改动再叫起一次、之后每建一个 inode 叫起一次。
#[derive(Debug)]
struct ReadingOfAHistory {
    gate_closed_before: bool,
    live_intent_before: bool,
    wakes: Vec<ReadingOfAWake>,
}

impl ReadingOfAHistory {
    fn first_wake(&self) -> &ReadingOfAWake {
        self.wakes.first().expect("门关之后至少叫起一次")
    }

    /// 意图在门关之后第几次用户改动之后没了（头一次叫起就删掉的是 0 或 1，看那一种建没建 inode）；建满上限还在交回 `None`。
    fn user_changes_until_the_intent_is_gone(&self) -> Option<usize> {
        self.wakes
            .iter()
            .find(|reading| !reading.live_intent_after)
            .map(|reading| reading.user_changes_after_the_closure)
    }

    /// 删掉意图的那一次叫起（发了版、意图没了）。
    fn wake_dropping_the_intent(&self) -> Option<&ReadingOfAWake> {
        self.wakes
            .iter()
            .find(|reading| reading.publishes_of_the_wake > 0 && !reading.live_intent_after)
    }

    /// 探针自己的读法认为门该拒、却发了版的叫起有几次。
    fn wakes_publishing_past_a_closed_gate(&self) -> usize {
        self.wakes
            .iter()
            .filter(|reading| {
                reading.gate_refuses_by_the_probe() && reading.publishes_of_the_wake > 0
            })
            .count()
    }
}

fn history(
    starting_point: &MemoryPool,
    user_changes_before: usize,
    after: AfterTheGateCloses,
) -> ReadingOfAHistory {
    let mut devices = devices_on(starting_point);
    let (mut session, mut txgs, caller) =
        prefix_up_to_the_closed_gate(&mut devices, user_changes_before);
    let gate_closed_before = the_gate_closed_last(&session);
    let live_intent_before = has_a_live_intent(&session);
    let mut user_changes_after_the_closure = 0_usize;
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
            user_changes_after_the_closure += 1;
        }
        AfterTheGateCloses::WakeWithoutASwitch => {}
    }
    let mut wakes = vec![wake(
        &mut session,
        &mut devices,
        &caller,
        &txgs,
        user_changes_after_the_closure,
    )];
    if after != AfterTheGateCloses::WakeWithoutASwitch {
        wakes.push(wake(
            &mut session,
            &mut devices,
            &caller,
            &txgs,
            user_changes_after_the_closure,
        ));
        while has_a_live_intent(&session)
            && user_changes_after_the_closure < USER_CHANGES_AFTER_THE_CLOSURE_AT_MOST
        {
            txgs.push(create_one_inode(&mut session, &mut devices));
            user_changes_after_the_closure += 1;
            wakes.push(wake(
                &mut session,
                &mut devices,
                &caller,
                &txgs,
                user_changes_after_the_closure,
            ));
        }
    }
    ReadingOfAHistory {
        gate_closed_before,
        live_intent_before,
        wakes,
    }
}

fn label_of_the_wake(after: AfterTheGateCloses, index: usize) -> &'static str {
    match (after, index) {
        (_, 0) => "first_after_the_closure",
        (AfterTheGateCloses::WakeWithoutASwitch, _) => "unexpected",
        (_, 1) => "again_without_a_change",
        (_, _) => "after_one_more_user_change",
    }
}

/// 两臂喂同一批输入：门前用户改动 4、6、8 三档 × 门关之后三种。观测行写每次叫起的发布数、意图在不在、门前后的状态数，
/// 与汇总（门关着时头一次叫起发没发、意图删没删、之后建 inode 等到门开删不删得掉）。只断言前提。
#[test]
fn ab_the_stopped_drop_at_a_closed_rollback_gate() {
    let starting_point = memory_pool_after_a_sequential_write();
    let mut geometries_judged = 0_usize;
    let mut first_wakes_at_a_closed_gate = 0_usize;
    let mut first_wakes_publishing_past_a_closed_gate = 0_usize;
    let mut first_wakes_holding_the_intent_at_a_closed_gate = 0_usize;
    let mut wakes_publishing_past_a_closed_gate = 0_usize;
    let mut histories_switched = 0_usize;
    let mut histories_with_the_intent_gone = 0_usize;
    let mut histories_with_the_intent_gone_after_a_wake_held_at_the_gate = 0_usize;
    let mut histories_left_with_the_intent = 0_usize;
    let mut drops_while_the_probe_reads_the_gate_closed = 0_usize;
    let mut contrast_wakes_publishing = 0_usize;
    for user_changes_before in [4_usize, 6, 8] {
        let readings: Vec<(AfterTheGateCloses, ReadingOfAHistory)> = [
            AfterTheGateCloses::SwitchToNoUsableGpuThenWake,
            AfterTheGateCloses::SwitchToNoUsableGpuThenAUserChangeThenWake,
            AfterTheGateCloses::WakeWithoutASwitch,
        ]
        .into_iter()
        .map(|after| (after, history(&starting_point, user_changes_before, after)))
        .collect();
        for (after, reading) in &readings {
            for (index, wake_reading) in reading.wakes.iter().enumerate() {
                println!(
                    "{PREFIX} stopped_drop_gate_wake user_changes_before={user_changes_before} after={after:?} wake={} user_changes_after_the_closure={} states_now_before={} states_left_past_the_next_publish_and_the_margin_before={} gate_refuses_by_the_probe={} started={} trigger={} outcome={} publishes_of_the_wake={} states_left_past_the_margin_after={} live_intent_after={}",
                    label_of_the_wake(*after, index),
                    wake_reading.user_changes_after_the_closure,
                    wake_reading.states_now_before,
                    wake_reading.states_left_past_the_next_publish_and_the_margin_before,
                    wake_reading.gate_refuses_by_the_probe(),
                    wake_reading.started,
                    wake_reading.trigger,
                    wake_reading.outcome,
                    wake_reading.publishes_of_the_wake,
                    wake_reading.states_left_past_the_margin_after,
                    wake_reading.live_intent_after,
                );
            }
            let first = reading.first_wake();
            println!(
                "{PREFIX} stopped_drop_gate user_changes_before={user_changes_before} after={after:?} gate_closed_before={} live_intent_before={} publishes_of_the_wake={} states_left_past_the_next_publish_and_the_margin={} live_intent_after_the_wake={} wakes={} user_changes_until_the_intent_is_gone={} wakes_publishing_past_a_closed_gate={}",
                reading.gate_closed_before,
                reading.live_intent_before,
                first.publishes_of_the_wake,
                first.states_left_past_the_next_publish_and_the_margin_before,
                first.live_intent_after,
                reading.wakes.len(),
                reading
                    .user_changes_until_the_intent_is_gone()
                    .map_or_else(|| String::from("never"), |changes| changes.to_string()),
                reading.wakes_publishing_past_a_closed_gate(),
            );
        }
        let is_this_geometry_judged = readings
            .iter()
            .all(|(_, reading)| reading.gate_closed_before && reading.live_intent_before);
        if !is_this_geometry_judged {
            println!(
                "{PREFIX} stopped_drop_gate_not_judged user_changes_before={user_changes_before} reason=gate_not_closed_or_no_live_intent"
            );
            continue;
        }
        geometries_judged += 1;
        for (after, reading) in &readings {
            let first = reading.first_wake();
            if *after == AfterTheGateCloses::WakeWithoutASwitch {
                if first.publishes_of_the_wake > 0 {
                    contrast_wakes_publishing += 1;
                }
                continue;
            }
            histories_switched += 1;
            if first.gate_refuses_by_the_probe() {
                first_wakes_at_a_closed_gate += 1;
                if first.publishes_of_the_wake > 0 {
                    first_wakes_publishing_past_a_closed_gate += 1;
                }
                if first.publishes_of_the_wake == 0 && first.live_intent_after {
                    first_wakes_holding_the_intent_at_a_closed_gate += 1;
                }
            }
            wakes_publishing_past_a_closed_gate += reading.wakes_publishing_past_a_closed_gate();
            if reading.user_changes_until_the_intent_is_gone().is_some() {
                histories_with_the_intent_gone += 1;
                if first.publishes_of_the_wake == 0 && first.live_intent_after {
                    histories_with_the_intent_gone_after_a_wake_held_at_the_gate += 1;
                }
            } else {
                histories_left_with_the_intent += 1;
            }
            if reading
                .wake_dropping_the_intent()
                .is_some_and(ReadingOfAWake::gate_refuses_by_the_probe)
            {
                drops_while_the_probe_reads_the_gate_closed += 1;
            }
        }
    }
    println!(
        "{PREFIX} stopped_drop_gate_ab_summary geometries_judged={geometries_judged} histories_switched={histories_switched} first_wakes_at_a_closed_gate={first_wakes_at_a_closed_gate} first_wakes_publishing_past_a_closed_gate={first_wakes_publishing_past_a_closed_gate} first_wakes_holding_the_intent_at_a_closed_gate={first_wakes_holding_the_intent_at_a_closed_gate} wakes_publishing_past_a_closed_gate={wakes_publishing_past_a_closed_gate} histories_with_the_intent_gone={histories_with_the_intent_gone} histories_with_the_intent_gone_after_a_wake_held_at_the_gate={histories_with_the_intent_gone_after_a_wake_held_at_the_gate} histories_left_with_the_intent={histories_left_with_the_intent} drops_while_the_probe_reads_the_gate_closed={drops_while_the_probe_reads_the_gate_closed} contrast_wakes_publishing={contrast_wakes_publishing}"
    );
    assert!(
        geometries_judged > 0,
        "前提不成立：三种用户改动数下都没叫起到门关、或门关时意图已不在（造法要改，不是读数）"
    );
}

/// H3：上一次挂载推到门关、意图活着时丢掉会话；重挂（两张卡都用得上，挂载续做照常），切到一张卡都用不上，读头一次后台调用之后
/// 会话记的已过发布数。补丁把挂载那一份预算读根环排到停没停那一判之前：打了补丁的那一臂这一次就读根环，没打的那一臂这一次不读。
#[test]
fn reading_of_the_publishes_of_the_mount_at_the_first_background_call_left_stopped() {
    let starting_point = memory_pool_after_a_sequential_write();
    let user_changes_before = 4_usize;
    let mut devices = devices_on(&starting_point);
    let (session, txgs, _caller_of_the_previous_mount) =
        prefix_up_to_the_closed_gate(&mut devices, user_changes_before);
    let gate_closed_in_the_previous_mount = the_gate_closed_last(&session);
    let live_intent_left_by_the_previous_mount = has_a_live_intent(&session);
    let publishes_counted_by_the_previous_mount =
        session.compaction.publishes_since_the_last_user_change;
    drop(session);
    let mut session = mounted_on_both_cards(&mut devices);
    let live_intent_after_the_mount = has_a_live_intent(&session);
    let publishes_counted_at_the_mount = session.compaction.publishes_since_the_last_user_change;
    let last_user_change = *txgs.last().expect("建了 inode");
    let roots_after_the_last_user_change_by_the_probe =
        roots_newer_than(&devices, last_user_change);
    session
        .switch_background_compute_from_the_next_batch(
            settings_naming_only_the_card_with_eight_gibibytes(),
            None,
        )
        .expect("只点名 card-eight 的一组照卡清单校验合法：不拒切换");
    let caller = CallerOfTheHistories {
        clock_in_milliseconds: Cell::new(0),
    };
    let first_call = wake(&mut session, &mut devices, &caller, &txgs, 0);
    let publishes_after_the_first_call = session.compaction.publishes_since_the_last_user_change;
    println!(
        "{PREFIX} mount_budget_first_stopped_call user_changes_before={user_changes_before} gate_closed_in_the_previous_mount={gate_closed_in_the_previous_mount} live_intent_left_by_the_previous_mount={live_intent_left_by_the_previous_mount} publishes_counted_by_the_previous_mount={publishes_counted_by_the_previous_mount} live_intent_after_the_mount={live_intent_after_the_mount} publishes_counted_at_the_mount={publishes_counted_at_the_mount} roots_after_the_last_user_change_by_the_probe={roots_after_the_last_user_change_by_the_probe} started={} trigger={} outcome={} publishes_of_the_wake={} publishes_since_the_last_user_change_after_the_first_call={publishes_after_the_first_call} read_at_the_first_call={}",
        first_call.started,
        first_call.trigger,
        first_call.outcome,
        first_call.publishes_of_the_wake,
        publishes_after_the_first_call != publishes_counted_at_the_mount,
    );
    assert!(
        gate_closed_in_the_previous_mount && live_intent_left_by_the_previous_mount,
        "前提不成立：上一次挂载没叫起到门关、或门关时意图已不在"
    );
}
