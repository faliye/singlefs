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
mod common;
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

/// 停着且最满的那一块盘超过阈值（挂载给 0%）：显式入口照常推，每一批走 CPU、记账写明是停着时补跑并带那一判的读数（请求的后端照旧是 `gpu`），
/// GPU 实现一次都没被调；出错停下时删了意图，这一趟在同一个 R 上从头签发、照常推进（R 里未释放的记录变少、意图还在 R 上）；整理照旧停着、报的还是那个错，补跑的批数等于这一趟的批数；
/// 会话读得出此刻最满的那一块盘、它的已分配字节与容量（与用例自己从分配器逐盘计数挑的相同）与阈值 0%。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_moves_its_intent_on_the_cpu_while_the_pool_is_above_the_threshold(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-on-the-cpu-above-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    let reading_before_the_run = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading_before_the_run,
        GcCpuCompensationReading {
            fullest_device: fullest_device_counted_by_the_test(&session),
            threshold: THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE,
        },
        "此刻的占用读得出、数对：最满的那一块盘与它的已分配、容量就是分配器逐盘计数挑出的那一块，阈值是挂载给的 0%"
    );
    assert!(
        reading_before_the_run.fullest_device.allocated_bytes > 0
            && reading_before_the_run.is_the_allocated_share_above_the_threshold(),
        "最满的那一块盘有已分配的字节：超过 0%（{reading_before_the_run:?}）"
    );
    let unreleased_in_the_region_before = unreleased_records_in(&session, region).len();
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        run.batches_published > 0 && !run.versions_published.is_empty(),
        "停着而超过阈值：照常推、搬了批（{:?}）",
        run.outcome
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget })
                if budget == PUBLISHES_OF_THE_COMPENSATING_RUN
        ),
        "推到预算用完，不停在「此前出错停着」上：{:?}",
        run.outcome
    );
    for batch in &run.background_compute_of_each_batch {
        let BackgroundComputePathTaken::Cpu(
            ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading),
        ) = batch.path_taken
        else {
            panic!("停着时补跑的每一批走 CPU、记成补跑：{batch:?}");
        };
        assert_eq!(
            (
                batch.task,
                batch.requested_backend,
                &batch.compute_error_that_abandoned_the_batch
            ),
            (
                BackgroundTask::CompactionRelocation,
                BackgroundComputeBackend::Gpu,
                &None
            ),
            "设置照旧是停下时生效的 gpu 那一组、这一批没出错：{batch:?}"
        );
        assert!(
            reading.threshold == THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE
                && reading.is_the_allocated_share_above_the_threshold(),
            "每一批的记账带那一判的读数：超过阈值 0%（{reading:?}）"
        );
    }
    let first_batch_reading = match run.background_compute_of_each_batch.first() {
        Some(BackgroundComputeOfOneBatch {
            path_taken:
                BackgroundComputePathTaken::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading),
                ),
            ..
        }) => *reading,
        other => panic!("至少一批、走的是补跑：{other:?}"),
    };
    assert_eq!(
        first_batch_reading, reading_before_the_run,
        "头一批那一判读的就是开跑之前那一刻的占用与阈值（删除队列是空的，其间没有发布）"
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "补跑不经 GPU 实现：一段都没交给它"
    );
    assert!(
        unreleased_records_in(&session, region).len() < unreleased_in_the_region_before,
        "从头签发、照常推进：R 里未释放的记录变少（{unreleased_in_the_region_before} 条起）"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, intent)| intent.region == region),
        "预算用完时意图还在 R 上，下一趟接着推"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: u64::try_from(
                run.background_compute_of_each_batch.len()
            )
            .expect("批数"),
        }],
        "整理照旧停着（报警那一态）、报的还是那个错，补跑的批数就是这一趟的批数"
    );
}

/// 停着且超过阈值（挂载给 0%）时，后台入口照已定项 1 的三水位照常判起跑（起跑强制开着，一次用户改动记下「该跑了」）：起跑、推至多一批，
/// 那一批在 CPU 上补跑；GPU 实现一次都没被调，补跑的批数加一。
#[test]
fn the_background_entry_starts_a_compaction_stopped_after_a_gpu_failure_and_runs_its_batch_on_the_cpu_while_the_pool_is_above_the_threshold(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-on-the-cpu-through-the-background-entry",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "用户改动之后起跑判据为真，记下「该跑了」"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("停着而超过阈值：后台入口照常判起跑、起跑了：{call:?}");
    };
    assert_eq!(trigger, BackgroundCompactionTrigger::DueAfterAUserChange);
    assert_eq!(
        run.batches_published, 1,
        "用户改动之后那一趟推至多一批：{:?}",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run
                .background_compute_of_each_batch
                .iter()
                .all(|batch| matches!(
                    batch.path_taken,
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    )
                )),
        "那一趟每一批在 CPU 上补跑：{:?}",
        run.background_compute_of_each_batch
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "补跑不经 GPU 实现"
    );
    assert_eq!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .iter()
            .map(|stopped| stopped.batches_compensated_on_the_cpu)
            .collect::<Vec<u64>>(),
        vec![u64::try_from(run.background_compute_of_each_batch.len()).expect("批数")],
        "整理照旧停着，补跑的批数就是这一趟的批数"
    );
}

/// 停着且不超过阈值（不给阈值，取默认 85%）：显式入口一个写都不发、一批都不开（没有一批的记账），停在「出错停着」上，盘上逐字节不变；
/// 一次用户改动记下「该跑了」之后后台入口也不起跑、「该跑了」留着；GPU 实现一次都没被调，补跑的批数是 0，出错停下时删掉的意图没再签；
/// 会话读得出阈值是默认的 85%、占用不超过它。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_stays_stopped_while_the_pool_is_not_above_the_threshold(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-stays-stopped-at-or-below-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    let reading = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading,
        GcCpuCompensationReading {
            fullest_device: fullest_device_counted_by_the_test(&session),
            threshold: BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
        },
        "不给阈值：取默认"
    );
    assert_eq!(
        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT.allocated_percent_of_the_capacity, 85,
        "默认 85%（用户 2026-10-28 定案）"
    );
    assert!(
        !reading.is_the_allocated_share_above_the_threshold(),
        "这个池子最满的那一块盘的占用也不超过 85%：{reading:?}"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.versions_published.is_empty()
            && run.background_compute_of_each_batch.is_empty(),
        "不超过阈值：照旧停着、一个写都不发、一批都不开：{:?}，{:?}",
        run.outcome,
        run.background_compute_of_each_batch
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变：系统配置槽、根环、录制流的步数"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    let after_the_user_change = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    assert!(
        matches!(
            call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "后台入口也不起跑：{call:?}"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "「该跑了」留着，解除之后下一次调用再取"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        after_the_user_change,
        "后台入口没起跑：盘上逐字节不变"
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "停着不换回 GPU 也不补跑：一段都没交给实现"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: 0,
        }],
        "整理照旧停着，一批都没补跑"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_none(),
        "出错停下时删了意图，停着不补跑就不再签：盘上与分配器这一侧都没有活意图（R 原来是 {region:?}）"
    );
}

/// 阈值超过 100%（101%）：选 `cpu` 的挂载也在读任何一块盘之前拒、报出给的阈值，盘上逐字节不变；100% 照收。
/// 挂着时带 101% 的切换同样被拒，生效的阈值照旧是 100%、没有待生效的切换。
#[test]
fn a_gc_cpu_compensation_threshold_above_one_hundred_percent_is_refused_at_the_mount_and_at_a_switch(
) {
    let mut pool = build_pool("gc-cpu-compensation-threshold-above-the-whole-capacity");
    let above_the_whole_capacity = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 101,
    };
    let the_whole_capacity = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 100,
    };
    let mut devices = pool.reopen_recorded();
    let before_the_mount = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused =
        mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
            &parameters(),
            &mut devices,
            BackgroundComputeSettings::PRODUCT,
            None,
            above_the_whole_capacity,
        );
    assert!(
        matches!(
            refused,
            Err(WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings(
                BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                    threshold
                }
            )) if threshold == above_the_whole_capacity
        ),
        "101% 拒绝挂载、报出给的阈值：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_mount,
        "在读任何一块盘之前拒：盘上逐字节不变"
    );
    let mounted =
        mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
            &parameters(),
            &mut devices,
            BackgroundComputeSettings::PRODUCT,
            None,
            the_whole_capacity,
        )
        .expect("100% 照收");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    assert_eq!(
        session
            .background_compute()
            .in_effect()
            .gc_cpu_compensation_threshold(),
        the_whole_capacity,
        "挂载给的阈值生效"
    );
    let switch = session
        .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            BackgroundComputeSettings::PRODUCT,
            None,
            above_the_whole_capacity,
        );
    pool.devices = Some(devices);
    assert_eq!(
        switch,
        Err(
            BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                threshold: above_the_whole_capacity
            }
        ),
        "101% 的切换被拒"
    );
    assert!(
        session
            .background_compute()
            .requested_from_the_next_batch()
            .is_none()
            && session
                .background_compute()
                .in_effect()
                .gc_cpu_compensation_threshold()
                == the_whole_capacity,
        "被拒的切换不留待生效的、生效的阈值照旧"
    );
}

