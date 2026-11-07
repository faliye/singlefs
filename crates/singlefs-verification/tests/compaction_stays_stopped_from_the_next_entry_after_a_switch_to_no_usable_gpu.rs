//! 挂着切换到参与的卡一张都用不上的一组设置（D26（后台整理与放置回收） 已定项 13 第 3、4 条：参与的卡因显存上限一张都不剩时不拒切换，
//! 后台任务照第 4 条进出错停着）：与挂载那一刻一样，下一次进整理的显式入口、
//! 后台入口在任何一个写之前就判出停着——不走删除续做、不取走「该跑了」，报的是挂载那一刻停着报的同一个成员
//! （`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`）。要是到下一批开头才记停着，那一趟会先把删除续做走了、后台入口会先取走「该跑了」。
//!
//! GPU 用一份背后照核心层 CPU 写法算的假实现（`common_background_gpu_compute`）外面再包一层：只点名「card-eight」的那一组设置交进来之后，
//! 它答「参与的卡因显存上限一张都不剩」，别的设置照常用得上。池：`common::build_pool` 起，另一次挂载里顺序写第一个文件，重挂选 `gpu`（两张卡都点名、用得上），
//! 整理一批至多搬 2 个、起跑强制开着；会话里 unlink 第一个文件（一次走完），再挂着回退到 unlink 意图那一版（队列带回第一个文件，攻方探针
//! `research/prompts/m4-gpugc-code-r1-opus-model/probes/harness_deletion_continuation_test.rs` 的造法），删除队列非空。
mod common;
mod common_background_gpu_compute;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{
    build_pool, disk_snapshot, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, card_named, CpuBackedBackgroundGpuComputeForTests,
    EIGHT_GIBIBYTES,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, AlarmOfABackgroundTaskStoppedAfterAComputeError, BackgroundComputeBackend,
    BackgroundComputeError, BackgroundComputeSettings, BackgroundGpuCompute,
    BackgroundGpuComputeFailed, BackgroundGpuComputeHandle, BackgroundTask,
    BackgroundTaskStoppedAfterAComputeError, ClusterSegmentEntryKey, GpuDeviceIdentifier,
    GpuEntryDecision, GpuInventory, GpuLeftOutUnderItsMemoryLimit, GpuMemoryInBytes,
    RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
    CompactionBatchUnits, CompactionPacing, CompactionRunForcing, CompactionRunStop,
    CompactionStartJudgement, CompactionSwitches, CompactionTimerFallback,
    CompactionWriteBandwidthLimit, ForegroundFsync, UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_background_compute_settings,
    roll_back_by_a_forward_publish, RollbackTarget,
};
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, BackgroundCompactionNotStarted, CompactionRunOutcome, MountedSession,
    UserChange,
};
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use singlefs_core::transaction::{PoolVersion, TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;

/// 另一次挂载里顺序写第一个文件的数据单元数。
const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 8;
/// 整理一批至多搬几个（只供测试的开关；产品路径批大小条款没写）。
const UNITS_PER_BATCH: usize = 2;
/// 「card-eight」答的那一张被晾下的卡：给它的上限与最小的一片（数是用例自己定的）。
const MEMORY_LIMIT_HOLDING_NO_PIECE: GpuMemoryInBytes = GpuMemoryInBytes(64);
const SMALLEST_PIECE_BYTES: u64 = 972;

/// 只点名「card-eight」的那一组交进来之后答「参与的卡因显存上限一张都不剩」的那一个出错。
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

/// 假实现外面包一层：交进来的最近那一组只点名「card-eight」时，答那一组下一张卡都用不上（[`every_participating_gpu_left_out`]），三段照旧交给里面那一份算。
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

/// 只点名「card-eight」、上限是它的显存总量的一组 `gpu` 设置（校验照收；包着的那一层答它一张卡都用不上）。
fn settings_naming_only_the_card_with_eight_gibibytes() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![card_named("card-eight")]),
        background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

/// 后台入口的调用方：钟停在起点、前台没有 fsync 在等。
struct CallerWithNoForegroundFsync;

impl CallerOfTheBackgroundCompaction for CallerWithNoForegroundFsync {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::ZERO)
    }

    fn foreground_fsync(&self) -> ForegroundFsync {
        ForegroundFsync::NoneWaiting
    }
}

