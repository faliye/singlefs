//! 整理选 `gpu` 时的一批（D26（后台整理与放置回收） 已定项 13 第 1、4、8 条）：甲（扫描里的校验和核对）、乙（区域扫描里逐条记录的单元头解析与映射反查）、
//! 丙（选段打分）三段经挂载时递进来的那一份 GPU 实现算，发布那一半照旧在 CPU 上、照发布路径逐条核计划与读盘核。
//! 实现用一份背后照核心层 CPU 写法算的假实现（`common_background_gpu_compute`），可注入出错与不确定：
//! 不注入时一趟整理的结局与选 `cpu` 时逐项相同（发的每一版、盘上的录制流、停在哪、观测）；某一段交回出错时这一批放弃、整理停下、盘上不动、
//! 不换回 CPU；两份核判得不一样的记录这一批不搬、下一批照判；判得不一样的段这一批不当选段候选；R 里只剩判得不一样的记录时这一批不发、意图照留；
//! 甲段判出整单元 CRC 对不上的照常交 CPU 发布路径、只计数。注入每一种出错，整理停下报的错里那一段、那张卡、那个原因逐项对得上。
//! 设置（第 1–3 条）：挂载那一刻核心层把校验过的那一组交给实现；一批读完设置之后请求的切换（缩卡、缩显存上限、改等的上限），实现在下一批开头、
//! 开算之前收到，请求切换的那一批算的每一段照的都是旧的那一组。
//! 池：`common::build_pool` 起，另一次挂载里顺序写第一个文件 40 个数据单元（那一次挂载的整理照产品开关，不起跑），重挂；整理一批至多搬 2 个、
//! 起跑强制开着。两个池走同一串操作时字节相同（逐项比之前先核）。
mod common;
mod common_background_gpu_compute;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use common::{
    build_pool, disk_snapshot, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, card_named,
    device_lost_on_the_card_with_eight_gibibytes, shared_cpu_backed_gpu_compute,
    CpuBackedBackgroundGpuComputeForTests,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    admit_gpu_settings_against_the_inventory, AdmittedGpuSettings, BackgroundComputeBackend,
    BackgroundComputeError, BackgroundComputeOfThisMount, BackgroundComputePathTaken,
    BackgroundComputeSettings, BackgroundGpuComputeFailed, BackgroundGpuComputeHandle,
    BackgroundTask, BackgroundTaskAfterTheComputeError, BackgroundTaskStoppedAfterAComputeError,
    ClusterSegmentEntryKey, GpuComputedSegment, GpuEntryCounts, GpuFailureOnTheDevice,
    GpuMemoryInBytes, GpuWorkOfTheBatch, InputNotSupportedOnTheGpu, RegionRecordEntryKey,
    SubmissionWaitedForPastTheLimit, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionRunForcing, CompactionRunStop,
    CompactionSwitches,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::{mount_writable, mount_writable_with_background_compute_settings};
use singlefs_core::mounted_session::{
    CompactionRun, CompactionRunOutcome, MountedSession, UserChange,
};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;

/// 另一次挂载里顺序写第一个文件的数据单元数（同 `compaction_off_the_calling_thread_and_admission.rs` 闲置池那一条：数据单元最多的那一段里有 29 个）。
const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;
/// 整理一批至多搬几个（只供测试的开关；产品路径批大小条款没写）。
const UNITS_PER_BATCH: usize = 2;
/// mkfs 写的每区根槽数 S（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`）。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
/// 显式入口一趟只发一次：「已经过去的发布数」取 `3S − 5`，`B_gc` = 1。
const PUBLISHES_PASSED_LEAVING_A_BUDGET_OF_ONE: u64 = 3 * ROOT_RING_SLOTS_PER_REGION - 4 - 1;

/// 重挂时这一次挂载的后台计算：`cpu`（不给实现），或 `gpu` 连同递进来的那一份实现。封闭集合。
enum BackgroundComputeOfTheRemount<'handle> {
    Cpu,
    GpuThrough(&'handle BackgroundGpuComputeHandle),
}

/// 另一次挂载里顺序写第一个文件 [`DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT`] 个数据单元，那一次的会话丢掉（「进程退出」）。
/// 交回池与那一次写之后第一块盘上两槽的数据单元最多的那一段（重挂不搬数据单元，重挂之后它还是那一段）。
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
    let fullest_segment = segment_with_the_most_data_units(&session);
    drop(session);
    pool.devices = Some(devices);
    (pool, fullest_segment)
}