/// 停着、不超过阈值（取默认 85%）时收到带新阈值的切换（选 `cpu`、阈值 90%）：当场解除停下；下一趟每一批照新设置走——请求的后端是 `cpu`、
/// 实际走的是「设置选的就是 cpu」（不是补跑），GPU 实现一次都没被调，生效的设置换成切换那一组、阈值 90%。
/// 之后不给阈值的切换沿用 90%。
#[test]
fn a_switch_with_a_new_gc_cpu_compensation_threshold_lifts_the_stop_and_the_next_batches_follow_the_new_settings(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-stop-lifted-by-a-switch-with-a-new-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    let switched_threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 90,
    };
    session
        .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            BackgroundComputeSettings::PRODUCT,
            None,
            switched_threshold,
        )
        .expect("选 cpu、阈值 90%：切换收下");
    assert!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "停着时收到通过校验的切换：当场解除停下"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    let unreleased_in_the_region_before = unreleased_records_in(&session, region).len();
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        run.batches_published > 0
            && unreleased_records_in(&session, region).len() < unreleased_in_the_region_before,
        "解除之后照常推、意图推进：{:?}",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run
                .background_compute_of_each_batch
                .iter()
                .all(|batch| *batch
                    == BackgroundComputeOfOneBatch {
                        task: BackgroundTask::CompactionRelocation,
                        requested_backend: BackgroundComputeBackend::Cpu,
                        path_taken: BackgroundComputePathTaken::Cpu(
                            ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
                        ),
                        compute_error_that_abandoned_the_batch: None,
                    }),
        "从下一批起照切换那一组走：请求 cpu、照设置走 CPU，不是补跑：{:?}",
        run.background_compute_of_each_batch
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "切到 cpu 之后不经 GPU 实现"
    );
    let in_effect = session.background_compute().in_effect();
    assert_eq!(
        (
            in_effect.settings().clone(),
            in_effect.gc_cpu_compensation_threshold()
        ),
        (BackgroundComputeSettings::PRODUCT, switched_threshold),
        "生效的换成切换那一组：cpu、阈值 90%"
    );
    session
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu：切换收下");
    assert_eq!(
        session
            .background_compute()
            .requested_from_the_next_batch()
            .map(|requested| requested.gc_cpu_compensation_threshold()),
        Some(switched_threshold),
        "不给阈值的切换沿用 90%，不回到默认"
    );
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

/// 「任一块盘超过就补跑」：停着、阈值 3%，第一块盘（R 那块）的占用抬过 3%，第二块盘不越线，两块盘的已分配与容量各自相加也不越线
/// （逐盘比与全池相加判得不一样）：会话读得出最满的是第一块盘、它的已分配字节与容量；显式入口照常推，每一批在 CPU 上补跑，记账里的读数报的也是第一块盘；
/// 补跑的批数等于这一趟的批数。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_runs_on_the_cpu_when_one_device_is_above_the_threshold_though_the_pool_as_a_whole_is_not(
) {
    let threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 3,
    };
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-when-one-device-is-above-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(threshold),
        );
    allocate_the_tail_of_a_device_just_above_percent(
        &mut session,
        DeviceIdentity(0),
        threshold.allocated_percent_of_the_capacity,
    );
    let every_device = every_device_counted_by_the_test(&session);
    let [region_device, other_device] = every_device.as_slice() else {
        panic!("池是两块盘：{every_device:?}");
    };
    assert!(
        region_device.device == DeviceIdentity(0)
            && share_is_above_percent(
                region_device.allocated_bytes,
                region_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            )
            && !share_is_above_percent(
                other_device.allocated_bytes,
                other_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            )
            && !share_is_above_percent(
                region_device.allocated_bytes + other_device.allocated_bytes,
                region_device.capacity_bytes + other_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            ),
        "第一块盘越过 3%、第二块盘不越、两块相加也不越：{every_device:?}"
    );
    let reading_before_the_run = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading_before_the_run,
        GcCpuCompensationReading {
            fullest_device: *region_device,
            threshold,
        },
        "会话读得出最满的是第一块盘、它的已分配字节与容量，阈值是挂载给的 3%"
    );
    assert!(
        reading_before_the_run.is_the_allocated_share_above_the_threshold(),
        "第一块盘越线：补跑那一判为真（{reading_before_the_run:?}）"
    );
    let gpu_calls_before = gpu_calls_so_far(&gpu_compute_kept_by_the_test);
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget })
                if budget == PUBLISHES_OF_THE_COMPENSATING_RUN
        ) && run.batches_published > 0,
        "一块盘越线就补跑：照常推、搬了批、推到预算用完（{:?}）",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run.background_compute_of_each_batch.iter().all(|batch| matches!(
                batch.path_taken,
                BackgroundComputePathTaken::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading)
                ) if reading.fullest_device.device == DeviceIdentity(0)
                    && reading.threshold == threshold
                    && reading.is_the_allocated_share_above_the_threshold()
            )),
        "每一批在 CPU 上补跑，读数报的是越线的第一块盘：{:?}",
        run.background_compute_of_each_batch
    );
    assert_eq!(
        gpu_calls_so_far(&gpu_compute_kept_by_the_test),
        gpu_calls_before,
        "补跑不经 GPU 实现"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: u64::try_from(
                run.background_compute_of_each_batch.len()
            )
            .expect("批数"),
        }],
        "整理照旧停着，补跑的批数就是这一趟的批数"
    );
}

/// 停着时整理照常推：一批读完设置、开算之前的钩子里数这一趟的第几批（从 1 数），数到 `batch_numbered` 那一批时调只供测试的开关，
/// 从那一批之后每一批开头都当占用不超过阈值判。
fn judge_the_batches_after_the_numbered_one_not_above_the_threshold(
    batch_numbered: u64,
    batches_seen: &Cell<u64>,
    background_compute: &mut BackgroundComputeOfThisMount,
) {
    batches_seen.set(batches_seen.get() + 1);
    if batches_seen.get() == batch_numbered {
        background_compute.judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests();
    }
}

/// 停着、阈值 0%（出错停下时删了意图）：一趟开头超过阈值，头一批在 CPU 上补跑、在同一个 R 上从头签发，第二批补跑、搬了单元
/// （两批都照开关排他 R）；只供测试的开关在第二批的钩子里让之后每一批都当不超过阈值判，第三批开头就不开：单发一次只删意图的发布丢掉
/// 补跑签下的那一条，停在「出错停着」上。只有补跑那两批的记账，没开的那一批没有；盘上与分配器这一侧都没有活意图；
/// 补跑的批数是 2，整理照旧停着、报的还是那个错。
#[test]
fn a_compensating_run_stops_at_the_next_batch_once_it_is_not_above_the_threshold_and_drops_the_intent_it_issued(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensation-ends-at-the-next-batch-of-the-run",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let batches_seen = Cell::new(0);
    let run = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut |background_compute| {
                judge_the_batches_after_the_numbered_one_not_above_the_threshold(
                    2,
                    &batches_seen,
                    background_compute,
                );
            },
        )
        .expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 1
            && run.versions_published.len() == 3,
        "签发与一批补跑发成，第三批开头不超过阈值、不开，删意图之后停在此前出错停着上：{:?}，发了 {} 版",
        run.outcome,
        run.versions_published.len()
    );
    let [issued, relocated, dropped] = run.versions_published.as_slice() else {
        panic!("发了三版：签发、一批、删意图");
    };
    assert!(
        [issued, relocated].iter().all(|version| {
            live_compaction_intent_of_the_entries(&version.logged_operations)
                .is_some_and(|(_, intent)| intent.region == region)
        }) && !relocated.units_relocated_by_this_publish.is_empty(),
        "补跑那两批在同一个 R 上从头签发、搬了单元"
    );
    assert!(
        dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none(),
        "没开的那一批开头单发一次只删意图的发布：不搬单元、logged_ops 里没有整理意图"
    );
    assert!(
        matches!(
            run.background_compute_of_each_batch.as_slice(),
            [
                BackgroundComputeOfOneBatch {
                    path_taken: BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    ),
                    compute_error_that_abandoned_the_batch: None,
                    ..
                },
                BackgroundComputeOfOneBatch {
                    path_taken: BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    ),
                    compute_error_that_abandoned_the_batch: None,
                    ..
                }
            ]
        ),
        "只有补跑那两批的记账，没开的那一批没有：{:?}",
        run.background_compute_of_each_batch
    );
    assert!(
        session.allocator.compaction().live_intent().is_none() && current_of(&session) == dropped,
        "分配器这一侧意图连同 R 排他一起撤了，会话的现行那一版就是删意图那一版"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: 2,
        }],
        "整理照旧停着、报的还是那个错，补跑了两批"
    );
}

