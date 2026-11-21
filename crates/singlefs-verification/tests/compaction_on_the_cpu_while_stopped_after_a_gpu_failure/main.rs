//! GC 因 GPU 出错停着时按池子占用在 CPU 上补跑（
//! 报警后必须进行垃圾回收时（比如磁盘超过 85% 的上限）用 cpu 补偿处理，挂载参数给默认值并提供可调节的参数；D26（后台整理与放置回收）
//! 已定项 13 第 4 条「那个后台任务停下并报错，不换回 CPU」）：
//! 挂载参数 `background_gc_cpu_compensation_threshold`（默认 85%）。整理这个后台任务因一批计算出错停着时，池子里有一块盘的已分配字节占它的容量超过阈值
//! （同日弹窗定「任一块盘超过就补跑」：比的是最满的那一块盘，不是全池相加），整理照常起跑、每一批在 CPU 上补跑：出错停下那一刻已删了意图，
//! 补跑从头签发、照常推进；每块盘都不超过就照旧停着，还有活意图（补跑签下、推着的）就单发一次只删意图的发布丢掉它（一趟开头判出的、
//! 一趟之中补跑过一批之后判出的都删；显式入口每一趟的预算至少 1，上一次用户改动给的那一份扣完了照样删），之后一个写都不发；阈值超过 100% 挂载与切换都拒；
//! 通过校验的切换照旧解除停下、从下一批起照新设置走。运行时读得出三样：整理停着、停下以来在 CPU 上补跑了几批、此刻最满的那一块盘的占用与阈值。
//! GPU 用一份背后照核心层 CPU 写法算的假实现（`common_background_gpu_compute`），注入甲段（对 R 重扫时核可搬单元的校验和）出错让整理停下。
//! 池照 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs`：`common::build_pool` 起，另一次挂载里顺序写第一个文件
//! 40 个数据单元，重挂选 `gpu`；整理一批至多搬 2 个、起跑强制开着、选段固定在数据单元最多的那一段，先在 GPU 上签发意图，再注入出错停下
//! （那一批放弃，单发一次只删意图的发布丢掉意图，整理停下）。
//! 这样的池子每块盘的占用都远低于 1%：「超过阈值」取阈值 0%，「不超过」取默认 85%；要一块盘越线、另一块不越的，用例把第一块盘单元区末尾的一段空槽
//! 直接在分配器里标成已分配（不发布、不写盘），把它的占用抬过整百分点的阈值。「一趟之中补跑过一批之后不补跑了」那一格经只供测试的开关
//! （`BackgroundComputeOfThisMount::judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests`）在那一批的钩子里强制进入。
#[path = "../common/mod.rs"]
mod common;
#[path = "../common_background_gpu_compute/mod.rs"]
mod common_background_gpu_compute;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{
    build_pool, disk_snapshot, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, card_named,
    device_lost_on_the_card_with_eight_gibibytes, shared_cpu_backed_gpu_compute,
    CpuBackedBackgroundGpuComputeForTests,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator, ReuseWindow};
