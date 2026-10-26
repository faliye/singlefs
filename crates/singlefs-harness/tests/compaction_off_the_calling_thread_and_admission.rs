//! 整理挪出用户改动的调用线程（用户 2026-10-26 弹窗定 GC「挪出调用线程，补齐三样保证」、C906「照条款：整理批走常规准入」、
//! 「停着时收到切换请求就当场解除停下」；D26（后台整理与放置回收） 已定项 3 第 2 句、已定项 13 第 1、4、5、6 条；D24（后台重活能不能卸给 GPU）
//! 已定项 2 第 2 条；第三轮判决 `research/prompts/m4-gpugc-design-r3-main-verification.md` 第二节 G5、G6、G8）：
//! 用户改动返回之前一批整理都不发，起跑判据只记下「该跑了」，后台入口 `MountedSession::run_compaction_in_the_background` 被调时才发；
//! 三样保证——一趟发布预算有上限（定时兜底与用户改动共用上一次用户改动给的那一份 `B_gc`，扣完等下一次用户改动；岔路单第 18 行甲，用户同日定）、
//! 签下一条意图那一判跨趟读会话上的上一条完成的意图、为一次用户改动推的抬 F 与再判之间插不进整理批；
//! 整理批走常规准入（批次最坏占用计入已承诺预留）；一批出错之后整理停下、通过校验的切换当场解除；定时兜底、写带宽上限、前台 fsync 让路。
//! 池照 `background_compute_settings_and_switch.rs` 的做法：`common::build_pool` 起，可写挂载，强制选第一个文件那次发布的提交内生块所在的段 50240、
//! 一批一个。要压准入余量的用例在每块盘单元区最高处隔离槽（只动分配器的隔离位，不写盘），把可用(d) 压进一个槽宽之内；需求与批次占用先在
//! 同样建、同样推的另一个池上量出来（两个池走同一串操作，字节相同）。
mod common;
mod common_background_gpu_compute;

use std::cell::Cell;
use std::num::NonZeroU64;
use std::time::Duration;

use common::{
    build_pool, disk_snapshot, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
};
use common_background_gpu_compute::{
    admissible_gpu_settings_naming_both_cards, shared_cpu_backed_gpu_compute,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::admission::admission_reading_before_a_publish;
use singlefs_core::allocator::UnitFootprint;
use singlefs_core::background_compute::{
    BackgroundComputeBackend, BackgroundComputeError, BackgroundComputeOfOneBatch,
    BackgroundComputeOfThisMount, BackgroundTask, BackgroundTaskAfterTheComputeError,
};
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
    CompactionBatchUnits, CompactionObservations, CompactionPacing, CompactionRunForcing,
    CompactionRunStop, CompactionStartJudgement, CompactionSwitches, CompactionTimerFallback,
    CompactionWriteBandwidthLimit, ForegroundFsync,
};
use singlefs_core::logged_operations::{live_compaction_intent_of_the_entries, CompactionRegion};
use singlefs_core::mount::{mount_writable, mount_writable_with_background_compute_settings};
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, BackgroundCompactionNotStarted, BackgroundCompactionTrigger,
    CompactionRun, CompactionRunOutcome, MountedSession, UserChange, UserChangePublished,
};
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;

/// 第一个文件那次发布的提交内生块落在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)）。
const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 50240;
/// mkfs 写的每区根槽数 S（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`）。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
/// 显式入口一趟只发一次：「已经过去的发布数」取 `3S − 5`，`B_gc` = `3S − 4 − 已经过去的发布数` = 1。
const PUBLISHES_PASSED_LEAVING_A_BUDGET_OF_ONE: u64 = 3 * ROOT_RING_SLOTS_PER_REGION - 4 - 1;
/// 一次用户改动给出的那一份 `B_gc`：`3S − 4`（`compaction_plan::publish_budget_of_a_compaction_run` 在已经过去的发布数为 0 时）。
const BUDGET_GIVEN_BY_A_USER_CHANGE: u64 = 3 * ROOT_RING_SLOTS_PER_REGION - 4;
/// 定时兜底的间隔（用例取的，不是默认值）。
const TIMER_FALLBACK_INTERVAL_IN_MILLISECONDS: u64 = 1000;
/// 闲置池那一条连着推几个兜底间隔：比一份 `B_gc` 推完所需的间隔数（用户改动之后那一趟之外再一趟）多得多。
const TIMER_FALLBACK_INTERVALS_PUSHED: u64 = 12;
/// 闲置池那一条之前（另一次挂载里）顺序写第一个文件的数据单元数。草稿探针在 `common::build_pool` 起的池上量过：写 40 个、重挂之后
/// 数据单元最多的那一段里有 29 个两槽的数据单元，强制选它、一批一个，一条意图要发 1 次签发加 29 批，比一份 `B_gc` 多。
const DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT: usize = 40;

/// 后台入口的调用方：钟读数与前台有没有 fsync 在等都由用例摆。
struct CallerForTests {
    clock_in_milliseconds: Cell<u64>,
    foreground_fsync: Cell<ForegroundFsync>,
}

impl CallerForTests {
    /// 钟在起点。
    fn at_the_start_with(foreground_fsync: ForegroundFsync) -> Self {
        CallerForTests {
            clock_in_milliseconds: Cell::new(0),
            foreground_fsync: Cell::new(foreground_fsync),
        }
    }

    fn move_the_clock_to(&self, milliseconds: u64) {
        self.clock_in_milliseconds.set(milliseconds);
    }
}

impl CallerOfTheBackgroundCompaction for CallerForTests {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::from_millis(self.clock_in_milliseconds.get()))
    }

    fn foreground_fsync(&self) -> ForegroundFsync {
        self.foreground_fsync.get()
    }
}

fn non_zero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("用例给的数不是 0")
}