/// 停着、阈值 0%（出错停下时删了意图）：先补跑一趟（预算 1：补跑那一批在同一个 R 上从头签发、照开关排他 R；钩子里让之后每一批都当
/// 不超过阈值判），下一趟开头就判出停着而不补跑：一批都不开，单发一次只删意图的发布丢掉补跑签下的那一条，停在「出错停着」上；
/// 没有一批的记账，盘上与分配器这一侧都没有活意图。
#[test]
fn a_run_left_stopped_at_its_start_drops_the_intent_a_compensating_run_issued() {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-left-stopped-at-the-start-drops-the-intent",
    );
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
        "补跑那一趟只发了签发那一次、推到预算用完：{:?}",
        compensating.outcome
    );
    assert_eq!(
        session.allocator.compaction().live_intent().map(|live| (
            live.intent.region,
            live.is_the_region_excluded_from_placement
        )),
        Some((region, true)),
        "补跑那一批在同一个 R 上从头签发，照开关排他 R"
    );
    let txg_before_the_run = current_of(&session).root.checkpoint_txg;
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let root_ring_slots_used_before_the_run = session.compaction.observations.root_ring_slots_used;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(
        (
            session.compaction.publishes_since_the_last_user_change,
            session.compaction.observations.root_ring_slots_used
        ),
        (
            BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN + 1,
            root_ring_slots_used_before_the_run + 1
        ),
        "删意图那一次是整理的一次发布：从 `B_gc` 里扣一次、用掉一个根槽"
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.background_compute_of_each_batch.is_empty(),
        "一趟开头就判出停着而不补跑：一批都不开：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "一趟开头恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_run.0 + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none(),
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none() && current_of(&session) == dropped,
        "分配器这一侧意图连同 R 排他一起撤了，会话的现行那一版就是删意图那一版"
    );
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

/// 后台入口判出停着而现行那一版有活意图（D26（后台整理与放置回收） 已定项 13 第 4 条）：
/// 上一次用户改动给的那一份 `B_gc` 还剩，后台入口起跑一趟，只单发一次删意图的发布（同显式入口一趟开头判出停着那一段），
/// 盘上与分配器这一侧意图连同 R 排他都撤了；之后再调不起跑，盘上逐字节不变。
/// 判别力：`crates/mutations.tsv`「后台入口停着不删」那一行。
#[test]
fn the_background_entry_left_stopped_drops_the_intent_a_compensating_run_issued_and_then_writes_nothing(
) {
    let (mut pool, _region, mut session, mut devices) =
        session_left_stopped_with_the_intent_a_compensating_run_issued(
            "compaction-background-entry-left-stopped-drops-the-intent",
        );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let txg_before_the_call = current_of(&session).root.checkpoint_txg;
    let intents_dropped_before_the_call = session.compaction.observations.intents_dropped;
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("停着而现行那一版有活意图：后台入口起跑一趟删它：{call:?}");
    };
    assert_eq!(
        trigger,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "起跑是为了删停着时的活意图"
    );
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.deletion_steps_published == 0
            && run.background_compute_of_each_batch.is_empty(),
        "那一趟开头判出停着：一批都不开、删除续做不走：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "那一趟恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_call.0 + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && current_of(&session) == dropped,
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图，会话的现行那一版就是它"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none()
            && session.compaction.observations.intents_dropped
                == intents_dropped_before_the_call + 1,
        "分配器这一侧意图连同 R 排他一起撤了，记一次丢意图"
    );
    let after_the_drop = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let call_after_the_drop = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    assert!(
        matches!(
            call_after_the_drop,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "删完之后照旧停着、不起跑：{call_after_the_drop:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        after_the_drop,
        "之后再调一个写都不发：系统配置槽、根环、录制流的步数逐字节不变"
    );
}

/// 停着而有活意图、预算扣完（D26（后台整理与放置回收） 已定项 13 第 4、5 条）：补跑那一趟用完上一次用户改动给的那一份 `B_gc`，后台入口连叫三次都不起跑、
/// 一个写都不发，意图与 R 排他留着（预算照后台入口别的起跑给）；用户改动发成、给了新的一份之后，后台入口那一次只单发一次删意图的发布，
/// 盘上与分配器这一侧意图连同 R 排他都撤了。
#[test]
fn the_background_entry_left_stopped_with_the_budget_spent_drops_the_intent_after_the_next_user_change(
) {
    let (mut pool, region, mut session, mut devices) =
        session_left_stopped_with_the_intent_a_compensating_run_issued(
            "compaction-background-entry-left-stopped-with-the-budget-spent",
        );
    let before_the_calls = disk_snapshot(&pool.memory_pool(), &pool.stream);
    for _call in 0..3 {
        let call = session
            .run_compaction_in_the_background(
                &mut devices,
                pacing_with_the_timer_fallback_off(),
                &CallerWithNoForegroundFsync,
            )
            .expect("后台入口");
        assert!(
            matches!(
                call,
                BackgroundCompactionCall::NotStarted {
                    reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                    ..
                }
            ),
            "那一份 `B_gc` 扣完、前台写得进：不起跑：{call:?}"
        );
    }
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_calls,
        "预算扣完时一个写都不发"
    );
    assert_eq!(
        session.allocator.region_excluded_from_placement(),
        Some(region),
        "意图与 R 排他留到下一次用户改动之后"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("停着时用户照常建一个 inode");
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("用户改动给了新的一份 `B_gc`：后台入口起跑一趟删意图：{call:?}");
    };
    assert_eq!(
        trigger,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "起跑是为了删停着时的活意图"
    );
    assert!(
        run.versions_published.len() == 1
            && run.batches_published == 0
            && matches!(
                run.outcome,
                CompactionRunOutcome::Stopped(
                    CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
                )
            ),
        "只单发一次删意图、停在停着上：{:?}，发了 {} 次",
        run.outcome,
        run.versions_published.len()
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations).is_none()
            && session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "盘上与分配器这一侧意图连同 R 排他都撤了"
    );
}