use singlefs_core::background_compute::{
    AlarmOfABackgroundTaskStoppedAfterAComputeError, AllocatedBytesAndCapacityOfADevice,
    BackgroundComputeBackend, BackgroundComputeError, BackgroundComputeOfOneBatch,
    BackgroundComputeOfThisMount, BackgroundComputePathTaken, BackgroundComputeSettings,
    BackgroundComputeSettingsRefused, BackgroundGcCpuCompensationThreshold,
    BackgroundGpuComputeHandle, BackgroundTask, BackgroundTaskAfterTheComputeError,
    BackgroundTaskStoppedAfterAComputeError, GcCpuCompensationReading, GpuComputedSegment,
    ReasonTheBatchRanOnTheCpu, BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::compaction_plan::{
    empty_cluster_segments_by_the_records, AnswerOfTheCallerBeforeTheNextBatch,
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
    CompactionBatchUnits, CompactionPacing, CompactionRunForcing, CompactionRunStop,
    CompactionStartJudgement, CompactionSwitches, CompactionTimerFallback,
    CompactionWriteBandwidthLimit, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_background_compute_settings,
    mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold,
    mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches,
    roll_back_by_a_forward_publish, RollbackTarget, WritableMountWithBackgroundComputeRefused,
};
use singlefs_core::mounted_session::{
    BackgroundCompactionAfterThePendingChangesRefused, BackgroundCompactionCall,
    BackgroundCompactionNotStarted, BackgroundCompactionTrigger, CompactionRun,
    CompactionRunOutcome, MountedSession, MountedSessionWithPendingChanges, UserChange,
};
use singlefs_core::transaction::{
    publish_compaction_step, CompactionIntentStep, PoolVersion, PoolWriter, TransactionOutput,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;

mod alarms_of_the_background_entry_with_pending_changes;
mod background_entry_and_explicit_entry_left_stopped;
mod compensating_on_the_cpu_while_the_pool_is_above_the_threshold;
mod compensating_run_compared_with_the_cpu_mode;
mod compensating_run_dropping_the_intent_it_issued;
mod compensation_threshold_at_the_mount_and_at_a_switch;
mod left_stopped_with_the_deletion_queue_not_empty;
mod one_device_above_the_threshold;
mod wake_counts_after_a_stop;
mod wakes_while_the_compaction_stays_stopped;

/// 另一次挂载里顺序写第一个文件的数据单元数（同 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs`）。
const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;

/// 整理一批至多搬几个（只供测试的开关）。
const UNITS_PER_BATCH: usize = 2;

/// mkfs 写的每区根槽数 S（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`）。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;

/// 一次用户改动给出的那一份 `B_gc`：`3S − 4`。
const BUDGET_GIVEN_BY_A_USER_CHANGE: u64 = 3 * ROOT_RING_SLOTS_PER_REGION - 4;

/// 补跑那一趟显式入口推几次发布（`B_gc` 压到它：「已经过去的发布数」取 `3S − 4 − 它`）。
const PUBLISHES_OF_THE_COMPENSATING_RUN: u64 = 3;

/// 删意图那一次取落点放开 R 的用例：签发之后几批把 R 里搬空、回收出的空槽（每块盘）至少这么多，R 之外标满之后删意图要在 R 里落得下。
const FREE_SLOTS_IN_THE_REGION_WANTED: usize = 32;

/// 回收 R 里空槽那几批的迭代上界：一批至多搬 [`UNITS_PER_BATCH`] 个两槽单元，32 槽至多 8 批；R 里的数据单元搬不空。
const BATCHES_FREEING_THE_REGION_AT_MOST: usize = 16;

/// 阈值 0%：池子里有一个字节已分配就超过。
const THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE: BackgroundGcCpuCompensationThreshold =
    BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 0,
    };

/// 挂载时给不给补跑的阈值。封闭集合。
enum CompensationThresholdAtTheMount {
    /// 经不带阈值的入口挂（`mount_writable_with_background_compute_settings`）：取默认。
    NotGivenTakingTheDefault,
    Given(BackgroundGcCpuCompensationThreshold),
}

/// 后台入口的调用方：钟停在起点、前台没有 fsync 在等。
struct CallerWithNoForegroundFsync;

impl CallerOfTheBackgroundCompaction for CallerWithNoForegroundFsync {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::ZERO)
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 定时兜底关着、写带宽不限：后台入口只看「该跑了」。
fn pacing_with_the_timer_fallback_off() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::Off,
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

/// 另一次挂载里顺序写第一个文件 [`DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT`] 个数据单元，那一次的会话丢掉。
/// 交回池与第一块盘上两槽的数据单元最多的那一段（重挂不搬数据单元，重挂之后它还是那一段）。
fn pool_after_a_sequential_write_in_an_earlier_mount(tag: &str) -> (BuiltPool, CompactionRegion) {
    let mut pool = build_pool(tag);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content: Vec<u8> = (0..DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT
        * data_unit_payload_capacity())
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
    let mut data_units_by_segment: BTreeMap<u64, u64> = BTreeMap::new();
    for record in current_of(&session)
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
    let (fullest_segment, _data_units) = data_units_by_segment
        .into_iter()
        .max_by_key(|(_, data_units)| *data_units)
        .expect("写了数据单元：至少一段里有");
    drop(session);
    pool.devices = Some(devices);
    (
        pool,
        CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(fullest_segment))
            .expect("按 64 取整的段起点"),
    )
}