/// 后台入口的节奏参数：每个字段都写出来。
fn pacing_with(
    timer_fallback: CompactionTimerFallback,
    write_bandwidth_limit: CompactionWriteBandwidthLimit,
) -> CompactionPacing {
    CompactionPacing {
        timer_fallback,
        write_bandwidth_limit,
    }
}

fn region_of_the_first_publish() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH),
    )
    .expect("64 的倍数")
}

/// 重开、可写挂载（产品默认 `cpu`）、开会话。
fn mounted_session_of(pool: &mut BuiltPool) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (session, devices)
}

/// 会话装上整理只供测试的开关：强制选段 50240、一批一个、起跑照 `run_forcing`，其余照产品值。
fn force_one_unit_batches_on_the_segment_of_the_first_publish(
    session: &mut MountedSession,
    run_forcing: CompactionRunForcing,
) {
    force_one_unit_batches_on(session, region_of_the_first_publish(), run_forcing);
}

/// 会话装上整理只供测试的开关：强制选 `region`、一批一个、起跑照 `run_forcing`，其余照产品值。
fn force_one_unit_batches_on(
    session: &mut MountedSession,
    region: CompactionRegion,
    run_forcing: CompactionRunForcing,
) {
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
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

fn txg_of(version: &TransactionOutput) -> u64 {
    version.root.checkpoint_txg.0
}

fn create_one_inode(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
) -> UserChangePublished {
    session
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("建一个 inode")
}

/// 用户改动交回时一批整理都没发：交回的每一版都没搬单元、没碰 logged_ops，会话的整理观测一样都没动、没有活意图，
/// 会话的现行那一版就是改动那一版，起跑判据为真只记下了「该跑了」。
fn assert_the_user_change_published_no_compaction_batch(
    session: &MountedSession,
    published: &UserChangePublished,
) {
    assert_eq!(
        published.versions_published.len(),
        1,
        "删除队列是空的：这次改动只发它自己那一版"
    );
    for version in &published.versions_published {
        assert!(
            version.units_relocated_by_this_publish.is_empty()
                && version.logged_operations.is_empty(),
            "用户改动交回的那一版没有搬单元、logged_ops 里没有整理意图"
        );
    }
    assert_eq!(
        session.compaction.observations,
        CompactionObservations::default(),
        "用户改动返回之前整理一趟都没起跑：起跑次数、签发、批数、用掉的根槽都是 0"
    );
    assert!(
        session.allocator.compaction().live_intent().is_none(),
        "没有签任何意图"
    );
    assert_eq!(
        Some(current_of(session)),
        published.versions_published.last(),
        "会话的现行那一版就是改动那一版"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "强制起跑：起跑判据为真，记成「该跑了」交给后台入口"
    );
}

/// 后台入口这一次起跑了、起跑原因是 `trigger`：交回那一趟。
fn run_started_by(
    call: BackgroundCompactionCall,
    trigger: BackgroundCompactionTrigger,
) -> CompactionRun {
    match call {
        BackgroundCompactionCall::Ran {
            trigger: trigger_of_the_call,
            run,
        } if trigger_of_the_call == trigger => *run,
        other
        @ (BackgroundCompactionCall::NotStarted(_) | BackgroundCompactionCall::Ran { .. }) => {
            panic!("后台入口应当因 {trigger:?} 起跑，实际 {other:?}")
        }
    }
}

fn not_started_reason_of(
    call: &BackgroundCompactionCall,
) -> Option<BackgroundCompactionNotStarted> {
    match call {
        BackgroundCompactionCall::NotStarted(reason) => Some(*reason),
        BackgroundCompactionCall::Ran { .. } => None,
    }
}

fn requested_backends_of(batches: &[BackgroundComputeOfOneBatch]) -> Vec<BackgroundComputeBackend> {
    batches
        .iter()
        .map(|batch| batch.requested_backend)
        .collect()
}

/// 用户改动交回、后台入口调过之后共有的那几条：那一趟只发一批、发在改动之后，会话的现行那一版是它，「该跑了」被取走。
fn assert_the_background_entry_published_the_batch_after_the_change(
    session: &MountedSession,
    published: &UserChangePublished,
    run: &CompactionRun,
) {
    assert_eq!(run.batches_published, 1, "用户改动之后那一趟至多一批");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone)
        ),
        "{:?}",
        run.outcome
    );
    let change_txg = txg_of(published.versions_published.last().expect("改动发了一版"));
    let first_compaction_publish = run
        .versions_published
        .first()
        .expect("后台入口发了整理的发布");
    assert!(
        txg_of(first_compaction_publish) > change_txg,
        "整理的发布在改动之后"
    );
    assert_eq!(
        Some(current_of(session)),
        run.versions_published.last(),
        "会话的现行那一版是那一趟最后发的一版"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::NotDue,
        "后台入口取走了「该跑了」"
    );
}

#[test]
fn a_user_change_under_cpu_settings_publishes_no_compaction_batch_before_returning_and_the_background_entry_publishes_it(
) {
    let mut pool = build_pool("compaction-off-the-calling-thread-under-cpu-settings");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::ForcedToStartForTests,
    );
    let published = create_one_inode(&mut session, &mut devices);
    assert_the_user_change_published_no_compaction_batch(&session, &published);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            CompactionPacing::BY_DEFAULT,
            &CallerForTests::at_the_start_with(ForegroundFsync::NoneWaiting),
        )
        .expect("盘表是挂载的那一份、现行那一版有树");
    pool.devices = Some(devices);
    let run = run_started_by(call, BackgroundCompactionTrigger::DueAfterAUserChange);
    assert_the_background_entry_published_the_batch_after_the_change(&session, &published, &run);
    assert!(
        requested_backends_of(&run.background_compute_of_each_batch)
            .iter()
            .all(|backend| *backend == BackgroundComputeBackend::Cpu),
        "cpu 设置下每一批请求的都是 cpu"
    );
}