/// 重挂、开会话，装上整理只供测试的开关：一批至多 [`UNITS_PER_BATCH`] 个、起跑强制开着、选段照 `forced_region`（`None` 是 greedy 打分选）。
fn remount(
    pool: &mut BuiltPool,
    background_compute: &BackgroundComputeOfTheRemount<'_>,
    forced_region: Option<CompactionRegion>,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = match background_compute {
        BackgroundComputeOfTheRemount::Cpu => {
            mount_writable(&parameters(), &mut devices).expect("可写挂载")
        }
        BackgroundComputeOfTheRemount::GpuThrough(gpu_compute) => {
            mount_writable_with_background_compute_settings(
                &parameters(),
                &mut devices,
                admissible_gpu_settings_naming_both_cards(),
                Some((*gpu_compute).clone()),
            )
            .expect("全合法的 gpu 设置连同实现：挂载照常做成")
        }
    };
    let (_, mut session) = MountedSession::of_the_mount(mounted);
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
                CompactionBatchUnits::new(UNITS_PER_BATCH).expect("至少 1"),
            ),
            forced_region,
            run_forcing: CompactionRunForcing::ForcedToStartForTests,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
    (session, devices)
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

/// 第一块盘上两槽的数据单元最多的那一段（按记录数）。
fn segment_with_the_most_data_units(session: &MountedSession) -> CompactionRegion {
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
    let (fullest_segment, _data_units) = data_units_by_segment
        .into_iter()
        .max_by_key(|(_, data_units)| *data_units)
        .expect("写了数据单元：至少一段里有");
    CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(fullest_segment))
        .expect("按 64 取整的段起点")
}

/// R 那块盘上起点在 R 里、未释放的记录，按槽升序。
fn unreleased_records_in(
    session: &MountedSession,
    region: CompactionRegion,
) -> Vec<AllocationRecord> {
    let mut records: Vec<AllocationRecord> = current_of(session)
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .copied()
        .collect();
    records.sort_by_key(|record| record.slot);
    records
}

/// 这一份记录此刻还在、未释放（没被搬走）。
fn is_still_unreleased(session: &MountedSession, record: &AllocationRecord) -> bool {
    current_of(session)
        .allocation_records
        .iter()
        .any(|now| now.device == record.device && now.slot == record.slot && !now.is_released)
}

/// 显式入口推一次（预算压到 1）。
fn run_one_compaction_publish(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
) -> CompactionRun {
    session.compaction.publishes_since_the_last_user_change =
        PUBLISHES_PASSED_LEAVING_A_BUDGET_OF_ONE;
    session.run_compaction(devices).expect("整理一趟")
}

/// 这一趟唯一那一批的 GPU 记账。
fn gpu_work_of_the_only_batch(run: &CompactionRun) -> GpuWorkOfTheBatch {
    let [batch] = run.background_compute_of_each_batch.as_slice() else {
        panic!(
            "预算 1：这一趟恰好一批，{:?}",
            run.background_compute_of_each_batch
        );
    };
    let BackgroundComputePathTaken::Gpu(gpu_work) = batch.path_taken else {
        panic!("选 gpu 的一批走 GPU：{batch:?}");
    };
    gpu_work
}

/// 选段固定在 `region` 上的 gpu 会话，先发签发那一次（预算 1；签发之前强制选的那一段也照 GPU 路径扫过）：交回会话与盘。
fn gpu_session_with_the_intent_issued_on(
    pool: &mut BuiltPool,
    region: CompactionRegion,
    gpu_compute: &BackgroundGpuComputeHandle,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let (mut session, mut devices) = remount(
        pool,
        &BackgroundComputeOfTheRemount::GpuThrough(gpu_compute),
        Some(region),
    );
    let issuing = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        issuing.versions_published.len() == 1
            && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_some_and(|(_, intent)| intent.region == region),
        "只发了签发那一次，意图在强制选的那一段上：{:?}",
        issuing.outcome
    );
    (session, devices)
}