/// R 那块盘上起点在 R 里、未释放的记录。
fn unreleased_records_in(
    session: &MountedSession,
    region: CompactionRegion,
) -> Vec<AllocationRecord> {
    current_of(session)
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .copied()
        .collect()
}

/// 显式入口推一次（`B_gc` 压到 1）。
fn run_one_compaction_publish(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
) -> CompactionRun {
    session.compaction.publishes_since_the_last_user_change = BUDGET_GIVEN_BY_A_USER_CHANGE - 1;
    session.run_compaction(devices).expect("整理一趟")
}

/// 重挂选 `gpu`（全合法的两张卡，递进 `gpu_compute`），补跑的阈值照 `threshold`；装上整理只供测试的开关：一批至多 [`UNITS_PER_BATCH`] 个、
/// 起跑强制开着、选段固定在 `region`。在 GPU 上签发意图（`B_gc` 1）。交回会话与盘。
fn gpu_session_with_the_intent_issued_on_the_gpu(
    pool: &mut BuiltPool,
    region: CompactionRegion,
    gpu_compute: &BackgroundGpuComputeHandle,
    threshold: &CompensationThresholdAtTheMount,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = match threshold {
        CompensationThresholdAtTheMount::NotGivenTakingTheDefault => {
            mount_writable_with_background_compute_settings(
                &parameters(),
                &mut devices,
                admissible_gpu_settings_naming_both_cards(),
                Some(gpu_compute.clone()),
            )
        }
        CompensationThresholdAtTheMount::Given(threshold) => {
            mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
                &parameters(),
                &mut devices,
                admissible_gpu_settings_naming_both_cards(),
                Some(gpu_compute.clone()),
                *threshold,
            )
        }
    }
    .expect("全合法的 gpu 设置连同实现、不超过 100% 的阈值：挂载照常做成");
    let (_, session) = MountedSession::of_the_mount(mounted);
    session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
        session,
        devices,
        region,
        "在 GPU 上",
    )
}

/// 整理只供测试的开关：一批至多 [`UNITS_PER_BATCH`] 个、起跑强制开着、选段固定在 `region`，其余照产品。
fn compaction_switches_of_the_test(region: CompactionRegion) -> CompactionSwitches {
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
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(UNITS_PER_BATCH).expect("至少 1"),
        ),
        forced_region: Some(region),
        run_forcing: CompactionRunForcing::ForcedToStartForTests,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    }
}

/// 给刚挂上的会话装上 [`compaction_switches_of_the_test`]、签发意图（`B_gc` 1）：只发了签发那一次、意图在 `region` 上、不报警。
/// `where_issued` 写进断言消息（「在 GPU 上」「在 CPU 上」）。交回会话与盘。
fn session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
    mut session: MountedSession,
    mut devices: Vec<(DeviceIdentity, Recorded)>,
    region: CompactionRegion,
    where_issued: &str,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    session
        .allocator
        .set_compaction_switches(compaction_switches_of_the_test(region));
    let issuing = run_one_compaction_publish(&mut session, &mut devices);
    assert_eq!(
        issuing.alarm_of_the_compaction_stopped_after_a_compute_error, None,
        "还没停着：不报警"
    );
    assert!(
        issuing.versions_published.len() == 1
            && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_some_and(|(_, intent)| intent.region == region),
        "{where_issued}只发了签发那一次，意图在强制选的那一段上：{:?}",
        issuing.outcome
    );
    (session, devices)
}