#[test]
fn a_user_change_under_gpu_settings_publishes_no_compaction_batch_before_returning_and_the_background_entry_publishes_it(
) {
    let mut pool = build_pool("compaction-off-the-calling-thread-under-gpu-settings");
    let mut devices = pool.reopen_recorded();
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(gpu_compute.clone()),
    )
    .expect("全合法的 gpu 设置：挂载照常做成");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::ForcedToStartForTests,
    );
    let published = create_one_inode(&mut session, &mut devices);
    assert_the_user_change_published_no_compaction_batch(&session, &published);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            CompactionPacing::BY_DEFAULT,
            &CallerForTests::at_the_start_with(ForegroundFsync::NoneWaiting),
        )
        .expect("盘表是挂载的那一份、现行那一版有树");
    pool.devices = Some(devices);
    let run = run_started_by(call, BackgroundCompactionTrigger::DueAfterAUserChange);
    assert_the_background_entry_published_the_batch_after_the_change(&session, &published, &run);
    assert!(
        requested_backends_of(&run.background_compute_of_each_batch)
            .iter()
            .all(|backend| *backend == BackgroundComputeBackend::Gpu),
        "gpu 设置下每一批请求的都是 gpu"
    );
}

/// 闲置池：另一次挂载里顺序写第一个文件 [`DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT`] 个数据单元（那一次挂载的整理照产品开关，不起跑），
/// 重挂、开会话。交回重挂之后的会话、盘，与那一次写之后第一块盘上两槽的数据单元最多的段和它里面的数据单元数（按记录数）。
fn idle_session_after_a_sequential_write_in_an_earlier_mount(
    pool: &mut BuiltPool,
) -> (
    MountedSession,
    Vec<(DeviceIdentity, Recorded)>,
    CompactionRegion,
    u64,
) {
    let (mut session_of_the_write, mut devices_of_the_write) = mounted_session_of(pool);
    let content: Vec<u8> = (0..DATA_UNITS_WRITTEN_IN_AN_EARLIER_MOUNT
        * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    session_of_the_write
        .publish_user_change(
            &mut devices_of_the_write,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .expect("顺序写第一个文件");
    drop(session_of_the_write);
    pool.devices = Some(devices_of_the_write);
    let (session, devices) = mounted_session_of(pool);
    let mut data_units_by_segment: std::collections::BTreeMap<u64, u64> =
        std::collections::BTreeMap::new();
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
    let (fullest_segment, data_units_in_it) = data_units_by_segment
        .into_iter()
        .max_by_key(|(_, data_units)| *data_units)
        .expect("写了数据单元：至少一段里有");
    let region =
        CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(fullest_segment))
            .expect("按 64 取整的段起点");
    (session, devices, region, data_units_in_it)
}

/// 一趟发布预算有上限那一样保证（岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，用户 2026-10-26 定「与用户改动共用 `B_gc`」）：
/// 闲置的池上开着定时兜底，只有一次用户改动。那一次改动给出一份 `B_gc`（`3S − 4` = 20），用户改动之后那一趟与之后各个兜底间隔起跑的
/// 那几趟都从这一份里扣：连着推 12 个间隔，整理发布合计正好 20 次，意图还没做完（R 里的数据单元比一份 `B_gc` 多），扣完之后的间隔都不起跑；
/// 下一次用户改动之后才再发。
#[test]
fn an_idle_pool_under_the_timer_fallback_publishes_no_more_than_the_budget_of_its_only_user_change_until_the_next_change(
) {
    let mut pool = build_pool("compaction-timer-fallback-shares-the-budget-of-the-user-change");
    let (mut session, mut devices, region, data_units_in_the_region) =
        idle_session_after_a_sequential_write_in_an_earlier_mount(&mut pool);
    assert!(
        data_units_in_the_region > BUDGET_GIVEN_BY_A_USER_CHANGE,
        "前提：R 里的数据单元 {data_units_in_the_region} 个，一批一个，一份 `B_gc` 搬不完"
    );
    force_one_unit_batches_on(
        &mut session,
        region,
        CompactionRunForcing::ForcedToStartForTests,
    );
    create_one_inode(&mut session, &mut devices);
    let pacing = pacing_with(
        CompactionTimerFallback::EveryMilliseconds(non_zero(
            TIMER_FALLBACK_INTERVAL_IN_MILLISECONDS,
        )),
        CompactionWriteBandwidthLimit::Unlimited,
    );
    let caller = CallerForTests::at_the_start_with(ForegroundFsync::NoneWaiting);
    let after_the_change = run_started_by(
        session
            .run_compaction_in_the_background(&mut devices, pacing, &caller)
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    assert_eq!(
        after_the_change.batches_published, 1,
        "用户改动之后那一趟至多一批"
    );
    let mut calls_of_each_interval: Vec<BackgroundCompactionCall> = Vec::new();
    for interval in 1..=TIMER_FALLBACK_INTERVALS_PUSHED {
        caller.move_the_clock_to(interval * TIMER_FALLBACK_INTERVAL_IN_MILLISECONDS);
        calls_of_each_interval.push(
            session
                .run_compaction_in_the_background(&mut devices, pacing, &caller)
                .expect("盘表是挂载的那一份"),
        );
    }
    assert_eq!(
        session.compaction.observations.root_ring_slots_used, BUDGET_GIVEN_BY_A_USER_CHANGE,
        "用户改动之后那一趟与 {TIMER_FALLBACK_INTERVALS_PUSHED} 个兜底间隔起跑的那几趟合起来，整理发布正好一份 `B_gc`"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
            .is_some_and(|(_, intent)| intent.region == region),
        "意图还没做完：停在预算上，不是没东西可搬"
    );
    let last_call = calls_of_each_interval.last().expect("推了间隔");
    assert_eq!(
        not_started_reason_of(last_call),
        Some(BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent),
        "扣完之后的兜底间隔不起跑，等下一次用户改动"
    );
    let intervals_that_started_a_run = calls_of_each_interval
        .iter()
        .filter(|call| not_started_reason_of(call).is_none())
        .count();
    assert_eq!(
        intervals_that_started_a_run, 1,
        "扣完那一份只要一个兜底间隔：之后的间隔一趟都不起跑"
    );
    let slots_used_before_the_next_change = session.compaction.observations.root_ring_slots_used;
    create_one_inode(&mut session, &mut devices);
    let after_the_next_change = run_started_by(
        session
            .run_compaction_in_the_background(&mut devices, pacing, &caller)
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    pool.devices = Some(devices);
    assert_eq!(
        after_the_next_change.batches_published, 1,
        "下一次用户改动给出新的一份：照常发一批"
    );
    assert!(
        session.compaction.observations.root_ring_slots_used > slots_used_before_the_next_change,
        "下一次用户改动之后整理发布又涨了"
    );
}

/// 定时兜底（D26 已定项 13 第 5 条）：开着时第一次调就查一次起跑判据；离上一次查不到一个间隔（999 毫秒）不查、不起跑；到了一个间隔（1000 毫秒）再查。
#[test]
fn the_timer_fallback_checks_the_start_judgement_again_only_once_an_interval_has_elapsed() {
    let mut pool = build_pool("compaction-timer-fallback-interval");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::ForcedToStartForTests,
    );
    let pacing = pacing_with(
        CompactionTimerFallback::EveryMilliseconds(non_zero(
            TIMER_FALLBACK_INTERVAL_IN_MILLISECONDS,
        )),
        CompactionWriteBandwidthLimit::Unlimited,
    );
    let caller = CallerForTests::at_the_start_with(ForegroundFsync::NoneWaiting);
    let at_the_start = session
        .run_compaction_in_the_background(&mut devices, pacing, &caller)
        .expect("盘表是挂载的那一份");
    assert!(
        !run_started_by(at_the_start, BackgroundCompactionTrigger::TimerFallback)
            .versions_published
            .is_empty(),
        "起点上第一次查：起跑、发了整理的发布"
    );
    caller.move_the_clock_to(TIMER_FALLBACK_INTERVAL_IN_MILLISECONDS - 1);
    let before_the_interval = session
        .run_compaction_in_the_background(&mut devices, pacing, &caller)
        .expect("盘表是挂载的那一份");
    assert_eq!(
        not_started_reason_of(&before_the_interval),
        Some(BackgroundCompactionNotStarted::TimerFallbackIntervalNotElapsed),
        "999 毫秒：还不到一个间隔"
    );
    caller.move_the_clock_to(TIMER_FALLBACK_INTERVAL_IN_MILLISECONDS);
    let at_the_interval = session
        .run_compaction_in_the_background(&mut devices, pacing, &caller)
        .expect("盘表是挂载的那一份");
    pool.devices = Some(devices);
    run_started_by(at_the_interval, BackgroundCompactionTrigger::TimerFallback);
    assert_eq!(
        session.compaction.observations.runs_started, 2,
        "1000 毫秒：到了一个间隔，再查一次、起跑；起点那一趟与这一趟合起来两趟"
    );
}

/// 默认的节奏参数（D26 已定项 13 第 5、6 条的默认值，第一轮正推的建议，待 C412 与 E10）：定时兜底关、写带宽不限；
/// 默认之下没有「该跑了」时后台入口不起跑，一个写都不发。
#[test]
fn the_background_entry_under_the_default_pacing_with_nothing_due_starts_no_run_and_writes_nothing()
{
    assert_eq!(
        CompactionPacing::BY_DEFAULT,
        pacing_with(
            CompactionTimerFallback::Off,
            CompactionWriteBandwidthLimit::Unlimited
        ),
        "默认：兜底关、写带宽不限"
    );
    let mut pool = build_pool("compaction-background-entry-default-pacing");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::ForcedToStartForTests,
    );
    let before_the_call = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            CompactionPacing::BY_DEFAULT,
            &CallerForTests::at_the_start_with(ForegroundFsync::NoneWaiting),
        )
        .expect("盘表是挂载的那一份");
    pool.devices = Some(devices);
    assert_eq!(
        not_started_reason_of(&call),
        Some(BackgroundCompactionNotStarted::NothingDueAndTheTimerFallbackOff),
        "没有用户改动、兜底关着：不起跑"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_call,
        "一个写都没发"
    );
}