/// 不注入故障（D26 已定项 13「两份实现在测试里对拍」）：同样建、同样写的两个池，一个重挂选 `cpu`、一个选 `gpu` 递进背后照 CPU 写法算的实现，
/// greedy 选段推一趟整理到停。两趟发的每一版、两条录制流、停在哪、搬了几批、整理观测逐项相同；`gpu` 那一趟每一批都走 GPU，三段都经实现算、全部确认。
#[test]
fn the_gpu_path_through_a_cpu_backed_implementation_ends_a_compaction_run_exactly_as_the_cpu_path()
{
    let (mut cpu_pool, _) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-path-equivalence-cpu");
    let (mut gpu_pool, _) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-path-equivalence-gpu");
    assert_eq!(
        cpu_pool.retained_operations(),
        gpu_pool.retained_operations(),
        "两个池走同一串操作：整理之前字节相同"
    );
    let (mut cpu_session, mut cpu_devices) =
        remount(&mut cpu_pool, &BackgroundComputeOfTheRemount::Cpu, None);
    let cpu_run = cpu_session
        .run_compaction(&mut cpu_devices)
        .expect("整理一趟");
    cpu_pool.devices = Some(cpu_devices);
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut gpu_session, mut gpu_devices) = remount(
        &mut gpu_pool,
        &BackgroundComputeOfTheRemount::GpuThrough(&gpu_compute),
        None,
    );
    let gpu_run = gpu_session
        .run_compaction(&mut gpu_devices)
        .expect("整理一趟");
    gpu_pool.devices = Some(gpu_devices);
    assert!(
        gpu_run.batches_published > 0,
        "这一趟签了意图、搬了批（不然这条用例什么都没比）：{:?}",
        gpu_run.outcome
    );
    assert_eq!(
        format!("{:?}", gpu_run.outcome),
        format!("{:?}", cpu_run.outcome),
        "停在同一条上"
    );
    assert_eq!(
        gpu_run.versions_published, cpu_run.versions_published,
        "发的每一版逐项相同"
    );
    assert_eq!(
        gpu_run.batches_published, cpu_run.batches_published,
        "搬了一样多批"
    );
    assert_eq!(
        gpu_session.compaction.observations, cpu_session.compaction.observations,
        "整理观测逐项相同"
    );
    assert_eq!(
        gpu_pool.retained_operations(),
        cpu_pool.retained_operations(),
        "两条录制流逐步相同：盘上逐字节相同"
    );
    assert!(
        cpu_run
            .background_compute_of_each_batch
            .iter()
            .all(|batch| matches!(batch.path_taken, BackgroundComputePathTaken::Cpu(_))),
        "cpu 那一趟每一批走 CPU"
    );
    let mut work_of_the_run = GpuEntryCounts::default();
    for batch in &gpu_run.background_compute_of_each_batch {
        let BackgroundComputePathTaken::Gpu(gpu_work) = batch.path_taken else {
            panic!("gpu 那一趟每一批走 GPU：{batch:?}");
        };
        let entries = gpu_work.entries_of_the_three_segments();
        assert_eq!(
            entries.uncertain_this_round + entries.judged_by_the_cpu_reference,
            0,
            "不注入故障：全部确认，{gpu_work:?}"
        );
        work_of_the_run.confirmed_by_both_kernels += entries.confirmed_by_both_kernels;
    }
    assert!(
        work_of_the_run.confirmed_by_both_kernels > 0,
        "确认的条目数报出来了"
    );
    let gpu_compute_for_tests = gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过");
    for segment in [
        GpuComputedSegment::LiveSlotsPerClusterSegment,
        GpuComputedSegment::RegionRecordClassification,
        GpuComputedSegment::UnitChecksumVerification,
    ] {
        assert!(
            gpu_compute_for_tests.entries_handed_in_to(segment) > 0,
            "{segment:?} 段经递进来的实现算了：{:?}",
            gpu_compute_for_tests.entries_handed_in
        );
    }
}

/// 丙段（签发之前的选段打分）交回 GPU 出错（第 4 条）：这一批放弃，一个写都不发、盘上逐字节不变；整理停下并报出是哪一段出的错；
/// 不换回 CPU（乙、甲两段一条都没交）。
#[test]
fn a_gpu_failure_while_scoring_the_segments_abandons_the_batch_stops_the_compaction_and_leaves_the_disk_unchanged(
) {
    let (mut pool, _) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-failure-while-scoring");
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .failing_segment = Some((
        GpuComputedSegment::LiveSlotsPerClusterSegment,
        device_lost_on_the_card_with_eight_gibibytes(),
    ));
    let (mut session, mut devices) = remount(
        &mut pool,
        &BackgroundComputeOfTheRemount::GpuThrough(&gpu_compute),
        None,
    );
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && run.versions_published.is_empty(),
        "出错的那一批放弃、一个写都没发：{:?}",
        run.outcome
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变：系统配置槽、根环、录制流的步数"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &[BackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            error: BackgroundComputeError::GpuSegmentFailed {
                segment: GpuComputedSegment::LiveSlotsPerClusterSegment,
                failure: device_lost_on_the_card_with_eight_gibibytes(),
            },
        }],
        "整理停下，报出是丙段、哪张卡、什么原因"
    );
    assert_eq!(
        gpu_work_of_the_only_batch(&run),
        GpuWorkOfTheBatch::default(),
        "丙段出错：这一批在 GPU 上一条都没算成"
    );
    let gpu_compute_for_tests = gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过");
    assert_eq!(
        gpu_compute_for_tests.entries_handed_in_to(GpuComputedSegment::RegionRecordClassification)
            + gpu_compute_for_tests
                .entries_handed_in_to(GpuComputedSegment::UnitChecksumVerification),
        0,
        "出错之后不换回 CPU 接着算、也不往下交别的段：{:?}",
        gpu_compute_for_tests.entries_handed_in
    );
}