/// 攒批的会话经放锁的后台入口（至多一次发布）判出停着而有活意图、起跑删意图那一趟（D26（后台整理与放置回收） 已定项 13 第 4 条），
/// 排空待发那一次写报 I/O 错被拒（待发并进冻结着的那一次发布）：这一趟不记成没完。重发冻结的那一次（一次用户改动，记下「该跑了」）、
/// 一次通过校验的切换解除停下之后，下一次调用照普通整理判起跑，触发是用户改动之后的「该跑了」，不顶着停着删意图那一趟的触发名推普通整理。
/// 判别力：`crates/mutations.tsv`「排空被拒时停着删意图那一趟照样记成没完」那一行。
#[test]
fn the_stopped_drop_refused_at_the_drain_is_not_continued_under_its_trigger_once_a_switch_lifts_the_stop(
) {
    let (mut pool, _region, mut session, mut devices_of_the_stopped_session) =
        session_left_stopped_with_the_intent_a_compensating_run_issued(
            "compaction-stopped-drop-refused-at-the-drain-is-judged-again-after-the-switch",
        );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    session_with_pending_changes
        .accept(
            &mut devices_of_the_stopped_session,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
            CallerMonotonicClockReading(Duration::ZERO),
        )
        .expect("建一个 inode 收进待发");
    assert_eq!(
        session_with_pending_changes.pending_changes_accepted(),
        1,
        "前提：收进待发、两个阈值都没到：没发布"
    );
    let mut devices_failing_every_write: Vec<(DeviceIdentity, DeviceFailingEveryWrite)> =
        devices_of_the_stopped_session
            .into_iter()
            .map(|(identity, device)| (identity, DeviceFailingEveryWrite { inner: device }))
            .collect();
    let drained = session_with_pending_changes
        .run_compaction_in_the_background_for_at_most_one_publish(
            &mut devices_failing_every_write,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        );
    let mut devices: Vec<(DeviceIdentity, Recorded)> = devices_failing_every_write
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    assert!(
        matches!(
            drained,
            Err(BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused { .. })
        ),
        "前提：停着而有活意图，起跑删意图那一趟之前排空待发、那一次写报错被拒：{drained:?}"
    );
    assert!(
        !session_with_pending_changes
            .session_for_tests_only()
            .compaction
            .has_a_background_trip_under_way(),
        "停着删意图那一趟排空被拒：不记成没完的一趟"
    );
    assert!(
        session_with_pending_changes
            .resend_the_frozen_publish(&mut devices)
            .expect("盘照常写得进：重发成了"),
        "前提：排空那一次冻结着，重发成了"
    );
    assert_eq!(
        session_with_pending_changes
            .session_for_tests_only()
            .compaction
            .start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "前提：重发的是一次用户改动，起跑判据为真，记下「该跑了」"
    );
    session_with_pending_changes
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu 的产品设置：切换收下");
    assert!(
        session_with_pending_changes
            .session_for_tests_only()
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "前提：停着时收到通过校验的切换请求，当场解除停下"
    );
    let after_the_switch = session_with_pending_changes
        .run_compaction_in_the_background_for_at_most_one_publish(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, .. } = &after_the_switch.call else {
        panic!(
            "解除停下、记着「该跑了」：后台入口起跑：{:?}",
            after_the_switch.call
        );
    };
    assert_eq!(
        *trigger,
        BackgroundCompactionTrigger::DueAfterAUserChange,
        "解除停下之后照普通整理判起跑：触发是用户改动之后的「该跑了」，不是停着删意图那一趟"
    );
}

/// 停着、不超过阈值（取默认 85%），出错停下那一刻删了意图之后删除队列带回第一个文件（会话 unlink、再挂着回退到 unlink 意图那一版），
/// 再在现行那一版上签下一条意图（经发布入口直接签：会话推整理会先走删除续做），分配器这一侧照会话的做法排他 R。交回池、R、会话与盘：
/// 整理停着而不补跑，现行那一版有活意图、删除队列不空，上一次用户改动给的那一份 `B_gc` 还剩。
fn session_left_stopped_with_a_live_intent_and_the_deletion_queue_not_empty(
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
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    bring_the_deletion_queue_back_by_rolling_back_an_unlink(&mut session, &mut devices);
    let previous = current_of(&session).clone();
    let unit_area = {
        let device_map = &session.allocator.devices[0];
        UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        }
    };
    let intent = CompactionIntent {
        region,
        allocation_generation_at_issue: previous.root.checkpoint_txg,
        empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
            &previous.allocation_records,
            unit_area,
        ),
    };
    session.allocator.compaction_mut().issue(intent);
    let publish_parameters = parameters();
    let issued = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_compaction_step(
            &mut writer,
            &mut session.allocator,
            &previous,
            &[],
            CompactionIntentStep::Issue(intent),
            session.instance,
        )
        .expect("在删除队列不空的那一版上签发")
    };
    session.current = PoolVersion::of_a_version_of_trees(issued);
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, logged)| logged == intent)
            && current_of(&session).deleted_inodes == vec![InodeNumber(FIRST_INODE_NUMBER)]
            && session.allocator.region_excluded_from_placement() == Some(region),
        "前提：签发那一版有活意图、照抄删除队列（带着第一个文件），分配器这一侧排他 R"
    );
    assert!(
        session
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "前提：整理照旧停着"
    );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    (pool, region, session, devices)
}

/// 停着删意图、删除队列不空时那一次发布的样子：恰好一版、不搬单元、logged_ops 里没有整理意图、照抄删除队列（带着第一个文件），删除续做一步都没走；
/// 会话的现行那一版就是它，分配器这一侧意图连同 R 排他都撤了。`entry` 写进断言消息。
fn assert_the_stopped_drop_carried_the_deletion_queue(
    session: &MountedSession,
    run: &CompactionRun,
    txg_before_the_call: u64,
    entry: &str,
) {
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.deletion_steps_published == 0
            && run.background_compute_of_each_batch.is_empty(),
        "{entry}：停着，删意图排在删除续做之前、续做一步都不走、一批都不开：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "{entry}：恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_call + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && dropped.deleted_inodes == vec![InodeNumber(FIRST_INODE_NUMBER)]
            && current_of(session) == dropped,
        "{entry}：紧接着的就是删意图那一版，不搬单元、logged_ops 里没有整理意图，删除队列照抄（带着第一个文件），会话的现行那一版就是它"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "{entry}：分配器这一侧意图连同 R 排他都撤了"
    );
}

/// 显式入口一趟开头判出停着而现行那一版有活意图、删除队列不空（D26（后台整理与放置回收） 已定项 13 第 4 条）：删意图排在删除续做之前，
/// 单发的那一版照抄删除队列，续做等解除之后。与暂停删意图流
/// （`crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs`）不同处：暂停那一趟删除续做排在批之前、先走空队列，
/// 删意图那一版之前队列已空；这里停着时续做不走，删意图那一版带着没走的队列落盘。
/// 判别力：`crates/mutations.tsv`「一趟开头停着、删除队列不空时不删意图」那一行。
#[test]
fn a_run_left_stopped_with_the_deletion_queue_not_empty_drops_the_intent_before_the_deletion_continuation_and_carries_the_queue(
) {
    let (mut pool, _region, mut session, mut devices) =
        session_left_stopped_with_a_live_intent_and_the_deletion_queue_not_empty(
            "compaction-left-stopped-drops-the-intent-carrying-the-deletion-queue",
        );
    let txg_before_the_run = current_of(&session).root.checkpoint_txg.0;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_the_stopped_drop_carried_the_deletion_queue(
        &session,
        &run,
        txg_before_the_run,
        "显式入口",
    );
}

/// 同一形在后台入口：判出停着而有活意图起跑的那一趟（[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]）排在
/// 「删除队列不空就起跑只走队列」那一判之前，单发的删意图那一版照抄删除队列。
/// 判别力：`crates/mutations.tsv`「后台入口停着、删除队列不空时不起跑删意图」那一行。
#[test]
fn the_background_entry_left_stopped_with_the_deletion_queue_not_empty_drops_the_intent_and_carries_the_queue(
) {
    let (mut pool, _region, mut session, mut devices) =
        session_left_stopped_with_a_live_intent_and_the_deletion_queue_not_empty(
            "compaction-background-entry-left-stopped-drops-the-intent-carrying-the-deletion-queue",
        );
    let txg_before_the_call = current_of(&session).root.checkpoint_txg.0;
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { trigger, run } = call else {
        panic!("停着而现行那一版有活意图：后台入口起跑一趟删它：{call:?}");
    };
    assert_eq!(
        trigger,
        BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,
        "起跑是为了删停着时的活意图，不是删除队列不空"
    );
    assert_the_stopped_drop_carried_the_deletion_queue(
        &session,
        &run,
        txg_before_the_call,
        "后台入口",
    );
}

/// 停着删意图走的入口。封闭集合。
#[derive(Clone, Copy)]
enum EntryLeftStopped {
    /// 显式入口：一趟开头判出停着那一段。
    Explicit,
    /// 后台入口：判出停着而有活意图起跑的那一趟。
    Background,
}

impl EntryLeftStopped {
    fn name(self) -> &'static str {
        match self {
            EntryLeftStopped::Explicit => "explicit",
            EntryLeftStopped::Background => "background",
        }
    }
}