/// 填开放段时留下几个空槽：签发那次发布要写的提交内生块比这多，就得开一个新聚簇段
/// （同 `compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs`）。
const FREE_SLOTS_LEFT_IN_THE_OPEN_SEGMENT: u64 = 2;
/// 一个聚簇段的槽数。
const SLOTS_OF_A_CLUSTER_SEGMENT: u64 = 64;

/// 分配器把开放段占到只剩两槽（只动分配器，不写盘）：之后签发那次发布要开一个新聚簇段。
fn fill_the_open_segment_up_to_two_free_slots(session: &mut MountedSession) {
    let open_segment = session
        .allocator
        .open_segment()
        .expect("可写挂载之后有开放段");
    let generation_of_the_filler = current_of(session).root.checkpoint_txg;
    // 迭代上界：开放段 64 槽，每轮多占一槽，占到只剩两槽为止。
    loop {
        let used_slots: u64 = session
            .allocator
            .records()
            .iter()
            .filter(|record| {
                record.device == DeviceIdentity(0)
                    && (open_segment.0..open_segment.0 + SLOTS_OF_A_CLUSTER_SEGMENT)
                        .contains(&record.slot.0)
            })
            .map(|record| u64::from(record.span_slots))
            .sum();
        if used_slots + FREE_SLOTS_LEFT_IN_THE_OPEN_SEGMENT >= SLOTS_OF_A_CLUSTER_SEGMENT {
            break;
        }
        session
            .allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, generation_of_the_filler)
            .expect("开放段里还有空槽");
    }
}