/// 甲段（对 R 重扫时核可搬单元的校验和）交回 GPU 出错（第 4 条）：意图已在盘上，这一批放弃、一个写都不发、盘上逐字节不变，
/// 意图留在 logged_ops 里、R 排他撤掉（同暂停），整理停下。
#[test]
fn a_gpu_failure_while_verifying_the_unit_checksums_of_a_live_intent_leaves_the_intent_on_disk_and_stops_the_compaction(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-gpu-failure-while-verifying-checksums",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) =
        gpu_session_with_the_intent_issued_on(&mut pool, region, &gpu_compute);
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .failing_segment = Some((
        GpuComputedSegment::UnitChecksumVerification,
        device_lost_on_the_card_with_eight_gibibytes(),
    ));
    let run = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ) && run.versions_published.is_empty(),
        "出错的那一批放弃、一个写都没发：{:?}",
        run.outcome
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, intent)| intent.region == region),
        "意图留在 logged_ops 里"
    );
    assert_eq!(
        session
            .allocator
            .compaction()
            .live_intent()
            .map(|live| live.is_the_region_excluded_from_placement),
        Some(false),
        "R 排他撤掉（同暂停），意图照留"
    );
    assert!(
        session
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "整理停下"
    );
    let gpu_work = gpu_work_of_the_only_batch(&run);
    assert!(
        gpu_work
            .region_record_classification
            .confirmed_by_both_kernels
            > 0
            && gpu_work.unit_checksum_verification == GpuEntryCounts::default(),
        "乙段算成了、甲段出错：{gpu_work:?}"
    );
}

/// 乙段对 R 里最前那一条可搬记录这一轮两份核判得不一样（第 8 条）：这一批不搬它，照搬其余判定了可搬的；去掉注入之后下一批照判、搬走它。
#[test]
fn a_region_record_classified_uncertain_this_round_is_not_moved_in_that_batch_and_is_moved_once_confirmed_in_the_next(
) {
    let (mut pool, region) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-region-record-uncertain");
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) =
        gpu_session_with_the_intent_issued_on(&mut pool, region, &gpu_compute);
    let first_record = *unreleased_records_in(&session, region)
        .first()
        .expect("R 里有记录");
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .region_records_uncertain =
        BTreeSet::from([RegionRecordEntryKey::of_the_record(&first_record)]);
    let uncertain_batch = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        uncertain_batch
            .versions_published
            .iter()
            .any(|version| !version.units_relocated_by_this_publish.is_empty()),
        "照搬其余判定了可搬的：{:?}",
        uncertain_batch.outcome
    );
    assert!(
        is_still_unreleased(&session, &first_record),
        "这一轮不确定的那一条这一批不搬：{first_record:?}"
    );
    assert_eq!(
        gpu_work_of_the_only_batch(&uncertain_batch)
            .region_record_classification
            .uncertain_this_round,
        1,
        "每批报的不确定条目数"
    );
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .region_records_uncertain = BTreeSet::new();
    let confirmed_batch = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    assert!(
        !is_still_unreleased(&session, &first_record),
        "下一批照判：确认可搬、它在 R 里最前，这一批搬走它：{:?}",
        confirmed_batch.outcome
    );
}