/// 包着假实现的那一份的把手（还没交来设置）。
fn gpu_compute_unusable_under_the_settings_naming_only_the_card_with_eight_gibibytes(
) -> BackgroundGpuComputeHandle {
    BackgroundGpuComputeHandle::of_a_shared_implementation(Arc::new(Mutex::new(
        GpuComputeUnusableUnderTheSettingsNamingOnlyTheCardWithEightGibibytes {
            computed_on_the_cpu: CpuBackedBackgroundGpuComputeForTests::with_nothing_injected(),
            settings_handed_in_last: None,
        },
    )))
}

/// 另一次挂载里顺序写第一个文件（[`DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT`] 个数据单元），那一次的会话丢掉。
fn write_the_first_file_in_an_earlier_mount(pool: &mut BuiltPool) {
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
    drop(session);
    pool.devices = Some(devices);
}

/// 挂载那一刻只点名「card-eight」（包着的那一层答一张卡都用不上）：挂载照常做成、整理一挂上就停着；头一次叫起（显式入口）就交回报警——
/// 「card-eight」、挂载那一刻停着报的那一个成员、叫起 1 次、补跑 0 批。
#[test]
fn the_first_wake_after_a_mount_with_no_usable_gpu_returns_the_alarm() {
    let mut pool =
        build_pool("compaction-alarm-on-the-first-wake-after-a-mount-with-no-usable-gpu");
    write_the_first_file_in_an_earlier_mount(&mut pool);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        settings_naming_only_the_card_with_eight_gibibytes(),
        Some(gpu_compute_unusable_under_the_settings_naming_only_the_card_with_eight_gibibytes()),
    )
    .expect("一张卡都用不上也不拒挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &the_compaction_stopped_for_no_usable_gpu(),
        "挂载那一刻就停着"
    );
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(
        run.alarm_of_the_compaction_stopped_after_a_compute_error,
        Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
            task: BackgroundTask::CompactionRelocation,
            gpu_devices_named_by_the_error: vec![card_named("card-eight")],
            error_that_stopped_the_task:
                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                    failure: every_participating_gpu_left_out(),
                },
            wakes_while_stopped: 1,
            batches_compensated_on_the_cpu: 0,
        }),
        "头一次叫起就报警（{:?}）",
        run.outcome
    );
}

/// 池（另一次挂载顺序写过第一个文件）重挂选 `gpu`、两张卡都点名（用得上），整理一批至多 [`UNITS_PER_BATCH`] 个、起跑强制开着；会话里 unlink 第一个文件
/// （一次走完、「该跑了」记下），再挂着回退到 unlink 意图那一版：删除队列带回第一个文件。最后挂着切换到只点名「card-eight」的那一组（一张卡都用不上），
/// 切换收下、还没换上。交回会话与盘。
fn session_switched_to_no_usable_gpu_with_the_deletion_queue_brought_back(
    pool: &mut BuiltPool,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    write_the_first_file_in_an_earlier_mount(pool);
    let mut devices = pool.reopen_recorded();
    let gpu_compute =
        gpu_compute_unusable_under_the_settings_naming_only_the_card_with_eight_gibibytes();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(gpu_compute),
    )
    .expect("两张卡都点名、都用得上：挂载照常做成");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    assert!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "挂载那一组用得上：没有停着的"
    );
    let CompactionSwitches {
        batch_size: _,
        forced_region,
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
    let unlinked = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .expect("会话照常 unlink：一次走完");
    let intent_root = unlinked.versions_published[0].root;
    let mut current = current_of(&session).clone();
    assert!(current.deleted_inodes.is_empty(), "unlink 一次走完：队列空");
    roll_back_by_a_forward_publish(
        &parameters(),
        &mut devices,
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
        current_of(&session).deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "回退到意图那一版：删除队列带回第一个文件"
    );
    session
        .switch_background_compute_from_the_next_batch(
            settings_naming_only_the_card_with_eight_gibibytes(),
            None,
        )
        .expect("只点名 card-eight 的一组照卡清单校验合法：不拒切换（一张卡都用不上也不拒）");
    (session, devices)
}

/// 整理照挂载那一刻一样记成停着、报的是同一个成员：「参与的卡因显存上限一张都不剩」。
fn the_compaction_stopped_for_no_usable_gpu() -> [BackgroundTaskStoppedAfterAComputeError; 1] {
    [BackgroundTaskStoppedAfterAComputeError {
        task: BackgroundTask::CompactionRelocation,
        error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
            failure: every_participating_gpu_left_out(),
        },
        batches_compensated_on_the_cpu: 0,
    }]
}