/// 第二样保证（岔路单第 11 行乙；攻方 K-h「上一条无产出就不签」）：开放段先填到只剩两槽，一趟显式入口在 50240 上签发（签发那次发布开了新段）、
/// 搬空 R、删意图：意图腾出的那一段只抵掉签发开的新段，全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假，这一趟停在「上一条无产出」，
/// 那一条完成的意图记在会话上。不来用户改动，下一趟进门读的是会话上跨趟留着的那一条，不签下一条，停在「上一条无产出」，一个写都不发。
/// 用户改动发成会清掉那一条（用户 2026-10-26 定「每次用户改动发成就清掉」），那一格在 `compaction_user_change_clears_the_last_completed_intent.rs`。
#[test]
fn the_completed_intent_kept_across_runs_stops_the_next_run_from_issuing_when_it_yielded_nothing() {
    let mut pool = build_pool("compaction-completed-intent-kept-across-runs");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::FollowTheWatermarks,
    );
    fill_the_open_segment_up_to_two_free_slots(&mut session);
    let emptying_the_region = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            emptying_the_region.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent { .. })
        ),
        "前提：签发开了新段、意图腾出的那一段只抵掉它，这一趟停在「上一条无产出」：{:?}",
        emptying_the_region.outcome
    );
    let completed = session
        .compaction
        .last_completed_intent
        .expect("那一趟完成的意图记在会话上");
    assert_eq!(completed.region, region_of_the_first_publish());
    let next_run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(
            next_run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent { .. })
        ),
        "下一趟读会话上的上一条完成的意图：无产出就不签，实际 {:?}",
        next_run.outcome
    );
    assert!(next_run.versions_published.is_empty(), "一个写都没发");
}

/// 抬 F 那一次改动之前建几次 inode（每次换下的 inode 叶容器与固定点进 defer，抬 F 之后回收得到）。只建 inode、不碰第一个文件：
/// 第一个文件那次发布留在段 50240 里的 extent 树节点还是现行那一版的，那一段照样有能搬的单元（覆盖写第一个文件会把它们都换掉）。
const INODE_CREATIONS_BEFORE_THE_CHANGE_RAISING_THE_FLOOR: u64 = 12;
/// 抬 F 之后再判的那一次改动：顺序写第一个文件，几个数据单元（比签发意图那一次的需求大得多）。
const DATA_UNITS_OF_THE_CHANGE_RAISING_THE_FLOOR: usize = 8;

/// 挂载、装上强制起跑的开关、建若干次 inode：强制起跑，每一次改动之后都记下「该跑了」（后台入口一次都没调）。
fn session_after_the_inode_creations(
    pool: &mut BuiltPool,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let (mut session, mut devices) = mounted_session_of(pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::ForcedToStartForTests,
    );
    for creation in 0..INODE_CREATIONS_BEFORE_THE_CHANGE_RAISING_THE_FLOOR {
        session
            .publish_user_change(
                &mut devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + creation + 1,
                },
            )
            .expect("建一个 inode");
    }
    (session, devices)
}

fn content_of_the_change_raising_the_floor() -> Vec<u8> {
    (0..DATA_UNITS_OF_THE_CHANGE_RAISING_THE_FLOOR * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 233).expect("小于 256"))
        .collect()
}

fn publish_the_change_raising_the_floor(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, Recorded)>,
    content: &[u8],
) -> UserChangePublished {
    session
        .publish_user_change(
            devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS
                    + INODE_CREATIONS_BEFORE_THE_CHANGE_RAISING_THE_FLOOR
                    + 1,
            },
        )
        .expect("顺序写第一个文件：被拒是空间不够就推抬 F 再判，发成")
}

fn slots_rewritten_by(version: &TransactionOutput) -> u64 {
    version.rewritten.iter().map(|role| role.span_slots()).sum()
}

/// 准入读数里每块盘的可用(d)（字节），按分配器里盘的次序：会话现行那一版之上、下一次发布之前的那一份。
fn available_bytes_on_each_device(session: &MountedSession) -> Vec<(DeviceIdentity, i128)> {
    admission_reading_before_a_publish(&session.allocator, session.current.file_version())
        .available_on_each_device()
        .into_iter()
        .map(|(device, available)| (device, available.0))
        .collect()
}

/// 在每块盘单元区的最高处隔离槽（只动分配器的隔离位，D28（挂载期承诺量） 已定项 1 第九项，不写盘），让每块盘的可用(d) 落进
/// [`lower_bound_in_bytes`, `lower_bound_in_bytes` + 16384)。隔离的槽在单元区最高处、离这一串发布要取的落点远，取落点照常取得到。
fn isolate_the_highest_slots_until_the_available_is_within_one_slot_above(
    session: &mut MountedSession,
    lower_bound_in_bytes: i128,
) {
    let slot_bytes = i128::from(SLOT_BYTES);
    for (device, available) in available_bytes_on_each_device(session) {
        let excess_in_bytes = available - lower_bound_in_bytes;
        assert!(
            excess_in_bytes >= 0,
            "{device:?}：可用 {available} 本来就低于要压到的 {lower_bound_in_bytes}"
        );
        let slots_to_isolate =
            u64::try_from(excess_in_bytes / slot_bytes).expect("非负、装得进 u64");
        if slots_to_isolate == 0 {
            continue;
        }
        let device_map = session
            .allocator
            .devices
            .iter()
            .find(|device_map| device_map.device == device)
            .expect("读数里的盘就是分配器的盘");
        let unit_area_end = device_map.unit_area_start().slot().0 + device_map.unit_area_slots();
        session.allocator.isolate_abandoned(
            device,
            SlotNumber(unit_area_end - slots_to_isolate),
            slots_to_isolate,
        );
    }
    for (device, available) in available_bytes_on_each_device(session) {
        assert!(
            available >= lower_bound_in_bytes && available < lower_bound_in_bytes + slot_bytes,
            "{device:?}：隔离之后可用 {available} 落在 [{lower_bound_in_bytes}, +16384) 里"
        );
    }
}