/// 停着删意图的两处入口写的一样：同一段历史（停着、补跑签下的意图活着、预算还剩）造两份池，
/// 一份走显式入口（一趟开头判出停着那一段），一份走后台入口（判出停着而有活意图起跑的那一趟），那一次调用录下的写逐步相同、发成的那一版相同。
/// 两处都只经 `drop_the_live_intent_on_a_stop` 单发一次只删意图的发布，同暂停删意图流里那一步
/// （`crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs` 的 `CompactionIntentDroppedOnAStop`）。
#[test]
fn the_background_entry_left_stopped_writes_exactly_what_the_explicit_entry_left_stopped_writes() {
    let mut operations_and_version_of_each_entry = Vec::new();
    for entry in [EntryLeftStopped::Explicit, EntryLeftStopped::Background] {
        let (pool, _region, mut session, mut devices) =
            session_left_stopped_with_the_intent_a_compensating_run_issued(&format!(
                "compaction-left-stopped-drop-through-the-{}-entry",
                entry.name()
            ));
        session.compaction.publishes_since_the_last_user_change =
            BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
        let start = pool.stream.operation_count();
        let run = match entry {
            EntryLeftStopped::Explicit => session.run_compaction(&mut devices).expect("整理一趟"),
            EntryLeftStopped::Background => match session
                .run_compaction_in_the_background(
                    &mut devices,
                    pacing_with_the_timer_fallback_off(),
                    &CallerWithNoForegroundFsync,
                )
                .expect("后台入口")
            {
                BackgroundCompactionCall::Ran { run, .. } => *run,
                not_started @ BackgroundCompactionCall::NotStarted { .. } => {
                    panic!("停着而有活意图：后台入口起跑：{not_started:?}")
                }
            },
        };
        let [dropped] = <[TransactionOutput; 1]>::try_from(run.versions_published)
            .expect("停着那一趟恰好发一次只删意图的发布");
        assert!(
            dropped.units_relocated_by_this_publish.is_empty()
                && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none(),
            "{}：只删意图、不搬单元",
            entry.name()
        );
        let operations = pool.stream.retained_operations().split_off(start);
        drop(devices);
        operations_and_version_of_each_entry.push((operations, dropped));
    }
    let (background_operations, background_dropped) = operations_and_version_of_each_entry
        .pop()
        .expect("后台入口那一份");
    let (explicit_operations, explicit_dropped) = operations_and_version_of_each_entry
        .pop()
        .expect("显式入口那一份");
    assert!(
        !explicit_operations.is_empty() && explicit_operations == background_operations,
        "两处入口那一次调用录下的写逐步相同（{} 步对 {} 步）",
        explicit_operations.len(),
        background_operations.len()
    );
    assert_eq!(
        explicit_dropped, background_dropped,
        "两处入口发成的删意图那一版相同"
    );
}

/// 只有 R 里有空槽时暂停删意图（D26（后台整理与放置回收） 已定项 2）：签发之后在复用窗口置 0 下推几批，把 R 里搬空的槽当场回收成空槽，回收到每块盘
/// R 里至少 [`FREE_SLOTS_IN_THE_REGION_WANTED`] 个；再在分配器里把 R 之外的每一个空槽标成已分配（不写盘）。整理推一趟：那一批在 R 之外取不到落点、
/// 被空间拒、暂停，单发一次只删意图的发布——它取落点时放开 R，在 R 里落得下，发成；盘上与分配器这一侧意图连同 R 排他都撤了。
/// 判别力：`crates/mutations.tsv`「删意图照旧排除 R」那一行。
#[test]
fn the_drop_on_a_pause_takes_its_placements_in_the_region_once_only_the_region_has_free_slots() {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-drop-on-a-pause-places-in-the-region",
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    let (mut session, mut devices) =
        session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
            session,
            devices,
            region,
            "在 CPU 上",
        );
    let region_slots = region.start_slot().0..region.end_slot().0;
    let free_slots_in_the_region_of_each_device = |allocator: &PoolAllocator| -> Vec<usize> {
        allocator
            .devices
            .iter()
            .map(|device_map| {
                region_slots
                    .clone()
                    .filter(|slot| device_map.is_free(SlotNumber(*slot)))
                    .count()
            })
            .collect()
    };
    session
        .allocator
        .set_reuse_window(ReuseWindow::ForcedToZero);
    for _batch_index in 0..BATCHES_FREEING_THE_REGION_AT_MOST {
        if free_slots_in_the_region_of_each_device(&session.allocator)
            .iter()
            .all(|free| *free >= FREE_SLOTS_IN_THE_REGION_WANTED)
        {
            break;
        }
        let freeing = run_one_compaction_publish(&mut session, &mut devices);
        assert!(
            freeing.batches_published == 1
                && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                    .is_some_and(|(_, intent)| intent.region == region),
            "前提：回收 R 里空槽的那几批各搬一批、意图照抄：{:?}",
            freeing.outcome
        );
    }
    session
        .allocator
        .set_reuse_window(ReuseWindow::GatedByTheRollbackFloor);
    for device_map in &mut session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let end = start + device_map.unit_area_slots();
        let mut slot = start;
        while slot < end {
            if region_slots.contains(&slot) || !device_map.is_free(SlotNumber(slot)) {
                slot += 1;
                continue;
            }
            let run_start = slot;
            while slot < end
                && !region_slots.contains(&slot)
                && device_map.is_free(SlotNumber(slot))
            {
                slot += 1;
            }
            device_map.mark_allocated(SlotNumber(run_start), slot - run_start);
        }
    }
    let free_slots_outside_the_region: Vec<usize> = session
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            let start = device_map.unit_area_start().slot().0;
            (start..start + device_map.unit_area_slots())
                .filter(|slot| {
                    !region_slots.contains(slot) && device_map.is_free(SlotNumber(*slot))
                })
                .count()
        })
        .collect();
    assert!(
        free_slots_in_the_region_of_each_device(&session.allocator)
            .iter()
            .all(|free| *free >= FREE_SLOTS_IN_THE_REGION_WANTED)
            && free_slots_outside_the_region.iter().all(|free| *free == 0)
            && session.allocator.region_excluded_from_placement() == Some(region),
        "前提：R 里每块盘至少 {FREE_SLOTS_IN_THE_REGION_WANTED} 个空槽、R 之外一个都没有，意图活着、R 排他着（R 之外 {free_slots_outside_the_region:?}）"
    );
    session.compaction.publishes_since_the_last_user_change = 0;
    let txg_before_the_run = current_of(&session).root.checkpoint_txg;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PausedShortOfSpace)
        ),
        "那一批在 R 之外取不到落点：暂停，删意图那一次发成：{:?}",
        run.outcome
    );
    let [dropped] = run.versions_published.as_slice() else {
        panic!(
            "暂停那一趟恰好发一次只删意图的发布，实际发了 {} 次",
            run.versions_published.len()
        );
    };
    assert!(
        dropped.root.checkpoint_txg.0 == txg_before_the_run.0 + 1
            && dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && current_of(&session) == dropped,
        "紧接着的就是删意图那一版：不搬单元、logged_ops 里没有整理意图"
    );
    assert!(
        dropped.allocation_records.iter().any(|record| {
            record.device == region.device()
                && !record.is_released
                && record.generation == dropped.root.checkpoint_txg
                && region.contains_slot(record.slot)
        }),
        "删意图那一次自己的新落点落在 R 里（R 之外一个空槽都没有）"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && session.allocator.region_excluded_from_placement().is_none(),
        "分配器这一侧意图连同 R 排他一起撤了"
    );
}

