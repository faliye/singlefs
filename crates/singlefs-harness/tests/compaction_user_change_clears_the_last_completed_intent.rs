//! 每次用户改动发成就清掉会话上跨趟留着的上一条完成的意图（用户 2026-10-26 定「每次用户改动发成就清掉」，预算照岔路单
//! `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲共用 `B_gc`；`records/2026-10-19-里程碑四GPU垃圾回收开工.md`「调查员交回」那一行；
//! D26（后台整理与放置回收） 已定项 13 第 6 条「签下一条意图那一判跨趟留着」）：上一条完成的意图
//! （`compaction_plan::CompactionOfThisMount::last_completed_intent`）留到下一次用户改动发成为止。没有用户改动时一趟趟推都停在
//! 「上一条无产出」、一个写都不发；一次用户改动发成之后，后台入口那一趟越过那一判、照常往下选段签发；被拒、没发成的改动不清它。
//! 池照 `compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs` 会话那一格的做法：从第一个文件起两次顺序写之后的镜像
//! 可写挂载，开放段先填到只剩两槽，强制选装着用户数据的那一段、一批三个，签发那次发布开了新段、意图腾空 R，全空段数回到签发之前那个数，
//! 这一趟停在「上一条无产出」（不靠用户改动占掉全空段）。之后换成产品的选段（不强制选段）。
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::allocator::UnitFootprint;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{
    CallerMonotonicClockReading, CallerOfTheBackgroundCompaction, CompactionBatchSize,
    CompactionBatchUnits, CompactionPacing, CompactionRunForcing, CompactionRunStop,
    CompactionStartJudgement, CompactionSwitches, CompactionTimerFallback,
    CompactionWriteBandwidthLimit, ForegroundFsync,
};
use singlefs_core::logged_operations::{CompactionIntent, CompactionRegion};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{
    BackgroundCompactionCall, BackgroundCompactionTrigger, CompactionRunOutcome, MountedSession,
    UserChange, UserChangeRefused,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::SharedStream;
use std::time::Duration;

/// 填开放段时留下几个空槽：签发那次发布要写的提交内生块比这多，就得开一个新聚簇段。
const FREE_SLOTS_LEFT_IN_THE_OPEN_SEGMENT: u64 = 2;
/// 一个聚簇段的槽数。
const SLOTS_OF_A_CLUSTER_SEGMENT: u64 = 64;
/// 两次顺序写之后 0 号盘上装着用户数据的那一段（同 `compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs`）。
const REGION_START_SLOT_AFTER_THE_TWO_SEQUENTIAL_WRITES: u64 = 50_496;
/// 停在「上一条无产出」之后、不来用户改动连推几趟。
const RUNS_PUSHED_WITHOUT_A_USER_CHANGE: usize = 3;
/// 用户改动写进记录的时间秒：比历史里那几次写都晚。
const WRITE_TIME_SECONDS_OF_THE_USER_CHANGE: u64 = 1_900_000_000;

/// 两盘的设备表，盘是内存里的 4 GiB 稀疏盘。
type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 从第一个文件起两次顺序写（152 个、290 个数据单元）之后的整份镜像（每一版与模型对得上）。
/// 抄自 `compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue.rs` 的同名函数：会话那一格「上一条无产出」是在这份镜像上量出来的。
fn image_after_the_two_sequential_writes() -> MemoryPool {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 152,
                    selector: 9_852_620_462_860_606_767,
                },
                fill_seed: 2_832_512_656_265_544_978,
            }),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 290,
                    selector: 15_334_540_613_569_234_429,
                },
                fill_seed: 10_383_977_899_441_184_023,
            }),
        ],
    };
    let mut image_after_the_last_step: Option<MemoryPool> = None;
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| image_after_the_last_step = Some(observation.image.clone()),
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "两次顺序写每一版与模型对得上：{:?}",
        run.outcomes
    );
    image_after_the_last_step.expect("走完的历史每一步之后都交镜像")
}