/// 甲段对 R 里最前那一条可搬单元的校验和这一轮两份核判得不一样（第 8 条）：这一批不搬它；去掉注入之后下一批照判、搬走它。
#[test]
fn a_unit_checksum_verified_uncertain_this_round_keeps_its_unit_in_place_for_that_batch_and_it_is_moved_once_confirmed(
) {
    let (mut pool, region) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-unit-checksum-uncertain");
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) =
        gpu_session_with_the_intent_issued_on(&mut pool, region, &gpu_compute);
    let first_record = *unreleased_records_in(&session, region)
        .first()
        .expect("R 里有记录");
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .unit_checksums_uncertain =
        BTreeSet::from([RegionRecordEntryKey::of_the_record(&first_record)]);
    let uncertain_batch = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        is_still_unreleased(&session, &first_record),
        "这一轮不确定的那一条这一批不搬：{:?}",
        uncertain_batch.outcome
    );
    assert_eq!(
        gpu_work_of_the_only_batch(&uncertain_batch)
            .unit_checksum_verification
            .uncertain_this_round,
        1,
        "每批报的不确定条目数"
    );
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .unit_checksums_uncertain = BTreeSet::new();
    let confirmed_batch = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    assert!(
        !is_still_unreleased(&session, &first_record),
        "下一批照判、搬走它：{:?}",
        confirmed_batch.outcome
    );
}

/// R 里每一条记录乙段这一轮都判得不一样：判定了可搬的一个都没有，这一批不发——不删意图、不因「一个能搬的都没有」丢意图——
/// 停在「只剩不确定的记录」上，报出几条；盘上逐字节不变，意图照留、R 照旧排他。
#[test]
fn a_region_left_with_only_records_uncertain_this_round_publishes_nothing_and_keeps_the_intent() {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-gpu-only-uncertain-records-left",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) =
        gpu_session_with_the_intent_issued_on(&mut pool, region, &gpu_compute);
    let records_in_the_region = unreleased_records_in(&session, region);
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .region_records_uncertain = records_in_the_region
        .iter()
        .map(RegionRecordEntryKey::of_the_record)
        .collect();
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let run = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    let records_uncertain = u64::try_from(records_in_the_region.len()).expect("一段 64 槽");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::OnlyRecordsUncertainThisRoundLeftToMove {
                records_uncertain_this_round
            }) if records_uncertain_this_round == records_uncertain
        ) && run.versions_published.is_empty(),
        "这一批不发，停在只剩不确定的记录上：{:?}",
        run.outcome
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变"
    );
    assert_eq!(
        session.allocator.compaction().live_intent().map(|live| (
            live.intent.region,
            live.is_the_region_excluded_from_placement
        )),
        Some((region, true)),
        "意图照留、R 照旧排他"
    );
}

/// 丙段对 cpu 那一趟会选的那一段这一轮数得不一样（第 8 条）：它这一批不当候选，gpu 这一趟签发在别的段上，报出 1 条不确定。
#[test]
fn a_segment_scored_uncertain_this_round_is_not_chosen_by_that_batch() {
    let (mut cpu_pool, _) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-segment-uncertain-cpu");
    let (mut gpu_pool, _) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-segment-uncertain-gpu");
    let (mut cpu_session, mut cpu_devices) =
        remount(&mut cpu_pool, &BackgroundComputeOfTheRemount::Cpu, None);
    run_one_compaction_publish(&mut cpu_session, &mut cpu_devices);
    cpu_pool.devices = Some(cpu_devices);
    let (_, region_chosen_by_the_cpu) =
        live_compaction_intent_of_the_entries(&current_of(&cpu_session).logged_operations)
            .map(|(issued_txg, intent)| (issued_txg, intent.region))
            .expect("cpu 那一趟签了一条意图");
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .cluster_segments_uncertain = BTreeSet::from([ClusterSegmentEntryKey {
        device: region_chosen_by_the_cpu.device(),
        start_slot: region_chosen_by_the_cpu.start_slot(),
    }]);
    let (mut gpu_session, mut gpu_devices) = remount(
        &mut gpu_pool,
        &BackgroundComputeOfTheRemount::GpuThrough(&gpu_compute),
        None,
    );
    let gpu_run = run_one_compaction_publish(&mut gpu_session, &mut gpu_devices);
    gpu_pool.devices = Some(gpu_devices);
    let region_chosen_on_the_gpu_path =
        live_compaction_intent_of_the_entries(&current_of(&gpu_session).logged_operations)
            .map(|(_, intent)| intent.region);
    assert_ne!(
        region_chosen_on_the_gpu_path,
        Some(region_chosen_by_the_cpu),
        "数得不确定的那一段这一批不选：{:?}",
        gpu_run.outcome
    );
    assert_eq!(
        gpu_work_of_the_only_batch(&gpu_run)
            .live_slots_per_cluster_segment
            .uncertain_this_round,
        1,
        "每批报的不确定条目数"
    );
}