/// 第三样保证（第三轮判决第二节 G8；岔路单第 11 行乙）：一次用户改动被准入拒、推一串抬 F、回收之后再判发成，抬 F 与再判之间插不进整理批。
/// 先在同样建、同样建过 inode 的另一个池上量出这次改动的需求；本池把每块盘的可用(d) 压到需求之下一个槽以内，改动第一次被式子那一判拒、推抬 F、
/// 回收之后再判发成。抬 F 那一串最后一次发布之后紧接着就是改动那一版（txg 连着），这一次调用里整理一趟都没起跑、一个意图都没签；
/// 整理在之后调后台入口时才起跑，发的每一版都在改动之后。
#[test]
fn a_user_change_refused_for_space_raises_the_floor_and_rejudges_without_a_compaction_batch_in_between(
) {
    let content = content_of_the_change_raising_the_floor();
    let mut pool_measuring_the_demand =
        build_pool("compaction-not-between-the-floor-raise-and-the-rejudge-measuring");
    let (mut session_measuring_the_demand, mut devices_measuring_the_demand) =
        session_after_the_inode_creations(&mut pool_measuring_the_demand);
    let unconstrained = publish_the_change_raising_the_floor(
        &mut session_measuring_the_demand,
        &mut devices_measuring_the_demand,
        &content,
    );
    pool_measuring_the_demand.devices = Some(devices_measuring_the_demand);
    assert!(
        unconstrained.floor_raises.is_empty() && unconstrained.versions_published.len() == 1,
        "量需求的那个池不压余量：一次就发成"
    );
    let demand_in_slots = slots_rewritten_by(&unconstrained.versions_published[0]);

    let mut pool = build_pool("compaction-not-between-the-floor-raise-and-the-rejudge");
    let (mut session, mut devices) = session_after_the_inode_creations(&mut pool);
    let slot_bytes = i128::from(SLOT_BYTES);
    isolate_the_highest_slots_until_the_available_is_within_one_slot_above(
        &mut session,
        i128::from(demand_in_slots) * slot_bytes - slot_bytes,
    );
    let published = publish_the_change_raising_the_floor(&mut session, &mut devices, &content);
    assert!(
        !published.refusals_that_pushed_the_floor_raises.is_empty(),
        "可用压到需求之下：第一次被准入拒、推了抬 F"
    );
    let last_floor_raise_publish = published
        .floor_raises
        .iter()
        .flat_map(|raised| raised.publishes.iter())
        .last()
        .expect("推了抬 F 的空发布");
    assert_eq!(
        published.versions_published.len(),
        1,
        "抬 F 之外只发了改动那一版"
    );
    let change = &published.versions_published[0];
    assert_eq!(
        txg_of(change),
        txg_of(last_floor_raise_publish) + 1,
        "抬 F 那一串最后一次发布之后紧接着就是再判发成的那一版：中间没有别的发布"
    );
    assert_eq!(
        session.compaction.observations,
        CompactionObservations::default(),
        "这一次调用里整理一趟都没起跑"
    );
    let run = run_started_by(
        session
            .run_compaction_in_the_background(
                &mut devices,
                CompactionPacing::BY_DEFAULT,
                &CallerForTests::at_the_start_with(ForegroundFsync::NoneWaiting),
            )
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    pool.devices = Some(devices);
    assert!(
        run.versions_published
            .iter()
            .all(|version| txg_of(version) > txg_of(change)),
        "整理由后台入口在改动之后起跑：它发的每一版都在改动之后（这一趟停在 {:?}）",
        run.outcome
    );
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

/// 挂载、强制选段 50240 一批一个，只发签发那一次：意图在 logged_ops 里、R 排他。
fn session_with_the_intent_issued_on_the_segment_of_the_first_publish(
    pool: &mut BuiltPool,
) -> (MountedSession, Vec<(DeviceIdentity, Recorded)>) {
    let (mut session, mut devices) = mounted_session_of(pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::FollowTheWatermarks,
    );
    let issuing = run_one_compaction_publish(&mut session, &mut devices);
    assert!(
        issuing.versions_published.len() == 1
            && live_compaction_intent_of_the_entries(&current_of(&session).logged_operations)
                .is_some_and(|(_, intent)| intent.region == region_of_the_first_publish()),
        "只发了签发那一次，意图在 50240 上：{:?}",
        issuing.outcome
    );
    (session, devices)
}

/// 一批搬到目的地的单元占的槽数：每个搬走的单元按这一版里它目的落点那条分配记录的跨度。
fn slots_relocated_by(version: &TransactionOutput) -> u64 {
    version
        .units_relocated_by_this_publish
        .iter()
        .map(|relocated| {
            let destination = relocated.destination[0];
            let record = version
                .allocation_records
                .iter()
                .find(|record| {
                    record.device == destination.device && record.slot == destination.slot
                })
                .expect("目的地记了一条分配记录");
            u64::from(record.span_slots)
        })
        .sum()
}

/// 签发之后那一批的需求（这次重写的角色的槽数）与批次最坏占用（搬到目的地的槽数）：在同样建、同样签发过的另一个池上发一次量出来。
fn demand_and_occupancy_in_slots_of_the_batch_after_the_issue(tag: &str) -> (u64, u64) {
    let mut pool = build_pool(tag);
    let (mut session, mut devices) =
        session_with_the_intent_issued_on_the_segment_of_the_first_publish(&mut pool);
    let batch = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    let published = batch
        .versions_published
        .first()
        .expect("不压余量：那一批照常发成");
    let occupancy_in_slots = slots_relocated_by(published);
    assert!(occupancy_in_slots >= 1, "那一批搬了单元");
    (slots_rewritten_by(published), occupancy_in_slots)
}

/// C906（D26 已定项 3 第 2 句「整理不是释放路径，它走常规准入，批次最坏占用计入『已承诺预留』」）：签发之后把每块盘的可用(d) 压到
/// 「需求 + 批次最坏占用」之下一个槽以内（仍不少于需求本身：不算批次占用就放得下）。那一批被式子那一判拒：暂停（撤 R 排他），意图留在 logged_ops 里，
/// 盘上逐字节不变（系统配置槽、根环里自证过的根、录制流的步数都与这一趟之前相同）。
#[test]
fn a_compaction_batch_that_would_push_the_available_below_the_demand_is_refused_by_the_admission_and_writes_nothing(
) {
    let (demand_in_slots, occupancy_in_slots) =
        demand_and_occupancy_in_slots_of_the_batch_after_the_issue(
            "compaction-batch-admission-refused-measuring",
        );
    let mut pool = build_pool("compaction-batch-admission-refused");
    let (mut session, mut devices) =
        session_with_the_intent_issued_on_the_segment_of_the_first_publish(&mut pool);
    let slot_bytes = i128::from(SLOT_BYTES);
    isolate_the_highest_slots_until_the_available_is_within_one_slot_above(
        &mut session,
        i128::from(demand_in_slots + occupancy_in_slots) * slot_bytes - slot_bytes,
    );
    for (device, available) in available_bytes_on_each_device(&session) {
        assert!(
            available >= i128::from(demand_in_slots) * slot_bytes,
            "{device:?}：可用 {available} 不少于需求本身，压到需求之下的是批次最坏占用"
        );
    }
    let intent_before =
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations);
    let before_the_batch = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    assert!(
        matches!(
            refused.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PausedShortOfSpace)
        ),
        "准入拒：暂停，实际 {:?}",
        refused.outcome
    );
    assert!(refused.versions_published.is_empty(), "那一批没发");
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_batch,
        "盘上逐字节不变"
    );
    assert_eq!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations),
        intent_before,
        "意图留在盘上，下次再试"
    );
    assert!(
        session
            .allocator
            .compaction()
            .live_intent()
            .is_some_and(|live| !live.is_the_region_excluded_from_placement),
        "暂停：撤了 R 排他、意图留着"
    );
}