/// 下一次 GPU 那一侧算甲段（对 R 重扫时核可搬单元的校验和）就交回「card-eight」设备丢了。
fn fail_the_unit_checksum_verification_on_the_card_with_eight_gibibytes(
    gpu_compute_kept_by_the_test: &Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
) {
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .failing_segment = Some((
        GpuComputedSegment::UnitChecksumVerification,
        device_lost_on_the_card_with_eight_gibibytes(),
    ));
}

/// 甲段注入的那一个出错停下时报的错。
fn error_of_the_failing_unit_checksum_verification() -> BackgroundComputeError {
    BackgroundComputeError::GpuSegmentFailed {
        segment: GpuComputedSegment::UnitChecksumVerification,
        failure: device_lost_on_the_card_with_eight_gibibytes(),
    }
}

/// 同 [`gpu_session_with_the_intent_issued_on_the_gpu`]，再注入甲段出错、显式入口推一次：这一批放弃、一个单元都没搬，整理停下，
/// 单发一次只删意图的发布丢掉意图（盘上与分配器这一侧都没有活意图，R 排他跟着撤）。交回会话、盘与停下报的错。
fn gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
    pool: &mut BuiltPool,
    region: CompactionRegion,
    gpu_compute_kept_by_the_test: &Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
    gpu_compute: &BackgroundGpuComputeHandle,
    threshold: &CompensationThresholdAtTheMount,
) -> (
    MountedSession,
    Vec<(DeviceIdentity, Recorded)>,
    BackgroundComputeError,
) {
    let (mut session, mut devices) =
        gpu_session_with_the_intent_issued_on_the_gpu(pool, region, gpu_compute, threshold);
    fail_the_unit_checksum_verification_on_the_card_with_eight_gibibytes(
        gpu_compute_kept_by_the_test,
    );
    let txg_before_the_failing_batch = current_of(&session).root.checkpoint_txg;
    let failing = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        matches!(
            failing.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && failing.batches_published == 0,
        "甲段出错：这一批放弃、一个单元都没搬，整理停下：{:?}",
        failing.outcome
    );
    let [dropped] = failing.versions_published.as_slice() else {
        panic!(
            "出错停下那一趟恰好发一次只删意图的发布，实际发了 {} 次",
            failing.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_failing_batch.0 + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && current_of(&session) == dropped,
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图，会话的现行那一版就是它"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.compaction.observations.intents_dropped == 1,
        "分配器这一侧意图连同 R 排他一起撤了，记一次丢意图"
    );
    let error = error_of_the_failing_unit_checksum_verification();
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error: error.clone(),
            batches_compensated_on_the_cpu: 0,
        }],
        "整理停下、报出甲段那张卡的错，还没补跑过"
    );
    assert_eq!(
        failing.alarm_of_the_compaction_stopped_after_a_compute_error,
        Some(alarm_of_the_stop(&error, 0, 0)),
        "出错停下的那一趟就报警：这一趟不数，叫起次数报 0（用户 2026-11-01 定）"
    );
    (session, devices, error)
}

/// 整理因 `error` 停着时一次叫起交回的报警：甲段那张卡（「card-eight」）、停下以来叫起 `wakes_while_stopped` 次、补跑了 `batches_compensated_on_the_cpu` 批。
fn alarm_of_the_stop(
    error: &BackgroundComputeError,
    wakes_while_stopped: u64,
    batches_compensated_on_the_cpu: u64,
) -> AlarmOfABackgroundTaskStoppedAfterAComputeError {
    AlarmOfABackgroundTaskStoppedAfterAComputeError {
        task: BackgroundTask::CompactionRelocation,
        gpu_devices_named_by_the_error: vec![card_named("card-eight")],
        error_that_stopped_the_task: error.clone(),
        wakes_while_stopped,
        batches_compensated_on_the_cpu,
    }
}