/// 会话装上整理只供测试的开关：一批三个、起跑判据恒为真（用户改动发成之后记下「该跑了」，交后台入口去跑），`forced_region` 照给的，其余照产品值。
fn force_three_unit_batches_on(
    session: &mut MountedSession,
    forced_region: Option<CompactionRegion>,
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
                CompactionBatchUnits::new(3).expect("至少 1"),
            ),
            forced_region,
            run_forcing: CompactionRunForcing::ForcedToStartForTests,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
}

/// 会话停在「上一条无产出」时的样子：镜像可写挂载，开放段填到只剩两槽，强制选 50496、一批三个推一趟：签发开了新段、意图腾空 R、同一次发布删掉，
/// 这一趟停在「上一条无产出」，那一条完成的意图记在会话上。之后换成产品的选段。交回会话、盘表与那一条完成的意图。
fn session_stopped_on_no_yield() -> (MountedSession, Devices, CompactionIntent) {
    let image = image_after_the_two_sequential_writes();
    let width = HistoryDeviceWidth::FourGibibytes;
    let mut devices: Devices = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect();
    let mounted =
        mount_writable(&width.parameters(), &mut devices).expect("两次顺序写之后的镜像可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let open_segment = session
        .allocator
        .open_segment()
        .expect("可写挂载之后有开放段");
    let generation_of_the_filler = session
        .current
        .version_of_trees()
        .expect("有树的一版")
        .root
        .checkpoint_txg;
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
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(REGION_START_SLOT_AFTER_THE_TWO_SEQUENTIAL_WRITES),
    )
    .expect("64 的倍数");
    force_three_unit_batches_on(&mut session, Some(region));
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent { .. })
        ),
        "前提：意图腾出的那一段只抵掉签发开的新段，这一趟停在「上一条无产出」，实际 {:?}",
        run.outcome
    );
    let completed = session
        .compaction
        .last_completed_intent
        .expect("前提：那一趟完成的意图记在会话上");
    assert_eq!(completed.region, region, "前提：完成的是 50496 上那一条");
    force_three_unit_batches_on(&mut session, None);
    (session, devices, completed)
}

/// 后台入口的调用方：钟停在起点，前台没有 fsync 在等。
struct CallerWithNoForegroundFsync;

impl CallerOfTheBackgroundCompaction for CallerWithNoForegroundFsync {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(Duration::ZERO)
    }

    fn foreground_fsync(&self) -> ForegroundFsync {
        ForegroundFsync::NoneWaiting
    }
}

/// 后台入口的节奏参数：定时兜底关着、写带宽不限（每个字段都写出来）。
fn pacing_with_the_timer_fallback_off_and_no_write_bandwidth_limit() -> CompactionPacing {
    CompactionPacing {
        timer_fallback: CompactionTimerFallback::Off,
        write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
    }
}