/// C906 照常放行那一面：可用(d) 压到正好不少于「需求 + 批次最坏占用」（一个槽以内），那一批照常发成、搬了单元。
#[test]
fn a_compaction_batch_whose_demand_and_worst_case_occupancy_fit_the_available_is_admitted_and_published(
) {
    let (demand_in_slots, occupancy_in_slots) =
        demand_and_occupancy_in_slots_of_the_batch_after_the_issue(
            "compaction-batch-admission-admitted-measuring",
        );
    let mut pool = build_pool("compaction-batch-admission-admitted");
    let (mut session, mut devices) =
        session_with_the_intent_issued_on_the_segment_of_the_first_publish(&mut pool);
    isolate_the_highest_slots_until_the_available_is_within_one_slot_above(
        &mut session,
        i128::from(demand_in_slots + occupancy_in_slots) * i128::from(SLOT_BYTES),
    );
    let admitted = run_one_compaction_publish(&mut session, &mut devices);
    pool.devices = Some(devices);
    assert!(
        matches!(
            admitted.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget: 1 })
        ),
        "照常发成、停在预算：{:?}",
        admitted.outcome
    );
    let published = admitted.versions_published.first().expect("那一批发成");
    assert_eq!(
        slots_relocated_by(published),
        occupancy_in_slots,
        "搬的就是量需求那个池上的那一批"
    );
}

/// 第一批读完设置之后注入一次「算着出了错」的钩子（只供测试，第 4 条那一支今天在产品路径上走不到）。
fn report_a_compute_error_in_the_batch_numbered(
    batch_to_fail: u64,
    batches_seen: &Cell<u64>,
    background_compute: &mut BackgroundComputeOfThisMount,
) {
    batches_seen.set(batches_seen.get() + 1);
    if batches_seen.get() == batch_to_fail {
        background_compute.report_a_compute_error_in_the_batch_under_way_for_tests();
    }
}

/// 停着切换（用户 2026-10-26 定「停着时收到切换请求就当场解除停下」；第三轮判决第二节 G5）：签发那一批出错，这一批放弃、一个写都不发，
/// 整理停下；停着时再调显式入口一个写都不发，停在「此前出错停着」；一次通过校验的切换请求当场解除，下一趟从第一批起用新设置照常推。
#[test]
fn a_switch_request_received_while_the_compaction_is_stopped_after_a_compute_error_lifts_the_stop()
{
    let mut pool = build_pool("compaction-stopped-after-a-compute-error-and-lifted-by-a-switch");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::FollowTheWatermarks,
    );
    let before_the_error = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let batches_seen = Cell::new(0);
    let erroring = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut |background_compute| {
                report_a_compute_error_in_the_batch_numbered(1, &batches_seen, background_compute);
            },
        )
        .expect("整理一趟");
    assert!(
        matches!(
            erroring.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ),
        "{:?}",
        erroring.outcome
    );
    assert!(
        erroring.versions_published.is_empty(),
        "出错的那一批放弃，一个写都没发"
    );
    assert!(
        session
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "整理停下"
    );
    let while_stopped = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            while_stopped.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && while_stopped.versions_published.is_empty(),
        "停着：一个写都不发，{:?}",
        while_stopped.outcome
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_error,
        "出错那一趟与停着那一趟都没动盘"
    );
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    session
        .switch_background_compute_from_the_next_batch(
            admissible_gpu_settings_naming_both_cards(),
            Some(gpu_compute.clone()),
        )
        .expect("全合法的 gpu 设置：切换收下");
    assert!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "停着时收到通过校验的切换请求：当场解除停下"
    );
    let after_the_switch = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        !after_the_switch.versions_published.is_empty(),
        "解除之后照常推：{:?}",
        after_the_switch.outcome
    );
    assert!(
        requested_backends_of(&after_the_switch.background_compute_of_each_batch)
            .iter()
            .all(|backend| *backend == BackgroundComputeBackend::Gpu),
        "从下一批起用切换到的 gpu 设置"
    );
}