/// 每块盘此刻的已分配字节与容量，用例自己从分配器的逐盘计数算：占着的槽数与单元区槽数各乘 16 KiB，按分配器里的次序。
fn every_device_counted_by_the_test(
    session: &MountedSession,
) -> Vec<AllocatedBytesAndCapacityOfADevice> {
    session
        .allocator
        .devices
        .iter()
        .map(|device_map| AllocatedBytesAndCapacityOfADevice {
            device: device_map.device,
            allocated_bytes: device_map.allocated_slots() * SLOT_BYTES,
            capacity_bytes: device_map.unit_area_slots() * SLOT_BYTES,
        })
        .collect()
}

/// 已分配占容量最满的那一块盘，用例自己挑：交叉相乘比占比，一样满的取前面那一块。
fn fullest_device_counted_by_the_test(
    session: &MountedSession,
) -> AllocatedBytesAndCapacityOfADevice {
    let every_device = every_device_counted_by_the_test(session);
    let mut fullest = every_device[0];
    for device in &every_device[1..] {
        if u128::from(device.allocated_bytes) * u128::from(fullest.capacity_bytes)
            > u128::from(fullest.allocated_bytes) * u128::from(device.capacity_bytes)
        {
            fullest = *device;
        }
    }
    fullest
}

/// `allocated_bytes × 100 > capacity_bytes × percent`：占比严格大于整百分点 `percent`。
fn share_is_above_percent(allocated_bytes: u64, capacity_bytes: u64, percent: u32) -> bool {
    u128::from(allocated_bytes) * 100 > u128::from(capacity_bytes) * u128::from(percent)
}

fn gpu_calls_so_far(
    gpu_compute_kept_by_the_test: &Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
) -> usize {
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .entries_handed_in
        .len()
}

/// `device` 那块盘单元区末尾的一段空槽直接在分配器里标成已分配，使这块盘的占用恰好严格大于整百分点 `percent`
/// （已分配槽数取满足「已分配 × 100 > 单元区槽数 × percent」的最小值）。不发布、不写盘：只为让这一块盘越线，另一块盘不动。交回标了几槽。
fn allocate_the_tail_of_a_device_just_above_percent(
    session: &mut MountedSession,
    device: DeviceIdentity,
    percent: u32,
) -> u64 {
    let device_map = session
        .allocator
        .devices
        .iter_mut()
        .find(|device_map| device_map.device == device)
        .expect("池里有这块盘");
    let unit_area_slots = device_map.unit_area_slots();
    let allocated_slots_just_above_percent = u64::from(percent) * unit_area_slots / 100 + 1;
    let slots_to_allocate =
        allocated_slots_just_above_percent.saturating_sub(device_map.allocated_slots());
    assert!(
        slots_to_allocate > 0,
        "这块盘原来没越过 {percent}%：要标的槽数不是 0"
    );
    device_map.mark_allocated(
        SlotNumber(device_map.unit_area_start().slot().0 + unit_area_slots - slots_to_allocate),
        slots_to_allocate,
    );
    slots_to_allocate
}

/// `device` 那块盘末尾已由 [`allocate_the_tail_of_a_device_just_above_percent`] 标了 `slots_marked_at_the_tail` 槽；这块盘此刻的占用不严格大于
/// 整百分点 `percent` 时，紧挨在那一段前面再标几槽，使占用恰好严格大于 `percent`（同一个最小值）；已经越线就一槽都不标。不发布、不写盘。交回这一次标了几槽。
fn allocate_slots_below_the_marked_tail_until_just_above_percent(
    session: &mut MountedSession,
    device: DeviceIdentity,
    percent: u32,
    slots_marked_at_the_tail: u64,
) -> u64 {
    let device_map = session
        .allocator
        .devices
        .iter_mut()
        .find(|device_map| device_map.device == device)
        .expect("池里有这块盘");
    let unit_area_slots = device_map.unit_area_slots();
    let allocated_slots_just_above_percent = u64::from(percent) * unit_area_slots / 100 + 1;
    let slots_to_allocate =
        allocated_slots_just_above_percent.saturating_sub(device_map.allocated_slots());
    if slots_to_allocate > 0 {
        device_map.mark_allocated(
            SlotNumber(
                device_map.unit_area_start().slot().0 + unit_area_slots
                    - slots_marked_at_the_tail
                    - slots_to_allocate,
            ),
            slots_to_allocate,
        );
    }
    slots_to_allocate
}