/// 甲段两份核一致判出 R 里可搬单元的整单元 CRC 与位置项对不上（第 8 条「确认的交 CPU，照发布路径逐条核」）：区域扫描不因它改分类，
/// 这一批与 cpu 那一趟发的那一版逐项相同，只把对不上的单元数报出来。
#[test]
fn unit_checksum_mismatches_found_on_the_gpu_are_handed_to_the_cpu_publish_path_and_only_counted() {
    let (mut cpu_pool, region) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-checksum-mismatch-cpu");
    let (mut gpu_pool, _) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-checksum-mismatch-gpu");
    let (mut cpu_session, mut cpu_devices) = remount(
        &mut cpu_pool,
        &BackgroundComputeOfTheRemount::Cpu,
        Some(region),
    );
    let records_in_the_region = unreleased_records_in(&cpu_session, region);
    let cpu_issue = run_one_compaction_publish(&mut cpu_session, &mut cpu_devices);
    let cpu_batch = run_one_compaction_publish(&mut cpu_session, &mut cpu_devices);
    cpu_pool.devices = Some(cpu_devices);
    let (gpu_compute_kept_by_the_test, gpu_compute): (
        Arc<Mutex<CpuBackedBackgroundGpuComputeForTests>>,
        BackgroundGpuComputeHandle,
    ) = shared_cpu_backed_gpu_compute();
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .unit_checksums_reported_mismatching = records_in_the_region
        .iter()
        .map(RegionRecordEntryKey::of_the_record)
        .collect();
    let (mut gpu_session, mut gpu_devices) = remount(
        &mut gpu_pool,
        &BackgroundComputeOfTheRemount::GpuThrough(&gpu_compute),
        Some(region),
    );
    let gpu_issue = run_one_compaction_publish(&mut gpu_session, &mut gpu_devices);
    let gpu_batch = run_one_compaction_publish(&mut gpu_session, &mut gpu_devices);
    gpu_pool.devices = Some(gpu_devices);
    assert_eq!(
        (&gpu_issue.versions_published, &gpu_batch.versions_published),
        (&cpu_issue.versions_published, &cpu_batch.versions_published),
        "签发那一次与搬的那一批，两边发的版逐项相同"
    );
    assert!(
        gpu_batch
            .versions_published
            .iter()
            .any(|version| !version.units_relocated_by_this_publish.is_empty()),
        "这一批照常搬了：{:?}",
        gpu_batch.outcome
    );
    assert!(
        gpu_work_of_the_only_batch(&gpu_batch).units_whose_checksum_mismatched_on_the_gpu > 0,
        "对不上的单元数报出来了"
    );
}

/// 签发之前对强制选的那一段扫描时乙段对其中一条这一轮判得不一样（第 8 条）：R 里要搬几个数不准，这一批不签——停在「签发被不确定的记录挡住」、
/// 报出 1 条，盘上逐字节不变、logged_ops 里没有意图；去掉注入之后下一批照判、签在这一段上。
#[test]
fn an_issue_on_a_segment_holding_a_record_uncertain_this_round_is_held_back_and_issued_once_confirmed(
) {
    let (mut pool, region) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-gpu-issue-held-back-by-an-uncertain-record",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) = remount(
        &mut pool,
        &BackgroundComputeOfTheRemount::GpuThrough(&gpu_compute),
        Some(region),
    );
    let first_record = *unreleased_records_in(&session, region)
        .first()
        .expect("R 里有记录");
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .region_records_uncertain =
        BTreeSet::from([RegionRecordEntryKey::of_the_record(&first_record)]);
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let held_back = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        matches!(
            held_back.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::IssueHeldBackByRecordsUncertainThisRound {
                    records_uncertain_this_round: 1
                }
            )
        ) && held_back.versions_published.is_empty(),
        "这一批不签：{:?}",
        held_back.outcome
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations).is_none(),
        "logged_ops 里没有意图"
    );
    gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过")
        .region_records_uncertain = BTreeSet::new();
    let issuing = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, intent)| intent.region == region),
        "下一批照判、签在这一段上：{:?}",
        issuing.outcome
    );
}