/// 切换到一张卡都用不上的那一组之后，下一次显式入口进来：在任何一个写之前判出停着，不走删除续做（队列照旧带着第一个文件）、一个写都不发、
/// 一批都不开，盘上逐字节不变；整理记成停着、报「参与的卡因显存上限一张都不剩」（同挂载那一刻）。
#[test]
fn the_explicit_entry_after_a_switch_to_no_usable_gpu_stays_stopped_before_continuing_the_deletion_queue(
) {
    let mut pool = build_pool("compaction-explicit-entry-after-a-switch-to-no-usable-gpu");
    let (mut session, mut devices) =
        session_switched_to_no_usable_gpu_with_the_deletion_queue_brought_back(&mut pool);
    let before_the_run = disk_snapshot(&pool.memory_pool(), &pool.stream);
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
        "下一次显式入口一进来就停着：一个写都不发、一批都不开：{:?}，发了 {} 版",
        run.outcome,
        run.versions_published.len()
    );
    assert_eq!(
        current_of(&session).deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "删除续做没走：队列照旧带着第一个文件"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_run,
        "盘上逐字节不变：系统配置槽、根环、录制流的步数"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &the_compaction_stopped_for_no_usable_gpu(),
        "与挂载那一刻同：报参与的卡一张都不剩"
    );
}

/// 同一处的后台入口：下一次调用在判起不起跑之前就判出停着，不起跑、「该跑了」留着（不取走），盘上逐字节不变；整理记成停着、报同一个成员。
#[test]
fn the_background_entry_after_a_switch_to_no_usable_gpu_stays_stopped_and_keeps_the_start_due() {
    let mut pool = build_pool("compaction-background-entry-after-a-switch-to-no-usable-gpu");
    let (mut session, mut devices) =
        session_switched_to_no_usable_gpu_with_the_deletion_queue_brought_back(&mut pool);
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "unlink 那一次用户改动之后起跑判据为真，记下「该跑了」"
    );
    let before_the_call = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            CompactionPacing {
                timer_fallback: CompactionTimerFallback::Off,
                write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
            },
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
        "后台入口一进来就判出停着、不起跑：{call:?}"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "「该跑了」留着，解除之后下一次调用再取"
    );
    assert_eq!(
        current_of(&session).deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "删除续做没走：队列照旧带着第一个文件"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_call,
        "盘上逐字节不变"
    );
    assert_eq!(
        session.background_compute().stopped_after_a_compute_error(),
        &the_compaction_stopped_for_no_usable_gpu(),
        "与挂载那一刻同：报参与的卡一张都不剩"
    );
}

/// 停着时一次通过校验的切换解除停下、叫起次数清掉：挂载那一刻就一张卡都用不上，叫起两次（报 1、2）之后切换到同样一张卡都用不上的那一组——
/// 切换当场解除、下一次叫起开头换上它又停着，这一次报的叫起次数从 1 起，不接着上一次停下数。
#[test]
fn a_switch_to_settings_still_with_no_usable_gpu_counts_the_wakes_while_stopped_from_one_again() {
    let mut pool =
        build_pool("compaction-alarm-wakes-from-one-after-a-switch-still-with-no-usable-gpu");
    write_the_first_file_in_an_earlier_mount(&mut pool);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        settings_naming_only_the_card_with_eight_gibibytes(),
        Some(gpu_compute_unusable_under_the_settings_naming_only_the_card_with_eight_gibibytes()),
    )
    .expect("一张卡都用不上也不拒挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let wakes_before_the_switch: Vec<Option<u64>> = (0..2)
        .map(|_wake| {
            session
                .run_compaction(&mut devices)
                .expect("整理一趟")
                .alarm_of_the_compaction_stopped_after_a_compute_error
                .map(|alarm| alarm.wakes_while_stopped)
        })
        .collect();
    assert_eq!(
        wakes_before_the_switch,
        vec![Some(1), Some(2)],
        "挂载之后叫起两次：报 1、2"
    );
    session
        .switch_background_compute_from_the_next_batch(
            settings_naming_only_the_card_with_eight_gibibytes(),
            None,
        )
        .expect("同样一张卡都用不上的那一组：切换照收");
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert_eq!(
        run.alarm_of_the_compaction_stopped_after_a_compute_error
            .map(|alarm| alarm.wakes_while_stopped),
        Some(1),
        "切换解除过停下：换上之后又停着，叫起次数从 1 起（{:?}）",
        run.outcome
    );
}