/// 停着时整理照常推：一批读完设置、开算之前的钩子里调只供测试的开关，从这一批之后每一批开头都当占用不超过阈值判。
fn judge_the_next_batches_not_above_the_threshold(
    background_compute: &mut BackgroundComputeOfThisMount,
) {
    background_compute.judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests();
}

/// 整理因 GPU 出错停着、阈值 0%（补跑）：补跑那一趟只发签发那一次就用完预算（同 [`a_run_left_stopped_at_its_start_drops_the_intent_a_compensating_run_issued`]），
/// 钩子里强制之后每一批都当不超过阈值。交回池、R、会话与盘：会话此刻停着而不补跑，盘上与分配器这一侧都有一条活意图，R 排他着。
fn session_left_stopped_with_the_intent_a_compensating_run_issued(
    tag: &str,
) -> (
    BuiltPool,
    CompactionRegion,
    MountedSession,
    Vec<(DeviceIdentity, Recorded)>,
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(tag);
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    session.compaction.publishes_since_the_last_user_change = BUDGET_GIVEN_BY_A_USER_CHANGE - 1;
    let compensating = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut judge_the_next_batches_not_above_the_threshold,
        )
        .expect("整理一趟");
    assert!(
        matches!(
            compensating.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
        ) && compensating.versions_published.len() == 1,
        "前提：补跑那一趟只发了签发那一次、推到预算用完：{:?}",
        compensating.outcome
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, intent)| intent.region == region)
            && session.allocator.region_excluded_from_placement() == Some(region),
        "前提：补跑那一批在 R 上签下意图，盘上活着、分配器这一侧排他 R"
    );
    (pool, region, session, devices)
}

/// 停着时会话里 unlink 第一个文件（一次走完），再挂着回退到 unlink 意图那一版：删除队列带回第一个文件
/// （攻方探针 `research/prompts/m4-gpugc-code-r1-opus-model/probes/harness_deletion_continuation_test.rs` 的造法）。
fn bring_the_deletion_queue_back_by_rolling_back_an_unlink(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
) {
    let unlinked = session
        .publish_user_change(
            devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("停着时会话照常 unlink：一次走完");
    let intent_root = unlinked.versions_published[0].root;
    let mut current = current_of(session).clone();
    assert!(current.deleted_inodes.is_empty(), "unlink 一次走完：队列空");
    roll_back_by_a_forward_publish(
        &parameters(),
        devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: intent_root.instance,
            checkpoint_txg: intent_root.checkpoint_txg,
        },
    )
    .expect("挂着时回退到 unlink 意图那一版");
    session.current = PoolVersion::of_a_version_of_trees(current);
    assert_eq!(
        current_of(session).deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "回退到意图那一版：删除队列带回第一个文件"
    );
}

/// 一块盘：照转给里面那块录着的盘，只是每一次写都报 I/O 错、不落盘（攒批的会话排空待发那一次写报错被拒；用例只在那一次调用上换上它）。
struct DeviceFailingEveryWrite {
    inner: Recorded,
}

impl BlockDevice for DeviceFailingEveryWrite {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        _offset: DeviceOffsetInBytes,
        _bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        Err(BlockDeviceError::InputOutput(std::io::Error::other(
            "用例设的写失败",
        )))
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}