/// 第 4 条「停下并报错」：实现交回的每一种错（卡出错的五种原因、等超过上限的两种、第一版不支持的三种输入）各注入一次到丙段，
/// 整理停下报的错里那一段、那张卡、那个原因与注入的逐项相同，这一批的记账也报出它。一个池、一个会话依次注入，每换一种先切换一次解除上一次的停下。
#[test]
fn every_kind_of_failure_of_the_gpu_compute_stops_the_compaction_reporting_its_segment_card_and_cause(
) {
    let (mut pool, _) =
        pool_after_a_sequential_write_in_an_earlier_mount("compaction-gpu-failure-kinds-reported");
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) = remount(
        &mut pool,
        &BackgroundComputeOfTheRemount::GpuThrough(&gpu_compute),
        None,
    );
    let card_eight = card_named("card-eight");
    let card_sixteen = card_named("card-sixteen");
    let failed_on = |device: &singlefs_core::background_compute::GpuDeviceIdentifier,
                     cause: GpuFailureOnTheDevice| {
        BackgroundGpuComputeFailed::GpuFailed {
            device: device.clone(),
            cause,
        }
    };
    let every_kind_of_failure = [
        failed_on(
            &card_eight,
            GpuFailureOnTheDevice::DeviceOrPipelineDidNotStart {
                reason: "假的：起不了设备".to_string(),
            },
        ),
        failed_on(
            &card_sixteen,
            GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                reason: "假的：设备丢了".to_string(),
            },
        ),
        failed_on(
            &card_eight,
            GpuFailureOnTheDevice::WaitForTheSubmissionFailed {
                reason: "假的：等不成".to_string(),
            },
        ),
        failed_on(
            &card_sixteen,
            GpuFailureOnTheDevice::ReadbackMappingFailed {
                reason: "假的：映射不成".to_string(),
            },
        ),
        failed_on(
            &card_eight,
            GpuFailureOnTheDevice::KernelLeftAnUnrecognizedVerdictWord {
                entry_position: 5,
                verdict_word: 9,
            },
        ),
        BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
            device: card_sixteen.clone(),
            wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
        },
        BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
            device: card_eight.clone(),
            wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            submission_waited_for:
                SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
        },
        BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                device: card_sixteen.clone(),
                entry_position: 2,
                input_bytes: 16,
            },
        ),
        BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::TablesLargerThanABindingOnTheDevice {
                device: card_eight.clone(),
                largest_buffer_bytes: 1 << 32,
            },
        ),
        BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
            InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold {
                entry_position: 0,
                key_bytes: 29,
            },
        ),
    ];
    for (position_in_the_list, failure) in every_kind_of_failure.iter().enumerate() {
        if position_in_the_list > 0 {
            session
                .switch_background_compute_from_the_next_batch(
                    admissible_gpu_settings_naming_both_cards(),
                    None,
                )
                .expect("全合法的 gpu 设置：切换收下，解除上一种出错的停下");
        }
        gpu_compute_kept_by_the_test
            .lock()
            .expect("假实现没在别处 panic 过")
            .failing_segment = Some((
            GpuComputedSegment::LiveSlotsPerClusterSegment,
            failure.clone(),
        ));
        let run = session.run_compaction(&mut devices).expect("整理一趟");
        let reported = BackgroundComputeError::GpuSegmentFailed {
            segment: GpuComputedSegment::LiveSlotsPerClusterSegment,
            failure: failure.clone(),
        };
        assert!(
            matches!(
                run.outcome,
                CompactionRunOutcome::Stopped(
                    CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                        task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                    }
                )
            ) && run.versions_published.is_empty(),
            "注入 {failure:?}：这一批放弃、整理停下：{:?}",
            run.outcome
        );
        assert_eq!(
            session.background_compute().stopped_after_a_compute_error(),
            &[BackgroundTaskStoppedAfterAComputeError {
                task: BackgroundTask::CompactionRelocation,
                error: reported.clone(),
            }],
            "整理停下报的错就是注入的那一个：那一段、那张卡、那个原因"
        );
        assert_eq!(
            run.background_compute_of_each_batch
                .last()
                .map(|batch| batch.compute_error_that_abandoned_the_batch.clone()),
            Some(Some(reported)),
            "这一批的记账也报出它"
        );
    }
    pool.devices = Some(devices);
}

/// 用例给的一组 `gpu` 设置照假实现的卡清单（「card-eight」8 GiB、「card-sixteen」16 GiB）校验过、交给实现的样子。
fn admitted_against_the_two_cards_of_the_fake(
    settings: &BackgroundComputeSettings,
) -> AdmittedGpuSettings {
    admit_gpu_settings_against_the_inventory(
        settings,
        &CpuBackedBackgroundGpuComputeForTests::with_nothing_injected(),
    )
    .expect("用例给的一组都合法")
}