/// 停在「上一条无产出」之后不来用户改动连推几趟：每一趟都读到会话上跨趟留着的那一条、停在「上一条无产出」、一个写都不发、一条意图都不签。
/// 接着发成一次用户改动（建一个空 inode）：会话上的上一条完成的意图清掉，起跑判据记下「该跑了」；后台入口因此起跑的那一趟不再停在
/// 「上一条无产出」，照常往下签新意图：产品选段选了 50496 之外的一段、扫过，走到签发前判。这个池子在这一刻 R 外空洞不够
/// （那一段 10 个单元、R 外空洞 6 个），签发前判不过、这一趟不签：这一条钉的是越过了「上一条无产出」那一判，不钉签成。
/// 判别力：发成之后不清（`MountedSession::publish_user_change` 不置空），后台那一趟读到那一条、停在「上一条无产出」，这一条红；
/// 每一趟开头就清（退回一趟里的局部变量），不来用户改动的第二趟就越过那一判、停在签发前判，这一条也红。
#[test]
fn a_published_user_change_clears_the_last_completed_intent_and_the_next_background_run_goes_past_the_no_yield_stop(
) {
    let (mut session, mut devices, completed) = session_stopped_on_no_yield();
    let intents_issued_when_stopped = session.compaction.observations.intents_issued;
    // 迭代上界是 RUNS_PUSHED_WITHOUT_A_USER_CHANGE；跨轮携带的是会话与盘。
    for run_number in 1..=RUNS_PUSHED_WITHOUT_A_USER_CHANGE {
        let pushed = session.run_compaction(&mut devices).expect("整理一趟");
        assert!(
            matches!(
                pushed.outcome,
                CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent { .. })
            ),
            "不来用户改动的第 {run_number} 趟照旧读到那一条、停在「上一条无产出」，实际 {:?}",
            pushed.outcome
        );
        assert!(
            pushed.versions_published.is_empty(),
            "不来用户改动的第 {run_number} 趟一个写都没发（不空转）"
        );
    }
    assert_eq!(
        session.compaction.observations.intents_issued, intents_issued_when_stopped,
        "不来用户改动的那几趟一条意图都没签"
    );
    assert_eq!(
        session.compaction.last_completed_intent,
        Some(completed),
        "不来用户改动，会话上的上一条完成的意图一直留着"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_USER_CHANGE,
            },
        )
        .expect("建一个空 inode");
    assert_eq!(
        session.compaction.last_completed_intent, None,
        "用户改动发成：会话上的上一条完成的意图清掉"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::DueAfterAUserChange,
        "前提：起跑判据恒为真，改动发成之后记下「该跑了」"
    );
    let call = session
        .run_compaction_in_the_background(
            &mut devices,
            pacing_with_the_timer_fallback_off_and_no_write_bandwidth_limit(),
            &CallerWithNoForegroundFsync,
        )
        .expect("后台入口");
    let run = match call {
        BackgroundCompactionCall::Ran {
            trigger: BackgroundCompactionTrigger::DueAfterAUserChange,
            run,
        } => *run,
        other
        @ (BackgroundCompactionCall::NotStarted(_) | BackgroundCompactionCall::Ran { .. }) => {
            panic!("后台入口应当因用户改动之后的「该跑了」起跑，实际 {other:?}")
        }
    };
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. }
            )
        ),
        "用户改动发成之后后台那一趟不停在「上一条无产出」，往下选了一段、走到签发前判（这个池子 R 外空洞不够），实际 {:?}",
        run.outcome
    );
}

/// 停在「上一条无产出」之后交一次被拒的用户改动（盘表次序与挂载核过的对调，在任何读写之前拒）：会话上的上一条完成的意图照旧留着，
/// 下一趟照旧停在「上一条无产出」、一个写都不发。判别力：在发成之前（进门就）清那一条，被拒之后它就空了、下一趟签新意图，这一条红。
#[test]
fn a_refused_user_change_keeps_the_last_completed_intent_and_the_next_run_still_stops_on_no_yield()
{
    let (mut session, mut devices, completed) = session_stopped_on_no_yield();
    devices.reverse();
    let refused = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_USER_CHANGE,
            },
        )
        .expect_err("盘表次序与挂载核过的不同：被拒");
    devices.reverse();
    assert!(
        matches!(
            refused,
            UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
        ),
        "被拒在盘表那一判：{refused:?}"
    );
    assert_eq!(
        session.compaction.last_completed_intent,
        Some(completed),
        "改动没发成：会话上的上一条完成的意图照旧留着"
    );
    let next_run = session.run_compaction(&mut devices).expect("整理一趟");
    assert!(
        matches!(
            next_run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoYieldFromTheLastIntent { .. })
        ),
        "下一趟照旧读到留着的那一条、停在「上一条无产出」，实际 {:?}",
        next_run.outcome
    );
    assert!(next_run.versions_published.is_empty(), "一个写都没发");
}