/// 停着、不超过阈值（取默认 85%），整理的批大小用产品那一种，删除队列非空（停着时会话 unlink 第一个文件、再挂着回退到 unlink 意图那一版，
/// T4-1 那一条路）：一趟开头那一判排在删除续做之前，这一趟报的是出错停着；删除续做没走，队列照旧带着第一个文件；
/// 一个写都不发，盘上逐字节不变。
#[test]
fn a_run_left_stopped_at_its_start_reports_the_stop_before_judging_the_batch_size_or_continuing_the_deletion_queue(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-left-stopped-at-the-start-before-the-batch-size",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    bring_the_deletion_queue_back_by_rolling_back_an_unlink(&mut session, &mut devices);
    let CompactionSwitches {
        batch_size: _,
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = session.allocator.compaction().switches;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionSwitches::PRODUCT.batch_size,
            forced_region,
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.versions_published.is_empty()
            && run.background_compute_of_each_batch.is_empty(),
        "报的是整理先前出错停着、一个写都不发：{:?}",
        run.outcome
    );
    assert_eq!(
        current_of(&session).deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "停着时删除续做不走：队列照旧带着第一个文件"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变"
    );
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

/// 「任一块盘超过就补跑」，最满的是第二块盘：停着、阈值 3%，把第二块盘的占用抬过 3%、第一块盘（R 那块）不越线——会话读得出最满的是
/// 第二块盘；显式入口照常推，每一批在 CPU 上补跑，记账里的读数报的是第二块盘。只看第一块盘（「永远取第一块」那一种改坏）就照旧停着。
#[test]
fn a_compaction_stopped_after_a_gpu_failure_runs_on_the_cpu_when_the_second_device_is_the_one_above_the_threshold(
) {
    let threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 3,
    };
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensated-when-the-second-device-is-above-the-threshold",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(threshold),
        );
    allocate_the_tail_of_a_device_just_above_percent(
        &mut session,
        DeviceIdentity(1),
        threshold.allocated_percent_of_the_capacity,
    );
    let every_device = every_device_counted_by_the_test(&session);
    let [region_device, second_device] = every_device.as_slice() else {
        panic!("池是两块盘：{every_device:?}");
    };
    assert!(
        second_device.device == DeviceIdentity(1)
            && share_is_above_percent(
                second_device.allocated_bytes,
                second_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            )
            && !share_is_above_percent(
                region_device.allocated_bytes,
                region_device.capacity_bytes,
                threshold.allocated_percent_of_the_capacity
            ),
        "第二块盘越过 3%、第一块盘不越：{every_device:?}"
    );
    let reading_before_the_run = session.gc_cpu_compensation_reading();
    assert_eq!(
        reading_before_the_run,
        GcCpuCompensationReading {
            fullest_device: *second_device,
            threshold,
        },
        "会话读得出最满的是第二块盘、它的已分配字节与容量"
    );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget })
                if budget == PUBLISHES_OF_THE_COMPENSATING_RUN
        ) && run.batches_published > 0,
        "第二块盘越线就补跑：照常推、搬了批、推到预算用完（{:?}）",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty()
            && run.background_compute_of_each_batch.iter().all(|batch| matches!(
                batch.path_taken,
                BackgroundComputePathTaken::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading)
                ) if reading.fullest_device.device == DeviceIdentity(1)
                    && reading.is_the_allocated_share_above_the_threshold()
            )),
        "每一批在 CPU 上补跑，读数报的是越线的第二块盘：{:?}",
        run.background_compute_of_each_batch
    );
}

/// 「补跑途中降到不超过，下一批起照旧停着」，不经只供测试的补跑开关：停着、阈值 3%，第一块盘（R 那块）的占用抬过 3%；
/// 复用窗口压成 0（只供测试，释放那一刻就回收，用户删文件当场让已分配降下来）。先补跑两趟，各推一次（预算 1）：头一趟在同一个 R 上从头签发；
/// 签发那一版回收了 defer 队列里待回收的槽、第一块盘可能落回 3% 以内，用例在已标的那一段前面再标几槽、抬回刚好越线，第二趟补跑一批。
/// 再由用户真删第一个文件——读数降到不超过 3%；下一趟开头就照旧停着：一批都不开，只单发一次删掉补跑签下的意图的发布，补跑的批数照旧是 2。
/// 补跑过一批就一直补跑下去（攻方锁存那一种改坏）这里就红。
#[test]
fn a_compensating_compaction_stays_stopped_from_the_next_batch_once_a_user_deletion_brings_the_reading_down(
) {
    let threshold = BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 3,
    };
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensation-ended-by-a-user-deletion",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(threshold),
        );
    session
        .allocator
        .set_reuse_window(ReuseWindow::ForcedToZero);
    let slots_marked_at_the_tail = allocate_the_tail_of_a_device_just_above_percent(
        &mut session,
        DeviceIdentity(0),
        threshold.allocated_percent_of_the_capacity,
    );
    assert!(
        session
            .gc_cpu_compensation_reading()
            .is_the_allocated_share_above_the_threshold(),
        "第一块盘越线：停着时补跑"
    );
    let issuing = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        matches!(
            issuing.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
        ) && issuing.batches_published == 0
            && issuing.versions_published.len() == 1
            && session
                .allocator
                .compaction()
                .live_intent()
                .is_some_and(|live| live.intent.region == region),
        "补跑的头一趟在同一个 R 上从头签发：{:?}，发了 {} 版",
        issuing.outcome,
        issuing.versions_published.len()
    );
    let reading_after_the_issue = session.gc_cpu_compensation_reading();
    let slots_marked_after_the_issue =
        allocate_slots_below_the_marked_tail_until_just_above_percent(
            &mut session,
            DeviceIdentity(0),
            threshold.allocated_percent_of_the_capacity,
            slots_marked_at_the_tail,
        );
    assert!(
        session
            .gc_cpu_compensation_reading()
            .is_the_allocated_share_above_the_threshold(),
        "签发之后第一块盘抬回刚好越线（签发之后的读数 {reading_after_the_issue:?}，又标了 {slots_marked_after_the_issue} 槽）"
    );
    let compensating = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        matches!(
            compensating.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
        ) && compensating.batches_published == 1,
        "补跑的第二趟发了一批：{:?}",
        compensating.outcome
    );
    assert_eq!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .iter()
            .map(|stopped| stopped.batches_compensated_on_the_cpu)
            .collect::<Vec<u64>>(),
        vec![2],
        "补跑了签发那一次与一批"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
        )
        .expect("停着时用户照常删第一个文件");
    let reading_after_the_deletion = session.gc_cpu_compensation_reading();
    assert!(
        !reading_after_the_deletion.is_the_allocated_share_above_the_threshold(),
        "删了文件、当场回收：读数降到不超过 3%（{reading_after_the_deletion:?}）"
    );
    session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && run.batches_published == 0
            && run.background_compute_of_each_batch.is_empty()
            && run.versions_published.len() == 1
            && run.versions_published.iter().all(|dropped| {
                dropped.units_relocated_by_this_publish.is_empty()
                    && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            }),
        "降到不超过之后下一批起照旧停着：一批都不开，只单发一次删意图的发布：{:?}，发了 {} 版",
        run.outcome,
        run.versions_published.len()
    );
    assert!(
        session.allocator.compaction().live_intent().is_none(),
        "补跑签下的那一条连同 R 排他一起撤了"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error,
            batches_compensated_on_the_cpu: 2,
        }],
        "整理照旧停着、报的还是那个错，补跑的批数照旧是 2"
    );
}

/// 三次失败后每次后台整理被叫起都报、不管后台有没有 gpu 可用，经返回值里必须处理的报警交回：
/// 出错停下（那一趟照报、不数，报 0）之后连着叫起三次——显式入口一趟在 CPU 上补跑一批（预算 1：从头签发那一次；
/// 阈值 0%，这一批的钩子里调只供测试的开关，之后都当不超过阈值）、显式入口一趟照旧停着（补跑那一趟扣完了上一次用户改动给的那一份 `B_gc`，
/// 显式入口每一趟的预算照旧至少 1：一趟开头单发一次只删意图的发布丢掉补跑签下的意图）、后台入口一次照旧停着——三次都交回报警，逐字段：
/// 甲段那张卡、停下的那一个错、叫起 1、2、3 次、补跑 1 批；
/// 一次通过校验的切换（选 `cpu`）解除停下之后，显式入口再叫起不交报警。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn every_wake_while_the_compaction_stays_stopped_after_a_gpu_failure_returns_an_alarm_until_a_switch_lifts_the_stop(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-on-every-wake-while-stopped",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
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
    assert_eq!(
        compensating.background_compute_of_each_batch.len(),
        1,
        "超过阈值 0%：在 CPU 上补跑了一批（{:?}）",
        compensating.outcome
    );
    assert_eq!(
        compensating.alarm_of_the_compaction_stopped_after_a_compute_error,
        Some(alarm_of_the_stop(&error, 1, 1)),
        "补跑了一批的那一次叫起照报"
    );
    let left_stopped = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            left_stopped.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && left_stopped.batches_published == 0
            && left_stopped.background_compute_of_each_batch.is_empty(),
        "当作不超过阈值：照旧停着、一批都不开（{:?}）",
        left_stopped.outcome
    );
    let [dropped] = left_stopped.versions_published.as_slice() else {
        panic!(
            "显式入口每一趟的预算至少 1：补跑那一趟扣完了上一次用户改动给的那一份，这一趟开头照样恰好发一次只删意图的发布，实际发了 {} 次",
            left_stopped.versions_published.len()
        );
    };
    assert!(
        dropped.units_relocated_by_this_publish.is_empty()
            && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            && session.allocator.compaction().live_intent().is_none(),
        "删的是补跑签下的那一条：不搬单元、logged_ops 里没有整理意图，分配器这一侧意图连同 R 排他一起撤了"
    );
    assert_eq!(
        left_stopped.alarm_of_the_compaction_stopped_after_a_compute_error,
        Some(alarm_of_the_stop(&error, 2, 1)),
        "显式入口照旧停着的那一次叫起照报"
    );
    let background_call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    assert!(
        matches!(
            background_call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "后台入口不起跑：{background_call:?}"
    );
    assert_eq!(
        background_call.alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 3, 1)),
        "后台入口没起跑的那一次叫起照报"
    );
    session
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu：切换收下，解除停下");
    let after_the_switch = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(
        after_the_switch.alarm_of_the_compaction_stopped_after_a_compute_error, None,
        "切换解除停下之后再叫起：不报警（{:?}）",
        after_the_switch.outcome
    );
}