/// 第 1–3 条：选 `gpu` 挂载那一刻实现就收到校验过的那一组（两张卡、各自的上限、等的上限）；头一批读完设置、开算之前请求切换，
/// 缩成只点「card-eight」、显存上限缩到 1 GiB、等的上限改成 3 秒。实现在下一批开头、开算之前收到切换那一组：
/// 请求切换的那一批算的每一段照的都是挂载那一组（一批之内不换），之后各批算的每一段只照切换那一组（只在剩下的那张卡上、照小的上限、照新的等的上限）。
#[test]
fn the_mount_and_a_switch_hand_the_admitted_gpu_settings_to_the_implementation_from_the_next_batch()
{
    let (mut pool, _) = pool_after_a_sequential_write_in_an_earlier_mount(
        "compaction-gpu-settings-handed-to-the-implementation",
    );
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    let (mut session, mut devices) = remount(
        &mut pool,
        &BackgroundComputeOfTheRemount::GpuThrough(&gpu_compute),
        None,
    );
    let at_the_mount =
        admitted_against_the_two_cards_of_the_fake(&admissible_gpu_settings_naming_both_cards());
    assert_eq!(
        gpu_compute_kept_by_the_test
            .lock()
            .expect("假实现没在别处 panic 过")
            .settings_received,
        vec![at_the_mount.clone()],
        "挂载那一刻：实现收到校验过的那一组"
    );
    let shrunk_settings = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_named("card-eight")]),
        background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(1 << 30)]),
        background_gpu_wait_limit: std::time::Duration::from_secs(3),
    };
    let after_the_switch = admitted_against_the_two_cards_of_the_fake(&shrunk_settings);
    let mut calls_before_each_batch_computes: Vec<usize> = Vec::new();
    let mut switch_in_the_middle_of_the_first_batch =
        |background_compute: &mut BackgroundComputeOfThisMount| {
            calls_before_each_batch_computes.push(
                gpu_compute_kept_by_the_test
                    .lock()
                    .expect("假实现没在别处 panic 过")
                    .settings_in_use_at_each_call
                    .len(),
            );
            if calls_before_each_batch_computes.len() == 1 {
                background_compute
                    .request_switch_from_the_next_batch(shrunk_settings.clone(), None)
                    .expect("全合法的 gpu 设置：切换收下，记成待生效");
            }
        };
    let run = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut switch_in_the_middle_of_the_first_batch,
        )
        .expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        calls_before_each_batch_computes.len() >= 2,
        "签发一次、至少搬一批：至少两批（{:?}）",
        run.outcome
    );
    let gpu_compute_for_tests = gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过");
    assert_eq!(
        gpu_compute_for_tests.settings_received,
        vec![at_the_mount.clone(), after_the_switch.clone()],
        "实现只收到两组：挂载那一组、切换那一组"
    );
    let calls = &gpu_compute_for_tests.settings_in_use_at_each_call;
    let first_batch_calls =
        &calls[calls_before_each_batch_computes[0]..calls_before_each_batch_computes[1]];
    let later_batch_calls = &calls[calls_before_each_batch_computes[1]..];
    assert!(
        !first_batch_calls.is_empty() && !later_batch_calls.is_empty(),
        "请求切换的那一批与之后的批都经实现算了：{calls_before_each_batch_computes:?}，共 {} 次",
        calls.len()
    );
    assert!(
        first_batch_calls
            .iter()
            .all(|(_segment, settings)| settings.as_ref() == Some(&at_the_mount)),
        "请求切换的那一批算的每一段照的都是挂载那一组（一批之内不换）：{first_batch_calls:?}"
    );
    assert!(
        later_batch_calls
            .iter()
            .all(|(_segment, settings)| settings.as_ref() == Some(&after_the_switch)),
        "之后各批算的每一段只照切换那一组：{later_batch_calls:?}"
    );
    let participating_after_the_switch = &gpu_compute_for_tests
        .participating_devices_at_each_call()[calls_before_each_batch_computes[1]..];
    assert!(
        participating_after_the_switch
            .iter()
            .all(|participating| *participating == vec![card_named("card-eight")]),
        "切换缩卡之后：只在剩下的那张卡上：{participating_after_the_switch:?}"
    );
    let switched_device = &after_the_switch.participating_devices()[0];
    assert_eq!(
        (switched_device.memory_limit, after_the_switch.wait_limit()),
        (
            Some(GpuMemoryInBytes(1 << 30)),
            std::time::Duration::from_secs(3)
        ),
        "切换那一组带着缩小的显存上限与新的等的上限"
    );
}