/// 出错之前就挂着的待生效切换（用户 2026-10-26 定）：签发那一批先请求切到 gpu（记成待生效）、再注入出错；那一刻它也算停着时收到的切换请求，
/// 当场解除，整理不记成停着，出错那一批的记账照样报出这个错；下一趟第一批就换上 gpu、照常推。
#[test]
fn a_switch_pending_before_a_compute_error_lifts_the_stop_at_the_moment_of_the_stop() {
    let mut pool = build_pool("compaction-switch-pending-before-a-compute-error");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::FollowTheWatermarks,
    );
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let batches_seen = Cell::new(0);
    let erroring = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut |background_compute| {
                if batches_seen.get() == 0 {
                    background_compute
                        .request_switch_from_the_next_batch(
                            admissible_gpu_settings_naming_both_cards(),
                            Some(gpu_compute.clone()),
                        )
                        .expect("全合法的 gpu 设置：切换收下，记成待生效");
                }
                report_a_compute_error_in_the_batch_numbered(1, &batches_seen, background_compute);
            },
        )
        .expect("整理一趟");
    assert!(
        matches!(
            erroring.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error:
                        BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError
                }
            )
        ),
        "{:?}",
        erroring.outcome
    );
    assert_eq!(
        erroring
            .background_compute_of_each_batch
            .last()
            .map(|batch| batch.compute_error_that_abandoned_the_batch.clone()),
        Some(Some(BackgroundComputeError::InjectedByTheTestOnlyHook)),
        "停下当场解除、整理不记成停着，这一批的记账照样报出这个错（第 4 条「停下并报错」）"
    );
    assert!(
        session
            .background_compute()
            .stopped_after_a_compute_error()
            .is_empty(),
        "出错之前挂着的切换在停下那一刻算数：不记成停着"
    );
    assert_eq!(
        session
            .background_compute()
            .requested_from_the_next_batch()
            .map(|admitted| admitted.settings().clone()),
        Some(admissible_gpu_settings_naming_both_cards()),
        "切换仍待生效，下一批开头换上"
    );
    let after_the_error = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        !after_the_error.versions_published.is_empty(),
        "照常推：{:?}",
        after_the_error.outcome
    );
    assert_eq!(
        requested_backends_of(&after_the_error.background_compute_of_each_batch).first(),
        Some(&BackgroundComputeBackend::Gpu),
        "下一趟第一批就用 gpu 设置"
    );
}

/// 前台有 fsync 在等（D26 已定项 13 第 6 条）：用户改动之后后台入口起跑，不发下一批——一批都没发、盘上不变，「该跑了」放回去；
/// fsync 不在等时再调，照常发那一批。
#[test]
fn the_background_entry_sends_no_batch_while_a_foreground_fsync_is_waiting() {
    let mut pool = build_pool("compaction-yields-to-a-foreground-fsync");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::ForcedToStartForTests,
    );
    create_one_inode(&mut session, &mut devices);
    let caller = CallerForTests::at_the_start_with(ForegroundFsync::Waiting);
    let before_the_call = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let yielded = run_started_by(
        session
            .run_compaction_in_the_background(&mut devices, CompactionPacing::BY_DEFAULT, &caller)
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    assert!(
        matches!(
            yielded.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::YieldedToAForegroundFsync)
        ) && yielded.versions_published.is_empty(),
        "fsync 在等：一批都不发，{:?}",
        yielded.outcome
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_call,
        "盘上不变"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "让路是缓发：「该跑了」放回去"
    );
    caller.foreground_fsync.set(ForegroundFsync::NoneWaiting);
    let resumed = run_started_by(
        session
            .run_compaction_in_the_background(&mut devices, CompactionPacing::BY_DEFAULT, &caller)
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    pool.devices = Some(devices);
    assert_eq!(resumed.batches_published, 1, "fsync 不在等：照常发那一批");
}

/// 写带宽上限（D26 已定项 13 第 6 条）：上限 1 字节每秒。用户改动之后后台入口起跑，签发那一次写了单元字节，这一秒的窗口就到了上限，
/// 不发下一批、「该跑了」放回去；同一秒里再调一批都不发；钟走过一秒，新窗口里照常发那一批。
#[test]
fn the_background_entry_sends_no_next_batch_once_the_write_bandwidth_limit_of_the_window_is_reached(
) {
    let mut pool = build_pool("compaction-write-bandwidth-limit");
    let (mut session, mut devices) = mounted_session_of(&mut pool);
    force_one_unit_batches_on_the_segment_of_the_first_publish(
        &mut session,
        CompactionRunForcing::ForcedToStartForTests,
    );
    create_one_inode(&mut session, &mut devices);
    let pacing = pacing_with(
        CompactionTimerFallback::Off,
        CompactionWriteBandwidthLimit::BytesPerSecond(non_zero(1)),
    );
    let caller = CallerForTests::at_the_start_with(ForegroundFsync::NoneWaiting);
    let issuing = run_started_by(
        session
            .run_compaction_in_the_background(&mut devices, pacing, &caller)
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    assert_eq!(issuing.versions_published.len(), 1, "只发了签发那一次");
    assert!(
        matches!(
            issuing.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::WriteBandwidthLimitReached {
                limit_in_bytes_per_second: 1,
                ..
            })
        ),
        "签发写了单元字节，这一秒到了上限：{:?}",
        issuing.outcome
    );
    let in_the_same_second = run_started_by(
        session
            .run_compaction_in_the_background(&mut devices, pacing, &caller)
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    assert!(
        in_the_same_second.versions_published.is_empty(),
        "同一秒里一批都不发：{:?}",
        in_the_same_second.outcome
    );
    caller.move_the_clock_to(1000);
    let next_second = run_started_by(
        session
            .run_compaction_in_the_background(&mut devices, pacing, &caller)
            .expect("盘表是挂载的那一份"),
        BackgroundCompactionTrigger::DueAfterAUserChange,
    );
    pool.devices = Some(devices);
    assert_eq!(next_second.batches_published, 1, "新窗口里照常发那一批");
}