/// 攒批的会话（`MountedSessionWithPendingChanges`）的后台入口同样每次叫起都报：停着、阈值取默认 85%（不超过），
/// 没起跑的那一次交回报警——甲段那张卡、停下的那一个错、叫起 1 次（停下那一趟不数）、补跑 0 批。
#[test]
fn the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_while_it_does_not_start(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-through-a-session-with-pending-changes-not-started",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (session, mut devices, error) = gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
        &mut pool,
        region,
        &gpu_compute_kept_by_the_test,
        &gpu_compute,
        &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
    );
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    let after_the_pending_changes = session_with_pending_changes
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    assert!(
        matches!(
            after_the_pending_changes.call,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::BackgroundTaskStoppedAfterAComputeError,
                ..
            }
        ),
        "停着而不超过阈值：不起跑（{:?}）",
        after_the_pending_changes.call
    );
    assert_eq!(
        after_the_pending_changes
            .call
            .alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 1, 0)),
        "没起跑的那一次叫起照报"
    );
}

/// 同上，阈值 0%（超过）、用户改动之后记下「该跑了」：攒批的会话的后台入口起跑，从头签发那一次与那一批都在 CPU 上补跑，起跑了的那一次也交回报警
/// （叫起 1 次、补跑 2 批）。
#[test]
fn the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_when_it_runs_a_compensating_batch(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-through-a-session-with-pending-changes-compensating",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    let after_the_pending_changes = session_with_pending_changes
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    pool.devices = Some(devices);
    let BackgroundCompactionCall::Ran { run, .. } = &after_the_pending_changes.call else {
        panic!(
            "停着而超过阈值：起跑了（{:?}）",
            after_the_pending_changes.call
        );
    };
    assert!(
        run.batches_published == 1 && run.background_compute_of_each_batch.len() == 2,
        "用户改动之后那一趟推至多一批：出错停下时删了意图，先从头签发、再一批，各一份记账（{:?}）",
        run.outcome
    );
    assert_eq!(
        after_the_pending_changes
            .call
            .alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 1, 2)),
        "起跑了、补跑了签发那一次与一批的那一次叫起照报"
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

/// 判决 m4-gpugc-code-r2 第三节第 3 条：攒批的会话的后台入口判过起跑、排空待发被拒的那一次也是一次叫起（不是入口在任何读写之前就拒的那一种），
/// 照数一次叫起、拒绝里带着这一次的报警。停着、阈值 0%（超过）：出错停下的那一趟报 0；后台入口一次没起跑（没有「该跑了」、定时兜底关着）报 1；
/// 建一个 inode 记下「该跑了」，攒批的会话收下一次建 inode 进待发，后台入口判过起跑、排空那一次写报 I/O 错被拒——拒绝里的报警：
/// 甲段那张卡、停下的那一个错、叫起接着数到 2、补跑 0 批（那一趟整理没跑）。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_when_draining_the_pending_changes_is_refused(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-through-a-session-with-pending-changes-drain-refused",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices, error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    let not_started = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    assert!(
        matches!(
            not_started,
            BackgroundCompactionCall::NotStarted {
                reason: BackgroundCompactionNotStarted::NothingDueAndTheTimerFallbackOff,
                ..
            }
        ),
        "没有「该跑了」、定时兜底关着：不起跑（{not_started:?}）"
    );
    assert_eq!(
        not_started.alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 1, 0)),
        "停下之后头一次叫起报 1"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    let mut session_with_pending_changes =
        MountedSessionWithPendingChanges::of_the_session(session);
    session_with_pending_changes
        .accept(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3,
            },
            CallerMonotonicClockReading(Duration::ZERO),
        )
        .expect("建一个 inode 收进待发");
    assert_eq!(
        session_with_pending_changes.pending_changes_accepted(),
        1,
        "收进待发、两个阈值都没到：没发布"
    );
    let mut devices_failing_every_write: Vec<(DeviceIdentity, DeviceFailingEveryWrite)> = devices
        .into_iter()
        .map(|(identity, device)| (identity, DeviceFailingEveryWrite { inner: device }))
        .collect();
    let drained = session_with_pending_changes.run_compaction_in_the_background(
        &mut devices_failing_every_write,
        pacing_with_the_timer_fallback_off(),
        &CallerWithNoForegroundFsync,
    );
    pool.devices = Some(
        devices_failing_every_write
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    match drained {
        Err(
            BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused {
                refusal: _,
                alarm_of_the_compaction_stopped_after_a_compute_error,
            },
        ) => assert_eq!(
            alarm_of_the_compaction_stopped_after_a_compute_error,
            Some(alarm_of_the_stop(&error, 2, 0)),
            "排空被拒的那一次叫起照数、照报：接着数到 2"
        ),
        Err(BackgroundCompactionAfterThePendingChangesRefused::CompactionRefused(refused)) => {
            panic!("后台入口自己在任何读写之前拒了：{refused:?}")
        }
        Ok(after_the_pending_changes) => {
            panic!("排空那一次写报错应当被拒：{after_the_pending_changes:?}")
        }
    }
}

/// 在后台入口上：在 GPU 上签发意图之后注入甲段出错，建一个 inode 记下「该跑了」，
/// 后台入口判过起跑、那一批出错放弃、整理停下——这一次叫起照报、不数这一趟（叫起 0 次）；之后后台入口连着叫起三次（停着、阈值取默认 85% 不超过，
/// 都不起跑）报 1、2、3。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn a_stop_in_a_wake_of_the_background_entry_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-stopped-in-a-wake-of-the-background-entry",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) = gpu_session_with_the_intent_issued_on_the_gpu(
        &mut pool,
        region,
        &gpu_compute,
        &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
    );
    fail_the_unit_checksum_verification_on_the_card_with_eight_gibibytes(
        &gpu_compute_kept_by_the_test,
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("建一个 inode");
    let stopping_call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    let BackgroundCompactionCall::Ran { run, .. } = &stopping_call else {
        panic!("用户改动之后记下了「该跑了」：起跑了（{stopping_call:?}）");
    };
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ),
        "甲段出错：这一批放弃、整理停下（{:?}）",
        run.outcome
    );
    let error = error_of_the_failing_unit_checksum_verification();
    assert_eq!(
        stopping_call.alarm_of_the_compaction_stopped_after_a_compute_error(),
        Some(&alarm_of_the_stop(&error, 0, 0)),
        "出错停下的那一次叫起照报、不数这一趟"
    );
    let wakes_after_the_stop: Vec<Option<u64>> = (0..3)
        .map(|_wake| {
            session
                .run_compaction_in_the_background(
                    &mut devices,
                    pacing_with_the_timer_fallback_off(),
                    &CallerWithNoForegroundFsync,
                )
                .expect("后台入口")
                .alarm_of_the_compaction_stopped_after_a_compute_error()
                .map(|alarm| alarm.wakes_while_stopped)
        })
        .collect();
    pool.devices = Some(devices);
    assert_eq!(
        wakes_after_the_stop,
        vec![Some(1), Some(2), Some(3)],
        "下一次叫起才是第 1 次"
    );
}

/// 在可写挂载续做上：另一次挂载里在 GPU 上签发意图之后丢掉会话
/// （意图留在盘上），注入甲段出错，再挂选 `gpu`、经只供测试的入口装上同一组整理开关——挂载续做推那一条意图，头一批在 GPU 上出错放弃、整理停下，
/// 续做只单发一次删那一条意图的发布、挂上之后盘上没有活意图；续做不是叫起，不交报警。挂上之后显式入口连着叫起三次（停着、阈值取默认 85% 不超过）报 1、2、3，
/// 与在叫起里停下那一种（`a_stop_in_a_wake_of_the_background_entry_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three`：
/// 停下那一次叫起报 0、下一次才是 1）同一个数法：停下之后的头一次叫起是第 1 次。
// harness-test-granularity:one-scenario 同一个会话上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
#[test]
fn a_stop_in_the_writable_mount_continuation_is_followed_by_three_wakes_counting_one_two_three() {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-alarm-stopped-in-the-writable-mount-continuation",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (session_of_the_earlier_mount, devices_of_the_earlier_mount) =
        gpu_session_with_the_intent_issued_on_the_gpu(
            &mut pool,
            region,
            &gpu_compute,
            &CompensationThresholdAtTheMount::NotGivenTakingTheDefault,
        );
    drop(session_of_the_earlier_mount);
    pool.devices = Some(devices_of_the_earlier_mount);
    fail_the_unit_checksum_verification_on_the_card_with_eight_gibibytes(
        &gpu_compute_kept_by_the_test,
    );
    let mut devices = pool.reopen_recorded();
    let mounted =
        mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches(
            &parameters(),
            &mut devices,
            admissible_gpu_settings_naming_both_cards(),
            Some(gpu_compute.clone()),
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            compaction_switches_of_the_test(region),
        )
        .expect("全合法的 gpu 设置连同实现、默认阈值：挂载照常做成（续做自己不让挂载失败）");
    let (output, mut session) = MountedSession::of_the_mount(mounted);
    let continuation = output
        .compaction_continuation
        .run
        .as_ref()
        .expect("盘上有一条未完成的意图、删除队列是空的：续做推了它");
    assert!(
        matches!(
            continuation.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && continuation.batches_published == 0
            && continuation.versions_published.len() == 1
            && continuation.versions_published.iter().all(|dropped| {
                dropped.units_relocated_by_this_publish.is_empty()
                    && live_compaction_intent_of_the_entries(&dropped.logged_operations).is_none()
            }),
        "续做的头一批在 GPU 上算甲段出错：这一批放弃、一个单元都没搬，单发一次删意图的发布，整理停下（{:?}）",
        continuation.outcome
    );
    assert!(
        session.allocator.compaction().live_intent().is_none()
            && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_none(),
        "挂上之后盘上与分配器这一侧都没有活意图"
    );
    assert_eq!(
        continuation.alarm_of_the_compaction_stopped_after_a_compute_error, None,
        "续做不是叫起：不交报警"
    );
    let error = error_of_the_failing_unit_checksum_verification();
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error: error.clone(),
            batches_compensated_on_the_cpu: 0,
        }],
        "挂上之后整理停着、报出甲段那张卡的错，还没补跑过"
    );
    let alarms_of_the_next_three_wakes: Vec<
        Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
    > = (0..3)
        .map(|_wake| {
            session
                .run_compaction(&mut devices)
                .expect("整理一趟")
                .alarm_of_the_compaction_stopped_after_a_compute_error
        })
        .collect();
    pool.devices = Some(devices);
    assert_eq!(
        alarms_of_the_next_three_wakes,
        vec![
            Some(alarm_of_the_stop(&error, 1, 0)),
            Some(alarm_of_the_stop(&error, 2, 0)),
            Some(alarm_of_the_stop(&error, 3, 0)),
        ],
        "停在续做里：停下之后的头一次叫起是第 1 次"
    );
}

/// D26（后台整理与放置回收） 已定项 13 第 4 条「占用超过阈值在 CPU 上补跑」与七·3 验收「结果与 CPU 模式相同」在替身上：
/// 同样建、同样写的两个池，一个选 `gpu` 在 GPU 上签发意图、注入甲段出错停下（那一批不搬单元、单发一次删意图的发布）、阈值 0% 下在 CPU 上
/// 补跑一趟（`B_gc` 压到 [`PUBLISHES_OF_THE_COMPENSATING_RUN`]，从头签发再推）；另一个选 `cpu`、同一组整理开关签发意图，经只供测试的钩子
/// 在下一批注入一次出错（同样单发一次删意图的发布、整理停下），选 `cpu` 的切换解除停下，再推同样预算的一趟。补跑那一趟每一批都记成停着时补跑，
/// 与 `cpu` 那一趟停在同一条上、发的每一版逐项相同、搬了一样多批，两条录制流逐步相同（盘上逐字节相同）。
#[test]
fn a_compensating_run_on_the_cpu_while_stopped_after_a_gpu_failure_ends_exactly_as_the_cpu_mode() {
    let (mut gpu_pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensating-run-against-the-cpu-mode-gpu",
    );
    let (mut cpu_pool, region_of_the_cpu_pool) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-compensating-run-against-the-cpu-mode-cpu",
    );
    assert_eq!(
        (gpu_pool.retained_operations(), region),
        (cpu_pool.retained_operations(), region_of_the_cpu_pool),
        "两个池走同一串操作：整理之前字节相同、数据单元最多的是同一段"
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut gpu_session, mut gpu_devices, _error) =
        gpu_session_stopped_after_a_gpu_failure_dropping_the_intent(
            &mut gpu_pool,
            region,
            &gpu_compute_kept_by_the_test,
            &gpu_compute,
            &CompensationThresholdAtTheMount::Given(THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE),
        );
    gpu_session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let compensating = gpu_session
        .run_compaction(&mut gpu_devices)
        .expect("整理一趟");
    gpu_pool.devices = Some(gpu_devices);
    let mut cpu_devices = cpu_pool.reopen_recorded();
    let cpu_mounted = mount_writable(&parameters(), &mut cpu_devices).expect("可写挂载，选 cpu");
    let (_, cpu_session) = MountedSession::of_the_mount(cpu_mounted);
    let (mut cpu_session, mut cpu_devices) =
        session_with_the_intent_issued_under_the_compaction_switches_of_the_test(
            cpu_session,
            cpu_devices,
            region,
            "在 CPU 上",
        );
    cpu_session.compaction.publishes_since_the_last_user_change = BUDGET_GIVEN_BY_A_USER_CHANGE - 1;
    let abandoning_on_the_cpu = cpu_session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut cpu_devices,
            &mut |background_compute| {
                background_compute.report_a_compute_error_in_the_batch_under_way_for_tests();
            },
        )
        .expect("整理一趟");
    assert!(
        matches!(
            abandoning_on_the_cpu.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && abandoning_on_the_cpu.versions_published.len() == 1,
        "选 cpu 那一边注入的出错同样放弃那一批、单发一次删意图的发布：{:?}",
        abandoning_on_the_cpu.outcome
    );
    cpu_session
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu：切换收下，解除停下");
    cpu_session.compaction.publishes_since_the_last_user_change =
        BUDGET_GIVEN_BY_A_USER_CHANGE - PUBLISHES_OF_THE_COMPENSATING_RUN;
    let cpu_run = cpu_session
        .run_compaction(&mut cpu_devices)
        .expect("整理一趟");
    cpu_pool.devices = Some(cpu_devices);
    assert!(
        compensating.batches_published > 0
            && compensating
                .background_compute_of_each_batch
                .iter()
                .all(|batch| matches!(
                    batch.path_taken,
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    )
                )),
        "停着而超过阈值 0%：搬了批，每一批都记成停着时在 CPU 上补跑（{:?}）",
        compensating.outcome
    );
    assert_eq!(
        format!("{:?}", compensating.outcome),
        format!("{:?}", cpu_run.outcome),
        "停在同一条上"
    );
    assert_eq!(
        compensating.versions_published, cpu_run.versions_published,
        "发的每一版逐项相同"
    );
    assert_eq!(
        compensating.batches_published, cpu_run.batches_published,
        "搬了一样多批"
    );
    assert_eq!(
        gpu_pool.retained_operations(),
        cpu_pool.retained_operations(),
        "两条录制流逐步相同：盘上逐字节相同"
    );
}
